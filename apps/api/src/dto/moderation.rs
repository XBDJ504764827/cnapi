//! Phase 3 read contracts for moderation.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::models::moderation::BanSource;
use crate::models::moderation::BanStatus;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BanSummary {
    pub id: i64,
    pub account_id: i64,
    pub ban_type: String,
    pub reason: String,
    pub issued_at: DateTime<Utc>,
    /// `None` = permanent ban.
    pub expires_at: Option<DateTime<Utc>>,
    pub status: BanStatus,
    pub source: BanSource,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BanRevocationResponse {
    pub id: i64,
    pub ban_id: i64,
    pub reason: String,
    pub revoked_at: DateTime<Utc>,
}
