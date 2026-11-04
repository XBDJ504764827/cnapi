//! Phase-7 game-server orchestration.
//!
//! SourceMod reports facts (heartbeat, players, GOKZ finish, replay metadata).
//! This service validates that data and delegates the authoritative PB/WR/points
//! work to PostgreSQL triggers and the phase-6 recalculation worker.

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::dto::game_server::{
    AdminServerDetailResponse, AdminServerListResponse, AdminServerQuery, AdminServerSummary,
    ConfigSyncQuery, ConfigSyncResponse, CreateServerRequest, CreateServerResponse,
    HeartbeatRequest, HeartbeatResponse, PlayerDisconnectRequest, PlayerEventRequest,
    PlayerSyncRequest, PlayerSyncResponse, RecordStatusQuery, RecordStatusResponse,
    RecordSubmissionRequest, RecordSubmissionResponse, ReplayRegistrationRequest,
    ReplayRegistrationResponse, RetryTaskResponse, RotateTokenResponse, ServerActionResponse,
    UpdateServerRequest,
};
use crate::dto::public::common::{PageMeta, seconds_string_3};
use crate::errors::AppError;
use crate::repositories::game_servers as repo;
use crate::state::AppState;

pub const HEARTBEAT_INTERVAL_SECONDS: i64 = 25;

