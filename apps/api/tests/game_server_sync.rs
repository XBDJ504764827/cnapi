//! Phase-7 end-to-end tests for the CS:GO game-server ingestion API.
//!
//! Each test creates a temporary database and exercises the real Axum router,
//! real migrations, real PostgreSQL triggers and the real token hashing.

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use cngokz_api::{config::AppConfig, routes, state};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::{AssertSqlSafe, PgPool, postgres::PgConnectOptions};
use tower::ServiceExt;

const MAINTENANCE: &str = "test-maintenance-token";

struct TestDatabase {
    base_url: String,
    pool: PgPool,
    name: String,
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        let base_url = self.base_url.clone();
        let name = self.name.clone();
        let cleanup = std::thread::spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            runtime.block_on(async move {
                let Ok(admin) = PgPool::connect(&base_url).await else {
                    return;
                };
                let _ = sqlx::raw_sql(AssertSqlSafe(format!(
                    "DROP DATABASE IF EXISTS \"{name}\" WITH (FORCE)"
                )))
                .execute(&admin)
                .await;
                admin.close().await;
            });
        });
        let _ = cleanup.join();
    }
}

async fn fixture_db() -> Option<(TestDatabase, String, Router)> {
    let base_url = std::env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.is_empty())?;
    let admin = PgPool::connect(&base_url).await.expect("admin connection");
    let name = format!("cngokz_sync_{}", uuid::Uuid::new_v4().simple());
    sqlx::raw_sql(AssertSqlSafe(format!("CREATE DATABASE \"{name}\"")))
        .execute(&admin)
        .await
        .expect("create database");
    let options: PgConnectOptions = base_url.parse().expect("valid DATABASE_URL");
    let pool = PgPool::connect_with(options.database(&name))
        .await
        .expect("fixture db");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations");
    let url = match base_url.rsplit_once('/') {
        Some((prefix, _)) => format!("{prefix}/{name}"),
        None => base_url.clone(),
    };
    let config = AppConfig::from_pairs([
        ("APP_ENV", "test"),
        ("APP_VERSION", "0.0.0-test"),
        ("DATABASE_URL", url.as_str()),
        ("REDIS_URL", "redis://127.0.0.1:1"),
        ("MAINTENANCE_TOKEN", MAINTENANCE),
        ("HEALTH_CHECK_TIMEOUT_MS", "300"),
    ])
    .expect("config");
    let redis = state::RedisClient::new(&config.redis_url, "test").expect("redis client");
    let app = routes::build_router(state::AppState::new(config, pool.clone(), redis));
    Some((
        TestDatabase {
            base_url,
            pool,
            name,
        },
        url,
        app,
    ))
}

async fn call(
    app: &Router,
    method: &str,
    path: &str,
    token: Option<&str>,
    maintenance: bool,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        builder = builder.header("x-server-token", token);
    }
    if maintenance {
        builder = builder.header("x-maintenance-token", MAINTENANCE);
    }
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    let request = builder
        .body(body.map_or_else(Body::empty, |value| Body::from(value.to_string())))
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "invalid JSON response ({error}): {}",
            String::from_utf8_lossy(&bytes)
        )
    });
    (status, json)
}

