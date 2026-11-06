//! Public leaderboard contracts.
//!
//! Points are intentionally `null`: CNGOKZAPI has no points algorithm yet, so
//! ranking uses the documented `completed_maps` metric instead of inventing
//! numbers. See `docs/public-portal/api-requirements.md`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::common::{ModeRef, PageMeta, PageParams, PageQuery};
use crate::errors::AppError;

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct LeaderboardQuery {
    /// Movement mode code (e.g. `vanilla`). Defaults to `vanilla`.
    pub mode: Option<String>,
    /// `all` (default), `nub` or `pro`.
    pub time_type: Option<String>,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

impl LeaderboardQuery {
    pub fn page_params(&self) -> Result<PageParams, AppError> {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
        }
        .resolve()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub account_id: i64,
    pub steam_id64: String,
    pub display_name: String,
    pub country: Option<String>,
    pub completed_maps: i64,
    pub wr_count: i64,
    pub total_records: i64,
    pub last_activity_at: Option<DateTime<Utc>>,
    /// Always `null` until a points algorithm is defined.
    pub points: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LeaderboardResponse {
    pub mode: ModeRef,
    /// `all`, `nub` or `pro`.
    pub time_type: String,
    /// Metric used for ordering: `completed_maps`.
    pub ranking_metric: String,
    pub points_available: bool,
    pub items: Vec<LeaderboardEntry>,
    pub meta: PageMeta,
}
