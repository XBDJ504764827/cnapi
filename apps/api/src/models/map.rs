//! Map identity, versions, courses and per-mode tiers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Map review state (`maps.status`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum MapStatus {
    InTesting,
    Approved,
    Disabled,
}

/// Course ranking state (`course_tiers.status`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum CourseTierStatus {
    Unranked,
    Pending,
    Ranked,
}

#[derive(Debug, Clone, FromRow)]
pub struct Map {
    pub id: i64,
    pub name: String,
    pub workshop_id: Option<i64>,
    pub description: Option<String>,
    pub status: MapStatus,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by_account_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct MapVersion {
    pub id: i64,
    pub map_id: i64,
    pub version_label: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub checksum: Option<Vec<u8>>,
    pub checksum_algorithm: Option<String>,
    pub is_current: bool,
    pub created_by_account_id: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct MapCourse {
    pub id: i64,
    pub map_id: i64,
    /// 0 = main course, 1..99 = bonus stages.
    pub stage: i16,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CourseTier {
    pub course_id: i64,
    pub mode_id: i16,
    pub nub_tier: Option<i16>,
    pub pro_tier: Option<i16>,
    pub status: CourseTierStatus,
    pub notes: Option<String>,
    pub reviewed_by_account_id: Option<i64>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