async fn create_server(app: &Router, name: &str, ip: &str, port: i32) -> (i64, String) {
    let (status, body) = call(
        app,
        "POST",
        "/api/v1/admin/game-servers",
        None,
        true,
        Some(json!({
            "name": name,
            "ip": ip,
            "port": port,
            "query_port": port + 1000,
            "region": "eu",
            "server_type": "community",
            "max_players": 32,
            "owner_steam_id64": "76561198000000001",
            "owner_display_name": "Owner"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let token = body["data"]["token"].as_str().expect("token").to_owned();
    assert!(token.starts_with("cngokz_srv_"));
    assert!(!body["data"]["server"].to_string().contains(&token));
    (body["data"]["server"]["id"].as_i64().expect("id"), token)
}

async fn heartbeat(app: &Router, token: &str, map: &str, online: i64) -> (StatusCode, Value) {
    call(
        app,
        "POST",
        "/api/v1/game-servers/heartbeat",
        Some(token),
        false,
        Some(json!({
            "server_name": "CS:GO KZ",
            "map": map,
            "mode": "KZT",
            "online_players": online,
            "max_players": 32,
            "plugin_version": "1.0.0",
            "game_version": "1.38.7.3"
        })),
    )
    .await
}

async fn seed_map(pool: &PgPool) {
    sqlx::raw_sql(
        r#"
        INSERT INTO maps (name, status, approved_at)
        VALUES ('kz_sync_test', 'approved', now());
        INSERT INTO map_courses (map_id, stage, name)
        SELECT id, 0, 'Main' FROM maps WHERE name = 'kz_sync_test';
        INSERT INTO map_modes (map_id, mode_id)
        SELECT m.id, mm.id FROM maps m CROSS JOIN movement_modes mm
         WHERE m.name = 'kz_sync_test' AND mm.code IN ('vanilla', 'simple', 'kztimer');
        INSERT INTO course_tiers (course_id, mode_id, nub_tier, pro_tier, status)
        SELECT c.id, mm.id, 4, 5, 'ranked'
          FROM map_courses c CROSS JOIN movement_modes mm
          JOIN maps m ON m.id = c.map_id
         WHERE m.name = 'kz_sync_test' AND mm.code = 'kztimer';
        INSERT INTO course_mode_tiers (map_id, course_id, mode_id, tier)
        SELECT c.map_id, c.id, mm.id, 5
          FROM map_courses c CROSS JOIN movement_modes mm
          JOIN maps m ON m.id = c.map_id
         WHERE m.name = 'kz_sync_test' AND mm.code = 'kztimer';
        "#,
    )
    .execute(pool)
    .await
    .expect("fixtures");
}

fn record_body(player: &str, identifier: &str, seconds: f64, teleports: i32) -> Value {
    json!({
        "steam_id64": player,
        "player_name": "Runner",
        "map": "kz_sync_test",
        "course": 0,
        "mode": "KZT",
        "style": "normal",
        "time": seconds,
        "tickrate": 128,
        "teleports": teleports,
        "record_identifier": identifier,
        "plugin_version": "1.0.0"
    })
}

#[tokio::test]
async fn admin_auth_heartbeat_and_token_lifecycle() {
    let Some((_db, _url, app)) = fixture_db().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/admin/game-servers",
        None,
        false,
        Some(json!({
            "name": "Unauthorized",
            "ip": "127.0.0.1",
            "port": 27015,
            "owner_steam_id64": "76561198000000001"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");

    let (server_id, token) = create_server(&app, "CNGOKZ EU 01", "127.0.0.1", 27015).await;

    let (status, _) = heartbeat(&app, "wrong-token", "kz_sync_test", 1).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, body) = heartbeat(&app, &token, "kz_sync_test", 3).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["status"], "online");
    assert_eq!(body["data"]["heartbeat_interval_seconds"], 25);

    let (status, body) = call(
        &app,
        "GET",
        "/api/v1/admin/game-servers?page=1&page_size=10",
        None,
        true,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["meta"]["total"], 1);
    assert_eq!(body["data"]["items"][0]["status"], "online");
    assert_eq!(body["data"]["items"][0]["online_players"], 3);

    let (status, body) = call(
        &app,
        "POST",
        &format!("/api/v1/admin/game-servers/{server_id}/token/rotate"),
        None,
        true,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let new_token = body["data"]["token"].as_str().expect("rotated token");

    let (status, _) = heartbeat(&app, &token, "kz_sync_test", 1).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "old token must be invalid"
    );
    let (status, _) = heartbeat(&app, new_token, "kz_sync_test", 1).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/admin/game-servers/{server_id}/disable"),
        None,
        true,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = heartbeat(&app, new_token, "kz_sync_test", 1).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/admin/game-servers/{server_id}/token/revoke"),
        None,
        true,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn players_are_isolated_per_server_and_become_publicly_online() {
    let Some((_db, _url, app)) = fixture_db().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };
    let (first_id, first_token) = create_server(&app, "Server A", "127.0.0.1", 27015).await;
    let (second_id, second_token) = create_server(&app, "Server B", "127.0.0.2", 27016).await;
    let _ = heartbeat(&app, &first_token, "kz_sync_test", 1).await;
    let _ = heartbeat(&app, &second_token, "kz_sync_test", 1).await;

    let player = json!({
        "steam_id64": "76561198000000042",
        "display_name": "Online Player",
        "map": "kz_sync_test",
        "mode": "KZT",
        "status": "connected"
    });
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/game-servers/players/connect",
        Some(&first_token),
        false,
        Some(player.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/game-servers/players/connect",
        Some(&second_token),
        false,
        Some(player),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // Full calibration on Server A sees no players and must not clear Server B.
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/game-servers/players/sync",
        Some(&first_token),
        false,
        Some(json!({ "players": [] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["disconnected"], 1);
    assert_eq!(body["data"]["online"], 0);

    let (status, body) = call(
        &app,
        "GET",
        "/api/v1/public/players/online",
        None,
        false,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["count"], 1, "{body}");
    assert_eq!(body["data"]["items"][0]["server_id"], second_id);
    assert_eq!(
        body["data"]["items"][0]["player"]["steam_id64"],
        "76561198000000042"
    );

    let (status, body) = call(
        &app,
        "GET",
        &format!("/api/v1/admin/game-servers/{first_id}"),
        None,
        true,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["server"]["online_players"], 0);
    assert!(
        !body["data"]["recent_heartbeats"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/game-servers/players/disconnect",
        Some(&second_token),
        false,
        Some(json!({ "steam_id64": "76561198000000042" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["disconnected"], 1);
}

#[tokio::test]
async fn record_idempotency_pb_wr_points_and_replay() {
    let Some((db, _url, app)) = fixture_db().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };
    seed_map(&db.pool).await;
    let (server_id, token) = create_server(&app, "Records Server", "127.0.0.1", 27015).await;
    let _ = heartbeat(&app, &token, "kz_sync_test", 1).await;

    let first = record_body("76561198000000010", "run-1", 10.0, 0);
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/game-servers/records",
        Some(&token),
        false,
        Some(first.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["duplicate"], false);
    assert_eq!(body["data"]["is_pb"], true);
    assert_eq!(body["data"]["is_world_record"], true);
    assert_eq!(body["data"]["points_status"], "queued");
    let record_id = body["data"]["record_id"].as_i64().expect("record id");

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/game-servers/records",
        Some(&token),
        false,
        Some(first),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["duplicate"], true);
    assert_eq!(body["data"]["record_id"], record_id);
    let rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM records WHERE server_id = $1 AND submission_key = 'run-1'",
    )
    .bind(server_id)
    .fetch_one(&db.pool)
    .await
    .expect("count");
    assert_eq!(rows, 1);

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/game-servers/records",
        Some(&token),
        false,
        Some(record_body("76561198000000011", "run-2", 12.0, 0)),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["is_pb"], true);
    assert_eq!(body["data"]["is_world_record"], false);

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/game-servers/records",
        Some(&token),
        false,
        Some(record_body("76561198000000010", "run-3", 11.0, 0)),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["is_pb"], false, "10s remains the PB");
    assert_eq!(body["data"]["is_world_record"], false);

    let wr_record: i64 = sqlx::query_scalar(
        "SELECT record_id FROM course_world_records WHERE course_id = (SELECT id FROM map_courses WHERE map_id = (SELECT id FROM maps WHERE name = 'kz_sync_test')) AND time_type = 'pro'",
    )
    .fetch_one(&db.pool)
    .await
    .expect("world record");
    assert_eq!(wr_record, record_id);

    let tasks: i64 =
        sqlx::query_scalar("SELECT count(*) FROM filter_recalc_tasks WHERE status = 'pending'")
            .fetch_one(&db.pool)
            .await
            .expect("recalc tasks");
    assert!(tasks >= 1, "record insert must enqueue a Filter recalc");

    let checksum = "ab".repeat(32);
    let replay = json!({
        "record_identifier": "run-1",
        "bucket": "replays",
        "object_key": "records/run-1.replay",
        "storage_provider": "r2",
        "size_bytes": 2048,
        "checksum_sha256": checksum,
        "replay_format_version": 2,
        "status": "available"
    });
    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/game-servers/replays",
        Some(&token),
        false,
        Some(replay.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["duplicate"], false);
    assert_eq!(body["data"]["replay_available"], true);

    let (status, body) = call(
        &app,
        "POST",
        "/api/v1/game-servers/replays",
        Some(&token),
        false,
        Some(replay),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["duplicate"], true);

    let (status, body) = call(
        &app,
        "GET",
        "/api/v1/game-servers/records/status?record_identifier=run-1",
        Some(&token),
        false,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["replay_status"], "available");
    assert_eq!(body["data"]["is_world_record"], true);

    let (status, body) = call(
        &app,
        "GET",
        "/api/v1/game-servers/config/sync?version=0",
        Some(&token),
        false,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["changed"], true);
    let version = body["data"]["version"].as_i64().expect("version");
    assert!(
        body["data"]["maps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|map| map["name"] == "kz_sync_test")
    );
    let (status, body) = call(
        &app,
        "GET",
        &format!("/api/v1/game-servers/config/sync?version={version}"),
        Some(&token),
        false,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["changed"], false);

    let (status, _) = call(
        &app,
        "POST",
        &format!("/api/v1/admin/game-servers/{server_id}/disable"),
        None,
        true,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = call(
        &app,
        "POST",
        "/api/v1/game-servers/records",
        Some(&token),
        false,
        Some(record_body("76561198000000012", "run-disabled", 9.0, 0)),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
