//! Movement modes, movement styles and jump types (reference data).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Movement mode / ruleset, e.g. `vanilla`, `simple`, `kztimer`, `classic`.
#[derive(Debug, Clone, FromRow)]
pub struct MovementMode {
    pub id: i16,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Movement style. Only `normal` is confirmed by the upstream sources.
#[derive(Debug, Clone, FromRow)]
pub struct MovementStyle {
    pub id: i16,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Jumpstat jump type with upstream enum mappings.
#[derive(Debug, Clone, FromRow)]
pub struct JumpType {
    pub id: i16,
    pub code: String,
    pub display_name: String,
    /// GOKZ `JumpType_*` value (gokz/include/gokz/jumpstats.inc).
    pub gokz_code: Option<i16>,
    /// cs2kz `JumpType` value (crates/cs2kz/src/jumpstats/jump_type.rs).
    pub cs2kz_code: Option<i16>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Time type of a run. Derived from the teleport count, never chosen manually.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum TimeType {
    /// Teleports > 0 (`GOKZ TimeType_Nub`).
    Nub,
    /// Zero teleports (`GOKZ TimeType_Pro`).
    Pro,
}

impl TimeType {
    /// GOKZ_GetTimeTypeEx: `teleports == 0` is PRO, everything else is NUB.
    pub const fn from_teleports(teleports: u32) -> Self {
        if teleports == 0 { Self::Pro } else { Self::Nub }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Nub => "nub",
            Self::Pro => "pro",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_type_follows_gokz_rules() {
        assert_eq!(TimeType::from_teleports(0), TimeType::Pro);
        assert_eq!(TimeType::from_teleports(1), TimeType::Nub);
        assert_eq!(TimeType::from_teleports(42), TimeType::Nub);
    }

    #[test]
    fn time_type_codes_match_sql_constraints() {
        assert_eq!(TimeType::Nub.as_str(), "nub");
        assert_eq!(TimeType::Pro.as_str(), "pro");
    }
}
