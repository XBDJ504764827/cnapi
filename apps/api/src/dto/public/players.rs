//! Public player contracts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::common::{ModeRef, PageMeta, PageParams, PageQuery, PlayerRef, StyleRef};
use super::records::RecordSummary;
use crate::errors::AppError;

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PlayerListQuery {
    /// Display name or SteamID64 fragment.
    pub search: Option<String>,
    /// Optional mode filter (default `all`).
    pub mode: Option<String>,
    /// `completion` (default), `wr`, `records`, `recent`, `name`.
    pub sort: Option<String>,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

impl PlayerListQuery {
    pub fn page_params(&self) -> Result<PageParams, AppError> {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
        }
        .resolve()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerSummary {
    /// `null` when the player has no approved records for the selected mode.
    pub rank: Option<i64>,
    pub account_id: i64,
    pub steam_id64: String,
    pub display_name: String,
    /// Steam 头像（medium，64×64）。后台同步填充前为 `null`。
    pub avatar_url: Option<String>,
    pub country: Option<String>,
    pub completed_maps: i64,
    pub wr_count: i64,
    pub total_records: i64,
    pub last_activity_at: Option<DateTime<Utc>>,
    /// Always `null` until a points algorithm is defined.
    pub points: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerListResponse {
    /// `null` means all modes.
    pub mode: Option<ModeRef>,
    /// Metric used for ordering: `completed_maps`.
    pub ranking_metric: String,
    pub points_available: bool,
    pub items: Vec<PlayerSummary>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PlayerDetailQuery {
    /// Mode used for the whole detail page (default `vanilla`).
    pub mode: Option<String>,
    /// `all` (default), `nub`, `pro` — used for the whole detail page.
    pub time_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerProfileDto {
    pub account_id: i64,
    pub steam_id64: String,
    pub display_name: String,
    /// Steam 头像（medium，64×64）。后台同步填充前为 `null`。
    pub avatar_url: Option<String>,
    pub country: Option<String>,
    pub steam_profile_url: String,
    pub first_record_at: Option<DateTime<Utc>>,
    pub last_record_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerOverallStats {
    /// `true` = these numbers are scoped to `scope`; `false` = all-mode summary.
    pub scoped: bool,
    pub global_rank: Option<i64>,
    pub completed_maps: i64,
    pub total_maps: i64,
    pub completion_rate: f64,
    pub wr_count: i64,
    pub pro_count: i64,
    pub tp_count: i64,
    pub total_records: i64,
    /// Always `null` until the points algorithm exists.
    pub points: Option<i64>,
    pub rating: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerModeStatsDto {
    pub mode: ModeRef,
    pub rank: Option<i64>,
    pub completed_maps: i64,
    pub total_maps: i64,
    pub completion_rate: f64,
    pub wr_count: i64,
    pub pro_count: i64,
    pub tp_count: i64,
    pub total_records: i64,
    pub points: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TierProgressDto {
    /// `null` = unrated.
    pub tier: Option<i16>,
    pub map_count: i64,
    pub completed_count: i64,
    pub uncompleted_count: i64,
    pub completion_rate: f64,
    pub points: Option<i64>,
    pub average_points: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerStatScope {
    pub mode: ModeRef,
    /// Synthetic "all styles" reference; style is an optional drill-down, not
    /// part of the default ranking/completion scope.
    pub style: StyleRef,
    /// `all`, `nub` or `pro`.
    pub time_type: String,
    /// Human-readable definition of what "completed" means for this page.
    pub completion_definition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerDetailResponse {
    pub profile: PlayerProfileDto,
    /// All-mode/all-type summary, explicitly marked `scoped=false`.
    pub overall: PlayerOverallStats,
    /// Per-mode breakdown across all time types/styles.
    pub modes: Vec<PlayerModeStatsDto>,
    /// Tier completion counts for the selected `scope`.
    pub tiers: Vec<TierProgressDto>,
    /// Scope shared by `tiers`, map progress and the scoped card.
    pub scope: PlayerStatScope,
    /// Summary for the selected `scope` (`scoped=true`).
    pub scoped: PlayerOverallStats,
    pub ranking_metric: String,
    pub points_available: bool,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PlayerMapsQuery {
    /// Mode (default `vanilla`).
    pub mode: Option<String>,
    /// `pro` (default), `nub` or `all`.
    pub time_type: Option<String>,
    /// Style (default `normal`).
    pub style: Option<String>,
    /// `all` (default), `completed`, `uncompleted`.
    pub status: Option<String>,
    pub search: Option<String>,
    pub tier: Option<i16>,
    /// `tier` (default), `name`, `time`, `recent`.
    pub sort: Option<String>,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

impl PlayerMapsQuery {
    pub fn page_params(&self) -> Result<PageParams, AppError> {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
        }
        .resolve()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerMapProgress {
    pub map_id: i64,
    pub name: String,
    pub slug: String,
    pub tier: Option<i16>,
    pub completed: bool,
    pub best_time_ns: Option<i64>,
    pub best_time_seconds: Option<String>,
    pub best_rank: Option<i64>,
    pub best_time_type: Option<String>,
    /// Upload time of the best record (wall-clock completion time is not stored).
    pub last_record_at: Option<DateTime<Utc>>,
    /// Number of bonus courses on the map.
    pub bonus_total: i64,
    /// Bonus courses completed by the player (same mode/time_type/style).
    pub bonus_completed: i64,
    pub points: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerMapsResponse {
    pub account: PlayerRef,
    pub mode: ModeRef,
    pub style: StyleRef,
    pub time_type: String,
    pub items: Vec<PlayerMapProgress>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PlayerRecordsQuery {
    pub mode: Option<String>,
    /// `all` (default), `nub`, `pro`.
    pub time_type: Option<String>,
    pub style: Option<String>,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

impl PlayerRecordsQuery {
    pub fn page_params(&self) -> Result<PageParams, AppError> {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
        }
        .resolve()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerRecordsResponse {
    pub account: PlayerRef,
    pub items: Vec<RecordSummary>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct CompareQuery {
    /// SteamID64 of player A.
    pub left: String,
    /// SteamID64 of player B.
    pub right: String,
    pub mode: Option<String>,
    /// `all` (default), `nub`, `pro`.
    pub time_type: Option<String>,
    pub style: Option<String>,
    /// `all` (default), `both`, `left_only`, `right_only`, `neither`,
    /// `left_faster`, `right_faster`.
    pub filter: Option<String>,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

impl CompareQuery {
    pub fn page_params(&self) -> Result<PageParams, AppError> {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
        }
        .resolve()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompareBest {
    pub completed: bool,
    pub time_ns: Option<i64>,
    pub time_seconds: Option<String>,
    pub rank: Option<i64>,
    pub points: Option<i64>,
    /// Upload time of the best record (not the run duration).
    pub last_record_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ComparePlayerSide {
    pub profile: PlayerProfileDto,
    pub overall: PlayerOverallStats,
    pub modes: Vec<PlayerModeStatsDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompareTier {
    pub tier: Option<i16>,
    pub total_maps: i64,
    pub left_completed: i64,
    pub right_completed: i64,
    pub left_points: Option<i64>,
    pub right_points: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompareMapRow {
    pub map_id: i64,
    pub name: String,
    pub slug: String,
    pub tier: Option<i16>,
    pub left: CompareBest,
    pub right: CompareBest,
    /// `left_null`, `both_completed`, `left_only`, `right_only`, `neither`.
    pub status: String,
    /// Positive = player A faster, negative = player B faster.
    pub time_diff_ns: Option<i64>,
    pub left_points_diff: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompareResponse {
    pub left: ComparePlayerSide,
    pub right: ComparePlayerSide,
    pub mode: ModeRef,
    pub time_type: String,
    pub tiers: Vec<CompareTier>,
    pub filter: String,
    pub maps: Vec<CompareMapRow>,
    pub meta: PageMeta,
    pub ranking_metric: String,
    pub points_available: bool,
}
