use thiserror::Error;

/// Unified error type for all subsystems in the governed coding agent.
#[derive(Error, Debug)]
pub enum AppError {
    // ── Configuration ──
    #[error("configuration error: {0}")]
    Config(String),

    // ── LLM / Nebius Token Factory ──
    #[error("LLM request failed: {0}")]
    Llm(String),

    #[error("LLM returned empty or unparseable response")]
    LlmEmptyResponse,

    // ── Sandbox Execution ──
    #[error("sandbox execution failed: {0}")]
    Sandbox(String),

    #[error("sandbox timeout after {0} seconds")]
    SandboxTimeout(u64),

    // ── GitHub ──
    #[error("GitHub API error: {0}")]
    GitHub(String),

    #[error("issue not found: {owner}/{repo}#{number}")]
    IssueNotFound {
        owner: String,
        repo: String,
        number: u64,
    },

    // ── Tavily ──
    #[error("Tavily API error: {0}")]
    Tavily(String),

    // ── Verification ──
    #[error("verification error: {0}")]
    Verification(String),

    // ── Agent Orchestration ──
    #[error("agent failed after {attempts} attempts: {reason}")]
    AgentMaxRetries { attempts: u32, reason: String },

    #[error("agent run not found: {0}")]
    RunNotFound(String),

    // ── Generic wrappers ──
    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Octocrab(Box<octocrab::Error>),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<octocrab::Error> for AppError {
    fn from(err: octocrab::Error) -> Self {
        AppError::Octocrab(Box::new(err))
    }
}

/// Convenience type alias used throughout the codebase.
pub type Result<T> = std::result::Result<T, AppError>;

// ── Axum integration: convert AppError into HTTP responses ──

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::IssueNotFound { .. } | AppError::RunNotFound(_) => {
                (axum::http::StatusCode::NOT_FOUND, self.to_string())
            }
            AppError::Config(_) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                self.to_string(),
            ),
            _ => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                self.to_string(),
            ),
        };

        let body = serde_json::json!({
            "error": message,
            "status": status.as_u16(),
        });

        (status, axum::Json(body)).into_response()
    }
}
