use crate::api::routes::AppState;
use crate::error::AppError;
use crate::verification::ReportGenerator;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

// ── Request / Response types ──

#[derive(Debug, Deserialize)]
pub struct ResolveIssueRequest {
    pub issue_url: String,
    #[serde(default)]
    pub github_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResolveIssueResponse {
    pub run_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub run_id: String,
    pub status: String,
    pub pr_url: Option<String>,
    pub has_report: bool,
    pub is_pr_preview: bool,
    pub pr_preview: Option<crate::agent::PrPreview>,
    pub iterations: Vec<crate::agent::IterationRecord>,
    pub telemetry: Option<crate::agent::Telemetry>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub service: String,
}

// ── Handlers ──

/// POST /api/resolve-issue — Start resolving a GitHub issue.
pub async fn resolve_issue(
    State(state): State<AppState>,
    Json(payload): Json<ResolveIssueRequest>,
) -> Result<Json<ResolveIssueResponse>, AppError> {
    tracing::info!(
        issue_url = %payload.issue_url,
        has_custom_token = payload.github_token.is_some(),
        "Received resolve-issue request"
    );

    let run_id = state
        .orchestrator
        .start_run_with_token(payload.issue_url, payload.github_token);

    Ok(Json(ResolveIssueResponse {
        run_id,
        status: "queued".into(),
        message: "Agent run started. Poll /api/status/{run_id} for updates.".into(),
    }))
}

/// GET /api/status/:run_id — Get the status of an agent run.
pub async fn get_status(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<StatusResponse>, AppError> {
    let run = state
        .orchestrator
        .get_run(&run_id)
        .ok_or_else(|| AppError::RunNotFound(run_id.clone()))?;

    Ok(Json(StatusResponse {
        run_id: run.run_id,
        status: format!("{:?}", run.status),
        pr_url: run.pr_url,
        has_report: run.report.is_some(),
        is_pr_preview: run.is_pr_preview,
        pr_preview: run.pr_preview,
        iterations: run.iterations,
        telemetry: run.telemetry,
    }))
}

/// GET /api/report/:run_id — Get the verification report as JSON.
pub async fn get_report(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let report = state
        .orchestrator
        .get_report(&run_id)
        .ok_or_else(|| AppError::RunNotFound(run_id.clone()))?;

    Ok(Json(report))
}

/// GET /api/report/:run_id/markdown — Get the verification report as Markdown.
pub async fn get_report_markdown(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let report = state
        .orchestrator
        .get_report(&run_id)
        .ok_or_else(|| AppError::RunNotFound(run_id.clone()))?;

    let markdown = ReportGenerator::to_markdown(&report);

    Ok((
        StatusCode::OK,
        [("Content-Type", "text/markdown; charset=utf-8")],
        markdown,
    ))
}

/// GET /api/health — Health check endpoint.
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        service: "governed-coding-agent".into(),
    })
}
