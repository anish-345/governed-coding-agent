use crate::error::{AppError, Result};

/// Application-wide configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    // ── Nebius Token Factory (LLM) ──
    pub nebius_api_key: String,
    pub nebius_base_url: String,
    pub nebius_model: String,

    // ── Nebius Sandbox ──
    pub nebius_sandbox_url: String,
    pub nebius_project_id: String,
    pub sandbox_backend: SandboxBackend,

    // ── Tavily ──
    pub tavily_api_key: String,

    // ── GitHub ──
    pub github_token: String,
    pub github_repo: String, // "owner/repo"

    // ── Server ──
    pub host: String,
    pub port: u16,
}

/// Which sandbox backend to use for code execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxBackend {
    /// Nebius Token Factory Sandboxes API
    Nebius,
    /// Local Docker containers (fallback)
    Docker,
    /// Local Host Process (isolated temp directory)
    Host,
}

impl AppConfig {
    /// Load configuration from environment variables.
    /// Call `dotenvy::dotenv().ok()` before invoking this.
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            nebius_api_key: required_env("NEBIUS_API_KEY")?,
            nebius_base_url: env_or("NEBIUS_BASE_URL", "https://api.tokenfactory.nebius.com/v1"),
            nebius_model: env_or("NEBIUS_MODEL", "nvidia/nemotron-3-super-120b-a12b"),

            nebius_sandbox_url: env_or(
                "NEBIUS_SANDBOX_URL",
                "https://api.tokenfactory.nebius.com/sandboxes",
            ),
            nebius_project_id: env_or("NEBIUS_PROJECT_ID", ""),
            sandbox_backend: match env_or("SANDBOX_BACKEND", "docker").to_lowercase().as_str() {
                "nebius" => SandboxBackend::Nebius,
                "host" => SandboxBackend::Host,
                _ => SandboxBackend::Docker,
            },

            tavily_api_key: env_or("TAVILY_API_KEY", ""),

            github_token: required_env("GITHUB_TOKEN")?,
            github_repo: required_env("GITHUB_REPO")?,

            host: env_or("HOST", "0.0.0.0"),
            port: env_or("PORT", "3000")
                .parse()
                .map_err(|_| AppError::Config("PORT must be a valid u16".into()))?,
        })
    }

    /// Parse "owner/repo" into (owner, repo).
    pub fn repo_parts(&self) -> Result<(&str, &str)> {
        self.github_repo
            .split_once('/')
            .ok_or_else(|| AppError::Config("GITHUB_REPO must be in 'owner/repo' format".into()))
    }
}

/// Read a required environment variable or return a config error.
fn required_env(key: &str) -> Result<String> {
    std::env::var(key).map_err(|_| AppError::Config(format!("{key} is required but not set")))
}

/// Read an environment variable with a default fallback.
fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
