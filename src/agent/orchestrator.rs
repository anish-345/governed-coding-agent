use crate::agent::codegen::CodeGenerator;
use crate::agent::planner::Planner;
use crate::config::AppConfig;
use crate::error::Result;
use crate::github_tools::{BranchManager, IssueFetcher, PrManager};
use crate::llm::LlmClient;
use crate::sandbox::SandboxExecutor;
use crate::tavily::TavilyClient;
use crate::verification::{
    Artifact, ArtifactKind, ClaimCategory, EvidenceCollector, EvidenceLink,
    ReportGenerator, VerificationReport,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Status of an agent run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentRunStatus {
    Queued,
    AnalyzingIssue,
    GeneratingCode,
    ExecutingTests,
    FixingCode { attempt: u32 },
    GeneratingReport,
    CreatingPr,
    Completed,
    Failed { reason: String },
}

/// Tracks the state of a single agent run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunState {
    pub run_id: String,
    pub issue_url: String,
    pub status: AgentRunStatus,
    pub pr_url: Option<String>,
    pub report: Option<VerificationReport>,
}

/// The main agent orchestrator. Ties all subsystems together.
pub struct AgentOrchestrator {
    config: AppConfig,
    llm: LlmClient,
    planner: Planner,
    codegen: CodeGenerator,
    sandbox: Arc<dyn SandboxExecutor>,
    issue_fetcher: IssueFetcher,
    branch_manager: BranchManager,
    pr_manager: PrManager,
    /// Active runs, keyed by run_id.
    runs: Arc<Mutex<HashMap<String, AgentRunState>>>,
}

