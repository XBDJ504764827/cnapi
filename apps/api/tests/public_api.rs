//! End-to-end tests for the public portal read API.
//!
//! Creates a temporary database, applies migrations, inserts deterministic
//! fixtures and exercises every public endpoint through the real router.
//! Skipped when DATABASE_URL is not configured (same convention as the other
//! integration tests).

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use cngokz_api::{
    config::AppConfig,
    routes,
    services::public as public_service,
    state::{self, AppState},
};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::{AssertSqlSafe, PgPool, postgres::PgConnectOptions};
use tower::ServiceExt;

const FIXTURES: &str = r#"
INSERT INTO accounts (steam_id64) VALUES
  (76561198000000001), (76561198000000002), (76561198000000003);
INSERT INTO player_profiles (account_id, display_name, country) VALUES
  (1, 'Alpha', 'SE'), (2, 'Bravo', 'DE'), (3, 'Charlie', 'US');

INSERT INTO maps (name, workshop_id, description, status, approved_at) VALUES
  ('kz_fixture_alpha', 111111111, 'Alpha fixture map', 'approved', now() - interval '10 days'),
  ('kz_fixture_beta', 222222222, 'Beta fixture map', 'approved', now() - interval '5 days');

INSERT INTO map_courses (map_id, stage, name) VALUES
  (1, 0, 'Main'), (1, 1, 'Bonus 1'), (2, 0, 'Main');

INSERT INTO map_modes (map_id, mode_id) VALUES (1, 1), (1, 2), (2, 1), (2, 2);
INSERT INTO course_tiers (course_id, mode_id, nub_tier, pro_tier, status) VALUES
  (1, 1, 3, 4, 'ranked'),
  (2, 1, 2, 3, 'ranked'),
  (3, 1, 5, 6, 'ranked'),
  (1, 2, 2, 3, 'ranked');

INSERT INTO servers (name, ip, port, owner_account_id, api_key_hash, api_key_prefix, plugin_version, current_map_id, is_online, last_heartbeat_at, approved_at)
VALUES
  ('Alpha Server', '10.0.0.1', 27015, 1, sha256('key-a'), 'aaaa1111', '1.2.3', 1, true, now() - interval '5 seconds', now() - interval '9 days'),
  ('Beta Server', '10.0.0.2', 27016, 2, sha256('key-b'), 'bbbb2222', '1.2.2', 2, false, now() - interval '2 hours', now() - interval '4 days');

INSERT INTO records (account_id, map_id, course_id, mode_id, style_id, server_id, tickrate, teleports, time_type, time_ns, status, source, created_at)
VALUES
  (1, 1, 1, 1, 1, 1, 128, 0, 'pro', 12500000000, 'approved', 'plugin', now() - interval '6 days'),
  (2, 1, 1, 1, 1, 1, 128, 0, 'pro', 13200000000, 'approved', 'plugin', now() - interval '5 days'),
  (1, 1, 1, 1, 1, 1, 128, 2, 'nub', 14000000000, 'approved', 'plugin', now() - interval '4 days'),
  (2, 1, 1, 1, 1, 1, 128, 0, 'pro', 13000000000, 'approved', 'plugin', now() - interval '3 days'),
  (1, 1, 2, 1, 1, 1, 128, 0, 'pro', 5000000000, 'approved', 'plugin', now() - interval '2 days'),
  (2, 2, 3, 1, 1, 2, 128, 0, 'pro', 20000000000, 'approved', 'plugin', now() - interval '1 day'),
  (1, 2, 3, 1, 1, 2, 128, 0, 'pro', 21000000000, 'approved', 'plugin', now() - interval '2 hours'),
  (3, 2, 3, 1, 1, 2, 128, 3, 'nub', 22000000000, 'approved', 'plugin', now() - interval '1 hour'),
  (3, 1, 1, 1, 1, 2, 128, 4, 'nub', 16000000000, 'approved', 'plugin', now() - interval '30 minutes'),
  (1, 1, 1, 2, 1, 1, 128, 0, 'pro', 11000000000, 'approved', 'plugin', now() - interval '3 hours');

