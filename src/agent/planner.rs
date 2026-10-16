use crate::error::Result;
use crate::llm::{LlmClient, Prompts};
use crate::tavily::TavilyClient;
use crate::verification::{
    Artifact, ArtifactKind, Claim, ClaimCategory, EvidenceCollector,
};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Phase 1: Analyze the issue and produce a structured plan.
pub struct Planner {
    llm: LlmClient,
    tavily: Option<TavilyClient>,
}

/// Structured plan output from the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub analysis: String,
    pub files_to_modify: Vec<String>,
    pub steps: Vec<PlanStep>,
    pub test_strategy: String,
    pub claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub step: u32,
    pub action: String,
    pub file: String,
}

impl Planner {
    pub fn new(llm: LlmClient, tavily: Option<TavilyClient>) -> Self {
        Self { llm, tavily }
    }

    /// Analyze the issue and produce a plan.
    /// Records all intermediate artifacts in the evidence collector.
    pub async fn create_plan(
        &self,
        issue_title: &str,
        issue_body: &str,
        collector: &EvidenceCollector,
    ) -> Result<Plan> {
        info!("Phase 1: Analyzing issue and creating plan");

        // ── Step 1: Tavily grounding ──
        let tavily_context = if let Some(tavily) = &self.tavily {
            let query = format!("{} {}", issue_title, truncate(issue_body, 200));
            let search_results = tavily.search(&query, 5).await?;

            // Record each Tavily source as an artifact.
            for result in &search_results.results {
                let artifact = Artifact::new(
                    ArtifactKind::TavilySource,
                    format!(
                        "Title: {}\nURL: {}\nContent: {}",
                        result.title, result.url, result.content
                    ),
                    Some(result.url.clone()),
                );
                collector.record_artifact(artifact);
            }

            TavilyClient::format_context(&search_results.results)
        } else {
            "Tavily not configured — no external documentation grounding.".into()
        };

        // ── Step 2: LLM planning ──
        let user_prompt = Prompts::planner_user(issue_title, issue_body, &tavily_context);

        let response = self
            .llm
            .ask(Prompts::PLANNER_SYSTEM, &user_prompt, 0.3)
            .await?;

        // Record the LLM interaction as an artifact.
        let interaction_content = format!(
            "System: {}\n\nUser: {}\n\nAssistant: {}",
            Prompts::PLANNER_SYSTEM,
            user_prompt,
            response.content
        );
        collector.record_artifact(Artifact::new(
            ArtifactKind::LlmInteraction,
            &interaction_content,
            None,
        ));

        // Parse the plan JSON.
        let plan: Plan = parse_json_response(&response.content)?;

        // Record the plan as an artifact.
        collector.record_artifact(Artifact::new(
            ArtifactKind::AgentPlan,
            serde_json::to_string_pretty(&plan).unwrap_or_default(),
            None,
        ));

        // Pre-register claims from the plan.
        for claim_text in &plan.claims {
            let category = categorise_claim(claim_text);
            collector.register_claim(Claim::new(claim_text, category));
        }

        // If Tavily was used, register a "based on documentation" claim.
        if self.tavily.is_some() {
            let doc_claim = Claim::new(
                "Analysis is grounded in relevant external documentation",
                ClaimCategory::BasedOnDocumentation,
            );
            let claim_id = collector.register_claim(doc_claim);

            // Link all Tavily artifacts to this claim.
            let state = collector.finalize();
            for artifact in state
                .artifacts
                .iter()
                .filter(|a| a.kind == ArtifactKind::TavilySource)
            {
                collector.link_evidence(crate::verification::EvidenceLink::supports(
                    &claim_id,
                    &artifact.id,
                    format!("Tavily source: {}", artifact.source_url.as_deref().unwrap_or("N/A")),
                ));
            }
        }

        info!(
            steps = plan.steps.len(),
            claims = plan.claims.len(),
            "Plan created"
        );
        Ok(plan)
    }
}

/// Try to parse JSON from an LLM response, handling markdown fences.
fn parse_json_response<T: serde::de::DeserializeOwned>(content: &str) -> Result<T> {
    // Strip markdown fences if present.
    let cleaned = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    serde_json::from_str(cleaned).map_err(|e| {
        crate::error::AppError::Llm(format!(
            "Failed to parse LLM JSON response: {e}\nRaw content:\n{content}"
        ))
    })
}

/// Heuristic to categorise a claim string.
fn categorise_claim(claim: &str) -> ClaimCategory {
    let lower = claim.to_lowercase();
    if lower.contains("test") && lower.contains("pass") {
        ClaimCategory::TestsPassing
    } else if lower.contains("compil") || lower.contains("check") || lower.contains("build") {
        ClaimCategory::NoCompileErrors
    } else if lower.contains("clippy") || lower.contains("lint") || lower.contains("warning") {
        ClaimCategory::NoClippyWarnings
    } else if lower.contains("doc") || lower.contains("reference") || lower.contains("based on") {
        ClaimCategory::BasedOnDocumentation
    } else {
        ClaimCategory::CodeChangeCorrect
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}
