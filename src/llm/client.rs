use crate::config::AppConfig;
use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// OpenAI-compatible client for Nebius Token Factory + Nemotron.
#[derive(Debug, Clone)]
pub struct LlmClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

// ── OpenAI-compatible request/response types ──

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
    max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".into(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
    usage: Option<UsageInfo>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Result of an LLM call, including the response and metadata.
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub usage: Option<UsageInfo>,
    /// The full prompt sent (for evidence trail).
    pub prompt_snapshot: Vec<ChatMessage>,
    pub latency_ms: u64,
}

impl LlmClient {
    /// Create a new client from application config.
    pub fn new(config: &AppConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: config.nebius_base_url.trim_end_matches('/').to_string(),
            api_key: config.nebius_api_key.clone(),
            model: config.nebius_model.clone(),
        }
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Send a chat completion request to Nebius Token Factory.
    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        temperature: f64,
        max_tokens: u32,
    ) -> Result<LlmResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let request_body = ChatRequest {
            model: self.model.clone(),
            messages: messages.clone(),
            temperature,
            max_tokens,
        };

        info!(
            model = %self.model,
            msg_count = messages.len(),
            "Sending chat completion to Nebius Token Factory"
        );
        debug!(url = %url, "LLM endpoint");

        let start = std::time::Instant::now();

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::Llm(format!("HTTP request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Llm(format!(
                "Nebius returned {status}: {body}"
            )));
        }

        let chat_resp: ChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::Llm(format!("Failed to parse response: {e}")))?;

        let latency_ms = start.elapsed().as_millis() as u64;

        let content = chat_resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or(AppError::LlmEmptyResponse)?;

        let usage = chat_resp.usage.or_else(|| {
            let prompt_chars: usize = messages.iter().map(|m| m.content.len()).sum();
            let prompt_tokens = (prompt_chars / 4).max(30) as u32;
            let completion_tokens = (content.len() / 4).max(20) as u32;
            Some(UsageInfo {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            })
        });

        info!(
            tokens = ?usage.as_ref().map(|u| u.total_tokens),
            latency_ms = latency_ms,
            "LLM response received from Nebius Token Factory"
        );

        Ok(LlmResponse {
            content,
            usage,
            prompt_snapshot: messages,
            latency_ms,
        })
    }

    /// Convenience: single system + user message.
    pub async fn ask(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f64,
    ) -> Result<LlmResponse> {
        let messages = vec![
            ChatMessage {
                role: "system".into(),
                content: system_prompt.into(),
            },
            ChatMessage {
                role: "user".into(),
                content: user_prompt.into(),
            },
        ];
        self.chat(messages, temperature, 4096).await
    }

    /// Convenience: multi-turn conversation with system prompt.
    pub async fn converse(
        &self,
        system_prompt: &str,
        history: &[ChatMessage],
        user_message: &str,
        temperature: f64,
    ) -> Result<LlmResponse> {
        let mut messages = vec![ChatMessage {
            role: "system".into(),
            content: system_prompt.into(),
        }];
        messages.extend_from_slice(history);
        messages.push(ChatMessage {
            role: "user".into(),
            content: user_message.into(),
        });
        self.chat(messages, temperature, 4096).await
    }
}
