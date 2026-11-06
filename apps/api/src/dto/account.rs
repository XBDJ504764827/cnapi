//! Phase 3 read contracts for accounts.
//!
//! These types are not wired into HTTP handlers yet; they exist so the shape of
//! the upcoming read API is reviewed together with the schema. They are added to
//! the OpenAPI document when the corresponding endpoints are implemented.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AccountSummary {
    pub id: i64,
    /// Decimal string: JSON numbers cannot safely represent every SteamID64.
    pub steam_id64: String,
    pub display_name: String,
    pub country: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PlayerModeStatsResponse {
    /// Movement mode code, e.g. `vanilla`.
    pub mode: String,
    pub points: i32,
    pub points_overall: i32,
    /// Decimal strings to avoid float precision loss in JSON.
    pub rating: String,
    pub average: String,
    pub finishes: i32,
    pub total_records: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PlayerProfileResponse {
    pub account: AccountSummary,
    pub prime_verified: bool,
    pub first_joined_at: DateTime<Utc>,
    pub last_joined_at: DateTime<Utc>,
    pub stats: Vec<PlayerModeStatsResponse>,
}
