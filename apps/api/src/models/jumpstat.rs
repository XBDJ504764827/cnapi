//! Jumpstat submissions.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use sqlx::types::BigDecimal;
use uuid::Uuid;

use crate::models::IngestionSource;

#[derive(Debug, Clone, FromRow)]
pub struct Jumpstat {
    pub id: i64,
    pub account_id: i64,
    pub map_id: i64,
    pub server_id: Option<i64>,
    pub mode_id: i16,
    pub style_id: i16,
    pub jump_type_id: i16,
    /// Jump distance in game units (exact decimal).
    pub distance_units: BigDecimal,
    pub strafe_count: Option<i16>,
    /// 0-100; only populated when the source reports it as a percentage.
    pub sync_percent: Option<BigDecimal>,
    pub airtime_ns: Option<i64>,
    pub block: Option<i32>,
    pub is_block_jump: bool,
    pub msl_count: Option<i16>,
    pub is_crouch_bind: bool,
    pub is_forward_bind: bool,
    pub is_crouch_boost: bool,
    pub tickrate: i16,
    pub source: IngestionSource,
    pub submission_id: Option<Uuid>,
    pub external_ref: Option<String>,
    /// Verbatim source metrics that are not promoted to columns yet.
    pub raw_metrics: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