fn presented_token(headers: &axum::http::HeaderMap) -> Option<&str> {
    if let Some(value) = headers
        .get("x-server-token")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(value);
    }
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub async fn authenticate(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<repo::AuthServer, AppError> {
    let token = presented_token(headers).ok_or_else(|| AppError::Unauthorized {
        message: "missing server token".to_owned(),
    })?;
    let server = repo::authenticate_server(state.db(), token)
        .await?
        .ok_or_else(|| AppError::Unauthorized {
            message: "invalid server token".to_owned(),
        })?;
    if server.api_key_revoked_at.is_some() {
        return Err(AppError::Unauthorized {
            message: "server token has been revoked".to_owned(),
        });
    }
    if !server.is_enabled {
        return Err(AppError::Forbidden {
            message: "server is disabled".to_owned(),
        });
    }
    Ok(server)
}

pub fn require_maintenance_token(
    state: &AppState,
    headers: &axum::http::HeaderMap,
) -> Result<(), AppError> {
    let Some(expected) = state.config().maintenance_token.as_deref() else {
        return Err(AppError::Forbidden {
            message: "game server administration is not configured".to_owned(),
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

fn normalize_mode(value: &str) -> Result<&'static str, AppError> {
    match value.trim().to_ascii_uppercase().as_str() {
        "KZT" | "KZTIMER" => Ok("kztimer"),
        "SKZ" | "SIMPLE" | "SIMPLEKZ" => Ok("simple"),
        "VNL" | "VANILLA" => Ok("vanilla"),
        "CLASSIC" => Ok("classic"),
        other => Err(AppError::validation(format!(
            "`mode` must be KZT, SKZ or VNL, got `{other}`"
        ))),
    }
}

fn validate_steam_id64(value: &str) -> Result<&str, AppError> {
    let value = value.trim();
    if value.is_empty() || value.len() > 20 || !value.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::validation(
            "`steam_id64` must be a decimal uint64 string",
        ));
    }
    if value.chars().all(|c| c == '0') {
        return Err(AppError::validation("`steam_id64` must be greater than 0"));
    }
    Ok(value)
}

fn validate_display_name(value: Option<&str>, fallback: &str) -> String {
    let value = value.unwrap_or(fallback).trim();
    if value.is_empty() {
        fallback.to_owned()
    } else {
        value.chars().take(64).collect()
    }
}

fn validate_record_identifier(value: &str) -> Result<&str, AppError> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 128 {
        return Err(AppError::validation(
            "`record_identifier` must be between 1 and 128 characters",
        ));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
    {
        return Err(AppError::validation(
            "`record_identifier` contains unsupported characters",
        ));
    }
    Ok(value)
}

fn time_ns_from_request(time: Option<f64>, time_ns: Option<i64>) -> Result<i64, AppError> {
    let value = match (time_ns, time) {
        (Some(value), _) => value,
        (None, Some(seconds)) => {
            if !seconds.is_finite() || seconds <= 0.0 {
                return Err(AppError::validation(
                    "`time` must be a positive finite number",
                ));
            }
            (seconds * 1_000_000_000.0).round() as i64
        }
        (None, None) => {
            return Err(AppError::validation(
                "one of `time` or `time_ns` is required",
            ));
        }
    };
    // 24 hours is a generous upper bound; it catches unit mistakes (ms -> ns,
    // seconds -> ns) without rejecting legitimate long KZ runs.
    if value <= 0 || value > 24 * 60 * 60 * 1_000_000_000 {
        return Err(AppError::validation(
            "run duration must be between 0 and 24 hours",
        ));
    }
    Ok(value)
}

pub async fn heartbeat(
    state: &AppState,
    server: &repo::AuthServer,
    request: HeartbeatRequest,
) -> Result<HeartbeatResponse, AppError> {
    if let Some(online) = request.online_players
        && !(0..=128).contains(&online)
    {
        return Err(AppError::validation(
            "`online_players` must be between 0 and 128",
        ));
    }
    if let Some(max) = request.max_players
        && !(1..=128).contains(&max)
    {
        return Err(AppError::validation(
            "`max_players` must be between 1 and 128",
        ));
    }
    if let Some(port) = request.server_port
        && !(1..=65535).contains(&port)
    {
        return Err(AppError::validation(
            "`server_port` must be between 1 and 65535",
        ));
    }
    let mode_id = match request.mode.as_deref() {
        Some(mode) => {
            let code = normalize_mode(mode)?;
            sqlx::query_scalar::<_, i16>("SELECT id FROM movement_modes WHERE code = $1")
                .bind(code)
                .fetch_optional(state.db())
                .await?
        }
        None => None,
    };

    repo::record_heartbeat(
        state.db(),
        server.id,
        request.server_name.as_deref(),
        request.map.as_deref(),
        mode_id,
        request.online_players,
        request.max_players,
        request.game_version.as_deref(),
        request.plugin_version.as_deref(),
        request.timestamp,
    )
    .await?;
    // Heartbeat history is kept in `server_heartbeats`; logging every success
    // here would grow the sync log without diagnostic value.
    let config_version = repo::config_version(state.db()).await?;
    Ok(HeartbeatResponse {
        server_id: server.id,
        status: "online".to_owned(),
        received_at: Utc::now(),
        heartbeat_interval_seconds: HEARTBEAT_INTERVAL_SECONDS,
        config_version,
    })
}

fn player_status(value: Option<&str>) -> Result<&str, AppError> {
    match value.unwrap_or("connected") {
        "connected" | "spectating" | "afk" => Ok(value.unwrap_or("connected")),
        other => Err(AppError::validation(format!(
            "`status` must be connected, spectating or afk, got `{other}`"
        ))),
    }
}

pub async fn connect_player(
    state: &AppState,
    server: &repo::AuthServer,
    request: PlayerEventRequest,
) -> Result<PlayerSyncResponse, AppError> {
    let steam_id64 = validate_steam_id64(&request.steam_id64)?;
    let status = player_status(request.status.as_deref())?;
    let name = validate_display_name(Some(&request.display_name), "Unknown player");
    repo::connect_player(
        state.db(),
        server.id,
        steam_id64,
        &name,
        request.map.as_deref(),
        request.mode.as_deref(),
        status,
        request.connected_at,
    )
    .await?;
    let online: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM server_player_sessions
          WHERE server_id = $1 AND disconnected_at IS NULL",
    )
    .bind(server.id)
    .fetch_one(state.db())
    .await?;
    Ok(PlayerSyncResponse {
        connected: 1,
        disconnected: 0,
        online,
    })
}

pub async fn disconnect_player(
    state: &AppState,
    server: &repo::AuthServer,
    request: PlayerDisconnectRequest,
) -> Result<PlayerSyncResponse, AppError> {
    let steam_id64 = validate_steam_id64(&request.steam_id64)?;
    let changed =
        repo::disconnect_player(state.db(), server.id, steam_id64, request.disconnected_at).await?;
    let online: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM server_player_sessions
          WHERE server_id = $1 AND disconnected_at IS NULL",
    )
    .bind(server.id)
    .fetch_one(state.db())
    .await?;
    Ok(PlayerSyncResponse {
        connected: 0,
        disconnected: i64::from(changed),
        online,
    })
}

