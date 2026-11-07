//! Public records contracts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::common::{CourseRef, MapRef, ModeRef, PageParams, PageQuery, PlayerRef, StyleRef};
use crate::errors::AppError;
use crate::models::TimeType;

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct RecentRecordsQuery {
    /// Number of records (1-50, default 20).
    #[param(default = 20, minimum = 1, maximum = 50)]
    pub limit: Option<u32>,
    /// Optional movement mode filter.
    pub mode: Option<String>,
}

impl RecentRecordsQuery {
    pub fn resolved_limit(&self) -> Result<u32, AppError> {
        let limit = self.limit.unwrap_or(20);
        if !(1..=50).contains(&limit) {
            return Err(AppError::validation("`limit` must be between 1 and 50"));
        }
        Ok(limit)
    }
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct RecordQuery {
    /// SteamID64 or display name fragment.
    pub player: Option<String>,
    /// Map name fragment.
    pub map: Option<String>,
    pub mode: Option<String>,
    pub style: Option<String>,
    /// `all` (default), `nub` or `pro`.
    pub time_type: Option<String>,
    /// Map tier (1-10) for the selected mode/time type.
    pub tier: Option<i16>,
    /// `true` = only current world records, `false` = only non-WR.
    pub world_record: Option<bool>,
    /// RFC3339 lower bound on upload time.
    pub from: Option<String>,
    /// RFC3339 upper bound on upload time.
    pub to: Option<String>,
    /// `uploaded` (default), `time`, `rank`.
    pub sort: Option<String>,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

impl RecordQuery {
    pub fn page_params(&self) -> Result<PageParams, AppError> {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
        }
        .resolve()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecordSummary {
    pub id: i64,
    pub player: PlayerRef,
    pub map: MapRef,
    pub course: CourseRef,
    pub mode: ModeRef,
    pub style: StyleRef,
    /// Map tier for the record's time type; `null` = unrated.
    pub tier: Option<i16>,
    pub time_type: TimeType,
    pub teleports: i32,
    /// Run duration as integer nanoseconds (ordering field).
    pub time_ns: i64,
    /// Run duration as decimal seconds with 3 fraction digits.
    pub time_seconds: String,
    /// Position on the course/mode/style/time-type leaderboard.
    pub rank: i64,
    /// Current world record pointer for this dimension.
    pub is_world_record: bool,
    pub replay_available: bool,
    /// 每条成绩对应的积分：该成绩是 PB 时取所属 Filter 的积分，否则为 0。
    pub points: Option<i64>,
    /// Upload time (not the run duration).
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecordListResponse {
    pub items: Vec<RecordSummary>,
    pub meta: super::common::PageMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecentRecordsResponse {
    pub items: Vec<RecordSummary>,
    pub count: usize,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecordDetailResponse {
    pub record: RecordSummary,
    pub server: Option<ServerRef>,
    pub replay_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServerRef {
    pub id: i64,
    pub name: String,
}

/// A current world record with optional historical improvement fields.
///
/// `previous_time_ns` / `improvement_ns` are always `null` in phase 4 because the
/// schema does not store WR transition history.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WorldRecordEntry {
    pub record: RecordSummary,
    pub achieved_at: DateTime<Utc>,
    pub previous_time_ns: Option<i64>,
    pub improvement_ns: Option<i64>,
    pub improvement_seconds: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct RecentWorldRecordsQuery {
    #[param(default = 10, minimum = 1, maximum = 50)]
    pub limit: Option<u32>,
    pub mode: Option<String>,
    /// `all` (default), `nub` or `pro`.
    pub time_type: Option<String>,
}

impl RecentWorldRecordsQuery {
    pub fn resolved_limit(&self) -> Result<u32, AppError> {
        let limit = self.limit.unwrap_or(10);
        if !(1..=50).contains(&limit) {
            return Err(AppError::validation("`limit` must be between 1 and 50"));
        }
        Ok(limit)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ActivePlayer {
    pub player: PlayerRef,
    pub records: i64,
    pub last_record_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ActivePlayersQuery {
    /// Lookback window in days (1-30, default 7).
    #[param(default = 7, minimum = 1, maximum = 30)]
    pub days: Option<u32>,
    #[param(default = 5, minimum = 1, maximum = 20)]
    pub limit: Option<u32>,
}

impl ActivePlayersQuery {
    pub fn resolve(&self) -> Result<(u32, u32), AppError> {
        let days = self.days.unwrap_or(7);
        let limit = self.limit.unwrap_or(5);
        if !(1..=30).contains(&days) {
            return Err(AppError::validation("`days` must be between 1 and 30"));
        }
        if !(1..=20).contains(&limit) {
            return Err(AppError::validation("`limit` must be between 1 and 20"));
        }
        Ok((days, limit))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ActivePlayersResponse {
    pub items: Vec<ActivePlayer>,
    pub days: u32,
}
