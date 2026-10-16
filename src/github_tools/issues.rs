use crate::error::{AppError, Result};
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Fetched GitHub issue details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueDetails {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub url: String,
    pub labels: Vec<String>,
    pub state: String,
}

/// Fetches issue details from GitHub.
pub struct IssueFetcher {
    client: Octocrab,
}

impl IssueFetcher {
    pub fn new(token: &str) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .map_err(|e| AppError::GitHub(format!("Failed to create Octocrab client: {e}")))?;

        Ok(Self { client })
    }

    /// Fetch a single issue by owner, repo, and issue number.
    pub async fn fetch(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<IssueDetails> {
        info!(owner = %owner, repo = %repo, issue = issue_number, "Fetching GitHub issue");

        let issue = self
            .client
            .issues(owner, repo)
            .get(issue_number)
            .await
            .map_err(|e| {
                if e.to_string().contains("404") {
                    AppError::IssueNotFound {
                        owner: owner.into(),
                        repo: repo.into(),
                        number: issue_number,
                    }
                } else {
                    AppError::GitHub(format!("Failed to fetch issue: {e}"))
                }
            })?;

        let labels = issue
            .labels
            .iter()
            .map(|l| l.name.clone())
            .collect();

        let details = IssueDetails {
            number: issue_number,
            title: issue.title.clone(),
            body: issue.body.clone().unwrap_or_default(),
            url: issue.html_url.to_string(),
            labels,
            state: format!("{:?}", issue.state).to_lowercase(),
        };

        info!(title = %details.title, "Issue fetched");
        Ok(details)
    }

    /// Parse an issue URL into (owner, repo, number).
    /// Supports: https://github.com/owner/repo/issues/42
    pub fn parse_issue_url(url: &str) -> Result<(String, String, u64)> {
        let parts: Vec<&str> = url
            .trim_end_matches('/')
            .split('/')
            .collect();

        // Expected: [..., "github.com", owner, repo, "issues", number]
        if parts.len() < 5 {
            return Err(AppError::GitHub(format!("Invalid issue URL: {url}")));
        }

        let number_str = parts.last().unwrap();
        let number: u64 = number_str
            .parse()
            .map_err(|_| AppError::GitHub(format!("Invalid issue number: {number_str}")))?;

        let issues_idx = parts
            .iter()
            .position(|&p| p == "issues")
            .ok_or_else(|| AppError::GitHub(format!("URL does not contain /issues/: {url}")))?;

        if issues_idx < 2 {
            return Err(AppError::GitHub(format!("Invalid issue URL: {url}")));
        }

        let owner = parts[issues_idx - 2].to_string();
        let repo = parts[issues_idx - 1].to_string();

        Ok((owner, repo, number))
    }
}
