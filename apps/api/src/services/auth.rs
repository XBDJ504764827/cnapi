//! Steam OpenID 2.0 login, session cookies and administrator authorization.
//!
//! ## Flow
//!
//! ```text
//! browser ──GET /api/v1/auth/steam/start?redirect=/admin/dashboard──► API
//!         ◄─302 https://steamcommunity.com/openid/login?openid.*────────┘
//! browser ──redirect (Steam login UI)──────────────────────────────► Steam
//!         ◄─302 /api/v1/auth/steam/callback?state=…&openid.mode=id_res─┘
//! API     ──POST openid.mode=check_authentication──────────────────► Steam
//!         ◄─ns:http://specs.openid.net/auth/2.0 is_valid:true──────────┘
//!         ──302 /admin/dashboard + Set-Cookie cngokz_session──────────► browser
//! ```
//!
//! The OpenID round trip is delegated to Steam's "dumb mode" verification:
//! instead of validating an RSA signature locally, the exact assertion is
//! posted back to the identity provider, which re-signs it. That keeps the
//! backend free of OpenID cryptography and works unchanged through the
//! Cloudflare relay used in production (see `docs/authentication.md`).
//!
//! Nothing here trusts the browser: the `state` nonce, `openid.return_to`,
//! `openid.op_endpoint` and the SteamID64 shape are all checked before a
//! session is minted.

use std::{collections::HashMap, net::IpAddr, sync::OnceLock, time::Duration};

use axum::http::{HeaderMap, header};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use reqwest::Url;
use sha2::{Digest, Sha256};
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::errors::AppError;
use crate::models::auth::{AdminPrincipal, AuthUser};
use crate::repositories::auth as repo;
use crate::state::AppState;

/// OpenID 2.0 namespace.
pub const OPENID_NS: &str = "http://specs.openid.net/auth/2.0";
/// "Let the provider choose the identifier" sentinel.
pub const OPENID_IDENTIFIER_SELECT: &str = "http://specs.openid.net/auth/2.0/identifier_select";
/// Path of the callback registered in `openid.return_to`.
pub const LOGIN_CALLBACK_PATH: &str = "/api/v1/auth/steam/callback";
/// How long a started login may stay unfinished.
const LOGIN_STATE_TTL_MINUTES: i64 = 10;
/// Outbound timeout for Steam (verification + profile lookup).
const HTTP_TIMEOUT: Duration = Duration::from_secs(8);
/// Where failed logins land when the state nonce is already unusable.
const LOGIN_ERROR_FALLBACK_PATH: &str = "/";

/// Successful login: where to send the browser and the cookie to set.
#[derive(Debug, Clone)]
pub struct LoginSuccess {
    pub redirect_url: String,
    pub cookie: String,
}

/// A rejected login. `code` is stable and safe to expose in the URL.
#[derive(Debug)]
pub struct LoginFailure {
    pub code: &'static str,
    pub source: AppError,
    pub redirect_path: Option<String>,
}

impl LoginFailure {
    fn new(code: &'static str, source: AppError) -> Self {
        Self {
            code,
            source,
            redirect_path: None,
        }
    }

    fn with_redirect(mut self, redirect_path: String) -> Self {
        self.redirect_path = Some(redirect_path);
        self
    }
}

// ---------------------------------------------------------------------------
// Login start
// ---------------------------------------------------------------------------

