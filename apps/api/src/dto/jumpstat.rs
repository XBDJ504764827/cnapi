//! Phase 3 read contract for jumpstats.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct JumpstatSummary {
    pub id: i64,
    pub account_id: i64,
    pub map_id: i64,
    pub jump_type: String,
    /// Exact game units as a decimal string.
    pub distance_units: String,
    pub sync_percent: Option<String>,
    pub strafe_count: Option<i16>,
    pub tickrate: i16,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct JumpstatDetailResponse {
    pub summary: JumpstatSummary,
    pub mode: String,
    pub style: String,
    pub airtime_ns: Option<i64>,
    pub block: Option<i32>,
    pub is_block_jump: bool,
    pub msl_count: Option<i16>,
    /// Source-specific metrics that are not promoted to columns yet.
    pub raw_metrics: serde_json::Value,
}