pub async fn sync_players(
    state: &AppState,
    server: &repo::AuthServer,
    request: PlayerSyncRequest,
) -> Result<PlayerSyncResponse, AppError> {
    if request.players.len() > 128 {
        return Err(AppError::validation(
            "`players` may contain at most 128 entries",
        ));
    }
    for player in &request.players {
        validate_steam_id64(&player.steam_id64)?;
        player_status(player.status.as_deref())?;
    }
    let (connected, disconnected, online) =
        repo::sync_players(state.db(), server.id, &request.players, None, None).await?;
    Ok(PlayerSyncResponse {
        connected,
        disconnected,
        online,
    })
}

pub async fn submit_record(
    state: &AppState,
    server: &repo::AuthServer,
    request: RecordSubmissionRequest,
) -> Result<RecordSubmissionResponse, AppError> {
    let steam_id64 = validate_steam_id64(&request.steam_id64)?;
    let record_identifier = validate_record_identifier(&request.record_identifier)?;
    let time_ns = time_ns_from_request(request.time, request.time_ns)?;
    if !(1..=1000).contains(&request.tickrate) {
        return Err(AppError::validation(
            "`tickrate` must be between 1 and 1000",
        ));
    }
    if request.teleports < 0 {
        return Err(AppError::validation("`teleports` must be non-negative"));
    }
    if !(0..100).contains(&request.course) {
        return Err(AppError::validation("`course` must be between 0 and 99"));
    }
    if request.map.trim().is_empty() || request.map.len() > 128 {
        return Err(AppError::validation(
            "`map` must be between 1 and 128 characters",
        ));
    }
    if let Some(created_at) = request.created_at
        && created_at > Utc::now() + Duration::minutes(5)
    {
        return Err(AppError::validation("`created_at` is in the future"));
    }
    let mode_code = normalize_mode(&request.mode)?;
    let style_code = request
        .style
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("normal")
        .to_ascii_lowercase();

    let context = repo::resolve_record_context(
        state.db(),
        request.map.trim(),
        request.course,
        mode_code,
        &style_code,
    )
    .await?;
    let Some(context) = context else {
        repo::log_sync(
            state.db(),
            Some(server.id),
            "record",
            "rejected",
            Some("map/course/mode/style not found"),
            None,
            serde_json::json!({
                "record_identifier": record_identifier,
                "map": request.map,
                "course": request.course,
                "mode": request.mode,
                "style": request.style
            }),
        )
        .await;
        return Err(AppError::validation(
            "map, course, mode or style does not exist",
        ));
    };

    let player_name = validate_display_name(request.player_name.as_deref(), "Unknown player");
    let result = repo::ingest_record(
        state.db(),
        &repo::RecordInput {
            server_id: server.id,
            server_plugin_version: request
                .plugin_version
                .clone()
                .or_else(|| server.plugin_version.clone()),
            steam_id64: steam_id64.to_owned(),
            player_name: Some(player_name),
            map_id: context.map_id,
            course_id: context.course_id,
            mode_id: context.mode_id,
            style_id: context.style_id,
            tickrate: request.tickrate,
            teleports: request.teleports,
            time_ns,
            record_identifier: record_identifier.to_owned(),
            created_at: request.created_at,
            replay_path: request.replay_path,
        },
    )
    .await?;

    Ok(RecordSubmissionResponse {
        accepted: true,
        duplicate: result.duplicate,
        record_id: result.record_id,
        record_identifier: record_identifier.to_owned(),
        time_ns: result.time_ns,
        time_seconds: seconds_string_3(result.time_ns),
        time_type: result.time_type,
        is_pb: result.is_pb,
        is_world_record: result.is_world_record,
        points_status: result.points_status,
        replay_status: result.replay_status,
    })
}

