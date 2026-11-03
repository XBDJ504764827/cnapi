//! System information endpoints mounted under `/api/v1`.

use axum::{Json, Router, extract::Query, extract::State, routing::get};

use crate::dto::response::{ApiErrorBody, ApiResponse};
use crate::dto::system::{
    RecentEventsQuery, SystemEventsResponse, SystemInfoResponse, SystemStatusResponse,
};
use crate::errors::AppError;
use crate::services::system as system_service;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/system/info", get(info))
        .route("/system/status", get(status))
        .route("/system/events", get(events))
}

/// `GET /api/v1/system/info`
#[utoipa::path(
    get,
    path = "/api/v1/system/info",
    tag = "system",
    summary = "System information",
    description = "Version, environment, start time and uptime of the running API instance.",
    responses(
        (status = 200, description = "System information", body = ApiResponse<SystemInfoResponse>),
    )
)]
pub async fn info(State(state): State<AppState>) -> Json<ApiResponse<SystemInfoResponse>> {
    Json(ApiResponse::ok(system_service::info(&state)))
}

/// `GET /api/v1/system/status`
#[utoipa::path(
    get,
    path = "/api/v1/system/status",
    tag = "system",
    summary = "System status",
    description = "Real, per-request connectivity status of PostgreSQL and Redis together with uptime information.",
    responses(
        (status = 200, description = "Current system status", body = ApiResponse<SystemStatusResponse>),
    )
)]
pub async fn status(State(state): State<AppState>) -> Json<ApiResponse<SystemStatusResponse>> {
    Json(ApiResponse::ok(system_service::status(&state).await))
}

/// `GET /api/v1/system/events`
#[utoipa::path(
    get,
    path = "/api/v1/system/events",
    tag = "system",
    summary = "Recent system events",
    description = "Most recent system events. A fresh installation has an empty list.",
    params(RecentEventsQuery),
    responses(
        (status = 200, description = "Recent system events", body = ApiResponse<SystemEventsResponse>),
        (status = 400, description = "Invalid query parameters", body = ApiErrorBody),
    )
)]
pub async fn events(
    State(state): State<AppState>,
    Query(query): Query<RecentEventsQuery>,
) -> Result<Json<ApiResponse<SystemEventsResponse>>, AppError> {
    let response = system_service::recent_events(&state, &query).await?;
    Ok(Json(ApiResponse::ok(response)))
}
