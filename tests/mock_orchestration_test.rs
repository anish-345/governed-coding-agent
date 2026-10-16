use governed_coding_agent::config::AppConfig;
use governed_coding_agent::llm::{ChatMessage, LlmClient};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_nebius_llm_chat_completion_success() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "id": "chatcmpl-test-123",
        "object": "chat.completion",
        "created": 1729000000,
        "model": "nvidia/nemotron-3-super-120b-a12b",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "{\"analysis\":\"Root cause is out of bounds index\",\"files_to_modify\":[\"src/lib.rs\"],\"steps\":[],\"test_strategy\":\"cargo test\",\"claims\":[\"Fixed boundary check\"]}"
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 128,
            "completion_tokens": 64,
            "total_tokens": 192
        }
    });

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("Authorization", "Bearer mock-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let config = AppConfig {
        nebius_api_key: "mock-key".into(),
        nebius_base_url: mock_server.uri(),
        nebius_model: "nvidia/nemotron-3-super-120b-a12b".into(),
        nebius_sandbox_url: "https://sandbox.api.nebius.ai".into(),
        nebius_project_id: "test-proj".into(),
        tavily_api_key: "".into(),
        github_token: "ghp_mock".into(),
        github_repo: "owner/repo".into(),
        host: "127.0.0.1".into(),
        port: 8080,
        sandbox_backend: governed_coding_agent::config::SandboxBackend::Docker,
    };

    let client = LlmClient::new(&config);
    let messages = vec![ChatMessage::user("Fix this issue: index out of bounds")];

    let result = client.chat(messages, 0.2, 512).await.expect("Chat completion should succeed");

    assert!(result.content.contains("Root cause is out of bounds index"));
    assert!(result.content.contains("Fixed boundary check"));
}

#[tokio::test]
async fn test_nebius_llm_chat_completion_error_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized: invalid API key"))
        .mount(&mock_server)
        .await;

    let config = AppConfig {
        nebius_api_key: "invalid-key".into(),
        nebius_base_url: mock_server.uri(),
        nebius_model: "nvidia/nemotron-3-super-120b-a12b".into(),
        nebius_sandbox_url: "https://sandbox.api.nebius.ai".into(),
        nebius_project_id: "test-proj".into(),
        tavily_api_key: "".into(),
        github_token: "ghp_mock".into(),
        github_repo: "owner/repo".into(),
        host: "127.0.0.1".into(),
        port: 8080,
        sandbox_backend: governed_coding_agent::config::SandboxBackend::Docker,
    };

    let client = LlmClient::new(&config);
    let messages = vec![ChatMessage::user("Test prompt")];

    let result = client.chat(messages, 0.2, 512).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("401") || err_msg.contains("Unauthorized"));
}
