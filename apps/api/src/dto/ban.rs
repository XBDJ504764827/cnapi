//! Admin ban console contracts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BanSummary {
    pub id: i64,
    pub account_id: i64,
    pub steam_id64: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub ban_type: String,
    pub reason: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    /// `active` | `expired` | `revoked`（expired 由 expires_at 推导）。
    pub status: String,
    pub source: String,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoke_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BanListResponse {
    pub items: Vec<BanSummary>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Deserialize, utoipa::IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct BanListQuery {
    pub search: Option<String>,
    /// active | expired | revoked | all
    pub status: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateBanRequest {
    /// SteamID64（17–20 位数字）。
    pub steam_id64: String,
    pub ban_type: Option<String>,
    pub reason: String,
    /// RFC3339；留空 = 永久封禁。
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BanMutationResponse {
    pub ban_id: i64,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RevokeBanRequest {
    pub reason: String,
}
