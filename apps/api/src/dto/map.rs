//! Phase 3 read contracts for maps.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::models::MapStatus;
use crate::models::map::CourseTierStatus;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MapSummary {
    pub id: i64,
    pub name: String,
    pub workshop_id: Option<i64>,
    pub status: MapStatus,
    pub approved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CourseSummary {
    pub id: i64,
    /// 0 = main course, >0 = bonus stage.
    pub stage: i16,
    pub name: String,
    pub tiers: Vec<CourseTierSummary>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CourseTierSummary {
    pub mode: String,
    pub nub_tier: Option<i16>,
    pub pro_tier: Option<i16>,
    pub status: CourseTierStatus,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MapDetailResponse {
    pub map: MapSummary,
    pub description: Option<String>,
    pub courses: Vec<CourseSummary>,
    pub supported_modes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MapVersionResponse {
    pub id: i64,
    pub version_label: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub checksum_algorithm: Option<String>,
    pub checksum_hex: Option<String>,
    pub is_current: bool,
    pub created_at: DateTime<Utc>,
}
