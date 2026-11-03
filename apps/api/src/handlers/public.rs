//! Public portal read-only HTTP handlers (`/api/v1/public/*`).

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};

use crate::dto::public::leaderboards::{LeaderboardQuery, LeaderboardResponse};
use crate::dto::public::maps::{
    MapDetailQuery, MapDetailResponse, MapLeaderboardQuery, MapLeaderboardResponse, MapListQuery,
    MapListResponse, MapStatistics,
};
use crate::dto::public::players::{
    CompareQuery, CompareResponse, PlayerDetailQuery, PlayerDetailResponse, PlayerListQuery,
    PlayerListResponse, PlayerMapsQuery, PlayerMapsResponse, PlayerRecordsQuery,
    PlayerRecordsResponse,
};
use crate::dto::public::records::{
    ActivePlayersQuery, ActivePlayersResponse, RecentRecordsQuery, RecentRecordsResponse,
    RecentWorldRecordsQuery, RecordDetailResponse, RecordListResponse, RecordQuery,
    WorldRecordEntry,
};
use crate::dto::public::search::{SearchQuery, SearchResponse};
use crate::dto::public::servers::{ServerListResponse, ServerQuery, ServerSummary};
use crate::dto::public::stats::{PublicModesResponse, PublicStatistics};
use crate::dto::response::{ApiErrorBody, ApiResponse};
use crate::errors::AppError;
use crate::services::public as public_service;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/statistics", get(statistics))
        .route("/modes", get(modes))
        .route("/leaderboards", get(leaderboard))
        .route("/search", get(search))
        .route("/records", get(records))
        .route("/records/recent", get(recent_records))
        .route("/records/world-records", get(recent_world_records))
        .route("/records/{id}", get(record_detail))
        .route("/players", get(players))
        .route("/players/active", get(active_players))
        .route("/players/compare", get(compare))
        .route("/players/{steamid}", get(player_detail))
        .route("/players/{steamid}/records", get(player_records))
        .route("/players/{steamid}/maps", get(player_maps))
        .route("/maps", get(maps))
        .route("/maps/{slug}", get(map_detail))
        .route("/maps/{slug}/leaderboard", get(map_leaderboard))
        .route("/maps/{slug}/statistics", get(map_statistics))
        .route("/servers", get(servers))
        .route("/servers/{id}", get(server_detail))
}

fn validate_identifier(value: &str, max: usize, field: &str) -> Result<(), AppError> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max {
        return Err(AppError::validation(format!(
            "`{field}` must be between 1 and {max} characters"
        )));
    }
    Ok(())
}

/// `GET /api/v1/public/statistics`
#[utoipa::path(
    get,
    path = "/api/v1/public/statistics",
    tag = "public",
    summary = "Public aggregate statistics",
    responses((status = 200, body = ApiResponse<PublicStatistics>)),
)]
pub async fn statistics(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<PublicStatistics>>, AppError> {
    Ok(Json(ApiResponse::ok(
        public_service::statistics(&state).await?,
    )))
}

/// `GET /api/v1/public/modes`
#[utoipa::path(
    get,
    path = "/api/v1/public/modes",
    tag = "public",
    summary = "List movement modes",
    responses((status = 200, body = ApiResponse<PublicModesResponse>)),
)]
pub async fn modes(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<PublicModesResponse>>, AppError> {
    Ok(Json(ApiResponse::ok(public_service::modes(&state).await?)))
}

/// `GET /api/v1/public/leaderboards`
#[utoipa::path(
    get,
    path = "/api/v1/public/leaderboards",
    tag = "public",
    summary = "Mode leaderboard",
    description = "Ranked by completed main-course maps. Points are not computed yet.",
    params(LeaderboardQuery),
    responses(
        (status = 200, body = ApiResponse<LeaderboardResponse>),
        (status = 400, body = ApiErrorBody),
        (status = 404, body = ApiErrorBody),
    ),
)]
pub async fn leaderboard(
    State(state): State<AppState>,
    Query(query): Query<LeaderboardQuery>,
) -> Result<Json<ApiResponse<LeaderboardResponse>>, AppError> {
    Ok(Json(ApiResponse::ok(
        public_service::leaderboard(&state, &query).await?,
    )))
}

