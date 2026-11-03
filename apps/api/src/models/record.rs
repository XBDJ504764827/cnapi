//! Runs, world-record pointers and the record audit trail.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::models::IngestionSource;
use crate::models::movement::TimeType;

/// Review state of a run (`records.status`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum RecordStatus {
    Pending,
    Approved,
    Rejected,
}

/// What changed in the audit trail (`record_history.action`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum RecordHistoryAction {
    Created,
    StatusChanged,
    TimeCorrected,
    Reviewed,
    SoftDeleted,
    Restored,
    Other,
}

#[derive(Debug, Clone, FromRow)]
pub struct Record {
    pub id: i64,
    pub account_id: i64,
    pub map_id: i64,
    pub course_id: i64,
    pub mode_id: i16,
    pub style_id: i16,
    pub server_id: Option<i64>,
    pub server_plugin_version: Option<String>,
    pub tickrate: i16,
    pub teleports: i32,
    pub time_type: TimeType,
    /// Run duration in nanoseconds (exact integer; sources send float seconds).
    pub time_ns: i64,
    pub status: RecordStatus,
    pub source: IngestionSource,
    pub submission_id: Option<Uuid>,
    pub external_ref: Option<String>,
    pub reviewed_by_account_id: Option<i64>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Record {
    /// Duration in microseconds, useful for display and legacy conversion.
    pub const fn time_us(&self) -> i64 {
        self.time_ns / 1_000
    }

    /// Duration as a decimal string with nanosecond precision (`"12.345678901"`).
    ///
    /// A string is used instead of `f64` so clients cannot lose precision and so
    /// the JSON contract does not depend on platform float formatting.
    pub fn time_seconds_string(&self) -> String {
        let seconds = self.time_ns / 1_000_000_000;
        let nanos = self.time_ns.unsigned_abs() % 1_000_000_000;
        format!("{seconds}.{nanos:09}")
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct RecordHistory {
    pub id: i64,
    pub record_id: i64,
    pub action: RecordHistoryAction,
    pub old_status: Option<RecordStatus>,
    pub new_status: Option<RecordStatus>,
    pub old_time_ns: Option<i64>,
    pub new_time_ns: Option<i64>,
    pub changed_by_account_id: Option<i64>,
    pub reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CourseWorldRecord {
    pub course_id: i64,
    pub mode_id: i16,
    pub style_id: i16,
    pub time_type: TimeType,
    pub record_id: i64,
    pub time_ns: i64,
    pub achieved_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record_with_time(time_ns: i64) -> Record {
        Record {
            id: 1,
            account_id: 1,
            map_id: 1,
            course_id: 1,
            mode_id: 1,
            style_id: 1,
            server_id: None,
            server_plugin_version: None,
            tickrate: 128,
            teleports: 0,
            time_type: TimeType::Pro,
            time_ns,
            status: RecordStatus::Approved,
            source: IngestionSource::Plugin,
            submission_id: None,
            external_ref: None,
            reviewed_by_account_id: None,
            reviewed_at: None,
            review_note: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    #[test]
    fn formats_seconds_with_nanosecond_precision() {
        assert_eq!(
            record_with_time(1_000_000_000).time_seconds_string(),
            "1.000000000"
        );
        assert_eq!(
            record_with_time(7_812_500).time_seconds_string(),
            "0.007812500"
        );
        assert_eq!(
            record_with_time(12_345_678_901).time_seconds_string(),
            "12.345678901"
        );
    }

    #[test]
    fn exposes_exact_microseconds() {
        assert_eq!(record_with_time(12_345_678_901).time_us(), 12_345_678);
    }
}
