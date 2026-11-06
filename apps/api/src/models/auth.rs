//! Authentication domain types.
//!
//! A session resolves to exactly one Steam account. Authorization is a
//! separate concern: [`AuthUser::is_admin`] is derived from `account_roles`,
//! never from the session itself, so revoking a role takes effect on the next
//! request without touching existing sessions.

use chrono::{DateTime, Utc};
use sqlx::types::BigDecimal;
use uuid::Uuid;

/// Fully resolved session: who is calling and what they may do.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuthUser {
    pub session_id: Uuid,
    pub account_id: i64,
    /// `NUMERIC(20,0)`: a SteamID64 does not fit into `i64`.
    pub steam_id64: BigDecimal,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_admin: bool,
    pub roles: Vec<String>,
    pub expires_at: DateTime<Utc>,
}

impl AuthUser {
    /// SteamID64 as a decimal string, matching every other public contract.
    pub fn steam_id64_string(&self) -> String {
        self.steam_id64.to_string()
    }

    /// Display name with a stable fallback, so the UI never renders `undefined`.
    pub fn display_name_or_fallback(&self) -> String {
        match self.display_name.as_deref().map(str::trim) {
            Some(name) if !name.is_empty() => name.to_owned(),
            _ => {
                let id = self.steam_id64_string();
                let suffix: String = id
                    .chars()
                    .rev()
                    .take(4)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
                format!("Player {suffix}")
            }
        }
    }
}

/// Who performed an administrative request.
///
/// The maintenance token is kept as an escape hatch for operations that cannot
/// hold a browser session (CI jobs, `curl`, monitoring). Both variants are
/// equally privileged; neither is stored in the browser bundle.
#[derive(Debug, Clone)]
pub enum AdminPrincipal {
    Session(AuthUser),
    MaintenanceToken,
}

impl AdminPrincipal {
    pub fn is_maintenance_token(&self) -> bool {
        matches!(self, Self::MaintenanceToken)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr as _;

    fn user(display_name: Option<&str>) -> AuthUser {
        AuthUser {
            session_id: Uuid::nil(),
            account_id: 7,
            steam_id64: BigDecimal::from_str("76561198000000001").unwrap(),
            display_name: display_name.map(ToOwned::to_owned),
            avatar_url: None,
            is_admin: false,
            roles: Vec::new(),
            expires_at: Utc::now(),
        }
    }

    #[test]
    fn display_name_falls_back_to_the_last_steam_id_digits() {
        assert_eq!(user(None).display_name_or_fallback(), "Player 0001");
        assert_eq!(user(Some("  ")).display_name_or_fallback(), "Player 0001");
        assert_eq!(user(Some("Runner")).display_name_or_fallback(), "Runner");
    }

    #[test]
    fn steam_id_is_rendered_as_a_decimal_string() {
        assert_eq!(user(None).steam_id64_string(), "76561198000000001");
    }
}