/// Creates a login state and returns the Steam redirect URL.
pub async fn start_login(state: &AppState, redirect: Option<&str>) -> Result<String, AppError> {
    let config = state.config();
    let redirect_path = normalize_redirect_path(redirect)?;
    let login_state = random_token();

    repo::create_login_state(
        state.db(),
        &login_state,
        &redirect_path,
        ChronoDuration::minutes(LOGIN_STATE_TTL_MINUTES),
    )
    .await?;

    // The browser may reach Steam directly even when this server cannot, so the
    // redirect target is configured separately from the verification endpoint.
    let mut url = Url::parse(&config.steam_openid_browser_endpoint).map_err(|error| {
        AppError::internal(anyhow::anyhow!(
            "STEAM_OPENID_BROWSER_ENDPOINT is not a valid URL: {error}"
        ))
    })?;
    url.query_pairs_mut()
        .append_pair("openid.ns", OPENID_NS)
        .append_pair("openid.mode", "checkid_setup")
        .append_pair("openid.return_to", &return_to_url(config, &login_state))
        .append_pair("openid.realm", &config.public_base_url)
        .append_pair("openid.identity", OPENID_IDENTIFIER_SELECT)
        .append_pair("openid.claimed_id", OPENID_IDENTIFIER_SELECT);

    Ok(url.to_string())
}

/// Absolute callback URL carrying the login state; also the exact value
/// echoed back in `openid.return_to`.
pub fn return_to_url(config: &AppConfig, login_state: &str) -> String {
    format!(
        "{}{}?state={}",
        config.public_base_url, LOGIN_CALLBACK_PATH, login_state
    )
}

/// Restricts the post-login redirect to a same-origin path.
///
/// Rejecting `//host`, absolute URLs and backslashes prevents an open redirect
/// through `?redirect=` and keeps the session cookie on our own origin.
fn normalize_redirect_path(redirect: Option<&str>) -> Result<String, AppError> {
    let raw = redirect.unwrap_or(LOGIN_ERROR_FALLBACK_PATH).trim();
    let invalid =
        || AppError::validation("`redirect` must be a same-origin path starting with `/`");
    if raw.is_empty() || raw.len() > 512 {
        return Err(invalid());
    }
    if !raw.starts_with('/') || raw.starts_with("//") || raw.starts_with("/\\") {
        return Err(invalid());
    }
    if raw.contains('\\') || raw.chars().any(|c| c.is_control()) {
        return Err(invalid());
    }
    Ok(raw.to_owned())
}

// ---------------------------------------------------------------------------
// Callback verification
// ---------------------------------------------------------------------------

/// Verifies the assertion returned by Steam and mints a session.
///
/// `params` are the raw query pairs of the callback request; duplicates are
/// preserved because OpenID is a form-encoded key/value protocol.
pub async fn complete_login(
    state: &AppState,
    params: &[(String, String)],
    user_agent: Option<&str>,
    ip: Option<IpAddr>,
) -> Result<LoginSuccess, LoginFailure> {
    let config = state.config();
    let fields: HashMap<&str, &str> = params
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();

    let Some(login_state) = fields
        .get("state")
        .copied()
        .filter(|value| !value.is_empty())
    else {
        return Err(LoginFailure::new(
            "invalid_state",
            AppError::Unauthorized {
                message: "missing login state".to_owned(),
            },
        ));
    };

    // Single use: a concurrent or replayed callback finds nothing.
    let redirect_path = match repo::consume_login_state(state.db(), login_state).await {
        Ok(Some(path)) => path,
        Ok(None) => {
            return Err(LoginFailure::new(
                "invalid_state",
                AppError::Unauthorized {
                    message: "login state is unknown, expired or already used".to_owned(),
                },
            ));
        }
        Err(error) => return Err(LoginFailure::new("login_failed", error)),
    };

    let reject = |code: &'static str, error: AppError| {
        LoginFailure::new(code, error).with_redirect(redirect_path.clone())
    };

    if fields.get("openid.mode").copied() == Some("cancel") {
        return Err(reject(
            "cancelled",
            AppError::Unauthorized {
                message: "login cancelled at Steam".to_owned(),
            },
        ));
    }

    let expected_return_to = return_to_url(config, login_state);
    if let Err(error) = validate_assertion(config, &fields, &expected_return_to) {
        return Err(reject("invalid_assertion", error));
    }

    let claimed_id = fields["openid.claimed_id"];
    let steam_id64 = match extract_steam_id64(config, claimed_id) {
        Ok(id) => id,
        Err(error) => return Err(reject("invalid_assertion", error)),
    };

    // Ask Steam whether the assertion really came from it.
    if let Err(error) = verify_with_provider(config, params).await {
        return Err(reject("verification_failed", error));
    }

    let account_id = match repo::ensure_account(state.db(), &steam_id64).await {
        Ok(id) => id,
        Err(error) => return Err(reject("login_failed", error)),
    };

    // Bootstrap administrators: `ADMIN_STEAM_IDS` is the only way to grant the
    // first role without direct database access.
    if config.admin_steam_ids.iter().any(|id| id == &steam_id64)
        && let Err(error) = repo::grant_role(state.db(), account_id, "admin").await
    {
        warn!(error = %error, "failed to grant bootstrap admin role");
    }

    if let Some(profile) = fetch_steam_profile(config, &steam_id64).await
        && let Err(error) = repo::sync_profile(
            state.db(),
            account_id,
            profile.persona_name.as_deref(),
            profile.avatar_url.as_deref(),
        )
        .await
    {
        warn!(error = %error, "failed to sync Steam profile");
    }

    let token = random_token();
    let expires_at = Utc::now()
        + ChronoDuration::from_std(config.session_ttl)
            .unwrap_or_else(|_| ChronoDuration::hours(24));
    let ip_text = ip.map(|addr| addr.to_string());
    if let Err(error) = repo::create_session(
        state.db(),
        account_id,
        &token_hash(&token),
        expires_at,
        user_agent,
        ip_text.as_deref(),
    )
    .await
    {
        return Err(reject("login_failed", error));
    }

    if let Err(error) = repo::purge_expired_sessions(state.db()).await {
        warn!(error = %error, "failed to purge expired sessions");
    }

    info!(account_id, steam_id64 = %steam_id64, "steam login succeeded");

    Ok(LoginSuccess {
        redirect_url: format!("{}{}", config.public_base_url, redirect_path),
        cookie: session_cookie(config, &token, expires_at),
    })
}

