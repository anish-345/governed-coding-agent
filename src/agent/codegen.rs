use crate::error::Result;
use crate::llm::{LlmClient, Prompts};
use crate::verification::{Artifact, ArtifactKind, EvidenceCollector};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Phase 2: Generate code changes from a plan.
pub struct CodeGenerator {
    llm: LlmClient,
}

/// Structured code generation output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChanges {
    pub files: Vec<FileChange>,
    #[serde(default)]
    pub new_tests: Vec<NewTestFile>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub action: String, // "modify", "create", "delete"
    pub content: String,
    pub diff_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTestFile {
    pub path: String,
    pub content: String,
}

impl CodeGenerator {
    pub fn new(llm: LlmClient) -> Self {
        Self { llm }
    }

    /// Generate code changes from a plan.
    pub async fn generate(
        &self,
        plan_json: &str,
        existing_code: &str,
        collector: &EvidenceCollector,
    ) -> Result<CodeChanges> {
        info!("Phase 2: Generating code changes");

        let user_prompt = Prompts::codegen_user(plan_json, existing_code);

        let response = self
            .llm
            .ask(Prompts::CODEGEN_SYSTEM, &user_prompt, 0.2)
            .await?;

        // Record the LLM interaction.
        collector.record_artifact(Artifact::new(
            ArtifactKind::LlmInteraction,
            format!(
                "System: {}\n\nUser: {}\n\nAssistant: {}",
                Prompts::CODEGEN_SYSTEM, user_prompt, response.content
            ),
            None,
        ));

        let changes: CodeChanges = parse_json_response(&response.content)?;

        // Record each generated file as an artifact.
        for file in &changes.files {
            collector.record_artifact(Artifact::new(
                ArtifactKind::GeneratedCode,
                format!(
                    "File: {}\nAction: {}\nSummary: {}\n\n{}",
                    file.path, file.action, file.diff_summary, file.content
                ),
                None,
            ));

            // Record a code diff artifact.
            collector.record_artifact(Artifact::new(
                ArtifactKind::CodeDiff,
                format!("--- {}\n+++ {}\n\n{}", file.path, file.path, file.diff_summary),
                None,
            ));
        }

        for test in &changes.new_tests {
            collector.record_artifact(Artifact::new(
                ArtifactKind::GeneratedCode,
                format!("New test file: {}\n\n{}", test.path, test.content),
                None,
            ));
        }

        info!(
            files = changes.files.len(),
            tests = changes.new_tests.len(),
            "Code generation complete"
        );

        Ok(changes)
    }

    /// Generate a fix for code that failed tests/compilation.
    pub async fn fix(
        &self,
        previous_code: &str,
        error_output: &str,
        collector: &EvidenceCollector,
    ) -> Result<CodeChanges> {
        info!("Generating fix for failed code");

        let user_prompt = Prompts::fix_user(previous_code, error_output);

        let response = self
            .llm
            .ask(Prompts::FIX_SYSTEM, &user_prompt, 0.2)
            .await?;

        // Record the fix interaction.
        collector.record_artifact(Artifact::new(
            ArtifactKind::LlmInteraction,
            format!(
                "FIX ATTEMPT\nSystem: {}\n\nUser: {}\n\nAssistant: {}",
                Prompts::FIX_SYSTEM, user_prompt, response.content
            ),
            None,
        ));

        let changes: CodeChanges = parse_json_response(&response.content)?;

        for file in &changes.files {
            collector.record_artifact(Artifact::new(
                ArtifactKind::CodeDiff,
                format!("FIX: --- {}\n+++ {}\n\n{}", file.path, file.path, file.diff_summary),
                None,
            ));
        }

        Ok(changes)
    }

    /// Convert code changes into a HashMap of path → content for the sandbox.
    pub fn to_file_map(changes: &CodeChanges) -> std::collections::HashMap<String, String> {
        let mut files = std::collections::HashMap::new();
        for file in &changes.files {
            if file.action != "delete" {
                files.insert(file.path.clone(), file.content.clone());
            }
        }
        for test in &changes.new_tests {
            files.insert(test.path.clone(), test.content.clone());
        }
        files
    }
}

/// Try to parse JSON from an LLM response, handling markdown fences.
fn parse_json_response<T: serde::de::DeserializeOwned>(content: &str) -> Result<T> {
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
