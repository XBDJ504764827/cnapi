//! Public map contracts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::common::{CourseRef, MapRef, ModeRef, PageMeta, PageParams, PageQuery, StyleRef};
use super::records::RecordSummary;
use crate::errors::AppError;

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct MapListQuery {
    pub search: Option<String>,
    /// Mode (default `vanilla`).
    pub mode: Option<String>,
    /// `nub` (default) or `pro` — selects which tier column is used.
    pub time_type: Option<String>,
    /// Tier 1-10; `0` = unrated.
    pub tier: Option<i16>,
    /// `completions` (default), `recent`, `name`, `records`.
    pub sort: Option<String>,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

impl MapListQuery {
    pub fn page_params(&self) -> Result<PageParams, AppError> {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
        }
        .resolve()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MapSummary {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub workshop_id: Option<i64>,
    pub description: Option<String>,
    /// Tier for the requested mode/time type; `null` = unrated.
    pub tier: Option<i16>,
    pub courses: i64,
    pub completed_players: i64,
    pub records_count: i64,
    pub wr_time_ns: Option<i64>,
    pub wr_time_seconds: Option<String>,
    pub wr_player_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MapListResponse {
    pub mode: ModeRef,
    pub time_type: String,
    pub items: Vec<MapSummary>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MapCourseDto {
    pub id: i64,
    pub stage: i16,
    pub name: String,
    pub nub_tier: Option<i16>,
    pub pro_tier: Option<i16>,
    pub completed_players: i64,
    pub records_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModeCount {
    pub mode: ModeRef,
    pub records: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TimeTypeCount {
    pub time_type: String,
    pub records: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MapStatistics {
    /// Distinct players with any approved record on the map.
    pub total_players: i64,
    /// Distinct players with a main-course record for the selected mode.
    pub completed_players: i64,
    pub pro_completed_players: i64,
    pub nub_completed_players: i64,
    /// completed_players / total registered players.
    pub completion_rate: f64,
    pub record_count: i64,
    pub records_by_mode: Vec<ModeCount>,
    pub records_by_time_type: Vec<TimeTypeCount>,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct MapDetailQuery {
    /// Mode (default `vanilla`).
    pub mode: Option<String>,
    /// `all` (default), `nub` or `pro`.
    pub time_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MapDetailResponse {
    pub map: MapSummary,
    pub supported_modes: Vec<ModeRef>,
    pub courses: Vec<MapCourseDto>,
    pub statistics: MapStatistics,
    /// Always `null` — the schema has no mapper table yet.
    pub authors: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct MapLeaderboardQuery {
    pub mode: Option<String>,
    /// `pro` (default), `nub`, `all`.
    pub time_type: Option<String>,
    /// Course stage; default `0` (main course).
    pub stage: Option<i16>,
    pub style: Option<String>,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

impl MapLeaderboardQuery {
    pub fn page_params(&self) -> Result<PageParams, AppError> {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
        }
        .resolve()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MapLeaderboardResponse {
    pub map: MapRef,
    pub course: CourseRef,
    pub mode: ModeRef,
    pub style: StyleRef,
    pub time_type: String,
    pub items: Vec<RecordSummary>,
    pub meta: PageMeta,
}
