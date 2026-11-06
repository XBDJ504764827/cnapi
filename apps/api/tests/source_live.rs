//! Optional live integration test against the real kzgo.eu API.
//!
//! Run explicitly:
//!   DATABASE_URL=... cargo test -p cngokz-api --test source_live -- --ignored --nocapture

use axum::Router;
use cngokz_api::{config::AppConfig, routes, services::sources, state};
use sqlx::{AssertSqlSafe, PgPool, postgres::PgConnectOptions};

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

async fn fixture() -> Option<(TestDatabase, state::AppState, Router)> {
    let base_url = std::env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.is_empty())?;
    let admin = PgPool::connect(&base_url).await.expect("admin");
    let name = format!("cngokz_live_{}", uuid::Uuid::new_v4().simple());
    sqlx::raw_sql(AssertSqlSafe(format!("CREATE DATABASE \"{name}\"")))
        .execute(&admin)
        .await
        .expect("create db");
    let options: PgConnectOptions = base_url.parse().expect("url");
    let pool = PgPool::connect_with(options.database(&name))
        .await
        .expect("db");
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
        ("DATABASE_URL", url.as_str()),
        ("REDIS_URL", "redis://127.0.0.1:1"),
        ("KZGO_SYNC_ENABLED", "true"),
        ("GOKZ_GLOBAL_SYNC_ENABLED", "false"),
        ("SOURCE_HTTP_TIMEOUT_MS", "30000"),
    ])
    .expect("config");
    let redis = state::RedisClient::new(&config.redis_url, "test").expect("redis");
    let app_state = state::AppState::new(config, pool.clone(), redis);
    let router = routes::build_router(app_state.clone());
    Some((
        TestDatabase {
            base_url,
            pool,
            name,
        },
        app_state,
        router,
    ))
}

#[tokio::test]
#[ignore = "requires live network access to kzgo.eu"]
async fn kzgo_live_sync_imports_real_maps_wrs_and_servers() {
    let Some((db, app, _router)) = fixture().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };
    let result = sources::sync_kzgo(&app, "manual").await.expect("live sync");
    eprintln!("{}", serde_json::to_string_pretty(&result.stats).unwrap());
    assert_eq!(result.status, "success");
    assert!(result.stats["maps_seen"].as_u64().unwrap_or(0) > 500);
    assert!(result.stats["wrs_seen"].as_u64().unwrap_or(0) > 500);
    assert!(result.stats["servers_seen"].as_u64().unwrap_or(0) > 0);

    let maps: i64 = sqlx::query_scalar("SELECT count(*) FROM maps")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    let records: i64 = sqlx::query_scalar("SELECT count(*) FROM records")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    let raw: i64 = sqlx::query_scalar("SELECT count(*) FROM raw_records WHERE source_id = (SELECT id FROM record_sources WHERE code = 'kzgo')")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    eprintln!("maps={maps} records={records} raw={raw}");
    assert!(maps > 500);
    assert!(records > 500);
    assert!(raw > 500);
}
