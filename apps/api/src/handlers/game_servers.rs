//! Phase-7 CS:GO game-server ingestion and administration HTTP handlers.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
};

use crate::dto::game_server::*;
use crate::dto::response::{ApiErrorBody, ApiResponse};
use crate::errors::AppError;
use crate::services::game_servers as service;
use crate::state::AppState;

/// Routes authenticated with the per-server token (`X-Server-Token`).
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/heartbeat", post(heartbeat))
        .route("/players/connect", post(connect_player))
        .route("/players/disconnect", post(disconnect_player))
        .route("/players/sync", post(sync_players))
        .route("/records", post(submit_record))
        .route("/records/status", get(record_status))
        .route("/replays", post(register_replay))
        .route("/maps/sync", get(config_sync))
        .route("/config/sync", get(config_sync))
}

/// Routes protected by the administrator maintenance token.
pub fn admin_router() -> Router<AppState> {
    Router::new()
        .route("/", get(admin_list).post(admin_create))
        .route("/{server_id}", get(admin_detail).patch(admin_update))
        .route("/{server_id}/enable", post(admin_enable))
        .route("/{server_id}/disable", post(admin_disable))
        .route("/{server_id}/token/rotate", post(admin_rotate_token))
        .route("/{server_id}/token/revoke", post(admin_revoke_token))
        .route("/tasks/{task_id}/retry", post(admin_retry_task))
}

#[utoipa::path(
    post, path = "/api/v1/game-servers/heartbeat", tag = "game-servers",
    summary = "Server heartbeat",
    description = "Authenticated with `X-Server-Token`. The API records its own receive time and derives ONLINE/STALE from it.",
    request_body = HeartbeatRequest,
    responses(
        (status = 200, body = ApiResponse<HeartbeatResponse>),
        (status = 401, body = ApiErrorBody),
        (status = 403, body = ApiErrorBody),
        (status = 400, body = ApiErrorBody),
    ),
)]
pub async fn heartbeat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<HeartbeatRequest>,
) -> Result<Json<ApiResponse<HeartbeatResponse>>, AppError> {
    let server = service::authenticate(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::heartbeat(&state, &server, request).await?,
    )))
}

#[utoipa::path(
    post, path = "/api/v1/game-servers/players/connect", tag = "game-servers",
    summary = "Player connected / authenticated",
    request_body = PlayerEventRequest,
    responses((status = 200, body = ApiResponse<PlayerSyncResponse>), (status = 400, body = ApiErrorBody), (status = 401, body = ApiErrorBody)),
)]
pub async fn connect_player(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PlayerEventRequest>,
) -> Result<Json<ApiResponse<PlayerSyncResponse>>, AppError> {
    let server = service::authenticate(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::connect_player(&state, &server, request).await?,
    )))
}

#[utoipa::path(
    post, path = "/api/v1/game-servers/players/disconnect", tag = "game-servers",
    summary = "Player disconnected",
    request_body = PlayerDisconnectRequest,
    responses((status = 200, body = ApiResponse<PlayerSyncResponse>), (status = 400, body = ApiErrorBody), (status = 401, body = ApiErrorBody)),
)]
pub async fn disconnect_player(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PlayerDisconnectRequest>,
) -> Result<Json<ApiResponse<PlayerSyncResponse>>, AppError> {
    let server = service::authenticate(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::disconnect_player(&state, &server, request).await?,
    )))
}

#[utoipa::path(
    post, path = "/api/v1/game-servers/players/sync", tag = "game-servers",
    summary = "Full online-player calibration",
    description = "Missing players are marked disconnected only on this server; sessions on other servers are untouched.",
    request_body = PlayerSyncRequest,
    responses((status = 200, body = ApiResponse<PlayerSyncResponse>), (status = 400, body = ApiErrorBody), (status = 401, body = ApiErrorBody)),
)]
pub async fn sync_players(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PlayerSyncRequest>,
) -> Result<Json<ApiResponse<PlayerSyncResponse>>, AppError> {
    let server = service::authenticate(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::sync_players(&state, &server, request).await?,
    )))
}

