use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Client for the Tavily Search and Extract APIs.
/// Used to ground agent decisions in real documentation.
#[derive(Debug, Clone)]
pub struct TavilyClient {
    http: reqwest::Client,
    api_key: String,
}

// ── Tavily Search API ──

#[derive(Debug, Serialize)]
struct TavilySearchRequest {
    query: String,
    max_results: u32,
    search_depth: String,
    include_answer: bool,
}

#[derive(Debug, Deserialize)]
pub struct TavilySearchResponse {
    pub answer: Option<String>,
    pub results: Vec<TavilySearchResult>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TavilySearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
    pub score: f64,
}

// ── Tavily Extract API ──

#[derive(Debug, Serialize)]
struct TavilyExtractRequest {
    urls: Vec<String>,
    extract_depth: String,
    include_images: bool,
}

#[derive(Debug, Deserialize)]
pub struct TavilyExtractResponse {
    pub results: Vec<TavilyExtractResult>,
    pub failed_results: Vec<TavilyExtractFailed>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TavilyExtractResult {
    pub url: String,
    pub raw_content: String,
}

#[derive(Debug, Deserialize)]
pub struct TavilyExtractFailed {
    pub url: String,
    pub error: String,
}

impl TavilyClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key: api_key.to_string(),
        }
    }

    /// Returns `None` if the API key is empty (Tavily is optional).
    pub fn new_optional(api_key: &str) -> Option<Self> {
        if api_key.is_empty() {
            None
        } else {
            Some(Self::new(api_key))
        }
    }

    /// Search for documentation relevant to a coding issue.
    pub async fn search(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<TavilySearchResponse> {
        info!(query = %query, max = max_results, "Tavily search");

        let request = TavilySearchRequest {
            query: query.to_string(),
            max_results,
            search_depth: "advanced".into(),
            include_answer: true,
        };

        let response = self
            .http
            .post("https://api.tavily.com/search")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Tavily(format!("Search request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Tavily(format!("Tavily search returned {status}: {body}")));
        }

        let result: TavilySearchResponse = response
            .json()
            .await
            .map_err(|e| AppError::Tavily(format!("Failed to parse search response: {e}")))?;

        info!(
            results = result.results.len(),
            has_answer = result.answer.is_some(),
            "Tavily search complete"
        );

        Ok(result)
    }

    /// Extract content from specific URLs (e.g., documentation pages).
    pub async fn extract(&self, urls: &[&str]) -> Result<TavilyExtractResponse> {
        info!(urls = ?urls, "Tavily extract");

        let request = TavilyExtractRequest {
            urls: urls.iter().map(|u| u.to_string()).collect(),
            extract_depth: "basic".into(),
            include_images: false,
        };

        let response = self
            .http
            .post("https://api.tavily.com/extract")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Tavily(format!("Extract request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Tavily(format!("Tavily extract returned {status}: {body}")));
        }

        response
            .json()
            .await
            .map_err(|e| AppError::Tavily(format!("Failed to parse extract response: {e}")))
    }

    /// Build a grounding context string from Tavily results.
    /// This is what gets injected into the LLM prompt.
    pub fn format_context(results: &[TavilySearchResult]) -> String {
        if results.is_empty() {
            return "No relevant documentation found.".into();
        }

        let mut ctx = String::with_capacity(2048);
        for (i, r) in results.iter().enumerate() {
            ctx.push_str(&format!(
                "[Source {}] {} (score: {:.2})\nURL: {}\n{}\n\n",
                i + 1,
                r.title,
                r.score,
                r.url,
                truncate(&r.content, 800),
            ));
        }
        ctx
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}
