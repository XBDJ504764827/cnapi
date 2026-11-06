//! End-to-end tests for the callback-style Steam relay (`STEAM_RELAY_URL`).
//!
//! A local mock stands in for the shared `cngokz-steam-auth` Cloudflare Worker:
//! it implements the same `/login` → `/verify` contract (302 back with a
//! single-use token, JSON identity on exchange), so the whole flow runs against
//! the real router without touching the network.
//!
//! Skipped when `DATABASE_URL` is not configured, like the other integration
//! tests in this crate.

use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use axum::{
    Json, Router,
    body::Body,
    extract::Query,
    http::{HeaderMap, Request, StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use cngokz_api::{config::AppConfig, routes, state, state::AppState};
use serde_json::{Value, json};
use sqlx::{AssertSqlSafe, PgPool, postgres::PgConnectOptions};
use tower::ServiceExt;

const ADMIN_STEAM_ID: &str = "76561198000000901";

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

async fn create_database() -> Option<(PgPool, String)> {
    let base_url = std::env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.is_empty())?;
    let admin = PgPool::connect(&base_url).await.expect("admin connection");
    let database_name = format!("cngokz_relay_{}", uuid::Uuid::new_v4().simple());

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

fn relay_state(pool: PgPool, database_url: &str, relay: &str) -> AppState {
    let config = AppConfig::from_pairs([
        ("APP_ENV", "test".to_owned()),
        ("APP_VERSION", "0.0.0-test".to_owned()),
        ("DATABASE_URL", database_url.to_owned()),
        ("REDIS_URL", "redis://127.0.0.1:1".to_owned()),
        ("PUBLIC_BASE_URL", "https://kz.test".to_owned()),
        ("STEAM_RELAY_URL", relay.to_owned()),
        ("ADMIN_STEAM_IDS", ADMIN_STEAM_ID.to_owned()),
    ])
    .expect("config");

    let redis = state::RedisClient::new(&config.redis_url, "test").expect("redis client");
    AppState::new(config, pool, redis)
}

#[derive(Default)]
struct RelayStats {
    logins: AtomicUsize,
    verifies: AtomicUsize,
}

/// Mock of the shared Worker: `/login` bounces back with a token, `/verify`
/// burns it exactly once.
async fn spawn_relay(steam_id: &'static str) -> (String, Arc<RelayStats>) {
    let stats = Arc::new(RelayStats::default());

    let login_stats = stats.clone();
    let login = move |Query(query): Query<HashMap<String, String>>| {
        let stats = login_stats.clone();
        async move {
            stats.logins.fetch_add(1, Ordering::SeqCst);
            let return_to = query.get("return_to").cloned().unwrap_or_default();
            let state = query.get("state").cloned().unwrap_or_default();
            let mut target = reqwest::Url::parse(&return_to).expect("return_to URL");
            target
                .query_pairs_mut()
                .append_pair("token", "relay-token-1")
                .append_pair("state", &state);
            (StatusCode::FOUND, [(header::LOCATION, target.to_string())])
        }
    };

    let verify_stats = stats.clone();
    let verify = move |Query(query): Query<HashMap<String, String>>| {
        let stats = verify_stats.clone();
        async move {
            let token = query.get("token").cloned().unwrap_or_default();
            let previous = stats.verifies.fetch_add(1, Ordering::SeqCst);
            // Single use: the second exchange fails, like the KV `delete` in the
            // real Worker.
            if token != "relay-token-1" || previous > 0 {
                return (StatusCode::UNAUTHORIZED, Json(json!({ "success": false })))
                    .into_response();
            }
            Json(json!({
                "success": true,
                "steamid": steam_id,
                "persona_name": "Relay Admin",
                "avatar": "https://avatars.steamstatic.com/small.jpg",
                "avatar_medium": "https://avatars.steamstatic.com/medium.jpg",
                "avatar_full": "https://avatars.steamstatic.com/full.jpg",
                "profile_url": format!("https://steamcommunity.com/profiles/{steam_id}"),
                "country_code": "CN",
                "steam_level": 42
            }))
            .into_response()
        }
    };

    let app = Router::new()
        .route("/login", get(login))
        .route("/verify", get(verify));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock relay");
    let address = listener.local_addr().expect("mock address");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    (format!("http://{address}"), stats)
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

async fn send(app: &Router, request: Request<Body>) -> axum::response::Response {
    app.clone().oneshot(request).await.expect("response")
}

fn location(response: &axum::response::Response) -> String {
    response
        .headers()
        .get(header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("Location header")
        .to_owned()
}

fn location_of(headers: &HeaderMap) -> String {
    headers
        .get(header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("Location header")
        .to_owned()
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

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&bytes).expect("json body")
}

fn query_param(url: &str, key: &str) -> Option<String> {
    reqwest::Url::parse(url)
        .expect("absolute URL")
        .query_pairs()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.into_owned())
}

async fn start_login(app: &Router, redirect: &str) -> String {
    let response = send(
        app,
        Request::builder()
            .uri(format!("/api/v1/auth/steam/start?redirect={redirect}"))
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    location(&response)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn relay_mode_round_trip_never_contacts_steam() {
    let Some((pool, database_url)) = create_database().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };
    let (relay, stats) = spawn_relay(ADMIN_STEAM_ID).await;
    let app = routes::build_router(relay_state(pool.clone(), &database_url, &relay));

    // 1. start: the browser is sent to the relay, not to Steam.
    let start = start_login(&app, "/admin/dashboard").await;
    assert!(start.starts_with(&format!("{relay}/login?")), "{start}");
    assert_eq!(query_param(&start, "mode").as_deref(), Some("login"));
    assert_eq!(
        query_param(&start, "return_to").as_deref(),
        Some("https://kz.test/api/v1/auth/steam/callback")
    );
    let state = query_param(&start, "state").expect("state");

    // 2. the browser hits the relay, which bounces back with a one-time token.
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("client");
    let bounced = client.get(&start).send().await.expect("relay login");
    assert_eq!(bounced.status(), 302);
    let back = location_of(bounced.headers());
    let back_url = reqwest::Url::parse(&back).expect("callback URL");
    assert_eq!(back_url.origin().ascii_serialization(), "https://kz.test");
    assert_eq!(back_url.path(), "/api/v1/auth/steam/callback");
    assert_eq!(query_param(&back, "state").as_deref(), Some(state.as_str()));
    let token = query_param(&back, "token").expect("relay token");
    assert_eq!(stats.logins.load(Ordering::SeqCst), 1);

    // 3. callback: exchange the token, mint the session.
    let response = send(
        &app,
        Request::builder()
            .uri(format!(
                "/api/v1/auth/steam/callback?token={token}&state={state}"
            ))
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&response), "https://kz.test/admin/dashboard");
    let cookie = session_cookie(&response).expect("session cookie");
    assert_eq!(stats.verifies.load(Ordering::SeqCst), 1);

    // 4. the Worker's profile data landed in the local profile.
    let response = send(
        &app,
        Request::builder()
            .uri("/api/v1/auth/me")
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    let body = body_json(response).await;
    assert_eq!(body["data"]["authenticated"], true);
    assert_eq!(body["data"]["user"]["steam_id64"], ADMIN_STEAM_ID);
    assert_eq!(body["data"]["user"]["display_name"], "Relay Admin");
    assert_eq!(
        body["data"]["user"]["avatar_url"],
        "https://avatars.steamstatic.com/full.jpg"
    );
    assert_eq!(body["data"]["user"]["is_admin"], true);

    // 5. the admin role travels with the relay session.
    let response = send(
        &app,
        Request::builder()
            .uri("/api/v1/admin/game-servers")
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    // 6. replaying the callback cannot mint a second session.
    let response = send(
        &app,
        Request::builder()
            .uri(format!(
                "/api/v1/auth/steam/callback?token={token}&state={state}"
            ))
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert!(location(&response).contains("auth_error=invalid_state"));
    assert_eq!(stats.verifies.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn relay_errors_are_mapped_to_stable_codes() {
    let Some((pool, database_url)) = create_database().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };
    let (relay, _) = spawn_relay(ADMIN_STEAM_ID).await;
    let app = routes::build_router(relay_state(pool.clone(), &database_url, &relay));

    // Cancel at Steam: the relay reports `steam_access_denied`.
    let state = query_param(&start_login(&app, "/players").await, "state").expect("state");
    let response = send(
        &app,
        Request::builder()
            .uri(format!(
                "/api/v1/auth/steam/callback?error=steam_access_denied&state={state}"
            ))
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(
        location(&response),
        "https://kz.test/players?auth_error=cancelled"
    );
    assert!(session_cookie(&response).is_none());

    // A refused verification is an assertion failure.
    let state = query_param(&start_login(&app, "/players").await, "state").expect("state");
    let response = send(
        &app,
        Request::builder()
            .uri(format!(
                "/api/v1/auth/steam/callback?error=steam_auth_failed&state={state}"
            ))
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert!(location(&response).contains("auth_error=invalid_assertion"));
}

#[tokio::test]
async fn an_unknown_relay_token_is_rejected() {
    let Some((pool, database_url)) = create_database().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };
    let (relay, stats) = spawn_relay(ADMIN_STEAM_ID).await;
    let app = routes::build_router(relay_state(pool.clone(), &database_url, &relay));

    let state = query_param(&start_login(&app, "/").await, "state").expect("state");
    let response = send(
        &app,
        Request::builder()
            .uri(format!(
                "/api/v1/auth/steam/callback?token=bogus&state={state}"
            ))
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert!(location(&response).contains("auth_error=invalid_assertion"));
    assert!(session_cookie(&response).is_none());
    assert_eq!(stats.verifies.load(Ordering::SeqCst), 1);
}