pub async fn record_status(
    state: &AppState,
    server: &repo::AuthServer,
    query: &RecordStatusQuery,
) -> Result<RecordStatusResponse, AppError> {
    let identifier = validate_record_identifier(&query.record_identifier)?;
    let row = repo::record_status(state.db(), server.id, identifier)
        .await?
        .ok_or_else(|| AppError::not_found("record"))?;
    let (is_pb, is_world_record, points_status) = repo::status_flags(state.db(), &row).await?;
    Ok(RecordStatusResponse {
        record_id: row.id,
        record_identifier: identifier.to_owned(),
        status: row.status,
        time_ns: row.time_ns,
        time_seconds: seconds_string_3(row.time_ns),
        is_pb,
        is_world_record,
        replay_status: row
            .replay_status
            .unwrap_or_else(|| "not_reported".to_owned()),
        replay_id: row.replay_id,
        points_status,
    })
}

fn parse_checksum(value: &str) -> Result<Vec<u8>, AppError> {
    let value = value.trim();
    if value.len() != 64 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::validation(
            "`checksum_sha256` must be 64 hexadecimal characters",
        ));
    }
    (0..64)
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .map_err(|_| AppError::validation("invalid `checksum_sha256`"))
        })
        .collect()
}

pub async fn register_replay(
    state: &AppState,
    server: &repo::AuthServer,
    request: ReplayRegistrationRequest,
) -> Result<ReplayRegistrationResponse, AppError> {
    let record_identifier = validate_record_identifier(&request.record_identifier)?;
    if request.bucket.trim().is_empty() || request.bucket.len() > 128 {
        return Err(AppError::validation(
            "`bucket` must be between 1 and 128 characters",
        ));
    }
    if request.object_key.trim().is_empty() || request.object_key.len() > 512 {
        return Err(AppError::validation(
            "`object_key` must be between 1 and 512 characters",
        ));
    }
    if request.size_bytes <= 0 {
        return Err(AppError::validation("`size_bytes` must be positive"));
    }
    let provider = request.storage_provider.as_deref().unwrap_or("r2");
    if !matches!(provider, "r2" | "s3" | "local") {
        return Err(AppError::validation(
            "`storage_provider` must be r2, s3 or local",
        ));
    }
    let status = request.status.as_deref().unwrap_or("available");
    if !matches!(status, "pending" | "available" | "failed") {
        return Err(AppError::validation(
            "`status` must be pending, available or failed",
        ));
    }
    if let Some(version) = request.replay_format_version
        && version <= 0
    {
        return Err(AppError::validation(
            "`replay_format_version` must be positive",
        ));
    }
    let checksum = parse_checksum(&request.checksum_sha256)?;
    let result = repo::register_replay(
        state.db(),
        server.id,
        record_identifier,
        request.bucket.trim(),
        request.object_key.trim(),
        provider,
        request.size_bytes,
        &checksum,
        request.replay_format_version,
        status,
        request.uploaded_at,
    )
    .await?
    .ok_or_else(|| AppError::not_found("record"))?;
    Ok(ReplayRegistrationResponse {
        id: result.0,
        record_id: result.1,
        status: result.2.clone(),
        replay_available: result.2 == "available",
        duplicate: result.3,
    })
}

pub async fn config_sync(
    state: &AppState,
    query: &ConfigSyncQuery,
) -> Result<ConfigSyncResponse, AppError> {
    let version = repo::config_version(state.db()).await?;
    if query.version == Some(version) {
        return Ok(ConfigSyncResponse {
            version,
            changed: false,
            generated_at: Utc::now(),
            modes: Vec::new(),
            styles: Vec::new(),
            maps: Vec::new(),
        });
    }
    Ok(ConfigSyncResponse {
        version,
        changed: true,
        generated_at: Utc::now(),
        modes: repo::config_modes(state.db()).await?,
        styles: repo::config_styles(state.db()).await?,
        maps: repo::config_maps(state.db()).await?,
    })
}

