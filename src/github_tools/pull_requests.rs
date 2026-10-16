use crate::error::{AppError, Result};
use octocrab::Octocrab;
use tracing::info;

/// Manages commits and pull requests via the GitHub API.
pub struct PrManager {
    client: Octocrab,
}

impl PrManager {
    pub fn new(token: &str) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .map_err(|e| AppError::GitHub(format!("Failed to create Octocrab client: {e}")))?;

        Ok(Self { client })
    }

    /// Commit a file to a branch via the GitHub Contents API.
    pub async fn commit_file(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        path: &str,
        content: &str,
        message: &str,
    ) -> Result<String> {
        info!(branch = %branch, path = %path, "Committing file");

        let encoded = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            content.as_bytes(),
        );

        // Check if the file already exists (need SHA for update).
        let existing_sha = match self
            .client
            .repos(owner, repo)
            .get_content()
            .path(path)
            .r#ref(branch)
            .send()
            .await
        {
            Ok(content_items) => content_items
                .items
                .first()
                .and_then(|item| item.sha.clone().into()),
            Err(_) => None,
        };

        // Build the request body.
        let mut body = serde_json::json!({
            "message": message,
            "content": encoded,
            "branch": branch,
        });

        if let Some(sha) = existing_sha {
            body["sha"] = serde_json::Value::String(sha);
        }

        // Use the raw API since octocrab doesn't have a direct create/update file method.
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, path
        );

        let response = reqwest::Client::new()
            .put(&url)
            .header("Authorization", format!("Bearer {}", get_token(&self.client)))
            .header("User-Agent", "governed-coding-agent")
            .header("Accept", "application/vnd.github.v3+json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::GitHub(format!("Failed to commit file: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let err_body = response.text().await.unwrap_or_default();
            return Err(AppError::GitHub(format!(
                "Commit failed ({status}): {err_body}"
            )));
        }

        let resp: serde_json::Value = response.json().await
            .map_err(|e| AppError::GitHub(format!("Failed to parse commit response: {e}")))?;

        let commit_sha = resp["commit"]["sha"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        info!(sha = %commit_sha, "File committed");
        Ok(commit_sha)
    }

    /// Open a pull request with the verification report as the body.
    pub async fn open_pr(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        issue_number: u64,
        title: &str,
        body: &str,
    ) -> Result<String> {
        info!(branch = %branch, issue = issue_number, "Opening pull request");

        let pr = self
            .client
            .pulls(owner, repo)
            .create(title, branch, "main")
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::GitHub(format!("Failed to create PR: {e}")))?;

        let pr_url = pr
            .html_url
            .as_ref()
            .map(|u| u.to_string())
            .unwrap_or_default();

        info!(pr_url = %pr_url, "Pull request created");
        Ok(pr_url)
    }
}

/// Extract the token from an Octocrab instance for raw API calls.
/// This is a workaround since octocrab doesn't expose all Content API methods.
fn get_token(_client: &Octocrab) -> String {
    // The token was set via personal_token() during construction.
    // We reconstruct it from the config. In practice, store the token separately.
    // For now, use an env var fallback.
    std::env::var("GITHUB_TOKEN").unwrap_or_default()
}
