//! Points & ranking HTTP endpoints.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
};

use crate::dto::points::*;
use crate::dto::response::{ApiErrorBody, ApiResponse};
use crate::errors::AppError;
use crate::repositories::points as repo;
use crate::repositories::public::directory;
use crate::services::{points as points_service, scoring::recalc};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // Public read endpoints (also exposed under /api/v1/public/* via alias).
        .route("/players/{steamid}/points", get(player_points))
        .route(
            "/players/{steamid}/points/filters",
            get(player_filter_points),
        )
        .route("/leaderboards/points/{mode}", get(mode_leaderboard))
        .route("/leaderboards/filters", get(filter_leaderboard))
        // Protected administration. No full-recalc endpoint is exposed without
        // the maintenance token; see `require_maintenance_token`.
        .route("/admin/points/status", get(points_status))
        .route("/admin/points/tiers", get(admin_tiers))
        .route(
            "/admin/points/tiers/{map_id}/{course_id}/{mode_id}",
            axum::routing::patch(update_tier),
        )
        .route("/admin/points/recalc-runs", get(admin_recalc_runs))
        .route(
            "/admin/points/recalc/filter/{filter_id}",
            post(recalc_filter),
        )
        .route("/admin/points/recalc/mode/{mode}", post(recalc_mode))
        .route("/admin/points/initialize", post(initialize))
}

fn require_maintenance_token(state: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
    let Some(expected) = state.config().maintenance_token.as_deref() else {
        return Err(AppError::Forbidden {
            message: "points administration is not configured".to_owned(),
        });
    };
    let presented = headers
        .get("x-maintenance-token")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if presented == expected {
        Ok(())
    } else {
        Err(AppError::Unauthorized {
            message: "invalid maintenance token".to_owned(),
        })
    }
}

async fn find_account(state: &AppState, steamid: &str) -> Result<i64, AppError> {
    if steamid.trim().is_empty() || steamid.chars().count() > 64 {
        return Err(AppError::validation("invalid `steamid`"));
    }
    directory::find_player(state.db(), steamid)
        .await?
        .map(|player| player.account_id)
        .ok_or_else(|| AppError::not_found("player"))
}

#[utoipa::path(
    get, path = "/api/v1/players/{steamid}/points", tag = "points",
    summary = "Player points for KZT / SKZ / VNL",
    params(("steamid" = String, Path, description = "SteamID64")),
    responses((status = 200, body = ApiResponse<PlayerPointsResponse>), (status = 404, body = ApiErrorBody)),
)]
pub async fn player_points(
    State(state): State<AppState>,
    Path(steamid): Path<String>,
) -> Result<Json<ApiResponse<PlayerPointsResponse>>, AppError> {
    let account_id = find_account(&state, &steamid).await?;
    Ok(Json(ApiResponse::ok(
        points_service::player_points(&state, account_id).await?,
    )))
}

#[utoipa::path(
    get, path = "/api/v1/players/{steamid}/points/filters", tag = "points",
    summary = "Player Filter points detail",
    params(("steamid" = String, Path, description = "SteamID64"), PlayerFilterPointsQuery),
    responses((status = 200, body = ApiResponse<PlayerFilterPointsResponse>), (status = 400, body = ApiErrorBody)),
)]
pub async fn player_filter_points(
    State(state): State<AppState>,
    Path(steamid): Path<String>,
    Query(query): Query<PlayerFilterPointsQuery>,
) -> Result<Json<ApiResponse<PlayerFilterPointsResponse>>, AppError> {
    let account_id = find_account(&state, &steamid).await?;
    Ok(Json(ApiResponse::ok(
        points_service::player_filter_points(&state, account_id, &query).await?,
    )))
}

#[utoipa::path(
    get, path = "/api/v1/leaderboards/points/{mode}", tag = "points",
    summary = "Independent points leaderboard for KZT / SKZ / VNL",
    params(("mode" = String, Path, description = "KZT, SKZ or VNL"), PointsLeaderboardQuery),
    responses((status = 200, body = ApiResponse<PointsLeaderboardResponse>), (status = 404, body = ApiErrorBody)),
)]
pub async fn mode_leaderboard(
    State(state): State<AppState>,
    Path(mode): Path<String>,
    Query(query): Query<PointsLeaderboardQuery>,
) -> Result<Json<ApiResponse<PointsLeaderboardResponse>>, AppError> {
    let mode = mode.to_ascii_uppercase();
    if crate::services::scoring::model::ScoringMode::parse(&mode).is_none() {
        return Err(AppError::not_found("scoring mode"));
    }
    let page = page_params(query.page, query.page_size)?;
    Ok(Json(ApiResponse::ok(
        points_service::mode_leaderboard(&state, &mode, page).await?,
    )))
}

#[utoipa::path(
    get, path = "/api/v1/leaderboards/filters", tag = "points",
    summary = "One Filter's points leaderboard",
    params(FilterLeaderboardQuery),
    responses((status = 200, body = ApiResponse<FilterLeaderboardResponse>), (status = 404, body = ApiErrorBody)),
)]
pub async fn filter_leaderboard(
    State(state): State<AppState>,
    Query(mut query): Query<FilterLeaderboardQuery>,
) -> Result<Json<ApiResponse<FilterLeaderboardResponse>>, AppError> {
    query.mode = query.mode.to_ascii_uppercase();
    query.record_type = query.record_type.to_ascii_lowercase();
    let page = page_params(query.page, query.page_size)?;
    Ok(Json(ApiResponse::ok(
        points_service::filter_leaderboard(&state, &query, page).await?,
    )))
}

#[utoipa::path(get, path = "/api/v1/admin/points/status", tag = "points", summary = "Points recalc status",
    responses((status = 200, body = ApiResponse<PointsSystemStatus>), (status = 401, body = ApiErrorBody)))]