INSERT INTO course_world_records (course_id, mode_id, style_id, time_type, record_id, time_ns, achieved_at) VALUES
  (1, 1, 1, 'pro', 1, 12500000000, now() - interval '6 days'),
  (1, 1, 1, 'nub', 3, 14000000000, now() - interval '4 days'),
  (2, 1, 1, 'pro', 5, 5000000000, now() - interval '2 days'),
  (3, 1, 1, 'pro', 6, 20000000000, now() - interval '1 day'),
  (3, 1, 1, 'nub', 8, 22000000000, now() - interval '1 hour'),
  (1, 2, 1, 'pro', 10, 11000000000, now() - interval '3 hours');

INSERT INTO replays (record_id, account_id, map_id, storage_provider, bucket, object_key, size_bytes, checksum_sha256, replay_format_version, status, uploaded_at, created_at)
VALUES (1, 1, 1, 'r2', 'fixtures', 'records/1/replay2.replay', 1024, sha256('fixture'), 2, 'available', now() - interval '6 days', now() - interval '6 days');
"#;

async fn create_fixture_database() -> Option<(PgPool, PgPool, String, String)> {
    let base_url = std::env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.is_empty())?;
    let admin = PgPool::connect(&base_url).await.expect("admin connection");
    let database_name = format!("cngokz_public_{}", uuid::Uuid::new_v4().simple());

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
    sqlx::raw_sql(AssertSqlSafe(FIXTURES.to_owned()))
        .execute(&pool)
        .await
        .expect("fixtures");

    let database_url = match base_url.rsplit_once('/') {
        Some((prefix, _)) => format!("{prefix}/{database_name}"),
        None => base_url.clone(),
    };

    Some((admin, pool, database_name, database_url))
}

fn test_state(pool: PgPool, database_url: &str) -> AppState {
    let config = AppConfig::from_pairs([
        ("APP_ENV", "test".to_owned()),
        ("APP_VERSION", "0.0.0-test".to_owned()),
        ("DATABASE_URL", database_url.to_owned()),
        ("REDIS_URL", "redis://127.0.0.1:1".to_owned()),
        ("HEALTH_CHECK_TIMEOUT_MS", "300".to_owned()),
    ])
    .expect("config");

    // Cache is unavailable on port 1 on purpose: every request must fall back to
    // PostgreSQL, which is what this test verifies.
    let redis = state::RedisClient::new(&config.redis_url, "test").expect("redis client");
    AppState::new(config.clone(), pool, redis)
}

async fn get_json(app: &Router, path: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(path)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    (status, serde_json::from_slice(&bytes).expect("json"))
}

