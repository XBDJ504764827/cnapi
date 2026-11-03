//! Phase 3 read contracts for records and leaderboards.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::models::movement::TimeType;
use crate::models::record::RecordStatus;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RecordSummary {
    pub id: i64,
    pub account_id: i64,
    pub map_id: i64,
    pub course_id: i64,
    /// Movement mode code, e.g. `vanilla`.
    pub mode: String,
    /// Movement style code, currently `normal`.
    pub style: String,
    pub teleports: i32,
    pub time_type: TimeType,
    /// Exact duration in nanoseconds (primary ordering field).
    pub time_ns: i64,
    /// Duration as a decimal string with nanosecond precision.
    pub time_seconds: String,
    pub status: RecordStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct WorldRecordResponse {
    pub course_id: i64,
    pub stage: i16,
    pub mode: String,
    pub style: String,
    pub time_type: TimeType,
    pub time_ns: i64,
    pub time_seconds: String,
    pub player: String,
    pub achieved_at: DateTime<Utc>,
}

/// Append-only audit entry exposed to moderators.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RecordHistoryResponse {
    pub id: i64,
    pub record_id: i64,
    pub action: String,
    pub old_status: Option<RecordStatus>,
    pub new_status: Option<RecordStatus>,
    pub old_time_ns: Option<i64>,
    pub new_time_ns: Option<i64>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}
