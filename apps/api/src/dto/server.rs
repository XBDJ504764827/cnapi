//! Phase 3 read contract for registered servers.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ServerSummary {
    pub id: i64,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub plugin_version: Option<String>,
    pub current_map_id: Option<i64>,
    pub is_online: bool,
    /// Readers must treat a stale heartbeat as offline; see `docs/database/servers.md`.
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
}
