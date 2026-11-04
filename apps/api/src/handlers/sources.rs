//! Phase-8 data source and record provenance administration.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, patch, post},
};

use crate::dto::response::{ApiErrorBody, ApiResponse};
use crate::dto::source::*;
use crate::errors::AppError;
use crate::repositories::sources as repo;
use crate::services::game_servers::require_maintenance_token;
use crate::services::sources as service;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sources", get(list_sources))
        .route("/sources/{code}", patch(update_source))
        .route("/sources/{code}/runs", get(list_runs))
        .route("/sources/{code}/sync", post(trigger_sync))
        .route("/records/{record_id}/sources", get(record_sources))
        .route("/external-servers", get(list_external_servers))
}

#[utoipa::path(get, path = "/api/v1/admin/sources", tag = "sources",
    summary = "List external data sources and health",
    responses((status = 200, body = ApiResponse<SourceListResponse>), (status = 401, body = ApiErrorBody)))]
pub async fn list_sources(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<SourceListResponse>>, AppError> {
    require_maintenance_token(&state, &headers)?;
    Ok(Json(ApiResponse::ok(SourceListResponse {
        items: repo::list_sources(state.db()).await?,
    })))
}

#[utoipa::path(patch, path = "/api/v1/admin/sources/{code}", tag = "sources",
    summary = "Enable or disable a data source",
    params(("code" = String, Path)),
    request_body = UpdateSourceRequest,
    responses((status = 200, body = ApiResponse<SourceSummary>), (status = 404, body = ApiErrorBody)))]
pub async fn update_source(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(code): Path<String>,
    Json(request): Json<UpdateSourceRequest>,
) -> Result<Json<ApiResponse<SourceSummary>>, AppError> {
    require_maintenance_token(&state, &headers)?;
    let enabled = request.is_enabled.unwrap_or(true);
    repo::set_source_enabled(state.db(), &code, enabled)
        .await?
        .map(|row| {
            Json(ApiResponse::ok(SourceSummary {
                id: row.id,
                code: row.code,
                source_type: row.source_type,
                display_name: row.display_name,
                is_enabled: row.is_enabled,
                priority: row.priority,
                status: row.status,
                last_attempt_at: row.last_attempt_at,
                last_success_at: row.last_success_at,
                last_failure_at: row.last_failure_at,
                last_error: row.last_error,
                consecutive_failures: row.consecutive_failures,
                cursor: row.cursor,
                config: row.config,
                raw_records: 0,
                canonical_links: 0,
                latest_run_status: None,
                latest_run_at: None,
            }))
        })
        .ok_or_else(|| AppError::not_found("source"))
}

#[utoipa::path(get, path = "/api/v1/admin/sources/{code}/runs", tag = "sources",
    summary = "Recent sync runs for a source",
    params(("code" = String, Path), SourceRunsQuery),
    responses((status = 200, body = ApiResponse<SourceRunsResponse>), (status = 404, body = ApiErrorBody)))]
pub async fn list_runs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(code): Path<String>,
    Query(query): Query<SourceRunsQuery>,
) -> Result<Json<ApiResponse<SourceRunsResponse>>, AppError> {
    require_maintenance_token(&state, &headers)?;
    let source = repo::find_source(state.db(), &code)
        .await?
        .ok_or_else(|| AppError::not_found("source"))?;
    let page = query.page_params()?;
    let (items, total) = repo::list_runs(state.db(), source.id, page.limit, page.offset).await?;
    Ok(Json(ApiResponse::ok(SourceRunsResponse {
        items,
        meta: crate::dto::public::common::PageMeta::from_total(total, page),
    })))
}

#[utoipa::path(post, path = "/api/v1/admin/sources/{code}/sync", tag = "sources",
    summary = "Trigger a source sync in the background",
    params(("code" = String, Path)),
    responses((status = 200, body = ApiResponse<SourceSyncResponse>), (status = 404, body = ApiErrorBody)))]
pub async fn trigger_sync(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<ApiResponse<SourceSyncResponse>>, AppError> {
    require_maintenance_token(&state, &headers)?;
    let source = repo::find_source(state.db(), &code)
        .await?
        .ok_or_else(|| AppError::not_found("source"))?;
    let state_clone = state.clone();
    let code_clone = code.clone();
    tokio::spawn(async move {
        if let Err(error) = service::sync_source(&state_clone, &code_clone, "manual").await {
            tracing::warn!(error = %error, source = %code_clone, "manual source sync failed");
        }
    });
    Ok(Json(ApiResponse::ok(SourceSyncResponse {
        source: source.code,
        run_id: None,
        status: "started".to_owned(),
        message: "sync started in background; poll source status/runs".to_owned(),
        stats: serde_json::json!({}),
    })))
}

#[utoipa::path(get, path = "/api/v1/admin/records/{record_id}/sources", tag = "sources",
    summary = "Raw record provenance for one canonical record",
    params(("record_id" = i64, Path)),
    responses((status = 200, body = ApiResponse<RecordProvenanceResponse>), (status = 404, body = ApiErrorBody)))]
pub async fn record_sources(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(record_id): Path<i64>,
) -> Result<Json<ApiResponse<RecordProvenanceResponse>>, AppError> {
    require_maintenance_token(&state, &headers)?;
    let sources = repo::record_provenance(state.db(), record_id).await?;
    if sources.is_empty() {
        return Err(AppError::not_found("record provenance"));
    }
    Ok(Json(ApiResponse::ok(RecordProvenanceResponse {
        record_id,
        sources,
    })))
}

#[utoipa::path(get, path = "/api/v1/admin/external-servers", tag = "sources",
    summary = "External server snapshot used for public fallback",
    responses((status = 200, body = ApiResponse<ExternalServerListResponse>), (status = 401, body = ApiErrorBody)))]
pub async fn list_external_servers(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<ExternalServerListResponse>>, AppError> {
    require_maintenance_token(&state, &headers)?;
    let max_age = state.config().source_fallback_max_age.as_secs() as i64;
    let items = repo::list_external_servers(state.db(), max_age).await?;
    Ok(Json(ApiResponse::ok(ExternalServerListResponse {
        items,
        source_healthy: repo::external_source_fresh(state.db(), max_age).await?,
        generated_at: chrono::Utc::now(),
    })))
}
