//! Public server contracts.
//!
//! The current schema does not store player counts, max slots or a server-level
//! mode; those fields are `null` instead of being invented.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::common::{PageMeta, PageParams, PageQuery, PlayerRef};
use crate::errors::AppError;

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ServerQuery {
    pub search: Option<String>,
    /// `true` = only servers with a fresh heartbeat.
    pub online: Option<bool>,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

impl ServerQuery {
    pub fn page_params(&self) -> Result<PageParams, AppError> {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
        }
        .resolve()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServerSummary {
    pub id: i64,
    pub name: String,
    /// Owner display name, used as the community label.
    pub community: Option<String>,
    pub ip: String,
    pub port: i32,
    /// Fresh heartbeat within the last 2 minutes.
    pub online: bool,
    /// `online` / `offline` / `expired` / `unknown`; derived from heartbeat age.
    pub status: String,
    /// Seconds since the last heartbeat; `null` when no heartbeat exists.
    pub heartbeat_age_seconds: Option<i64>,
    pub current_map_id: Option<i64>,
    pub current_map_name: Option<String>,
    pub current_map_slug: Option<String>,
    pub plugin_version: Option<String>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub players: Option<i64>,
    pub max_players: Option<i64>,
    pub mode: Option<String>,
    pub region: Option<String>,
    pub server_type: String,
    pub game: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServerListResponse {
    pub items: Vec<ServerSummary>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct OnlinePlayersQuery {
    /// Restrict to one server.
    pub server_id: Option<i64>,
    #[param(default = 100, minimum = 1, maximum = 200)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OnlinePlayerSummary {
    pub player: PlayerRef,
    pub server_id: i64,
    pub server_name: String,
    pub current_map_name: Option<String>,
    pub mode: Option<String>,
    pub status: String,
    pub connected_at: DateTime<Utc>,
    pub online_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OnlinePlayersResponse {
    pub items: Vec<OnlinePlayerSummary>,
    pub count: usize,
}
