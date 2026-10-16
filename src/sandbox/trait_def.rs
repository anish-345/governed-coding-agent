use async_trait::async_trait;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of executing a command in a sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// The command that was executed.
    pub command: String,
    /// Combined stdout output.
    pub stdout: String,
    /// Combined stderr output.
    pub stderr: String,
    /// Process exit code (0 = success).
    pub exit_code: i32,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
}

impl ExecutionResult {
    /// Whether the command completed successfully.
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Combined stdout + stderr for evidence artifacts.
    pub fn full_output(&self) -> String {
        let mut out = String::new();
        if !self.stdout.is_empty() {
            out.push_str("=== STDOUT ===\n");
            out.push_str(&self.stdout);
            out.push('\n');
        }
        if !self.stderr.is_empty() {
            out.push_str("=== STDERR ===\n");
            out.push_str(&self.stderr);
            out.push('\n');
        }
        out.push_str(&format!("=== EXIT CODE: {} ===\n", self.exit_code));
        out.push_str(&format!("=== DURATION: {}ms ===\n", self.duration_ms));
        out
    }
}

/// Trait for pluggable sandbox execution backends.
/// Implement this for Nebius Token Factory Sandboxes, Docker, or any other backend.
#[async_trait]
pub trait SandboxExecutor: Send + Sync {
    /// Human-readable name of this backend.
    fn name(&self) -> &str;

    /// Prepare the sandbox with project files.
    /// `files` is a map of relative_path → file_content.
    async fn prepare(&self, files: &HashMap<String, String>) -> Result<String>;

    /// Execute a shell command inside the sandbox.
    /// `sandbox_id` is returned by `prepare`.
    async fn execute(&self, sandbox_id: &str, command: &str) -> Result<ExecutionResult>;

    /// Run the standard verification suite: check, clippy, test.
    async fn run_verification_suite(
        &self,
        sandbox_id: &str,
    ) -> Result<Vec<ExecutionResult>> {
        let commands = vec![
            "cargo check 2>&1",
            "cargo clippy -- -D warnings 2>&1",
            "cargo test 2>&1",
        ];

        let mut results = Vec::new();
        for cmd in commands {
            let result = self.execute(sandbox_id, cmd).await?;
            results.push(result);
        }
        Ok(results)
    }

    /// Clean up the sandbox after use.
    async fn cleanup(&self, sandbox_id: &str) -> Result<()>;
}
