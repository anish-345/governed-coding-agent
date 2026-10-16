use crate::agent::AgentOrchestrator;
use crate::api::handlers;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

/// Application state shared across all handlers.
#[derive(Clone)]
pub struct AppState {
    pub orchestrator: Arc<AgentOrchestrator>,
}

/// Build the Axum router with all routes.
pub fn create_router(orchestrator: Arc<AgentOrchestrator>) -> Router {
    let state = AppState { orchestrator };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // API routes
        .route("/api/resolve-issue", post(handlers::resolve_issue))
        .route("/api/status/:run_id", get(handlers::get_status))
        .route("/api/report/:run_id", get(handlers::get_report))
        .route("/api/report/:run_id/markdown", get(handlers::get_report_markdown))
        .route("/api/health", get(handlers::health_check))
        // Static frontend files
        .fallback_service(ServeDir::new("static"))
        .layer(cors)
        .with_state(state)
}
