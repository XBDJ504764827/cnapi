//! Tests against real PostgreSQL / Redis instances.
//!
//! They are skipped (with a message) when the corresponding environment
//! variable is missing, so `cargo test` also works on a machine without
//! infrastructure. Set `DATABASE_URL` and `REDIS_URL` to run them for real.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use cngokz_api::{config::AppConfig, routes, state, state::AppState};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::{AssertSqlSafe, PgPool, postgres::PgConnectOptions};
use tower::ServiceExt;

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.is_empty())
}

fn test_config() -> AppConfig {
    let mut pairs: Vec<(String, String)> = std::env::vars().collect();
    pairs.push(("APP_ENV".to_owned(), "test".to_owned()));
    pairs.push(("APP_VERSION".to_owned(), "0.0.0-test".to_owned()));
    if std::env::var("REDIS_URL").is_err() {
        pairs.push(("REDIS_URL".to_owned(), "redis://127.0.0.1:1".to_owned()));
    }
    AppConfig::from_pairs(pairs).expect("test configuration")
}

async fn connect_state() -> Option<AppState> {
    if database_url().is_none() {
        eprintln!("skipped: DATABASE_URL is not set");
        return None;
    }
    let state = state::connect(&test_config())
        .await
        .expect("state must initialize");
    Some(state)
}

async fn get_json(state: &AppState, path: &str) -> (StatusCode, Value) {
    let response = routes::build_router(state.clone())
        .oneshot(
            Request::builder()
                .uri(path)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("request must succeed");

    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    (status, serde_json::from_slice(&bytes).expect("json body"))
}

#[tokio::test]
async fn migrations_apply_to_a_fresh_database() {
    let Some(base_url) = database_url() else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };

    let admin = PgPool::connect(&base_url).await.expect("admin connection");
    let database_name = format!("cngokz_migrate_{}", uuid::Uuid::new_v4().simple());

    sqlx::raw_sql(AssertSqlSafe(format!(
        "CREATE DATABASE \"{database_name}\""
    )))
    .execute(&admin)
    .await
    .expect("temporary database");

    let options: PgConnectOptions = base_url.parse().expect("valid DATABASE_URL");
    let options = options.database(&database_name);
    let pool = PgPool::connect_with(options)
        .await
        .expect("connection to temporary database");

    let result = async {
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrations must succeed");

        let applied: i64 =
            sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations WHERE success")
                .fetch_one(&pool)
                .await
                .expect("migration table");
        assert!(applied >= 1, "expected at least one applied migration");

        let tables: Vec<String> = sqlx::query_scalar(
            "SELECT tablename::text FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename",
        )
        .fetch_all(&pool)
        .await
        .expect("table list");

        for expected in [
            "system_events",
            "app_settings",
            "accounts",
            "map_courses",
            "records",
            "jumpstats",
            "replays",
            "bans",
        ] {
            assert!(
                tables.contains(&expected.to_owned()),
                "missing table {expected}; tables: {tables:?}"
            );
        }

        // Phase 2 reference data must be seeded by the migrations.
        let modes: i64 = sqlx::query_scalar("SELECT count(*) FROM movement_modes")
            .fetch_one(&pool)
            .await
            .expect("movement modes");
        assert_eq!(modes, 4);

        let jump_types: i64 = sqlx::query_scalar("SELECT count(*) FROM jump_types")
            .fetch_one(&pool)
            .await
            .expect("jump types");
        assert_eq!(jump_types, 12);
    }
    .await;

    pool.close().await;
    let _ = sqlx::raw_sql(AssertSqlSafe(format!(
        "DROP DATABASE IF EXISTS \"{database_name}\" WITH (FORCE)"
    )))
    .execute(&admin)
    .await;
    admin.close().await;

    result
}

#[tokio::test]
async fn readiness_is_ready_when_dependencies_are_reachable() {
    if database_url().is_none() || std::env::var("REDIS_URL").is_err() {
        eprintln!("skipped: DATABASE_URL and REDIS_URL are required");
        return;
    }

    let Some(state) = connect_state().await else {
        return;
    };
    let (status, body) = get_json(&state, "/health/ready").await;

    assert_eq!(status, StatusCode::OK, "unexpected body: {body}");
    assert_eq!(body["data"]["status"], "ready");
    assert_eq!(body["data"]["checks"].as_array().map(Vec::len), Some(2));

    let (status, body) = get_json(&state, "/api/v1/system/status").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["status"], "healthy", "unexpected body: {body}");
}

#[tokio::test]
async fn system_events_are_read_from_the_database() {
    let Some(state) = connect_state().await else {
        return;
    };

    sqlx::query("DELETE FROM system_events WHERE event_type = $1")
        .bind("test.integration")
        .execute(state.db())
        .await
        .expect("cleanup");

    // A fresh installation returns an empty list, no fabricated rows.
    let (status, body) = get_json(&state, "/api/v1/system/events?limit=10").await;
    assert_eq!(status, StatusCode::OK, "unexpected body: {body}");
    assert_eq!(body["data"]["count"], 0);
    assert!(body["data"]["items"].as_array().is_some_and(Vec::is_empty));

    sqlx::query(
        "INSERT INTO system_events (event_type, severity, message, metadata) VALUES ($1, $2, $3, $4)",
    )
    .bind("test.integration")
    .bind("info")
    .bind("integration test event")
    .bind(serde_json::json!({ "source": "cargo test" }))
    .execute(state.db())
    .await
    .expect("insert event");

    let (status, body) = get_json(&state, "/api/v1/system/events?limit=10").await;
    assert_eq!(status, StatusCode::OK, "unexpected body: {body}");
    assert_eq!(body["data"]["count"], 1);
    assert_eq!(body["data"]["items"][0]["event_type"], "test.integration");
    assert_eq!(body["data"]["items"][0]["metadata"]["source"], "cargo test");

    sqlx::query("DELETE FROM system_events WHERE event_type = $1")
        .bind("test.integration")
        .execute(state.db())
        .await
        .expect("cleanup");

    state.db().close().await;
}