#[tokio::test]
async fn public_portal_endpoints_work_against_real_schema() {
    let Some((admin, pool, database_name, database_url)) = create_fixture_database().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };

    let app = routes::build_router(test_state(pool.clone(), &database_url));

    // statistics
    let (status, body) = get_json(&app, "/api/v1/public/statistics").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["players"], 3);
    assert_eq!(body["data"]["maps"], 2);
    assert_eq!(body["data"]["records"], 10);
    assert_eq!(body["data"]["world_records"], 6);
    assert_eq!(body["data"]["servers_online"], 1);

    // leaderboard: Alpha and Bravo both completed 2 maps; Alpha has more WRs.
    let (status, body) = get_json(&app, "/api/v1/public/leaderboards?mode=vanilla").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let items = body["data"]["items"].as_array().expect("items");
    assert_eq!(items.len(), 3);
    assert_eq!(items[0]["display_name"], "Alpha");
    assert_eq!(items[0]["rank"], 1);
    assert_eq!(items[0]["wr_count"], 3);
    assert!(items[0]["points"].is_null());
    assert_eq!(body["data"]["points_available"], false);
    assert_eq!(body["data"]["ranking_metric"], "completed_maps");

    // player directory: optional mode + sorting
    let (status, body) = get_json(&app, "/api/v1/public/players?sort=records&page_size=10").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["meta"]["total"], 3);
    assert_eq!(body["data"]["items"][0]["display_name"], "Alpha");
    assert!(body["data"]["items"][0]["rank"].is_number());

    // player detail: overall + per-mode + tiers
    let (status, body) = get_json(&app, "/api/v1/public/players/76561198000000001").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["overall"]["completed_maps"], 2);
    assert_eq!(body["data"]["overall"]["total_maps"], 2);
    assert_eq!(body["data"]["overall"]["wr_count"], 4);
    assert_eq!(body["data"]["overall"]["global_rank"], 1);
    assert!(body["data"]["overall"]["points"].is_null());
    assert_eq!(body["data"]["modes"].as_array().map(Vec::len), Some(4));
    assert_eq!(body["data"]["tiers"].as_array().map(Vec::len), Some(2));
    assert_eq!(body["data"]["tiers"][0]["tier"], 3);
    assert_eq!(body["data"]["tiers"][1]["tier"], 5);

    // player map progress: completed and uncompleted (main course only)
    let (status, body) = get_json(
        &app,
        "/api/v1/public/players/76561198000000001/maps?mode=vanilla&time_type=pro",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["meta"]["total"], 2);
    assert_eq!(body["data"]["items"][0]["completed"], true);
    assert_eq!(body["data"]["items"][0]["bonus_total"], 1);
    assert_eq!(body["data"]["items"][0]["bonus_completed"], 1);
    assert_eq!(body["data"]["items"][0]["best_time_seconds"], "12.500");

    let (status, body) = get_json(
        &app,
        "/api/v1/public/players/76561198000000003/maps?mode=vanilla&time_type=pro&status=uncompleted",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["meta"]["total"], 2);
    assert!(
        body["data"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["completed"] == false)
    );

    // player records
    let (status, body) = get_json(
        &app,
        "/api/v1/public/players/76561198000000001/records?mode=vanilla&page_size=20",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["meta"]["total"], 4);

    // compare: Alpha vs Bravo, PRO, vanilla
    let (status, body) = get_json(
        &app,
        "/api/v1/public/players/compare?left=76561198000000001&right=76561198000000002&mode=vanilla&time_type=pro",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["maps"].as_array().map(Vec::len), Some(2));
    assert_eq!(body["data"]["maps"][0]["status"], "both");
    assert_eq!(body["data"]["maps"][0]["time_diff_ns"], -500000000);
    assert_eq!(body["data"]["tiers"].as_array().map(Vec::len), Some(2));

    // maps directory
    let (status, body) = get_json(&app, "/api/v1/public/maps?mode=vanilla").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["meta"]["total"], 2);
    assert_eq!(body["data"]["items"][0]["completed_players"], 3);
    assert_eq!(body["data"]["items"][0]["tier"], 3);
    assert_eq!(body["data"]["items"][0]["wr_player_name"], "Alpha");

    // map detail + statistics
    let (status, body) = get_json(&app, "/api/v1/public/maps/kz_fixture_alpha?mode=vanilla").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["courses"].as_array().map(Vec::len), Some(2));
    assert_eq!(body["data"]["statistics"]["completed_players"], 3);
    assert_eq!(body["data"]["statistics"]["pro_completed_players"], 2);
    assert_eq!(body["data"]["statistics"]["nub_completed_players"], 2);
    assert_eq!(body["data"]["map"]["tier"], 3);

    let (status, body) = get_json(
        &app,
        "/api/v1/public/maps/kz_fixture_alpha/statistics?mode=vanilla",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["record_count"], 6);

    // map leaderboard: vanilla PRO main course
    let (status, body) = get_json(
        &app,
        "/api/v1/public/maps/kz_fixture_alpha/leaderboard?mode=vanilla&time_type=pro&stage=0",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let items = body["data"]["items"].as_array().expect("items");
    assert_eq!(items.len(), 3);
    assert_eq!(items[0]["player"]["display_name"], "Alpha");
    assert_eq!(items[0]["time_seconds"], "12.500000000");
    assert_eq!(items[0]["rank"], 1);
    assert_eq!(items[0]["is_world_record"], true);

    // records: tier filter uses course_tiers for the record time type
    let (status, body) = get_json(
        &app,
        "/api/v1/public/records?mode=vanilla&time_type=pro&tier=4",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["meta"]["total"], 3);
    assert!(
        body["data"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["tier"] == 4)
    );

    // recent uploads are ordered by created_at desc
    let (status, body) = get_json(&app, "/api/v1/public/records/recent?limit=5").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["count"], 5);
    assert_eq!(body["data"]["items"][0]["id"], 9);

    // world records + active players
    let (status, body) = get_json(&app, "/api/v1/public/records/world-records?limit=3").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"].as_array().map(Vec::len), Some(3));

    let (status, body) = get_json(&app, "/api/v1/public/players/active?days=7&limit=5").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["items"][0]["player"]["display_name"], "Alpha");
    assert_eq!(body["data"]["items"][0]["records"], 5);

    // servers + search + record detail
    let (status, body) = get_json(&app, "/api/v1/public/servers?online=true").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["items"].as_array().map(Vec::len), Some(1));
    assert!(body["data"]["items"][0]["players"].is_null());

    let (status, body) = get_json(&app, "/api/v1/public/search?q=fixture").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["maps"].as_array().map(Vec::len), Some(2));

    let (status, body) = get_json(&app, "/api/v1/public/search?q=Alpha").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["players"][0]["display_name"], "Alpha");

    let (status, body) = get_json(&app, "/api/v1/public/records/1").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["record"]["replay_available"], true);
    assert_eq!(body["data"]["replay_id"], 1);
    assert_eq!(body["data"]["server"]["name"], "Alpha Server");

    // validation and not-found semantics
    let (status, body) = get_json(&app, "/api/v1/public/players?page_size=1000").await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["error"]["code"], "validation_error");

    let (status, _) = get_json(&app, "/api/v1/public/players/76561198000000009").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = get_json(&app, "/api/v1/public/leaderboards?mode=nope").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // cleanup
    pool.close().await;
    sqlx::raw_sql(AssertSqlSafe(format!(
        "DROP DATABASE IF EXISTS \"{database_name}\" WITH (FORCE)"
    )))
    .execute(&admin)
    .await
    .expect("drop temporary database");
    admin.close().await;
}