fn validate_assertion(
    config: &AppConfig,
    fields: &HashMap<&str, &str>,
    expected_return_to: &str,
) -> Result<(), AppError> {
    let invalid = |message: String| AppError::Unauthorized { message };

    if fields.get("openid.mode").copied() != Some("id_res") {
        return Err(invalid("openid.mode must be `id_res`".to_owned()));
    }
    for key in [
        "openid.op_endpoint",
        "openid.claimed_id",
        "openid.identity",
        "openid.return_to",
        "openid.response_nonce",
        "openid.signed",
        "openid.sig",
    ] {
        if fields.get(key).copied().unwrap_or("").is_empty() {
            return Err(invalid(format!("`{key}` is missing")));
        }
    }

    if fields["openid.op_endpoint"] != config.steam_openid_op_endpoint {
        return Err(invalid("unexpected openid.op_endpoint".to_owned()));
    }
    if fields["openid.claimed_id"] != fields["openid.identity"] {
        return Err(invalid(
            "openid.claimed_id and openid.identity differ".to_owned(),
        ));
    }
    if fields["openid.return_to"] != expected_return_to {
        return Err(invalid("openid.return_to does not match".to_owned()));
    }
    Ok(())
}

/// Extracts the SteamID64 from `openid.claimed_id`.
fn extract_steam_id64(config: &AppConfig, claimed_id: &str) -> Result<String, AppError> {
    let Some(rest) = claimed_id.strip_prefix(config.steam_openid_claimed_id_prefix.as_str()) else {
        return Err(AppError::Unauthorized {
            message: "openid.claimed_id is not a Steam identity".to_owned(),
        });
    };
    let looks_like_steam_id = (17..=20).contains(&rest.len())
        && rest.chars().all(|c| c.is_ascii_digit())
        && rest.chars().any(|c| c != '0');
    if !looks_like_steam_id {
        return Err(AppError::Unauthorized {
            message: "openid.claimed_id is not a valid SteamID64".to_owned(),
        });
    }
    Ok(rest.to_owned())
}