#[utoipa::path(
    post, path = "/api/v1/game-servers/records", tag = "game-servers",
    summary = "Submit a GOKZ finish",
    description = "Idempotent on (`server_id`, `record_identifier`). PB/WR/points are decided by the API and phase-6 triggers, never by the plugin.",
    request_body = RecordSubmissionRequest,
    responses(
        (status = 200, body = ApiResponse<RecordSubmissionResponse>),
        (status = 400, body = ApiErrorBody),
        (status = 401, body = ApiErrorBody),
        (status = 403, body = ApiErrorBody),
    ),
)]
pub async fn submit_record(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<RecordSubmissionRequest>,
) -> Result<Json<ApiResponse<RecordSubmissionResponse>>, AppError> {
    let server = service::authenticate(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::submit_record(&state, &server, request).await?,
    )))
}

#[utoipa::path(
    get, path = "/api/v1/game-servers/records/status", tag = "game-servers",
    summary = "Record / replay / points status",
    params(RecordStatusQuery),
    responses((status = 200, body = ApiResponse<RecordStatusResponse>), (status = 404, body = ApiErrorBody)),
)]
pub async fn record_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<RecordStatusQuery>,
) -> Result<Json<ApiResponse<RecordStatusResponse>>, AppError> {
    let server = service::authenticate(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::record_status(&state, &server, &query).await?,
    )))
}

#[utoipa::path(
    post, path = "/api/v1/game-servers/replays", tag = "game-servers",
    summary = "Register replay metadata",
    description = "The replay bytes stay in R2/S3. Registration is idempotent on (bucket, object_key); a failed upload leaves the record valid and creates a retryable sync task.",
    request_body = ReplayRegistrationRequest,
    responses((status = 200, body = ApiResponse<ReplayRegistrationResponse>), (status = 400, body = ApiErrorBody), (status = 404, body = ApiErrorBody)),
)]
pub async fn register_replay(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ReplayRegistrationRequest>,
) -> Result<Json<ApiResponse<ReplayRegistrationResponse>>, AppError> {
    let server = service::authenticate(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::register_replay(&state, &server, request).await?,
    )))
}

#[utoipa::path(
    get, path = "/api/v1/game-servers/config/sync", tag = "game-servers",
    summary = "Map, course, mode, style and tier snapshot",
    description = "Send the last `version` to receive `changed=false`; heartbeat never carries the full map list.",
    params(ConfigSyncQuery),
    responses((status = 200, body = ApiResponse<ConfigSyncResponse>), (status = 401, body = ApiErrorBody)),
)]
pub async fn config_sync(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ConfigSyncQuery>,
) -> Result<Json<ApiResponse<ConfigSyncResponse>>, AppError> {
    let _server = service::authenticate(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::config_sync(&state, &query).await?,
    )))
}

// ---------------------------------------------------------------------------
// Admin
// ---------------------------------------------------------------------------

#[utoipa::path(
    get, path = "/api/v1/admin/game-servers", tag = "game-servers",
    summary = "List registered game servers",
    params(AdminServerQuery),
    responses((status = 200, body = ApiResponse<AdminServerListResponse>), (status = 401, body = ApiErrorBody)),
)]
pub async fn admin_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AdminServerQuery>,
) -> Result<Json<ApiResponse<AdminServerListResponse>>, AppError> {
    crate::services::auth::require_admin(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::admin_list(&state, &query).await?,
    )))
}

#[utoipa::path(
    post, path = "/api/v1/admin/game-servers", tag = "game-servers",
    summary = "Create a game server and return its token once",
    request_body = CreateServerRequest,
    responses((status = 200, body = ApiResponse<CreateServerResponse>), (status = 400, body = ApiErrorBody), (status = 401, body = ApiErrorBody)),
)]
pub async fn admin_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateServerRequest>,
) -> Result<Json<ApiResponse<CreateServerResponse>>, AppError> {
    crate::services::auth::require_admin(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::admin_create(&state, request).await?,
    )))
}

