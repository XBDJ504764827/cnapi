//! Phase 7 contracts for the CS:GO game-server integration.
//!
//! These endpoints are consumed by `cngokz_sync` (SourceMod) running on a
//! CS:GO / LinuxGSM server. The server never decides points, PB or WR; it only
//! reports facts and the API applies the phase-6 scoring rules.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::dto::public::common::{PageMeta, PageParams, PageQuery};
use crate::errors::AppError;

// ---------------------------------------------------------------------------
// Server authentication / heartbeat
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct HeartbeatRequest {
    pub server_name: Option<String>,
    /// BSP name without `.bsp`, e.g. `kz_example`.
    pub map: Option<String>,
    /// GOKZ / scoring mode: `KZT`, `SKZ`, `VNL` (or `kztimer` / `simple` / `vanilla`).
    pub mode: Option<String>,
    pub online_players: Option<i64>,
    pub max_players: Option<i64>,
    pub server_port: Option<i32>,
    pub game_version: Option<String>,
    pub plugin_version: Option<String>,
    /// Plugin clock; the API records its own receive time as the source of truth.
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HeartbeatResponse {
    pub server_id: i64,
    pub status: String,
    pub received_at: DateTime<Utc>,
    pub heartbeat_interval_seconds: i64,
    pub config_version: i64,
}

// ---------------------------------------------------------------------------
// Online players
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct PlayerEventRequest {
    pub steam_id64: String,
    pub display_name: String,
    pub map: Option<String>,
    pub mode: Option<String>,
    pub connected_at: Option<DateTime<Utc>>,
    /// `connected`, `spectating` or `afk`; default `connected`.
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct PlayerDisconnectRequest {
    pub steam_id64: String,
    pub disconnected_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct PlayerSyncRequest {
    /// Full snapshot of the players currently on this server. The API marks
    /// missing sessions disconnected only for this server.
    pub players: Vec<PlayerEventRequest>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PlayerSyncResponse {
    pub connected: i64,
    pub disconnected: i64,
    pub online: i64,
}

// ---------------------------------------------------------------------------
// Record submission / status
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RecordSubmissionRequest {
    pub steam_id64: String,
    pub player_name: Option<String>,
    pub map: String,
    pub course: i16,
    /// `KZT`, `SKZ`, `VNL` (case-insensitive).
    pub mode: String,
    /// Defaults to `normal`; GOKZ only has Style_Normal.
    pub style: Option<String>,
    /// Run duration in seconds. Exactly one of `time` / `time_ns` is required.
    pub time: Option<f64>,
    /// Run duration in nanoseconds (preferred; no float conversion).
    pub time_ns: Option<i64>,
    pub tickrate: i16,
    pub teleports: i32,
    /// SourceMod plugin version reporting this run (optional).
    pub plugin_version: Option<String>,
    /// Stable plugin-side idempotency key (unique per server).
    pub record_identifier: String,
    pub created_at: Option<DateTime<Utc>>,
    /// Optional path reported by GOKZ replays; stored only in audit metadata.
    pub replay_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RecordSubmissionResponse {
    pub accepted: bool,
    pub duplicate: bool,
    pub record_id: i64,
    pub record_identifier: String,
    pub time_ns: i64,
    pub time_seconds: String,
    pub time_type: String,
    pub is_pb: bool,
    pub is_world_record: bool,
    /// `queued` when the database trigger enqueued a Filter recalculation.
    pub points_status: String,
    /// `pending` / `available` / `failed` / `not_reported`.
    pub replay_status: String,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct RecordStatusQuery {
    pub record_identifier: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RecordStatusResponse {
    pub record_id: i64,
    pub record_identifier: String,
    pub status: String,
    pub time_ns: i64,
    pub time_seconds: String,
    pub is_pb: bool,
    pub is_world_record: bool,
    pub replay_status: String,
    pub replay_id: Option<i64>,
    pub points_status: String,
}

// ---------------------------------------------------------------------------
// Replay metadata
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ReplayRegistrationRequest {
    pub record_identifier: String,
    pub bucket: String,
    pub object_key: String,
    /// `r2`, `s3` or `local`; default `r2`.
    pub storage_provider: Option<String>,
    pub size_bytes: i64,
    /// Lowercase or uppercase hex SHA-256 (64 characters).
    pub checksum_sha256: String,
    pub replay_format_version: Option<i16>,
    /// `pending`, `available` or `failed`; default `available`.
    pub status: Option<String>,
    pub uploaded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ReplayRegistrationResponse {
    pub id: i64,
    pub record_id: i64,
    pub status: String,
    pub replay_available: bool,
    pub duplicate: bool,
}

// ---------------------------------------------------------------------------
// Config sync
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ConfigSyncQuery {
    /// Version returned by the previous sync; omit for a full snapshot.
    pub version: Option<i64>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct ConfigMode {
    pub id: i16,
    pub code: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct ConfigStyle {
    pub id: i16,
    pub code: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct ConfigCourse {
    pub id: i64,
    pub stage: i16,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ConfigTier {
    pub course_id: i64,
    /// Scoring mode code: KZT, SKZ or VNL.
    pub mode: String,
    pub tier: i16,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ConfigMap {
    pub id: i64,
    pub name: String,
    pub status: String,
    pub modes: Vec<String>,
    pub courses: Vec<ConfigCourse>,
    pub tiers: Vec<ConfigTier>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ConfigSyncResponse {
    pub version: i64,
    pub changed: bool,
    pub generated_at: DateTime<Utc>,
    pub modes: Vec<ConfigMode>,
    pub styles: Vec<ConfigStyle>,
    pub maps: Vec<ConfigMap>,
}

// ---------------------------------------------------------------------------
// Admin API
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct AdminServerQuery {
    pub search: Option<String>,
    pub enabled: Option<bool>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl AdminServerQuery {
    pub fn page_params(&self) -> Result<PageParams, AppError> {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
        }
        .resolve()
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateServerRequest {
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub query_port: Option<i32>,
    pub region: Option<String>,
    /// `community`, `official`, `event` or `testing`.
    pub server_type: Option<String>,
    pub max_players: Option<i32>,
    /// Account that owns the server. Created if it does not exist yet.
    pub owner_steam_id64: String,
    pub owner_display_name: Option<String>,
    pub plugin_version: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub region: Option<String>,
    pub server_type: Option<String>,
    pub max_players: Option<i32>,
    pub query_port: Option<i32>,
    pub plugin_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminServerSummary {
    pub id: i64,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub query_port: Option<i32>,
    pub game: String,
    pub region: Option<String>,
    pub server_type: String,
    pub max_players: i32,
    pub online_players: i32,
    pub current_map_id: Option<i64>,
    pub current_map_name: Option<String>,
    pub current_mode: Option<String>,
    pub status: String,
    pub is_enabled: bool,
    pub is_online: bool,
    pub plugin_version: Option<String>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub heartbeat_age_seconds: Option<i64>,
    pub api_key_prefix: String,
    pub api_key_created_at: DateTime<Utc>,
    pub api_key_revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminServerListResponse {
    pub items: Vec<AdminServerSummary>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CreateServerResponse {
    pub server: AdminServerSummary,
    /// Returned exactly once. Store it on the CS:GO server and never commit it.
    pub token: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RotateTokenResponse {
    pub server_id: i64,
    pub token: String,
    pub prefix: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct AdminHeartbeatRow {
    pub id: i64,
    pub received_at: DateTime<Utc>,
    pub map_name: Option<String>,
    pub mode_code: Option<String>,
    pub online_players: i32,
    pub max_players: i32,
    pub plugin_version: Option<String>,
    pub game_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct AdminPlayerRow {
    pub account_id: i64,
    pub steam_id64: String,
    pub display_name: String,
    pub status: String,
    pub connected_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct AdminRecordRow {
    pub id: i64,
    pub account_id: i64,
    pub steam_id64: String,
    pub display_name: Option<String>,
    pub map_name: String,
    pub course_stage: i16,
    pub mode_code: String,
    pub time_type: String,
    pub time_ns: i64,
    pub time_seconds: String,
    pub status: String,
    pub is_world_record: bool,
    pub replay_status: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct AdminReplayRow {
    pub id: i64,
    pub record_id: i64,
    pub bucket: String,
    pub object_key: String,
    pub size_bytes: i64,
    pub status: String,
    pub uploaded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct AdminSyncLogRow {
    pub id: i64,
    pub kind: String,
    pub status: String,
    pub message: Option<String>,
    pub record_id: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct AdminSyncTaskRow {
    pub id: i64,
    pub record_id: Option<i64>,
    pub task_type: String,
    pub status: String,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AdminServerDetailResponse {
    pub server: AdminServerSummary,
    pub recent_heartbeats: Vec<AdminHeartbeatRow>,
    pub online_players: Vec<AdminPlayerRow>,
    pub recent_records: Vec<AdminRecordRow>,
    pub recent_replays: Vec<AdminReplayRow>,
    pub sync_logs: Vec<AdminSyncLogRow>,
    pub sync_tasks: Vec<AdminSyncTaskRow>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ServerActionResponse {
    pub server_id: i64,
    pub enabled: bool,
    pub token_revoked: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RetryTaskResponse {
    pub task_id: i64,
    pub status: String,
    pub message: String,
}