/// Phase 5 integration checks: cache namespace invalidation, rate limiting,
/// server heartbeat classification and the explicit player stat scope.
#[tokio::test]
async fn phase5_cache_rate_limit_and_scope() {
    let Some((admin, pool, database_name, database_url)) = create_fixture_database().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };

    // A dedicated config with a tiny public rate limit and a maintenance token.
    let config = AppConfig::from_pairs([
        ("APP_ENV", "test".to_owned()),
        ("APP_VERSION", "0.0.0-test".to_owned()),
        ("DATABASE_URL", database_url.clone()),
        ("REDIS_URL", "redis://127.0.0.1:56379".to_owned()),
        ("PUBLIC_RATE_LIMIT_PER_MINUTE", "3".to_owned()),
        ("MAINTENANCE_TOKEN", "phase5-test-token".to_owned()),
    ])
    .expect("config");
    let redis = state::RedisClient::new(&config.redis_url, "test-phase5").expect("redis client");
    let app_state = AppState::new(config, pool.clone(), redis);
    let app = routes::build_router(app_state.clone());

    async fn get_with(
        app: &Router,
        path: &str,
        headers: &[(&str, &str)],
    ) -> (StatusCode, Value, axum::http::HeaderMap) {
        let mut builder = Request::builder().uri(path);
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        let response = app
            .clone()
            .oneshot(builder.body(Body::empty()).expect("request"))
            .await
            .expect("response");
        let status = response.status();
        let response_headers = response.headers().clone();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).expect("json"),
            response_headers,
        )
    }

    // Same-origin requests share one peer key; 4th request is rejected.
    let mut rate_limited = false;
    for _ in 0..4 {
        let (status, body, headers) = get_with(&app, "/api/v1/public/modes", &[]).await;
        if status == StatusCode::TOO_MANY_REQUESTS {
            assert_eq!(body["error"]["code"], "rate_limited");
            assert!(headers.contains_key("retry-after"));
            rate_limited = true;
            break;
        }
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    assert!(rate_limited, "public rate limiter did not trigger");

    // Health probes are outside the limiter and still work.
    let (status, body, _) = get_with(&app, "/health", &[]).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // Build a fresh router (and thus a fresh in-memory limiter) for the
    // remaining functional assertions.
    let app = routes::build_router(app_state.clone());

    // Cache maintenance requires a token.
    let (status, _, _) = get_with(&app, "/api/v1/public/maintenance/cache/version", &[]).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Initialise the namespace version so that a later INCR is observable even
    // when Redis starts empty (missing version defaults to "1").
    let auth = [("x-maintenance-token", "phase5-test-token")];
    let init = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/public/maintenance/cache/invalidate")
                .header("x-maintenance-token", "phase5-test-token")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(init.status(), StatusCode::OK);

    // Authorized version read + invalidation + hot statistics warm.
    let (status, body, _) = get_with(&app, "/api/v1/public/maintenance/cache/version", &auth).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let version_before: i64 = body["data"]["version"]
        .as_str()
        .unwrap_or("1")
        .parse()
        .expect("numeric version");

    // Prime the statistics cache without rate limiting (direct service call).
    let stats_before = public_service::statistics(&app_state)
        .await
        .expect("statistics");
    assert_eq!(stats_before.players, 3);

    let invalidate = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/public/maintenance/cache/invalidate")
                .header("x-maintenance-token", "phase5-test-token")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(invalidate.status(), StatusCode::OK);
    let bytes = invalidate
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let body: Value = serde_json::from_slice(&bytes).expect("json");
    eprintln!("invalidate response: {body}");
    assert_eq!(body["data"]["invalidated"], true);
    let version_after: i64 = body["data"]["version"]
        .as_str()
        .unwrap_or("1")
        .parse()
        .expect("numeric version");
    assert!(
        version_after > version_before,
        "cache namespace version must advance: {version_before} -> {version_after}"
    );

    // Player detail now exposes an explicit scope; scoped and unscoped numbers
    // are both present and clearly labelled.
    let (status, body, _) = get_with(&app, "/api/v1/public/players/76561198000000001", &[]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["overall"]["scoped"], false);
    assert_eq!(body["data"]["scoped"]["scoped"], true);
    assert_eq!(body["data"]["scope"]["mode"]["code"], "vanilla");
    assert_eq!(body["data"]["scope"]["style"]["code"], "all");
    assert_eq!(body["data"]["scope"]["time_type"], "all");
    assert_eq!(body["data"]["tiers"].as_array().map(Vec::len), Some(2));

    // Server list status classification.
    let (status, body, _) = get_with(&app, "/api/v1/public/servers", &[]).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let servers = body["data"]["items"].as_array().expect("servers");
    let alpha = servers
        .iter()
        .find(|item| item["name"] == "Alpha Server")
        .expect("alpha server");
    let beta = servers
        .iter()
        .find(|item| item["name"] == "Beta Server")
        .expect("beta server");
    assert_eq!(alpha["status"], "online");
    assert_eq!(beta["status"], "expired");
    assert!(alpha["heartbeat_age_seconds"].is_number());
    assert!(alpha["players"].is_null());

    pool.close().await;
    sqlx::raw_sql(AssertSqlSafe(format!(
        "DROP DATABASE IF EXISTS \"{database_name}\" WITH (FORCE)"
    )))
    .execute(&admin)
    .await
    .expect("drop temporary database");
    admin.close().await;
}
