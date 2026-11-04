//! Registered game servers.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use sqlx::types::ipnetwork::IpNetwork;

#[derive(Debug, Clone, FromRow)]
pub struct Server {
    pub id: i64,
    pub name: String,
    pub ip: IpNetwork,
    pub port: i32,
    pub owner_account_id: i64,
    /// Hash of the API key; never serialized.
    pub api_key_hash: Vec<u8>,
    pub api_key_hash_algorithm: String,
    pub api_key_prefix: String,
    pub api_key_created_at: DateTime<Utc>,
    pub api_key_revoked_at: Option<DateTime<Utc>>,
    pub plugin_version: Option<String>,
    pub game: String,
    pub region: Option<String>,
    pub server_type: String,
    pub query_port: Option<i32>,
    pub max_players: i32,
    pub online_players: i32,
    pub current_map_id: Option<i64>,
    pub current_mode_id: Option<i16>,
    pub game_version: Option<String>,
    pub is_enabled: bool,
    pub is_online: bool,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