/// `GET /api/v1/public/records`
#[utoipa::path(
    get,
    path = "/api/v1/public/records",
    tag = "public",
    summary = "Search approved records",
    params(RecordQuery),
    responses(
        (status = 200, body = ApiResponse<RecordListResponse>),
        (status = 400, body = ApiErrorBody),
    ),
)]
pub async fn records(
    State(state): State<AppState>,
    Query(query): Query<RecordQuery>,
) -> Result<Json<ApiResponse<RecordListResponse>>, AppError> {
    Ok(Json(ApiResponse::ok(
        public_service::records(&state, &query).await?,
    )))
}

/// `GET /api/v1/public/records/recent`
#[utoipa::path(
    get,
    path = "/api/v1/public/records/recent",
    tag = "public",
    summary = "Recent uploads",
    params(RecentRecordsQuery),
    responses(
        (status = 200, body = ApiResponse<RecentRecordsResponse>),
        (status = 400, body = ApiErrorBody),
    ),
)]
pub async fn recent_records(
    State(state): State<AppState>,
    Query(query): Query<RecentRecordsQuery>,
) -> Result<Json<ApiResponse<RecentRecordsResponse>>, AppError> {
    Ok(Json(ApiResponse::ok(
        public_service::recent_records(&state, &query).await?,
    )))
}

/// `GET /api/v1/public/records/world-records`
#[utoipa::path(
    get,
    path = "/api/v1/public/records/world-records",
    tag = "public",
    summary = "Recent world records",
    params(RecentWorldRecordsQuery),
    responses(
        (status = 200, body = ApiResponse<Vec<WorldRecordEntry>>),
        (status = 400, body = ApiErrorBody),
    ),
)]
pub async fn recent_world_records(
    State(state): State<AppState>,
    Query(query): Query<RecentWorldRecordsQuery>,
) -> Result<Json<ApiResponse<Vec<WorldRecordEntry>>>, AppError> {
    Ok(Json(ApiResponse::ok(
        public_service::recent_world_records(&state, &query).await?,
    )))
}

/// `GET /api/v1/public/records/{id}`
#[utoipa::path(
    get,
    path = "/api/v1/public/records/{id}",
    tag = "public",
    summary = "Record detail",
    params(("id" = i64, Path, description = "Record id")),
    responses(
        (status = 200, body = ApiResponse<RecordDetailResponse>),
        (status = 404, body = ApiErrorBody),
    ),
)]
pub async fn record_detail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<RecordDetailResponse>>, AppError> {
    Ok(Json(ApiResponse::ok(
        public_service::record_detail(&state, id).await?,
    )))
}

/// `GET /api/v1/public/players`
#[utoipa::path(
    get,
    path = "/api/v1/public/players",
    tag = "public",
    summary = "Player directory",
    params(PlayerListQuery),
    responses(
        (status = 200, body = ApiResponse<PlayerListResponse>),
        (status = 400, body = ApiErrorBody),
    ),
)]
pub async fn players(
    State(state): State<AppState>,
    Query(query): Query<PlayerListQuery>,
) -> Result<Json<ApiResponse<PlayerListResponse>>, AppError> {
    Ok(Json(ApiResponse::ok(
        public_service::players(&state, &query).await?,
    )))
}

/// `GET /api/v1/public/players/active`
#[utoipa::path(
    get,
    path = "/api/v1/public/players/active",
    tag = "public",
    summary = "Most active players",
    params(ActivePlayersQuery),
    responses(
        (status = 200, body = ApiResponse<ActivePlayersResponse>),
        (status = 400, body = ApiErrorBody),
    ),
)]
pub async fn active_players(
    State(state): State<AppState>,
    Query(query): Query<ActivePlayersQuery>,
) -> Result<Json<ApiResponse<ActivePlayersResponse>>, AppError> {
    Ok(Json(ApiResponse::ok(
        public_service::active_players(&state, &query).await?,
    )))
}