impl AgentOrchestrator {
    /// Create a new orchestrator from config.
    pub fn new(config: AppConfig, sandbox: Arc<dyn SandboxExecutor>) -> Result<Self> {
        let llm = LlmClient::new(&config);
        let tavily = TavilyClient::new_optional(&config.tavily_api_key);
        let planner = Planner::new(llm.clone(), tavily);
        let codegen = CodeGenerator::new(llm.clone());
        let issue_fetcher = IssueFetcher::new(&config.github_token)?;
        let branch_manager = BranchManager::new(&config.github_token)?;
        let pr_manager = PrManager::new(&config.github_token)?;

        Ok(Self {
            config,
            llm,
            planner,
            codegen,
            sandbox,
            issue_fetcher,
            branch_manager,
            pr_manager,
            runs: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Get the status of a run.
    pub fn get_run(&self, run_id: &str) -> Option<AgentRunState> {
        self.runs.lock().unwrap().get(run_id).cloned()
    }

    /// Get a report for a completed run.
    pub fn get_report(&self, run_id: &str) -> Option<VerificationReport> {
        self.runs
            .lock()
            .unwrap()
            .get(run_id)
            .and_then(|r| r.report.clone())
    }

    fn update_status(&self, run_id: &str, status: AgentRunStatus) {
        if let Some(run) = self.runs.lock().unwrap().get_mut(run_id) {
            info!(run_id = %run_id, status = ?status, "Status update");
            run.status = status;
        }
    }

    /// Main entry point: resolve a GitHub issue end-to-end.
    /// Returns the run_id immediately; work happens in the background.
    pub fn start_run(self: &Arc<Self>, issue_url: String) -> String {
        let run_id = Uuid::new_v4().to_string();

        let state = AgentRunState {
            run_id: run_id.clone(),
            issue_url: issue_url.clone(),
            status: AgentRunStatus::Queued,
            pr_url: None,
            report: None,
        };

        self.runs.lock().unwrap().insert(run_id.clone(), state);

        let this = Arc::clone(self);
        let rid = run_id.clone();
        tokio::spawn(async move {
            if let Err(e) = this.execute_run(&rid).await {
                error!(run_id = %rid, error = %e, "Agent run failed");
                this.update_status(
                    &rid,
                    AgentRunStatus::Failed {
                        reason: e.to_string(),
                    },
                );
            }
        });

        run_id
    }

    /// Execute the full agent pipeline.
    async fn execute_run(self: &Arc<Self>, run_id: &str) -> Result<()> {
        let issue_url = {
            let runs = self.runs.lock().unwrap();
            runs.get(run_id).unwrap().issue_url.clone()
        };

        // ── Parse issue URL ──
        let (owner, repo, issue_number) = IssueFetcher::parse_issue_url(&issue_url)?;

        // ── Fetch issue ──
        self.update_status(run_id, AgentRunStatus::AnalyzingIssue);
        let issue = self.issue_fetcher.fetch(&owner, &repo, issue_number).await?;

        // ── Create evidence collector ──
        let collector = EvidenceCollector::new(run_id, &issue.url, &issue.title);

        // ── Phase 1: Plan ──
        let plan = self
            .planner
            .create_plan(&issue.title, &issue.body, &collector)
            .await?;

        let plan_json =
            serde_json::to_string_pretty(&plan).unwrap_or_else(|_| "{}".into());

        // ── Phase 2: Code Generation ──
        self.update_status(run_id, AgentRunStatus::GeneratingCode);

        // For now, pass empty existing code context (would need repo cloning for real use).
        let existing_code = format!(
            "Repository: {}/{}\nFiles to modify: {:?}",
            owner, repo, plan.files_to_modify
        );

        let mut changes = self
            .codegen
            .generate(&plan_json, &existing_code, &collector)
            .await?;

        // ── Phase 3: Sandbox Execution + Retry Loop ──
        const MAX_RETRIES: u32 = 3;
        let mut last_error_output = String::new();

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                self.update_status(
                    run_id,
                    AgentRunStatus::FixingCode { attempt },
                );
                changes = self
                    .codegen
                    .fix(
                        &serde_json::to_string(&changes).unwrap_or_default(),
                        &last_error_output,
                        &collector,
                    )
                    .await?;
            }

            self.update_status(run_id, AgentRunStatus::ExecutingTests);

            let file_map = CodeGenerator::to_file_map(&changes);
            let sandbox_id = self.sandbox.prepare(&file_map).await?;

            let results = self.sandbox.run_verification_suite(&sandbox_id).await?;
            self.sandbox.cleanup(&sandbox_id).await?;

            // Record sandbox results as artifacts and link to claims.
            let mut all_passed = true;

            for result in &results {
                let (kind, category) = if result.command.contains("check") {
                    (ArtifactKind::CheckOutput, ClaimCategory::NoCompileErrors)
                } else if result.command.contains("clippy") {
                    (ArtifactKind::ClippyOutput, ClaimCategory::NoClippyWarnings)
                } else {
                    (ArtifactKind::TestOutput, ClaimCategory::TestsPassing)
                };

                let artifact = Artifact::new(kind, result.full_output(), None);
                let artifact_id = collector.record_artifact(artifact);

                // Find matching claim.
                let report = collector.finalize();
                if let Some(claim) = report
                    .claims
                    .iter()
                    .find(|c| c.category == category)
                {
                    if result.success() {
                        collector.link_evidence(EvidenceLink::supports(
                            &claim.id,
                            &artifact_id,
                            format!("{} passed (exit code 0)", result.command),
                        ));
                    } else {
                        all_passed = false;
                        collector.link_evidence(EvidenceLink::contradicts(
                            &claim.id,
                            &artifact_id,
                            format!("{} failed (exit code {})", result.command, result.exit_code),
                        ));
                    }
                }
            }

            if all_passed {
                info!(attempt = attempt, "All checks passed");
                break;
            }

            if attempt == MAX_RETRIES {
                warn!("Max retries reached, proceeding with best-effort report");
                break;
            }

            // Collect error output for the fix attempt.
            last_error_output = results
                .iter()
                .filter(|r| !r.success())
                .map(|r| r.full_output())
                .collect::<Vec<_>>()
                .join("\n---\n");
        }

        // ── Phase 4: Verification Report ──
        self.update_status(run_id, AgentRunStatus::GeneratingReport);
        let report = collector.finalize();
        let report_md = ReportGenerator::to_markdown(&report);
        let pr_body = ReportGenerator::to_pr_summary(&report);

        // ── Phase 5: GitHub PR ──
        self.update_status(run_id, AgentRunStatus::CreatingPr);

        let branch = self
            .branch_manager
            .create_branch(&owner, &repo, issue_number)
            .await?;

        // Commit each changed file.
        for file in &changes.files {
            if file.action != "delete" {
                self.pr_manager
                    .commit_file(
                        &owner,
                        &repo,
                        &branch,
                        &file.path,
                        &file.content,
                        &format!("fix(#{}): {}", issue_number, file.diff_summary),
                    )
                    .await?;
            }
        }

        // Commit the verification report.
        self.pr_manager
            .commit_file(
                &owner,
                &repo,
                &branch,
                &format!(".gca/verification-report-{}.md", run_id),
                &report_md,
                &format!("docs: add verification report for #{}", issue_number),
            )
            .await?;

        // Open the PR.
        let pr_url = self
            .pr_manager
            .open_pr(
                &owner,
                &repo,
                &branch,
                issue_number,
                &format!("fix: resolve #{} — {}", issue_number, issue.title),
                &pr_body,
            )
            .await?;

        // ── Done ──
        {
            let mut runs = self.runs.lock().unwrap();
            if let Some(run) = runs.get_mut(run_id) {
                run.status = AgentRunStatus::Completed;
                run.pr_url = Some(pr_url.clone());
                run.report = Some(report);
            }
        }

        info!(run_id = %run_id, pr_url = %pr_url, "Agent run completed successfully");
        Ok(())
    }
}