pub async fn admin_list(
    state: &AppState,
    query: &AdminServerQuery,
) -> Result<AdminServerListResponse, AppError> {
    let page = query.page_params()?;
    let search = query
        .search
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let rows = repo::admin_list_servers(state.db(), search, query.enabled, page.limit, page.offset)
        .await?;
    let total = repo::admin_count_servers(state.db(), search, query.enabled).await?;
    Ok(AdminServerListResponse {
        items: rows
            .into_iter()
            .map(repo::ServerAdminRow::into_dto)
            .collect(),
        meta: PageMeta::from_total(total, page),
    })
}

fn validate_ip(value: &str) -> Result<&str, AppError> {
    let value = value.trim();
    if value.parse::<std::net::IpAddr>().is_err() {
        return Err(AppError::validation(
            "`ip` must be a valid IPv4/IPv6 address",
        ));
    }
    Ok(value)
}

fn validate_server_type(value: Option<&str>) -> Result<&str, AppError> {
    let value = value.unwrap_or("community");
    if !matches!(value, "community" | "official" | "event" | "testing") {
        return Err(AppError::validation(
            "`server_type` must be community, official, event or testing",
        ));
    }
    Ok(value)
}

fn map_admin_write_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(database) = &error
        && database.is_unique_violation()
    {
        return AppError::validation("server name or address already exists");
    }
    AppError::from(error)
}

pub async fn admin_create(
    state: &AppState,
    request: CreateServerRequest,
) -> Result<CreateServerResponse, AppError> {
    let name = request.name.trim();
    if name.is_empty() || name.chars().count() > 128 {
        return Err(AppError::validation(
            "`name` must be between 1 and 128 characters",
        ));
    }
    let ip = validate_ip(&request.ip)?;
    if !(1..=65535).contains(&request.port) {
        return Err(AppError::validation("`port` must be between 1 and 65535"));
    }
    if let Some(query_port) = request.query_port
        && !(1..=65535).contains(&query_port)
    {
        return Err(AppError::validation(
            "`query_port` must be between 1 and 65535",
        ));
    }
    if let Some(region) = request.region.as_deref()
        && (region.trim().is_empty() || region.chars().count() > 32)
    {
        return Err(AppError::validation(
            "`region` must be between 1 and 32 characters",
        ));
    }
    let server_type = validate_server_type(request.server_type.as_deref())?;
    let max_players = request.max_players.unwrap_or(24);
    if !(1..=128).contains(&max_players) {
        return Err(AppError::validation(
            "`max_players` must be between 1 and 128",
        ));
    }
    let owner_steam_id64 = validate_steam_id64(&request.owner_steam_id64)?;
    let owner_display_name =
        validate_display_name(request.owner_display_name.as_deref(), "Server owner");
    let token = format!(
        "cngokz_srv_{}{}",
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple()
    );
    let server_id = repo::admin_create_server(
        state.db(),
        name,
        ip,
        request.port,
        request.query_port,
        request.region.as_deref().map(str::trim),
        server_type,
        max_players,
        owner_steam_id64,
        &owner_display_name,
        request.plugin_version.as_deref(),
        &token,
    )
    .await
    .map_err(map_admin_write_error)?;
    let row = repo::admin_find_server(state.db(), server_id)
        .await?
        .ok_or_else(|| AppError::not_found("server"))?;
    Ok(CreateServerResponse {
        server: row.into_dto(),
        token,
    })
}

