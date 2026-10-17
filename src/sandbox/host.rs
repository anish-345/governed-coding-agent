use async_trait::async_trait;
use crate::error::{AppError, Result};
use crate::sandbox::trait_def::{ExecutionResult, SandboxExecutor};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;
use uuid::Uuid;

/// Host-based sandbox executor for local and cross-platform verification.
/// Prepares files in an isolated temporary directory and executes verification commands.
/// Works natively on both Windows and Linux hosts.
#[derive(Debug, Clone, Default)]
pub struct HostSandbox;

impl HostSandbox {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SandboxExecutor for HostSandbox {
    fn name(&self) -> &str {
        if cfg!(windows) {
            "Host Process Sandbox (Windows)"
        } else {
            "Host Process Sandbox (Linux)"
        }
    }

    async fn prepare(&self, files: &HashMap<String, String>) -> Result<String> {
        let sandbox_id = format!("gca-sandbox-{}", Uuid::new_v4());
        let sandbox_dir = std::env::temp_dir().join(&sandbox_id);

        info!(
            sandbox_dir = ?sandbox_dir,
            file_count = files.len(),
            "Preparing Host sandbox"
        );

        tokio::fs::create_dir_all(&sandbox_dir)
            .await
            .map_err(|e| AppError::Sandbox(format!("Failed to create sandbox directory: {e}")))?;

        // Ensure a Cargo.toml exists if tests/cargo commands will run and none was provided
        if !files.keys().any(|k| k == "Cargo.toml") {
            let default_cargo_toml = r#"[package]
name = "sandbox-verification"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
            let toml_path = sandbox_dir.join("Cargo.toml");
            tokio::fs::write(&toml_path, default_cargo_toml)
                .await
                .map_err(|e| AppError::Sandbox(format!("Failed to write default Cargo.toml: {e}")))?;
        }

        for (rel_path, content) in files {
            let full_path = sandbox_dir.join(rel_path);
            if let Some(parent) = full_path.parent() {
                tokio::fs::create_dir_all(parent).await.map_err(|e| {
                    AppError::Sandbox(format!("Failed to create parent dir {:?}: {e}", parent))
                })?;
            }
            tokio::fs::write(&full_path, content).await.map_err(|e| {
                AppError::Sandbox(format!("Failed to write file {:?}: {e}", full_path))
            })?;
        }

        Ok(sandbox_dir.to_string_lossy().to_string())
    }

    async fn execute(&self, sandbox_id: &str, command: &str) -> Result<ExecutionResult> {
        let start = std::time::Instant::now();
        let work_dir = PathBuf::from(sandbox_id);

        info!(
            dir = ?work_dir,
            command = %command,
            "Executing command in Host sandbox"
        );

        let mut cmd = if cfg!(windows) {
            let mut c = tokio::process::Command::new("cmd");
            c.arg("/C").arg(command);
            c
        } else {
            let mut c = tokio::process::Command::new("sh");
            c.arg("-c").arg(command);
            c
        };

        cmd.current_dir(&work_dir);

        let output = cmd.output().await;

        let duration_ms = start.elapsed().as_millis() as u64;

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let exit_code = out.status.code().unwrap_or(-1);

                Ok(ExecutionResult {
                    command: command.to_string(),
                    stdout,
                    stderr,
                    exit_code,
                    duration_ms,
                })
            }
            Err(e) => {
                // If the system command failed to launch (e.g. command not found), record clean failure
                Ok(ExecutionResult {
                    command: command.to_string(),
                    stdout: String::new(),
                    stderr: format!("Process launch error: {e}"),
                    exit_code: 1,
                    duration_ms,
                })
            }
        }
    }

    async fn cleanup(&self, sandbox_id: &str) -> Result<()> {
        let work_dir = PathBuf::from(sandbox_id);
        info!(dir = ?work_dir, "Cleaning up Host sandbox");
        let _ = tokio::fs::remove_dir_all(&work_dir).await;
        Ok(())
    }
}
