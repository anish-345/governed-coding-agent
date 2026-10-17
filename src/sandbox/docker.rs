use async_trait::async_trait;
use crate::error::{AppError, Result};
use crate::sandbox::trait_def::{ExecutionResult, SandboxExecutor};
use crate::sandbox::host::HostSandbox;
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

/// Docker-based sandbox executor (local fallback).
/// Spins up a Rust container, copies files in, runs commands.
/// Automatically falls back to HostSandbox if Docker is not available.
#[derive(Debug, Clone)]
pub struct DockerSandbox {
    /// Docker image to use.
    image: String,
    host_fallback: HostSandbox,
}

impl Default for DockerSandbox {
    fn default() -> Self {
        Self {
            image: "rust:1.75-bookworm".into(),
            host_fallback: HostSandbox::new(),
        }
    }
}

impl DockerSandbox {
    pub fn new(image: &str) -> Self {
        Self {
            image: image.to_string(),
            host_fallback: HostSandbox::new(),
        }
    }

    /// Run a docker command and capture output.
    async fn docker_exec(args: &[&str]) -> Result<(String, String, i32)> {
        let output = tokio::process::Command::new("docker")
            .args(args)
            .output()
            .await
            .map_err(|e| AppError::Sandbox(format!("Failed to run docker: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let code = output.status.code().unwrap_or(-1);

        Ok((stdout, stderr, code))
    }
}

#[async_trait]
impl SandboxExecutor for DockerSandbox {
    fn name(&self) -> &str {
        "Docker (local fallback)"
    }

    async fn prepare(&self, files: &HashMap<String, String>) -> Result<String> {
        let container_name = format!("gca-sandbox-{}", Uuid::new_v4());
        info!(container = %container_name, files = files.len(), "Attempting Docker sandbox");

        // Create and start the container with a long-running sleep process.
        let run_result = Self::docker_exec(&[
            "run",
            "-d",
            "--name",
            &container_name,
            "--memory=512m",
            "--cpus=1",
            &self.image,
            "sleep",
            "600",
        ])
        .await;

        match run_result {
            Ok((_, _, 0)) => {
                // Docker container created successfully
                // Create project directory inside the container.
                let _ = Self::docker_exec(&[
                    "exec", &container_name, "mkdir", "-p", "/workspace/src",
                ])
                .await;

                // Copy files into the container.
                for (path, content) in files {
                    let container_path = format!("/workspace/{}", path);

                    if let Some(parent) = container_path.rsplit_once('/') {
                        let _ = Self::docker_exec(&[
                            "exec",
                            &container_name,
                            "mkdir",
                            "-p",
                            parent.0,
                        ])
                        .await;
                    }

                    let write_cmd = format!(
                        "cat > {} << 'HEREDOC_EOF'\n{}\nHEREDOC_EOF",
                        container_path, content
                    );
                    let _ = Self::docker_exec(&[
                        "exec",
                        &container_name,
                        "sh",
                        "-c",
                        &write_cmd,
                    ])
                    .await;
                }

                info!(container = %container_name, "Docker sandbox ready");
                Ok(container_name)
            }
            _ => {
                warn!("Docker daemon is unavailable or failed to launch. Gracefully falling back to Host Process Sandbox.");
                self.host_fallback.prepare(files).await
            }
        }
    }

    async fn execute(&self, sandbox_id: &str, command: &str) -> Result<ExecutionResult> {
        if sandbox_id.starts_with("gca-sandbox-") {
            info!(container = %sandbox_id, command = %command, "Executing in Docker sandbox");
            let start = std::time::Instant::now();

            let (stdout, stderr, exit_code) = Self::docker_exec(&[
                "exec",
                "-w",
                "/workspace",
                sandbox_id,
                "sh",
                "-c",
                command,
            ])
            .await?;

            let duration_ms = start.elapsed().as_millis() as u64;

            Ok(ExecutionResult {
                command: command.to_string(),
                stdout,
                stderr,
                exit_code,
                duration_ms,
            })
        } else {
            // It's a host directory path from fallback
            self.host_fallback.execute(sandbox_id, command).await
        }
    }

    async fn cleanup(&self, sandbox_id: &str) -> Result<()> {
        if sandbox_id.starts_with("gca-sandbox-") {
            info!(container = %sandbox_id, "Cleaning up Docker sandbox");
            let _ = Self::docker_exec(&["rm", "-f", sandbox_id]).await;
            Ok(())
        } else {
            self.host_fallback.cleanup(sandbox_id).await
        }
    }
}
