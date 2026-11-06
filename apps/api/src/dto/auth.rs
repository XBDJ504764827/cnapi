//! Public authentication contracts.
//!
//! The API exposes only what the SPA needs to render the account menu and to
//! decide between "log in", "forbidden" and "allowed".

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::models::auth::AuthUser;

/// The signed-in Steam account.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthUserResponse {
    pub account_id: i64,
    /// Decimal string; a SteamID64 does not fit into a JSON number safely.
    pub steam_id64: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    /// `true` when the account holds the `admin` role.
    pub is_admin: bool,
    pub roles: Vec<String>,
    pub steam_profile_url: String,
}

/// Response of `GET /api/v1/auth/me`.
///
/// Anonymous callers get `200` with `authenticated: false` instead of `401`, so
/// the SPA can render the "log in" state without logging a network error.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthSessionResponse {
    pub authenticated: bool,
    pub user: Option<AuthUserResponse>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl AuthSessionResponse {
    pub fn anonymous() -> Self {
        Self {
            authenticated: false,
            user: None,
            expires_at: None,
        }
    }

    pub fn from_user(user: AuthUser) -> Self {
        Self {
            authenticated: true,
            user: Some(AuthUserResponse::from(&user)),
            expires_at: Some(user.expires_at),
        }
    }
}

impl From<&AuthUser> for AuthUserResponse {
    fn from(user: &AuthUser) -> Self {
        let steam_id64 = user.steam_id64_string();
        Self {
            account_id: user.account_id,
            steam_profile_url: format!("https://steamcommunity.com/profiles/{steam_id64}"),
            steam_id64,
            display_name: user.display_name_or_fallback(),
            avatar_url: user.avatar_url.clone(),
            is_admin: user.is_admin,
            roles: user.roles.clone(),
        }
    }
}

/// Response of `POST /api/v1/auth/logout`.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LogoutResponse {
    pub logged_out: bool,
}

/// Query of `GET /api/v1/auth/steam/start`.
#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
pub struct LoginStartQuery {
    /// Same-origin path to return to after a successful login.
    pub redirect: Option<String>,
}
