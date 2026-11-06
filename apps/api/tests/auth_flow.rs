//! End-to-end tests for Steam OpenID login and the administrator gate.
//!
//! A local mock identity provider stands in for `steamcommunity.com`, so the
//! whole round trip (start → callback → `check_authentication` → session →
//! logout) runs against the real router and a real PostgreSQL schema.
//!
//! Skipped when `DATABASE_URL` is not configured, like the other integration
//! tests in this crate.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    response::IntoResponse,
};
use cngokz_api::{config::AppConfig, routes, state, state::AppState};
use serde_json::Value;
use sqlx::{AssertSqlSafe, PgPool, postgres::PgConnectOptions};
use tower::ServiceExt;

const ADMIN_STEAM_ID: &str = "76561198000000901";
const PLAYER_STEAM_ID: &str = "76561198000000902";

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

async fn create_database() -> Option<(PgPool, String)> {
    let base_url = std::env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.is_empty())?;
    let admin = PgPool::connect(&base_url).await.expect("admin connection");
    let database_name = format!("cngokz_auth_{}", uuid::Uuid::new_v4().simple());

    sqlx::raw_sql(AssertSqlSafe(format!(
        "CREATE DATABASE \"{database_name}\""
    )))
    .execute(&admin)
    .await
    .expect("create temporary database");

    let options: PgConnectOptions = base_url.parse().expect("valid DATABASE_URL");
    let pool = PgPool::connect_with(options.database(&database_name))
        .await
        .expect("temporary database connection");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations");

    let database_url = match base_url.rsplit_once('/') {
        Some((prefix, _)) => format!("{prefix}/{database_name}"),
        None => base_url.clone(),
    };
    Some((pool, database_url))
}

fn test_state(pool: PgPool, database_url: &str, provider: &str) -> AppState {
    let config = AppConfig::from_pairs([
        ("APP_ENV", "test".to_owned()),
        ("APP_VERSION", "0.0.0-test".to_owned()),
        ("DATABASE_URL", database_url.to_owned()),
        ("REDIS_URL", "redis://127.0.0.1:1".to_owned()),
        ("PUBLIC_BASE_URL", "https://kz.test".to_owned()),
        ("STEAM_OPENID_ENDPOINT", provider.to_owned()),
        ("ADMIN_STEAM_IDS", ADMIN_STEAM_ID.to_owned()),
    ])
    .expect("config");

    let redis = state::RedisClient::new(&config.redis_url, "test").expect("redis client");
    AppState::new(config, pool, redis)
}

