//! Bans and ban revocations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::ipnetwork::IpNetwork;
use utoipa::ToSchema;

/// Stored ban state. `expired` is derived from `expires_at` and never stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum BanStatus {
    Active,
    Revoked,
}

/// Origin of the ban decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum BanSource {
    Admin,
    Server,
    GlobalapiImport,
}

#[derive(Debug, Clone, FromRow)]
pub struct Ban {
    pub id: i64,
    pub account_id: i64,
    pub ip: Option<IpNetwork>,
    pub ban_type: String,
    pub reason: String,
    pub stats: Option<String>,
    pub issued_by_account_id: Option<i64>,
    pub issued_by_server_id: Option<i64>,
    pub issued_at: DateTime<Utc>,
    /// `None` means permanent.
    pub expires_at: Option<DateTime<Utc>>,
    pub status: BanStatus,
    pub source: BanSource,
    pub external_ref: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Ban {
    /// Whether the ban still applies at `now` (revocation and expiry included).
    pub fn is_effective_at(&self, now: DateTime<Utc>) -> bool {
        self.status == BanStatus::Active
            && self.expires_at.is_none_or(|expires_at| expires_at > now)
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct BanRevocation {
    pub id: i64,
    pub ban_id: i64,
    pub revoked_by_account_id: Option<i64>,
    pub revoked_by_server_id: Option<i64>,
    pub reason: String,
    pub revoked_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn ban(status: BanStatus, expires_at: Option<DateTime<Utc>>) -> Ban {
        let now = Utc::now();
        Ban {
            id: 1,
            account_id: 1,
            ip: None,
            ban_type: "global".to_owned(),
            reason: "test".to_owned(),
            stats: None,
            issued_by_account_id: Some(1),
            issued_by_server_id: None,
            issued_at: now,
            expires_at,
            status,
            source: BanSource::Admin,
            external_ref: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn permanent_active_bans_are_effective() {
        assert!(ban(BanStatus::Active, None).is_effective_at(Utc::now()));
    }

    #[test]
    fn expired_and_revoked_bans_are_not_effective() {
        let now = Utc::now();
        assert!(!ban(BanStatus::Active, Some(now - Duration::seconds(1))).is_effective_at(now));
        assert!(ban(BanStatus::Active, Some(now + Duration::seconds(1))).is_effective_at(now));
        assert!(!ban(BanStatus::Revoked, None).is_effective_at(now));
    }
}
