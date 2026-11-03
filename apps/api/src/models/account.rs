//! Accounts, player profiles, derived stats and linked identities.

use chrono::{DateTime, Utc};
use sqlx::types::BigDecimal;
use sqlx::types::ipnetwork::IpNetwork;

/// Canonical Steam account identity.
///
/// `steam_id64` uses [`BigDecimal`] because the database column is
/// `NUMERIC(20,0)`: a SteamID64 can be larger than `i64::MAX`.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Account {
    pub id: i64,
    pub steam_id64: BigDecimal,
    pub steam_id2: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Account {
    /// SteamID64 as a decimal string, matching the GlobalAPI JSON representation.
    pub fn steam_id64_string(&self) -> String {
        self.steam_id64.to_string()
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PlayerProfile {
    pub account_id: i64,
    pub display_name: String,
    pub country: Option<String>,
    pub last_ip: Option<IpNetwork>,
    pub preferences: serde_json::Value,
    pub prime_verified: bool,
    pub first_joined_at: DateTime<Utc>,
    pub last_joined_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Derived, rebuildable aggregates per movement mode.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PlayerModeStats {
    pub account_id: i64,
    pub mode_id: i16,
    pub points: i32,
    pub points_overall: i32,
    pub rating: BigDecimal,
    pub average: BigDecimal,
    pub finishes: i32,
    pub total_records: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PlayerIdentity {
    pub id: i64,
    pub account_id: i64,
    pub provider: String,
    pub identifier: String,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr as _;

    #[test]
    fn steam_id64_survives_the_uint64_range() {
        let max = BigDecimal::from_str("18446744073709551615").expect("valid decimal");
        let account = Account {
            id: 1,
            steam_id64: max,
            steam_id2: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        assert_eq!(account.steam_id64_string(), "18446744073709551615");
    }
}