/// Mock Steam OpenID provider: records how often it was asked to verify.
async fn spawn_provider() -> (String, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();

    let app = Router::new().route(
        "/openid/login",
        axum::routing::post(move || {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                (
                    [(header::CONTENT_TYPE, "text/plain")],
                    "ns:http://specs.openid.net/auth/2.0\nis_valid:true\n",
                )
                    .into_response()
            }
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock provider");
    let address = listener.local_addr().expect("mock address");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    (format!("http://{address}/openid/login"), calls)
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

async fn send(app: &Router, request: Request<Body>) -> axum::response::Response {
    app.clone().oneshot(request).await.expect("response")
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&bytes).expect("json body")
}

fn start_login_request(redirect: &str) -> Request<Body> {
    Request::builder()
        .uri(format!("/api/v1/auth/steam/start?redirect={redirect}"))
        .body(Body::empty())
        .expect("request")
}

/// Reads `state` and `openid.return_to` out of the redirect issued by the API.
fn parse_start_redirect(location: &str) -> (String, String) {
    let url = reqwest::Url::parse(location).expect("absolute redirect");
    let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    assert_eq!(
        pairs.get("openid.mode").map(String::as_str),
        Some("checkid_setup")
    );
    assert_eq!(
        pairs.get("openid.realm").map(String::as_str),
        Some("https://kz.test")
    );
    let return_to = pairs.get("openid.return_to").expect("return_to").clone();
    let state = reqwest::Url::parse(&return_to)
        .expect("return_to URL")
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.into_owned())
        .expect("state in return_to");
    (state, return_to)
}

/// Builds the callback URL Steam would send the browser to.
fn callback_uri(
    state: &str,
    return_to: &str,
    steam_id: &str,
    overrides: &[(&str, &str)],
) -> String {
    let mut fields = vec![
        ("state".to_owned(), state.to_owned()),
        ("openid.mode".to_owned(), "id_res".to_owned()),
        (
            "openid.op_endpoint".to_owned(),
            "https://steamcommunity.com/openid/login".to_owned(),
        ),
        (
            "openid.claimed_id".to_owned(),
            format!("https://steamcommunity.com/openid/id/{steam_id}"),
        ),
        (
            "openid.identity".to_owned(),
            format!("https://steamcommunity.com/openid/id/{steam_id}"),
        ),
        ("openid.return_to".to_owned(), return_to.to_owned()),
        (
            "openid.response_nonce".to_owned(),
            "2026-11-06T10:00:00Zabc".to_owned(),
        ),
        (
            "openid.signed".to_owned(),
            "signed,op_endpoint,claimed_id,identity,return_to,response_nonce,assoc_handle"
                .to_owned(),
        ),
        ("openid.sig".to_owned(), "c2lnbmF0dXJl".to_owned()),
    ];
    for (key, value) in overrides {
        fields.retain(|(name, _)| name != key);
        fields.push(((*key).to_owned(), (*value).to_owned()));
    }

    let query = fields
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&");
    format!("/api/v1/auth/steam/callback?{query}")
}

fn session_cookie(response: &axum::response::Response) -> Option<String> {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find(|value| value.starts_with("cngokz_session=") && !value.contains("Max-Age=0"))
        .map(|value| value.split(';').next().unwrap_or_default().to_owned())
}

fn location(response: &axum::response::Response) -> String {
    response
        .headers()
        .get(header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("Location header")
        .to_owned()
}

async fn me(app: &Router, cookie: Option<&str>) -> Value {
    let mut builder = Request::builder().uri("/api/v1/auth/me");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = send(app, builder.body(Body::empty()).expect("request")).await;
    assert_eq!(response.status(), StatusCode::OK);
    body_json(response).await
}

async fn admin_gate(app: &Router, cookie: Option<&str>) -> StatusCode {
    let mut builder = Request::builder().uri("/api/v1/admin/game-servers");
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let response = send(app, builder.body(Body::empty()).expect("request")).await;
    response.status()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn steam_round_trip_logs_in_an_admin_and_protects_the_console() {
    let Some((pool, database_url)) = create_database().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };
    let (provider, provider_calls) = spawn_provider().await;
    let app = routes::build_router(test_state(pool.clone(), &database_url, &provider));

    // 1. start: the browser is sent to the configured identity provider.
    let response = send(&app, start_login_request("/admin/dashboard")).await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let (state, return_to) = parse_start_redirect(&location(&response));
    assert!(location(&response).starts_with(&provider));

    // 2. anonymous access: no cookie, no admin API.
    assert_eq!(admin_gate(&app, None).await, StatusCode::UNAUTHORIZED);
    let body = me(&app, None).await;
    assert_eq!(body["data"]["authenticated"], false);
    assert!(body["data"]["user"].is_null());

    // 3. callback: Steam assertion verified by the provider, session minted.
    let response = send(
        &app,
        Request::builder()
            .uri(callback_uri(&state, &return_to, ADMIN_STEAM_ID, &[]))
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&response), "https://kz.test/admin/dashboard");
    let cookie = session_cookie(&response).expect("session cookie");
    assert_eq!(provider_calls.load(Ordering::SeqCst), 1);

    // 4. the session resolves to the Steam account, with the admin role.
    let body = me(&app, Some(&cookie)).await;
    assert_eq!(body["data"]["authenticated"], true);
    assert_eq!(body["data"]["user"]["steam_id64"], ADMIN_STEAM_ID);
    assert_eq!(body["data"]["user"]["is_admin"], true);
    assert_eq!(body["data"]["user"]["roles"][0], "admin");

    // 5. the admin API accepts the session cookie.
    assert_eq!(admin_gate(&app, Some(&cookie)).await, StatusCode::OK);

    // 6. replaying the same state is impossible.
    let replayed = send(
        &app,
        Request::builder()
            .uri(callback_uri(&state, &return_to, ADMIN_STEAM_ID, &[]))
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(replayed.status(), StatusCode::SEE_OTHER);
    assert!(location(&replayed).contains("auth_error=invalid_state"));
    // The provider was never asked twice for the replayed state.
    assert_eq!(provider_calls.load(Ordering::SeqCst), 1);

    // 7. logout revokes the session immediately.
    let response = send(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/logout")
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let cleared = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .any(|value| value.contains("Max-Age=0"));
    assert!(cleared, "logout must clear the cookie");
    assert_eq!(
        me(&app, Some(&cookie)).await["data"]["authenticated"],
        false
    );
    assert_eq!(
        admin_gate(&app, Some(&cookie)).await,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn non_admin_players_are_authenticated_but_forbidden() {
    let Some((pool, database_url)) = create_database().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };
    let (provider, _) = spawn_provider().await;
    let app = routes::build_router(test_state(pool.clone(), &database_url, &provider));

    let response = send(&app, start_login_request("/players")).await;
    let (state, return_to) = parse_start_redirect(&location(&response));

    let response = send(
        &app,
        Request::builder()
            .uri(callback_uri(&state, &return_to, PLAYER_STEAM_ID, &[]))
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(location(&response), "https://kz.test/players");
    let cookie = session_cookie(&response).expect("session cookie");

    let body = me(&app, Some(&cookie)).await;
    assert_eq!(body["data"]["authenticated"], true);
    assert_eq!(body["data"]["user"]["is_admin"], false);
    assert_eq!(
        body["data"]["user"]["roles"].as_array().map(Vec::len),
        Some(0)
    );

    // Logged in, but not an administrator.
    assert_eq!(admin_gate(&app, Some(&cookie)).await, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn forged_assertions_never_reach_the_provider() {
    let Some((pool, database_url)) = create_database().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };
    let (provider, provider_calls) = spawn_provider().await;
    let app = routes::build_router(test_state(pool.clone(), &database_url, &provider));

    // A foreign `op_endpoint` and a foreign `return_to` are rejected locally.
    for overrides in [
        &[("openid.op_endpoint", "https://evil.example/openid/login")][..],
        &[("openid.return_to", "https://evil.example/callback?state=x")][..],
        &[(
            "openid.identity",
            "https://steamcommunity.com/openid/id/76561198000000002",
        )][..],
    ] {
        let response = send(&app, start_login_request("/")).await;
        let (state, return_to) = parse_start_redirect(&location(&response));
        let response = send(
            &app,
            Request::builder()
                .uri(callback_uri(&state, &return_to, ADMIN_STEAM_ID, overrides))
                .body(Body::empty())
                .expect("request"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert!(location(&response).contains("auth_error=invalid_assertion"));
        assert!(session_cookie(&response).is_none());
    }
    assert_eq!(provider_calls.load(Ordering::SeqCst), 0);

    // An unknown state is refused before any network call as well.
    let response = send(
        &app,
        Request::builder()
            .uri(callback_uri(
                "not-a-real-state",
                "https://kz.test/x",
                ADMIN_STEAM_ID,
                &[],
            ))
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert!(location(&response).contains("auth_error=invalid_state"));
    assert_eq!(provider_calls.load(Ordering::SeqCst), 0);

    // Open redirect: the login never starts.
    let response = send(&app, start_login_request("https://evil.example/")).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn the_maintenance_token_still_works_for_automation() {
    let Some((pool, database_url)) = create_database().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };
    let (provider, _) = spawn_provider().await;

    let config = AppConfig::from_pairs([
        ("APP_ENV", "test".to_owned()),
        ("DATABASE_URL", database_url.clone()),
        ("REDIS_URL", "redis://127.0.0.1:1".to_owned()),
        ("PUBLIC_BASE_URL", "https://kz.test".to_owned()),
        ("STEAM_OPENID_ENDPOINT", provider.clone()),
        ("MAINTENANCE_TOKEN", "automation-token".to_owned()),
    ])
    .expect("config");
    let redis = state::RedisClient::new(&config.redis_url, "test").expect("redis client");
    let app = routes::build_router(AppState::new(config, pool, redis));

    let response = send(
        &app,
        Request::builder()
            .uri("/api/v1/admin/game-servers")
            .header("x-maintenance-token", "automation-token")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    let response = send(
        &app,
        Request::builder()
            .uri("/api/v1/admin/game-servers")
            .header("x-maintenance-token", "wrong-token")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
