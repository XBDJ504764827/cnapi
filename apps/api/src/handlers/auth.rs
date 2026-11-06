//! Steam login, session status and logout.
//!
//! Only Steam OpenID 2.0 is offered: there is no password, no e-mail and no
//! local user table. See [`crate::services::auth`] for the protocol details.

use axum::{
    Json, Router,
    extract::{Query, RawQuery, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use tracing::warn;

use crate::dto::auth::{AuthSessionResponse, LoginStartQuery, LogoutResponse};
use crate::dto::response::{ApiErrorBody, ApiResponse};
use crate::errors::AppError;
use crate::services::auth as service;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/steam/start", get(steam_start))
        .route("/auth/steam/callback", get(steam_callback))
        .route("/auth/me", get(me))
        .route("/auth/logout", post(logout))
}

/// Starts the OpenID round trip.
///
/// Answers with a `303` instead of JSON because the caller is the browser
/// itself: the SPA navigates to this URL with `window.location.assign`.
#[utoipa::path(
    get, path = "/api/v1/auth/steam/start", tag = "auth",
    summary = "Start Steam OpenID login",
    description = "Redirects the browser to Steam. `redirect` must be a same-origin path and is where the browser lands after a successful login.",
    params(LoginStartQuery),
    responses(
        (status = 303, description = "Redirect to the Steam login page"),
        (status = 400, body = ApiErrorBody, description = "`redirect` is not a same-origin path"),
    )
)]
pub async fn steam_start(
    State(state): State<AppState>,
    Query(query): Query<LoginStartQuery>,
) -> Result<Redirect, AppError> {
    let url = service::start_login(&state, query.redirect.as_deref()).await?;
    Ok(Redirect::to(&url))
}

/// OpenID callback. Always answers with a redirect so a browser never sees JSON.
#[utoipa::path(
    get, path = "/api/v1/auth/steam/callback", tag = "auth",
    summary = "Steam OpenID callback",
    description = "Verifies the assertion with Steam, creates the account when needed and sets the session cookie. Failures redirect back to the site with `?auth_error=<code>`.",
    responses(
        (status = 303, description = "Redirect back to the site, with the session cookie on success"),
    )
)]
pub async fn steam_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Response {
    let params = match parse_query(raw_query.as_deref()) {
        Ok(params) => params,
        Err(()) => {
            return redirect_with_error(state.config(), None, "invalid_assertion");
        }
    };

    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.chars().take(512).collect::<String>());
    let ip = client_ip(&headers);

    match service::complete_login(&state, &params, user_agent.as_deref(), ip).await {
        Ok(success) => {
            let mut response = Redirect::to(&success.redirect_url).into_response();
            match HeaderValue::from_str(&success.cookie) {
                Ok(value) => {
                    response.headers_mut().append(header::SET_COOKIE, value);
                }
                Err(error) => {
                    // Cannot happen: the cookie only contains hex, `;`, `=` and
                    // a small attribute set. Log loudly if it ever does.
                    warn!(error = %error, "failed to build the session cookie header");
                    return redirect_with_error(state.config(), None, "login_failed");
                }
            }
            response
        }
        Err(failure) => {
            let redirect_path = failure.redirect_path;
            warn!(
                code = failure.code,
                error = ?failure.source,
                "steam login rejected"
            );
            redirect_with_error(state.config(), redirect_path.as_deref(), failure.code)
        }
    }
}

/// Current session, or `{ authenticated: false }` for anonymous callers.
#[utoipa::path(
    get, path = "/api/v1/auth/me", tag = "auth",
    summary = "Current Steam session",
    description = "Anonymous callers receive `200` with `authenticated: false` so the SPA can render the logged-out state without a failed request.",
    responses((status = 200, body = ApiResponse<AuthSessionResponse>))
)]
pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<AuthSessionResponse>>, AppError> {
    let session = match service::current_user(&state, &headers).await? {
        Some(user) => AuthSessionResponse::from_user(user),
        None => AuthSessionResponse::anonymous(),
    };
    Ok(Json(ApiResponse::ok(session)))
}

/// Revokes the current session and clears the cookie.
#[utoipa::path(
    post, path = "/api/v1/auth/logout", tag = "auth",
    summary = "Log out",
    description = "Idempotent: logging out twice still returns `logged_out: false` the second time.",
    responses((status = 200, body = ApiResponse<LogoutResponse>))
)]
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let logged_out = service::logout(&state, &headers).await?;
    let mut response = Json(ApiResponse::ok(LogoutResponse { logged_out })).into_response();
    match HeaderValue::from_str(&service::cleared_cookie(state.config())) {
        Ok(value) => {
            response.headers_mut().append(header::SET_COOKIE, value);
        }
        Err(error) => warn!(error = %error, "failed to clear the session cookie"),
    }
    Ok(response)
}

/// Redirects a failed login back to the SPA with a stable error code.
fn redirect_with_error(
    config: &crate::config::AppConfig,
    redirect_path: Option<&str>,
    code: &'static str,
) -> Response {
    let path = redirect_path.unwrap_or("/");
    let separator = if path.contains('?') { '&' } else { '?' };
    let location = format!(
        "{}{}{}auth_error={}",
        config.public_base_url, path, separator, code
    );
    let mut response = (StatusCode::SEE_OTHER, [(header::LOCATION, location)]).into_response();
    response
        .headers_mut()
        .append(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

/// Parses the raw query string, preserving duplicate keys.
///
/// `serde_urlencoded` collapses duplicates, and OpenID is a form protocol where
/// a repeated `openid.*` key must reach Steam unchanged.
fn parse_query(raw: Option<&str>) -> Result<Vec<(String, String)>, ()> {
    let Some(raw) = raw.filter(|value| !value.is_empty()) else {
        return Ok(Vec::new());
    };
    let url = reqwest::Url::parse(&format!("http://localhost/?{raw}")).map_err(|_| ())?;
    Ok(url
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect())
}

/// Best-effort client IP for the session record (audit only, never enforced).
fn client_ip(headers: &HeaderMap) -> Option<std::net::IpAddr> {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|value| value.to_str().ok())
        })
        .map(str::trim)
        .and_then(|value| value.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_parsing_keeps_duplicates_and_decodes_values() {
        let params = parse_query(Some(
            "state=abc&openid.claimed_id=https%3A%2F%2Fsteam.example%2Fid%2F1&openid.signed=a,b",
        ))
        .expect("valid query");
        assert_eq!(params[0], ("state".to_owned(), "abc".to_owned()));
        assert_eq!(
            params[1],
            (
                "openid.claimed_id".to_owned(),
                "https://steam.example/id/1".to_owned()
            )
        );
        assert!(parse_query(None).unwrap().is_empty());
    }

    #[test]
    fn client_ip_prefers_the_first_forwarded_address() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "203.0.113.7, 10.0.0.1".parse().unwrap());
        assert_eq!(
            client_ip(&headers).map(|ip| ip.to_string()),
            Some("203.0.113.7".to_owned())
        );
    }
}