/// `GET /api/v1/public/players/compare`
#[utoipa::path(
    get,
    path = "/api/v1/public/players/compare",
    tag = "public",
    summary = "Compare two players",
    params(CompareQuery),
    responses(
        (status = 200, body = ApiResponse<CompareResponse>),
        (status = 400, body = ApiErrorBody),
        (status = 404, body = ApiErrorBody),
    ),
)]
pub async fn compare(
    State(state): State<AppState>,
    Query(query): Query<CompareQuery>,
) -> Result<Json<ApiResponse<CompareResponse>>, AppError> {
    Ok(Json(ApiResponse::ok(
        public_service::compare(&state, &query).await?,
    )))
}

/// `GET /api/v1/public/players/{steamid}`
#[utoipa::path(
    get,
    path = "/api/v1/public/players/{steamid}",
    tag = "public",
    summary = "Player profile and statistics",
    params(("steamid" = String, Path, description = "SteamID64")),
    params(PlayerDetailQuery),
    responses(
        (status = 200, body = ApiResponse<PlayerDetailResponse>),
        (status = 400, body = ApiErrorBody),
        (status = 404, body = ApiErrorBody),
    ),
)]
pub async fn player_detail(
    State(state): State<AppState>,
    Path(steamid): Path<String>,
    Query(query): Query<PlayerDetailQuery>,
) -> Result<Json<ApiResponse<PlayerDetailResponse>>, AppError> {
    validate_identifier(&steamid, 64, "steamid")?;
    Ok(Json(ApiResponse::ok(
        public_service::player_detail(&state, &steamid, &query).await?,
    )))
}

/// `GET /api/v1/public/players/{steamid}/records`
#[utoipa::path(
    get,
    path = "/api/v1/public/players/{steamid}/records",
    tag = "public",
    summary = "Player records",
    params(("steamid" = String, Path, description = "SteamID64")),
    params(PlayerRecordsQuery),
    responses(
        (status = 200, body = ApiResponse<PlayerRecordsResponse>),
        (status = 404, body = ApiErrorBody),
    ),
)]
pub async fn player_records(
    State(state): State<AppState>,
    Path(steamid): Path<String>,
    Query(query): Query<PlayerRecordsQuery>,
) -> Result<Json<ApiResponse<PlayerRecordsResponse>>, AppError> {
    validate_identifier(&steamid, 64, "steamid")?;
    Ok(Json(ApiResponse::ok(
        public_service::player_records(&state, &steamid, &query).await?,
    )))
}

/// `GET /api/v1/public/players/{steamid}/maps`
#[utoipa::path(
    get,
    path = "/api/v1/public/players/{steamid}/maps",
    tag = "public",
    summary = "Player map progress (completed and uncompleted)",
    params(("steamid" = String, Path, description = "SteamID64")),
    params(PlayerMapsQuery),
    responses(
        (status = 200, body = ApiResponse<PlayerMapsResponse>),
        (status = 400, body = ApiErrorBody),
        (status = 404, body = ApiErrorBody),
    ),
)]
pub async fn player_maps(
    State(state): State<AppState>,
    Path(steamid): Path<String>,
    Query(query): Query<PlayerMapsQuery>,
) -> Result<Json<ApiResponse<PlayerMapsResponse>>, AppError> {
    validate_identifier(&steamid, 64, "steamid")?;
    Ok(Json(ApiResponse::ok(
        public_service::player_maps(&state, &steamid, &query).await?,
    )))
}

/// `GET /api/v1/public/maps`
#[utoipa::path(
    get,
    path = "/api/v1/public/maps",
    tag = "public",
    summary = "Map directory",
    params(MapListQuery),
    responses(
        (status = 200, body = ApiResponse<MapListResponse>),
        (status = 400, body = ApiErrorBody),
    ),
)]
pub async fn maps(
    State(state): State<AppState>,
    Query(query): Query<MapListQuery>,
) -> Result<Json<ApiResponse<MapListResponse>>, AppError> {
    Ok(Json(ApiResponse::ok(
        public_service::maps(&state, &query).await?,
    )))
}