pub async fn admin_update(
    state: &AppState,
    server_id: i64,
    request: UpdateServerRequest,
) -> Result<AdminServerSummary, AppError> {
    if let Some(name) = request.name.as_deref()
        && (name.trim().is_empty() || name.chars().count() > 128)
    {
        return Err(AppError::validation(
            "`name` must be between 1 and 128 characters",
        ));
    }
    if let Some(max_players) = request.max_players
        && !(1..=128).contains(&max_players)
    {
        return Err(AppError::validation(
            "`max_players` must be between 1 and 128",
        ));
    }
    if let Some(query_port) = request.query_port
        && !(1..=65535).contains(&query_port)
    {
        return Err(AppError::validation(
            "`query_port` must be between 1 and 65535",
        ));
    }
    let server_type = match request.server_type.as_deref() {
        Some(value) => Some(validate_server_type(Some(value))?),
        None => None,
    };
    let updated = repo::admin_update_server(
        state.db(),
        server_id,
        request.name.as_deref().map(str::trim),
        request.region.as_deref().map(str::trim),
        server_type,
        request.max_players,
        request.query_port,
        request.plugin_version.as_deref(),
    )
    .await
    .map_err(map_admin_write_error)?;
    if !updated {
        return Err(AppError::not_found("server"));
    }
    let row = repo::admin_find_server(state.db(), server_id)
        .await?
        .ok_or_else(|| AppError::not_found("server"))?;
    Ok(row.into_dto())
}

pub async fn admin_set_enabled(
    state: &AppState,
    server_id: i64,
    enabled: bool,
) -> Result<ServerActionResponse, AppError> {
    if !repo::admin_set_enabled(state.db(), server_id, enabled).await? {
        return Err(AppError::not_found("server"));
    }
    repo::log_sync(
        state.db(),
        Some(server_id),
        "other",
        "success",
        Some(if enabled {
            "server enabled"
        } else {
            "server disabled"
        }),
        None,
        serde_json::json!({ "enabled": enabled }),
    )
    .await;
    Ok(ServerActionResponse {
        server_id,
        enabled,
        token_revoked: false,
        message: if enabled {
            "server enabled".to_owned()
        } else {
            "server disabled; submissions are rejected".to_owned()
        },
    })
}

pub async fn admin_rotate_token(
    state: &AppState,
    server_id: i64,
) -> Result<RotateTokenResponse, AppError> {
    let token = format!(
        "cngokz_srv_{}{}",
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple()
    );
    let (prefix, created_at) = repo::admin_rotate_token(state.db(), server_id, &token)
        .await?
        .ok_or_else(|| AppError::not_found("server"))?;
    repo::log_sync(
        state.db(),
        Some(server_id),
        "auth",
        "success",
        Some("server token rotated"),
        None,
        serde_json::json!({}),
    )
    .await;
    Ok(RotateTokenResponse {
        server_id,
        token,
        prefix,
        created_at,
    })
}

pub async fn admin_revoke_token(
    state: &AppState,
    server_id: i64,
) -> Result<ServerActionResponse, AppError> {
    let revoked = repo::admin_revoke_token(state.db(), server_id).await?;
    if !revoked {
        // Either the server does not exist or it was already revoked.
        if repo::admin_find_server(state.db(), server_id)
            .await?
            .is_none()
        {
            return Err(AppError::not_found("server"));
        }
    }
    repo::log_sync(
        state.db(),
        Some(server_id),
        "auth",
        "success",
        Some("server token revoked"),
        None,
        serde_json::json!({}),
    )
    .await;
    Ok(ServerActionResponse {
        server_id,
        enabled: true,
        token_revoked: true,
        message: "server token revoked".to_owned(),
    })
}

pub async fn admin_detail(
    state: &AppState,
    server_id: i64,
) -> Result<AdminServerDetailResponse, AppError> {
    let server = repo::admin_find_server(state.db(), server_id)
        .await?
        .ok_or_else(|| AppError::not_found("server"))?;
    Ok(repo::admin_server_detail(state.db(), server).await?)
}

pub async fn admin_retry_task(
    state: &AppState,
    task_id: i64,
) -> Result<RetryTaskResponse, AppError> {
    let status = repo::admin_retry_task(state.db(), task_id)
        .await?
        .ok_or_else(|| AppError::not_found("sync task"))?;
    Ok(RetryTaskResponse {
        task_id,
        status,
        message: "task queued for retry".to_owned(),
    })
}
