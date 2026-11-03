//! Health endpoints.

use axum::{Json, Router, extract::State, routing::get};
use chrono::Utc;

use crate::dto::health::{
    DependencyStatusDto, HealthResponse, LivenessResponse, ReadinessResponse,
};
use crate::dto::response::{ApiErrorBody, ApiResponse};
use crate::errors::AppError;
use crate::services::health as health_service;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
}

/// `GET /health`
///
/// Cheap check: the process answered, nothing else is verified.
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    summary = "API health",
    description = "Returns `ok` when the process can serve HTTP requests. Dependencies are not checked, use `/health/ready` for that.",
    responses(
        (status = 200, description = "API is running", body = ApiResponse<HealthResponse>),
    )
)]
pub async fn health(State(state): State<AppState>) -> Json<ApiResponse<HealthResponse>> {
    let config = state.config();
    Json(ApiResponse::ok(HealthResponse {
        status: "ok".to_owned(),
        version: config.version.clone(),
        timestamp: Utc::now(),
    }))
}

/// `GET /health/live`
///
/// Kubernetes style liveness probe.
#[utoipa::path(
    get,
    path = "/health/live",
    tag = "health",
    summary = "Process liveness",
    description = "Returns `alive` while the process is running. Intended for orchestrator liveness probes.",
    responses(
        (status = 200, description = "Process is alive", body = ApiResponse<LivenessResponse>),
    )
)]
pub async fn liveness(State(state): State<AppState>) -> Json<ApiResponse<LivenessResponse>> {
    Json(ApiResponse::ok(LivenessResponse {
        status: "alive".to_owned(),
        uptime_seconds: state.uptime_seconds(),
    }))
}

/// `GET /health/ready`
///
/// Probes PostgreSQL and Redis. Returns `503 Service Unavailable` when at least
/// one dependency is down, so load balancers can take the instance out of
/// rotation.
#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "health",
    summary = "Dependency readiness",
    description = "Probes PostgreSQL and Redis. Responds 503 with sanitized check details when any dependency is unavailable.",
    responses(
        (status = 200, description = "All dependencies are reachable", body = ApiResponse<ReadinessResponse>),
        (status = 503, description = "At least one dependency is unavailable", body = ApiErrorBody),
    )
)]
pub async fn readiness(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ReadinessResponse>>, AppError> {
    let report = health_service::readiness(&state).await;
    let checks: Vec<DependencyStatusDto> = report
        .checks
        .iter()
        .map(DependencyStatusDto::from)
        .collect();

    if !report.ready {
        tracing::warn!("readiness check failed");
        return Err(AppError::NotReady {
            details: serde_json::json!({ "checks": checks }),
        });
    }

    Ok(Json(ApiResponse::with_message(
        ReadinessResponse {
            status: "ready".to_owned(),
            version: state.config().version.clone(),
            checks,
        },
        "Service is ready",
    )))
}