pub async fn points_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<PointsSystemStatus>>, AppError> {
    require_maintenance_token(&state, &headers)?;
    Ok(Json(ApiResponse::ok(
        points_service::system_status(&state).await?,
    )))
}

#[utoipa::path(get, path = "/api/v1/admin/points/tiers", tag = "points", summary = "Course tiers and first completion",
    params(AdminTiersQuery),
    responses((status = 200, body = ApiResponse<AdminTiersResponse>), (status = 401, body = ApiErrorBody)))]
pub async fn admin_tiers(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AdminTiersQuery>,
) -> Result<Json<ApiResponse<AdminTiersResponse>>, AppError> {
    require_maintenance_token(&state, &headers)?;
    let page = page_params(query.page, query.page_size)?;
    let mode_id = match query.mode.as_deref() {
        Some(code) => Some(
            repo::scoring_mode_id(state.db(), &code.to_ascii_uppercase())
                .await?
                .ok_or_else(|| AppError::validation("`mode` must be one of KZT, SKZ, VNL"))?,
        ),
        None => None,
    };
    let items = repo::tiers_for_admin(state.db(), mode_id, page.limit, page.offset).await?;
    let total = repo::tiers_total(state.db(), mode_id).await?;
    Ok(Json(ApiResponse::ok(AdminTiersResponse {
        items,
        meta: crate::dto::public::common::PageMeta::from_total(total, page),
    })))
}

#[utoipa::path(patch, path = "/api/v1/admin/points/tiers/{map_id}/{course_id}/{mode_id}", tag = "points",
    summary = "Update a course tier (triggers Filter recalculation)",
    params(("map_id" = i64, Path), ("course_id" = i64, Path), ("mode_id" = i16, Path),),
    request_body = UpdateTierRequest,
    responses((status = 200, body = ApiResponse<UpdateTierResponse>), (status = 400, body = ApiErrorBody), (status = 404, body = ApiErrorBody)))]
pub async fn update_tier(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((map_id, course_id, mode_id)): Path<(i64, i64, i16)>,
    Json(body): Json<UpdateTierRequest>,
) -> Result<Json<ApiResponse<UpdateTierResponse>>, AppError> {
    require_maintenance_token(&state, &headers)?;
    if !(1..=8).contains(&body.tier) {
        return Err(AppError::validation("`tier` must be between 1 and 8"));
    }
    let updated = repo::update_tier(state.db(), map_id, course_id, mode_id, body.tier).await?;
    if !updated {
        return Err(AppError::not_found("course tier"));
    }
    Ok(Json(ApiResponse::ok(UpdateTierResponse {
        updated: true,
        tier: body.tier,
    })))
}

#[utoipa::path(get, path = "/api/v1/admin/points/recalc-runs", tag = "points", summary = "Recent recalculation runs",
    params(RecalcRunsQuery),
    responses((status = 200, body = ApiResponse<RecalcRunsResponse>), (status = 401, body = ApiErrorBody)))]
pub async fn admin_recalc_runs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<RecalcRunsQuery>,
) -> Result<Json<ApiResponse<RecalcRunsResponse>>, AppError> {
    require_maintenance_token(&state, &headers)?;
    Ok(Json(ApiResponse::ok(
        points_service::recalc_runs(&state, page_params(query.page, query.page_size)?).await?,
    )))
}

#[utoipa::path(post, path = "/api/v1/admin/points/recalc/filter/{filter_id}", tag = "points", summary = "Queue one Filter recalculation",
    params(("filter_id" = i64, Path)),
    responses((status = 200, body = ApiResponse<serde_json::Value>), (status = 404, body = ApiErrorBody)))]
pub async fn recalc_filter(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(filter_id): Path<i64>,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_maintenance_token(&state, &headers)?;
    if !recalc::mark_filter_dirty(state.db(), filter_id).await? {
        return Err(AppError::not_found("filter"));
    }
    Ok(Json(ApiResponse::ok(
        serde_json::json!({ "queued": true, "filter_id": filter_id }),
    )))
}

#[utoipa::path(post, path = "/api/v1/admin/points/recalc/mode/{mode}", tag = "points", summary = "Queue all Filters of one mode",
    params(("mode" = String, Path, description = "KZT / SKZ / VNL"),),
    responses((status = 200, body = ApiResponse<RecalcResponse>), (status = 400, body = ApiErrorBody)))]
pub async fn recalc_mode(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(mode): Path<String>,
) -> Result<Json<ApiResponse<RecalcResponse>>, AppError> {
    require_maintenance_token(&state, &headers)?;
    let mode = mode.to_ascii_uppercase();
    let mode_id = repo::scoring_mode_id(state.db(), &mode)
        .await?
        .ok_or_else(|| AppError::validation("`mode` must be one of KZT, SKZ, VNL"))?;
    let queued = recalc::mark_all_dirty(state.db(), mode_id).await?;
    Ok(Json(ApiResponse::ok(RecalcResponse {
        queued_filters: queued,
        mode,
    })))
}

#[utoipa::path(post, path = "/api/v1/admin/points/initialize", tag = "points",
    summary = "Initialize Filters from existing records (no T9 backfill)",
    responses((status = 200, body = ApiResponse<serde_json::Value>), (status = 401, body = ApiErrorBody)))]
pub async fn initialize(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<serde_json::Value>>, AppError> {
    require_maintenance_token(&state, &headers)?;
    let filters = recalc::initialize_existing(state.db()).await?;
    Ok(Json(ApiResponse::ok(serde_json::json!({
        "initialized_filters": filters,
        "historical_t9_backfill": false
    }))))
}