#[utoipa::path(
    get, path = "/api/v1/admin/game-servers/{server_id}", tag = "game-servers",
    summary = "Server detail with heartbeats, players, records, replays and sync logs",
    params(("server_id" = i64, Path)),
    responses((status = 200, body = ApiResponse<AdminServerDetailResponse>), (status = 404, body = ApiErrorBody)),
)]
pub async fn admin_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(server_id): Path<i64>,
) -> Result<Json<ApiResponse<AdminServerDetailResponse>>, AppError> {
    crate::services::auth::require_admin(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::admin_detail(&state, server_id).await?,
    )))
}

#[utoipa::path(
    patch, path = "/api/v1/admin/game-servers/{server_id}", tag = "game-servers",
    summary = "Update server metadata",
    params(("server_id" = i64, Path)),
    request_body = UpdateServerRequest,
    responses((status = 200, body = ApiResponse<AdminServerSummary>), (status = 404, body = ApiErrorBody)),
)]
pub async fn admin_update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(server_id): Path<i64>,
    Json(request): Json<UpdateServerRequest>,
) -> Result<Json<ApiResponse<AdminServerSummary>>, AppError> {
    crate::services::auth::require_admin(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::admin_update(&state, server_id, request).await?,
    )))
}

#[utoipa::path(
    post, path = "/api/v1/admin/game-servers/{server_id}/enable", tag = "game-servers",
    summary = "Enable a server",
    params(("server_id" = i64, Path)),
    responses((status = 200, body = ApiResponse<ServerActionResponse>), (status = 404, body = ApiErrorBody)),
)]
pub async fn admin_enable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(server_id): Path<i64>,
) -> Result<Json<ApiResponse<ServerActionResponse>>, AppError> {
    crate::services::auth::require_admin(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::admin_set_enabled(&state, server_id, true).await?,
    )))
}

#[utoipa::path(
    post, path = "/api/v1/admin/game-servers/{server_id}/disable", tag = "game-servers",
    summary = "Disable a server; all ingestion is rejected immediately",
    params(("server_id" = i64, Path)),
    responses((status = 200, body = ApiResponse<ServerActionResponse>), (status = 404, body = ApiErrorBody)),
)]
pub async fn admin_disable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(server_id): Path<i64>,
) -> Result<Json<ApiResponse<ServerActionResponse>>, AppError> {
    crate::services::auth::require_admin(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::admin_set_enabled(&state, server_id, false).await?,
    )))
}

#[utoipa::path(
    post, path = "/api/v1/admin/game-servers/{server_id}/token/rotate", tag = "game-servers",
    summary = "Rotate the server token; the new token is returned once",
    params(("server_id" = i64, Path)),
    responses((status = 200, body = ApiResponse<RotateTokenResponse>), (status = 404, body = ApiErrorBody)),
)]
pub async fn admin_rotate_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(server_id): Path<i64>,
) -> Result<Json<ApiResponse<RotateTokenResponse>>, AppError> {
    crate::services::auth::require_admin(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::admin_rotate_token(&state, server_id).await?,
    )))
}

#[utoipa::path(
    post, path = "/api/v1/admin/game-servers/{server_id}/token/revoke", tag = "game-servers",
    summary = "Revoke the server token",
    params(("server_id" = i64, Path)),
    responses((status = 200, body = ApiResponse<ServerActionResponse>), (status = 404, body = ApiErrorBody)),
)]
pub async fn admin_revoke_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(server_id): Path<i64>,
) -> Result<Json<ApiResponse<ServerActionResponse>>, AppError> {
    crate::services::auth::require_admin(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::admin_revoke_token(&state, server_id).await?,
    )))
}

#[utoipa::path(
    post, path = "/api/v1/admin/game-servers/tasks/{task_id}/retry", tag = "game-servers",
    summary = "Retry a failed replay / sync task",
    params(("task_id" = i64, Path)),
    responses((status = 200, body = ApiResponse<RetryTaskResponse>), (status = 404, body = ApiErrorBody)),
)]
pub async fn admin_retry_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(task_id): Path<i64>,
) -> Result<Json<ApiResponse<RetryTaskResponse>>, AppError> {
    crate::services::auth::require_admin(&state, &headers).await?;
    Ok(Json(ApiResponse::ok(
        service::admin_retry_task(&state, task_id).await?,
    )))
}