/// `GET /api/v1/public/maps/{slug}`
#[utoipa::path(
    get,
    path = "/api/v1/public/maps/{slug}",
    tag = "public",
    summary = "Map detail",
    params(("slug" = String, Path, description = "Map slug (map name)")),
    params(MapDetailQuery),
    responses(
        (status = 200, body = ApiResponse<MapDetailResponse>),
        (status = 404, body = ApiErrorBody),
    ),
)]
pub async fn map_detail(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<MapDetailQuery>,
) -> Result<Json<ApiResponse<MapDetailResponse>>, AppError> {
    validate_identifier(&slug, 128, "slug")?;
    Ok(Json(ApiResponse::ok(
        public_service::map_detail(&state, &slug, &query).await?,
    )))
}

/// `GET /api/v1/public/maps/{slug}/leaderboard`
#[utoipa::path(
    get,
    path = "/api/v1/public/maps/{slug}/leaderboard",
    tag = "public",
    summary = "Map course leaderboard",
    params(("slug" = String, Path, description = "Map slug (map name)")),
    params(MapLeaderboardQuery),
    responses(
        (status = 200, body = ApiResponse<MapLeaderboardResponse>),
        (status = 404, body = ApiErrorBody),
    ),
)]
pub async fn map_leaderboard(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<MapLeaderboardQuery>,
) -> Result<Json<ApiResponse<MapLeaderboardResponse>>, AppError> {
    validate_identifier(&slug, 128, "slug")?;
    Ok(Json(ApiResponse::ok(
        public_service::map_leaderboard(&state, &slug, &query).await?,
    )))
}

/// `GET /api/v1/public/maps/{slug}/statistics`
#[utoipa::path(
    get,
    path = "/api/v1/public/maps/{slug}/statistics",
    tag = "public",
    summary = "Map completion statistics",
    params(("slug" = String, Path, description = "Map slug (map name)")),
    params(MapDetailQuery),
    responses(
        (status = 200, body = ApiResponse<MapStatistics>),
        (status = 404, body = ApiErrorBody),
    ),
)]
pub async fn map_statistics(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<MapDetailQuery>,
) -> Result<Json<ApiResponse<MapStatistics>>, AppError> {
    validate_identifier(&slug, 128, "slug")?;
    let detail = public_service::map_detail(&state, &slug, &query).await?;
    Ok(Json(ApiResponse::ok(detail.statistics)))
}

/// `GET /api/v1/public/servers`
#[utoipa::path(
    get,
    path = "/api/v1/public/servers",
    tag = "public",
    summary = "Server list",
    params(ServerQuery),
    responses(
        (status = 200, body = ApiResponse<ServerListResponse>),
        (status = 400, body = ApiErrorBody),
    ),
)]
pub async fn servers(
    State(state): State<AppState>,
    Query(query): Query<ServerQuery>,
) -> Result<Json<ApiResponse<ServerListResponse>>, AppError> {
    Ok(Json(ApiResponse::ok(
        public_service::servers(&state, &query).await?,
    )))
}

/// `GET /api/v1/public/servers/{id}`
#[utoipa::path(
    get,
    path = "/api/v1/public/servers/{id}",
    tag = "public",
    summary = "Server detail",
    params(("id" = i64, Path, description = "Server id")),
    responses(
        (status = 200, body = ApiResponse<ServerSummary>),
        (status = 404, body = ApiErrorBody),
    ),
)]
pub async fn server_detail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<ServerSummary>>, AppError> {
    Ok(Json(ApiResponse::ok(
        public_service::server_detail(&state, id).await?,
    )))
}

/// `GET /api/v1/public/search`
#[utoipa::path(
    get,
    path = "/api/v1/public/search",
    tag = "public",
    summary = "Global player/map search",
    params(SearchQuery),
    responses(
        (status = 200, body = ApiResponse<SearchResponse>),
        (status = 400, body = ApiErrorBody),
    ),
)]
pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<ApiResponse<SearchResponse>>, AppError> {
    Ok(Json(ApiResponse::ok(
        public_service::search(&state, &query).await?,
    )))
}
