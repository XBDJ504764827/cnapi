//! 头像后台同步的集成测试。
//!
//! 用本地 mock 代替中转 Worker 的 `/profiles`，验证：批量拉取、写入
//! `player_profiles.avatar_url`、记录 `avatar_synced_at`、以及「本轮已处理完」后
//! 不再重复处理。
//!
//! 未配置 `DATABASE_URL` 时跳过，与其他集成测试一致。

use std::collections::HashMap;

use axum::{Json, Router, extract::Query, routing::get};
use cngokz_api::{config::AppConfig, services::steam_profiles, state, state::AppState};
use serde_json::json;
use sqlx::{AssertSqlSafe, PgPool, postgres::PgConnectOptions};

async fn create_database() -> Option<(PgPool, String)> {
    let base_url = std::env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.is_empty())?;
    let admin = PgPool::connect(&base_url).await.expect("admin connection");
    let database_name = format!("cngokz_avatar_{}", uuid::Uuid::new_v4().simple());

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

/// mock 中转：只实现 /profiles
async fn spawn_relay() -> String {
    let app = Router::new().route(
        "/profiles",
        get(|Query(query): Query<HashMap<String, String>>| async move {
            let requested: Vec<String> = query
                .get("steamids")
                .cloned()
                .unwrap_or_default()
                .split(',')
                .map(str::to_owned)
                .collect();

            // 只认识 76561198000000001 和 76561198000000002；
            // 000003 故意不返回，模拟 Steam 已不存在的账号。
            let players: Vec<_> = requested
                .iter()
                .filter(|id| id.as_str() != "76561198000000003")
                .map(|id| {
                    json!({
                        "steamid": id,
                        "persona_name": format!("Persona {id}"),
                        "avatar": "https://avatars.steamstatic.com/small.jpg",
                        "avatar_medium": "https://avatars.steamstatic.com/medium.jpg",
                        "avatar_full": "https://avatars.steamstatic.com/full.jpg",
                        "profile_url": format!("https://steamcommunity.com/profiles/{id}"),
                        "country_code": "CN",
                    })
                })
                .collect();

            Json(json!({ "success": true, "players": players }))
        }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock relay");
    let address = listener.local_addr().expect("address");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    format!("http://{address}")
}

fn test_state(pool: PgPool, database_url: &str, relay: &str) -> AppState {
    let config = AppConfig::from_pairs([
        ("APP_ENV", "test".to_owned()),
        ("DATABASE_URL", database_url.to_owned()),
        ("REDIS_URL", "redis://127.0.0.1:1".to_owned()),
        ("PUBLIC_BASE_URL", "https://kz.test".to_owned()),
        ("STEAM_RELAY_URL", relay.to_owned()),
        ("AVATAR_SYNC_BATCHES", "1".to_owned()),
    ])
    .expect("config");

    let redis = state::RedisClient::new(&config.redis_url, "test").expect("redis client");
    AppState::new(config, pool, redis)
}

#[tokio::test]
async fn avatars_are_fetched_once_and_missing_ids_are_marked() {
    let Some((pool, database_url)) = create_database().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };

    // 两个有资料行的玩家 + 一个 Steam 不再认识的账号
    sqlx::raw_sql(AssertSqlSafe(
        r#"
        INSERT INTO accounts (steam_id64) VALUES
          (76561198000000001), (76561198000000002), (76561198000000003);
        INSERT INTO player_profiles (account_id, display_name) VALUES
          (1, 'Alpha'), (2, 'Bravo'), (3, 'Charlie');
        "#
        .to_owned(),
    ))
    .execute(&pool)
    .await
    .expect("fixtures");

    let relay = spawn_relay().await;
    let state = test_state(pool.clone(), &database_url, &relay);

    let handled = steam_profiles::sync_once(&state).await.expect("sync");
    assert_eq!(handled, 3, "本轮应处理全部 3 个账号");

    let rows: Vec<(i64, Option<String>, bool)> = sqlx::query_as(
        r#"
        SELECT account_id,
               avatar_url,
               avatar_synced_at IS NOT NULL
          FROM player_profiles
         ORDER BY account_id
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("read back");

    assert_eq!(
        rows[0].1.as_deref(),
        Some("https://avatars.steamstatic.com/medium.jpg")
    );
    assert_eq!(
        rows[1].1.as_deref(),
        Some("https://avatars.steamstatic.com/medium.jpg")
    );
    // Steam 不认识 000003：头像为空，但同步时间已记录，避免每轮重试
    assert_eq!(rows[2].1, None);
    assert!(rows[2].2, "unknown ids must still be marked as synced");
    assert!(
        rows.iter().all(|row| row.2),
        "every row must carry a sync timestamp"
    );

    // 同一天再跑一次：都还没到刷新窗口，不应该重复处理
    let handled = steam_profiles::sync_once(&state)
        .await
        .expect("second sync");
    assert_eq!(handled, 0, "已同步的账号在刷新周期内不应重复处理");

    // 后台任务不应覆盖插件上报的昵称
    let names: Vec<String> =
        sqlx::query_scalar("SELECT display_name FROM player_profiles ORDER BY account_id")
            .fetch_all(&pool)
            .await
            .expect("names");
    assert_eq!(names, vec!["Alpha", "Bravo", "Charlie"]);
}
