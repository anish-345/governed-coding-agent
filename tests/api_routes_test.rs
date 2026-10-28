use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use governed_coding_agent::agent::AgentOrchestrator;
use governed_coding_agent::api::create_router;
use governed_coding_agent::config::{AppConfig, SandboxBackend};
use governed_coding_agent::sandbox::DockerSandbox;
use std::sync::Arc;

fn create_test_config() -> AppConfig {
    AppConfig {
        nebius_api_key: "test-key".into(),
        nebius_base_url: "http://localhost:8080".into(),
        nebius_model: "nvidia/nemotron-3-super-120b-a12b".into(),
        nebius_sandbox_url: "http://localhost:8081".into(),
        nebius_project_id: "test".into(),
        tavily_api_key: "".into(),
        github_token: "test-token".into(),
        github_repo: "owner/repo".into(),
        host: "127.0.0.1".into(),
        port: 8080,
        sandbox_backend: SandboxBackend::Docker,
    }
}

#[tokio::test]
async fn test_health_check_endpoint() {
    let config = create_test_config();
    let sandbox = Arc::new(DockerSandbox::default());
    let orchestrator = Arc::new(AgentOrchestrator::new(config, sandbox).unwrap());
    let app = create_router(orchestrator);

    let req = Request::builder()
        .uri("/api/health")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_status_endpoint_not_found() {
    let config = create_test_config();
    let sandbox = Arc::new(DockerSandbox::default());
    let orchestrator = Arc::new(AgentOrchestrator::new(config, sandbox).unwrap());
    let app = create_router(orchestrator);

    let req = Request::builder()
        .uri("/api/status/non-existent-run-id")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_resolve_issue_with_custom_token_endpoint() {
    let config = create_test_config();
    let sandbox = Arc::new(DockerSandbox::default());
    let orchestrator = Arc::new(AgentOrchestrator::new(config, sandbox).unwrap());
    let app = create_router(orchestrator);

    let payload = serde_json::json!({
        "issue_url": "https://github.com/my-org/private-repo/issues/10",
        "github_token": "ghp_mock_token_for_private_repo"
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/resolve-issue")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