/// Posts the assertion back to the identity provider (`check_authentication`).
///
/// The response is a tiny key/value document; only `is_valid:true` counts.
async fn verify_with_provider(
    config: &AppConfig,
    params: &[(String, String)],
) -> Result<(), AppError> {
    let form: Vec<(&str, &str)> = params
        .iter()
        .filter(|(key, _)| key.starts_with("openid."))
        .map(|(key, value)| {
            if key == "openid.mode" {
                ("openid.mode", "check_authentication")
            } else {
                (key.as_str(), value.as_str())
            }
        })
        .collect();

    let response = http_client()
        .post(&config.steam_openid_endpoint)
        .form(&form)
        .send()
        .await
        .map_err(|error| {
            warn!(error = %error, endpoint = %config.steam_openid_endpoint, "openid verification request failed");
            AppError::DependencyUnavailable {
                dependency: "steam_openid",
            }
        })?;

    if !response.status().is_success() {
        warn!(
            status = response.status().as_u16(),
            "openid verification returned an error"
        );
        return Err(AppError::DependencyUnavailable {
            dependency: "steam_openid",
        });
    }

    let body = response.text().await.map_err(|error| {
        warn!(error = %error, "openid verification body could not be read");
        AppError::DependencyUnavailable {
            dependency: "steam_openid",
        }
    })?;

    let valid = body
        .lines()
        .filter_map(|line| line.split_once(':'))
        .any(|(key, value)| key.trim() == "is_valid" && value.trim() == "true");

    if !valid {
        warn!(body = %truncate(&body, 200), "openid assertion rejected by the provider");
        return Err(AppError::Unauthorized {
            message: "Steam rejected the assertion".to_owned(),
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Steam Web API profile (best effort)
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct SteamProfile {
    persona_name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(serde::Deserialize)]
struct PlayerSummariesResponse {
    #[serde(default)]
    response: PlayerSummaries,
}

#[derive(serde::Deserialize, Default)]
struct PlayerSummaries {
    #[serde(default)]
    players: Vec<SteamPlayer>,
}

#[derive(serde::Deserialize, Default)]
struct SteamPlayer {
    #[serde(default)]
    personaname: Option<String>,
    #[serde(default)]
    avatarfull: Option<String>,
}

/// Fetches the persona name and avatar. Failures are logged and ignored: a
/// profile lookup must never block a login.
async fn fetch_steam_profile(config: &AppConfig, steam_id64: &str) -> Option<SteamProfile> {
    let api_key = config.steam_api_key.as_deref()?;
    let url = format!(
        "{}/ISteamUser/GetPlayerSummaries/v2/?key={}&steamids={}",
        config.steam_api_base_url, api_key, steam_id64
    );

    let response = match http_client().get(&url).send().await {
        Ok(response) => response,
        Err(error) => {
            warn!(error = %error, "steam profile lookup failed");
            return None;
        }
    };
    if !response.status().is_success() {
        warn!(
            status = response.status().as_u16(),
            "steam profile lookup returned an error"
        );
        return None;
    }

    let body: PlayerSummariesResponse = match response.json().await {
        Ok(body) => body,
        Err(error) => {
            warn!(error = %error, "steam profile response was not JSON");
            return None;
        }
    };
    let player = body.response.players.into_iter().next()?;
    let sanitized_name = player.personaname.and_then(|name| {
        let name = name.trim().chars().take(64).collect::<String>();
        (!name.is_empty()).then_some(name)
    });
    let avatar_url = player
        .avatarfull
        .filter(|url| url.starts_with("https://") && url.len() <= 512);

    Some(SteamProfile {
        persona_name: sanitized_name,
        avatar_url,
    })
}

// ---------------------------------------------------------------------------
// Sessions
// ---------------------------------------------------------------------------

/// Resolves the caller, or `None` when the request is anonymous.
pub async fn current_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<AuthUser>, AppError> {
    let Some(token) = read_cookie(headers, &state.config().session_cookie_name) else {
        return Ok(None);
    };
    let user = repo::find_session(state.db(), &token_hash(&token)).await?;
    if let Some(user) = user.as_ref()
        && let Err(error) = repo::touch_session(state.db(), user.session_id).await
    {
        warn!(error = %error, "failed to refresh session timestamp");
    }
    Ok(user)
}

/// Authorization gate for every `/api/v1/admin/*` endpoint.
///
/// Accepts a session cookie carrying the `admin` role, or the
/// `X-Maintenance-Token` header for automation. Anything else is rejected.
pub async fn require_admin(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AdminPrincipal, AppError> {
    if let Some(expected) = state.config().maintenance_token.as_deref() {
        let presented = headers
            .get("x-maintenance-token")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");
        if !presented.is_empty() && constant_time_eq(presented.as_bytes(), expected.as_bytes()) {
            return Ok(AdminPrincipal::MaintenanceToken);
        }
    }

    let user = current_user(state, headers)
        .await?
        .ok_or_else(|| AppError::Unauthorized {
            message: "login is required".to_owned(),
        })?;

    if !user.is_admin {
        return Err(AppError::Forbidden {
            message: "the admin role is required".to_owned(),
        });
    }

    Ok(AdminPrincipal::Session(user))
}

/// Revokes the presented session. Returns whether a session existed.
pub async fn logout(state: &AppState, headers: &HeaderMap) -> Result<bool, AppError> {
    let Some(token) = read_cookie(headers, &state.config().session_cookie_name) else {
        return Ok(false);
    };
    repo::revoke_session(state.db(), &token_hash(&token)).await
}

/// `Set-Cookie` value that clears the session cookie.
pub fn cleared_cookie(config: &AppConfig) -> String {
    format!(
        "{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{}",
        config.session_cookie_name,
        cookie_security_attributes(config)
    )
}

fn session_cookie(config: &AppConfig, token: &str, expires_at: DateTime<Utc>) -> String {
    let max_age = (expires_at - Utc::now()).num_seconds().max(0);
    format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}",
        config.session_cookie_name,
        token,
        max_age,
        cookie_security_attributes(config)
    )
}

fn cookie_security_attributes(config: &AppConfig) -> String {
    let mut attributes = String::new();
    if config.session_cookie_secure {
        attributes.push_str("; Secure");
    }
    if let Some(domain) = config.session_cookie_domain.as_deref() {
        attributes.push_str("; Domain=");
        attributes.push_str(domain);
    }
    attributes
}

/// Reads one cookie out of the `Cookie` header(s).
fn read_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.split_once('='))
        .find(|(key, _)| key.trim() == name)
        .map(|(_, value)| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

/// 256-bit opaque token rendered as 64 hex characters.
fn random_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

/// SHA-256 digest of a session token; the only form persisted.
pub fn token_hash(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (a, b) in left.iter().zip(right) {
        difference |= a ^ b;
    }
    difference == 0
}

fn truncate(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .user_agent(concat!("CNGOKZAPI/", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn test_config() -> AppConfig {
        AppConfig::from_pairs(vec![
            ("APP_ENV", "development".to_owned()),
            (
                "DATABASE_URL",
                "postgres://user:secret@localhost:5432/db".to_owned(),
            ),
            ("REDIS_URL", "redis://localhost:6379".to_owned()),
            ("PUBLIC_BASE_URL", "https://kz.example".to_owned()),
        ])
        .expect("config loads")
    }

    #[test]
    fn redirect_paths_stay_on_the_same_origin() {
        assert_eq!(normalize_redirect_path(None).unwrap(), "/");
        assert_eq!(
            normalize_redirect_path(Some("/admin/dashboard?tab=1")).unwrap(),
            "/admin/dashboard?tab=1"
        );

        for blocked in [
            "https://evil.example/admin",
            "//evil.example",
            "/\\evil.example",
            "admin",
            "/admin\nSet-Cookie: x=1",
        ] {
            assert!(
                normalize_redirect_path(Some(blocked)).is_err(),
                "`{blocked}` should be rejected"
            );
        }
    }

    #[test]
    fn return_to_is_built_from_the_public_origin() {
        let config = test_config();
        assert_eq!(
            return_to_url(&config, "abc123"),
            "https://kz.example/api/v1/auth/steam/callback?state=abc123"
        );
    }

    #[test]
    fn assertion_validation_rejects_foreign_endpoints_and_return_urls() {
        let config = test_config();
        let expected = return_to_url(&config, "state-1");
        let mut fields = HashMap::from([
            ("openid.mode", "id_res"),
            (
                "openid.op_endpoint",
                "https://steamcommunity.com/openid/login",
            ),
            (
                "openid.claimed_id",
                "https://steamcommunity.com/openid/id/76561198000000001",
            ),
            (
                "openid.identity",
                "https://steamcommunity.com/openid/id/76561198000000001",
            ),
            ("openid.return_to", expected.as_str()),
            ("openid.response_nonce", "2026-11-06T00:00:00Zabc"),
            ("openid.signed", "signed,op_endpoint,claimed_id"),
            ("openid.sig", "sig"),
        ]);
        assert!(validate_assertion(&config, &fields, &expected).is_ok());

        fields.insert("openid.op_endpoint", "https://evil.example/openid/login");
        assert!(validate_assertion(&config, &fields, &expected).is_err());

        fields.insert(
            "openid.op_endpoint",
            "https://steamcommunity.com/openid/login",
        );
        fields.insert("openid.identity", "https://steamcommunity.com/openid/id/1");
        assert!(validate_assertion(&config, &fields, &expected).is_err());

        fields.insert(
            "openid.identity",
            "https://steamcommunity.com/openid/id/76561198000000001",
        );
        fields.insert("openid.return_to", "https://evil.example/callback");
        assert!(validate_assertion(&config, &fields, &expected).is_err());

        fields.insert("openid.return_to", expected.as_str());
        fields.insert("openid.mode", "cancel");
        assert!(validate_assertion(&config, &fields, &expected).is_err());
    }

    #[test]
    fn claimed_id_must_carry_a_steam_id64() {
        let config = test_config();
        assert_eq!(
            extract_steam_id64(
                &config,
                "https://steamcommunity.com/openid/id/76561198000000001"
            )
            .unwrap(),
            "76561198000000001"
        );
        for bad in [
            "https://example.com/openid/id/76561198000000001",
            "https://steamcommunity.com/openid/id/",
            "https://steamcommunity.com/openid/id/76561198abc",
            "https://steamcommunity.com/openid/id/00000000000000000",
        ] {
            assert!(
                extract_steam_id64(&config, bad).is_err(),
                "`{bad}` should be rejected"
            );
        }
    }

    #[test]
    fn tokens_are_hashed_and_cookies_are_http_only() {
        let config = test_config();
        let token = random_token();
        assert_eq!(token.len(), 64);
        assert_ne!(token_hash(&token), token_hash("other"));
        assert_eq!(token_hash(&token).len(), 32);

        let cookie = session_cookie(&config, &token, Utc::now() + ChronoDuration::hours(1));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Lax"));
        assert!(cookie.contains("Path=/"));
        // Secure is off in development so http://localhost keeps working.
        assert!(!cookie.contains("Secure"));

        let cleared = cleared_cookie(&config);
        assert!(cleared.contains("Max-Age=0"));
    }

    #[test]
    fn cookie_header_parsing_ignores_unrelated_cookies() {
        let mut headers = HeaderMap::new();
        headers.append(
            header::COOKIE,
            "theme=dark; cngokz_session=abc123; other=1"
                .parse()
                .unwrap(),
        );
        assert_eq!(
            read_cookie(&headers, "cngokz_session").as_deref(),
            Some("abc123")
        );
        assert!(read_cookie(&headers, "missing").is_none());
    }

    #[test]
    fn constant_time_comparison_is_exact() {
        assert!(constant_time_eq(b"token", b"token"));
        assert!(!constant_time_eq(b"token", b"Token"));
        assert!(!constant_time_eq(b"token", b"token "));
        assert!(!constant_time_eq(b"", b"token"));
    }
}
