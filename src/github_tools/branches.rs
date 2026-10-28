use crate::error::{AppError, Result};
use octocrab::Octocrab;
use tracing::info;

/// Manages Git branches via the GitHub API.
#[derive(Clone)]
pub struct BranchManager {
    client: Octocrab,
}

impl BranchManager {
    pub fn new(token: &str) -> Result<Self> {
        let client = Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .map_err(|e| AppError::GitHub(format!("Failed to create Octocrab client: {e}")))?;

        Ok(Self { client })
    }

    /// Create a new branch from the default branch (main/master).
    /// Returns the new branch name.
    pub async fn create_branch(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<String> {
        let branch_name = format!("gca/fix-issue-{}", issue_number);
        info!(branch = %branch_name, "Creating branch");

        // Get the SHA of the default branch head.
        let repo_data = self
            .client
            .repos(owner, repo)
            .get()
            .await
            .map_err(|e| AppError::GitHub(format!("Failed to get repo: {e}")))?;

        let default_branch = repo_data
            .default_branch
            .as_deref()
            .unwrap_or("main");

        let reference = self
            .client
            .repos(owner, repo)
            .get_ref(&octocrab::params::repos::Reference::Branch(
                default_branch.to_string(),
            ))
            .await
            .map_err(|e| {
                AppError::GitHub(format!("Failed to get ref for {default_branch}: {e}"))
            })?;

        let sha = match &reference.object {
            octocrab::models::repos::Object::Commit { sha, .. } => sha.clone(),
            octocrab::models::repos::Object::Tag { sha, .. } => sha.clone(),
            _ => {
                return Err(AppError::GitHub(
                    "Unexpected ref object type".into(),
                ))
            }
        };

        // Create the new branch ref.
        self.client
            .repos(owner, repo)
            .create_ref(
                &octocrab::params::repos::Reference::Branch(branch_name.clone()),
                sha,
            )
            .await
            .map_err(|e| AppError::GitHub(format!("Failed to create branch: {e}")))?;

        info!(branch = %branch_name, "Branch created");
        Ok(branch_name)
    }
}
