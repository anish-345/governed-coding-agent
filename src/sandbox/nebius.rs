use async_trait::async_trait;
use crate::error::{AppError, Result};
use crate::sandbox::trait_def::{ExecutionResult, SandboxExecutor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// Nebius Token Factory Sandbox executor.
///
/// Uses the Sandboxes API at `https://api.tokenfactory.nebius.com/sandboxes`.
///
/// ⚠️  Some endpoints are based on available documentation and may need
///     adjustment once you have full API access. Placeholders are clearly marked.
#[derive(Debug, Clone)]
pub struct NebiusSandbox {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    project_id: String,
}

// ── API request/response types (adjust to match actual Nebius Sandbox API) ──

#[derive(Debug, Serialize)]
struct CreateSandboxRequest {
    /// Sandbox image or template to use.
    image: String,
    /// Files to pre-load into the sandbox.
    files: Vec<SandboxFile>,
}

#[derive(Debug, Serialize)]
struct SandboxFile {
    path: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct CreateSandboxResponse {
    /// Unique sandbox/session ID.
    id: String,
}

#[derive(Debug, Serialize)]
struct RunCommandRequest {
    command: String,
    timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
struct RunCommandResponse {
    stdout: String,
    stderr: String,
    exit_code: i32,
    duration_ms: u64,
}

impl NebiusSandbox {
    pub fn new(base_url: &str, api_key: &str, project_id: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            project_id: project_id.to_string(),
        }
    }

    fn auth_headers(&self) -> Vec<(&str, String)> {
        vec![
            ("Authorization", format!("Bearer {}", self.api_key)),
            ("Project", self.project_id.clone()),
            ("Content-Type", "application/json".into()),
        ]
    }
}

#[async_trait]
impl SandboxExecutor for NebiusSandbox {
    fn name(&self) -> &str {
        "Nebius Token Factory Sandbox"
    }

    async fn prepare(&self, files: &HashMap<String, String>) -> Result<String> {
        info!(file_count = files.len(), "Creating Nebius sandbox");

        let sandbox_files: Vec<SandboxFile> = files
            .iter()
            .map(|(path, content)| SandboxFile {
                path: path.clone(),
                content: content.clone(),
            })
            .collect();

        // ⚠️ PLACEHOLDER: Adjust this endpoint path to match the actual Nebius Sandboxes API.
        // The documented base is: POST {base_url}/v1/sandboxes
        let url = format!("{}/v1/sandboxes", self.base_url);

        let request = CreateSandboxRequest {
            image: "rust:1.75-bookworm".into(), // ⚠️ Adjust to available Nebius sandbox images
            files: sandbox_files,
        };

        let mut req_builder = self.http.post(&url);
        for (key, value) in self.auth_headers() {
            req_builder = req_builder.header(key, value);
        }

        let response = req_builder
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Sandbox(format!("Failed to create sandbox: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Sandbox(format!(
                "Nebius sandbox creation returned {status}: {body}"
            )));
        }

        let resp: CreateSandboxResponse = response
            .json()
            .await
            .map_err(|e| AppError::Sandbox(format!("Failed to parse sandbox response: {e}")))?;

        info!(sandbox_id = %resp.id, "Nebius sandbox created");
        Ok(resp.id)
    }

    async fn execute(&self, sandbox_id: &str, command: &str) -> Result<ExecutionResult> {
        info!(sandbox_id = %sandbox_id, command = %command, "Executing in Nebius sandbox");

        // ⚠️ PLACEHOLDER: Adjust this endpoint path to match the actual Nebius Sandboxes API.
        // Likely: POST {base_url}/v1/sandboxes/{sandbox_id}/execute
        let url = format!("{}/v1/sandboxes/{}/execute", self.base_url, sandbox_id);

        let request = RunCommandRequest {
            command: command.to_string(),
            timeout_seconds: 120,
        };

        let mut req_builder = self.http.post(&url);
        for (key, value) in self.auth_headers() {
            req_builder = req_builder.header(key, value);
        }

        let response = req_builder
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Sandbox(format!("Sandbox execute failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Sandbox(format!(
                "Nebius sandbox execute returned {status}: {body}"
            )));
        }

        let resp: RunCommandResponse = response
            .json()
            .await
            .map_err(|e| AppError::Sandbox(format!("Failed to parse execute response: {e}")))?;

        Ok(ExecutionResult {
            command: command.to_string(),
            stdout: resp.stdout,
            stderr: resp.stderr,
            exit_code: resp.exit_code,
            duration_ms: resp.duration_ms,
        })
    }

    async fn cleanup(&self, sandbox_id: &str) -> Result<()> {
        // ⚠️ PLACEHOLDER: DELETE {base_url}/v1/sandboxes/{sandbox_id}
        let url = format!("{}/v1/sandboxes/{}", self.base_url, sandbox_id);

        let mut req_builder = self.http.delete(&url);
        for (key, value) in self.auth_headers() {
            req_builder = req_builder.header(key, value);
        }

        match req_builder.send().await {
            Ok(resp) if resp.status().is_success() => {
                info!(sandbox_id = %sandbox_id, "Nebius sandbox cleaned up");
            }
            Ok(resp) => {
                warn!(sandbox_id = %sandbox_id, status = %resp.status(), "Sandbox cleanup returned non-200");
            }
            Err(e) => {
                warn!(sandbox_id = %sandbox_id, error = %e, "Sandbox cleanup failed");
            }
        }
        Ok(())
    }
}
