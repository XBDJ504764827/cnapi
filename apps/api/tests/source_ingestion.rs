//! Phase-8 raw ingestion, cross-source dedupe and source health tests.
//!
//! The KZGO fixture below is a real sample returned by
//! `https://kzgo.eu/api/wrs/kz_timer` during the interface investigation.

use chrono::{TimeZone, Utc};
use sqlx::{AssertSqlSafe, PgPool, postgres::PgConnectOptions};

use cngokz_api::dto::source::ExternalServerSnapshot;
use cngokz_api::repositories::sources::{self as repo, ExternalRecordInput};

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

async fn fixture_db() -> Option<TestDatabase> {
    let base_url = std::env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.is_empty())?;
    let admin = PgPool::connect(&base_url).await.expect("admin");
    let name = format!("cngokz_sources_{}", uuid::Uuid::new_v4().simple());
    sqlx::raw_sql(AssertSqlSafe(format!("CREATE DATABASE \"{name}\"")))
        .execute(&admin)
        .await
        .expect("create db");
    let options: PgConnectOptions = base_url.parse().expect("url");
    let pool = PgPool::connect_with(options.database(&name))
        .await
        .expect("fixture db");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations");
    sqlx::raw_sql(
        r#"
        INSERT INTO maps (name, status, approved_at)
        VALUES ('bkz_apricity_v3', 'approved', '2018-01-09T00:00:00Z');
        INSERT INTO map_courses (map_id, stage, name)
        SELECT id, 0, 'Main' FROM maps WHERE name = 'bkz_apricity_v3';
        INSERT INTO map_modes (map_id, mode_id)
        SELECT m.id, mm.id FROM maps m CROSS JOIN movement_modes mm
         WHERE m.name = 'bkz_apricity_v3' AND mm.code IN ('vanilla', 'simple', 'kztimer');
        INSERT INTO course_tiers (course_id, mode_id, nub_tier, pro_tier, status)
        SELECT c.id, mm.id, 5, 5, 'ranked' FROM map_courses c
          JOIN maps m ON m.id = c.map_id CROSS JOIN movement_modes mm
         WHERE m.name = 'bkz_apricity_v3' AND mm.code = 'kztimer';
        INSERT INTO course_mode_tiers (map_id, course_id, mode_id, tier)
        SELECT c.map_id, c.id, mm.id, 5 FROM map_courses c
          JOIN maps m ON m.id = c.map_id CROSS JOIN movement_modes mm
         WHERE m.name = 'bkz_apricity_v3' AND mm.code = 'kztimer';
        "#,
    )
    .execute(&pool)
    .await
    .expect("fixtures");
    Some(TestDatabase {
        base_url,
        pool,
        name,
    })
}

fn sample(
    source_code: &str,
    external_record_id: &str,
    recorded_at: chrono::DateTime<Utc>,
) -> ExternalRecordInput {
    ExternalRecordInput {
        source_code: source_code.to_owned(),
        external_record_id: external_record_id.to_owned(),
        raw_payload: serde_json::json!({
            "source": source_code,
            "map_id": 200,
            "map_name": "bkz_apricity_v3",
            "player_name": "smieszneznaczki",
            "steam_id64": "76561198325578948",
            "server_name": "catgirl galaxy",
            "created_on": recorded_at.to_rfc3339(),
            "time": 169.422,
            "teleports": 0
        }),
        steam_id64: "76561198325578948".to_owned(),
        display_name: "smieszneznaczki".to_owned(),
        map_name: "bkz_apricity_v3".to_owned(),
        course_stage: 0,
        mode_code: "kztimer".to_owned(),
        style_code: "normal".to_owned(),
        time_ns: 169_422_000_000,
        teleports: 0,
        recorded_at: Some(recorded_at),
        server_name: Some("catgirl galaxy".to_owned()),
    }
}

#[tokio::test]
async fn same_record_from_kzgo_and_global_is_deduplicated_with_provenance() {
    let Some(db) = fixture_db().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };
    let recorded_at = Utc.with_ymd_and_hms(2021, 11, 18, 21, 11, 34).unwrap();

    let first = repo::ingest_external(
        &db.pool,
        &sample(
            "kzgo",
            "wr:kztimer:200:pro:76561198325578948:1637270000:169422000000",
            recorded_at,
        ),
    )
    .await
    .expect("kzgo sample");
    assert!(!first.duplicate);
    assert!(first.is_world_record);

    let second = repo::ingest_external(
        &db.pool,
        &sample("gokz_global", "record:123456", recorded_at),
    )
    .await
    .expect("global sample");
    assert!(second.duplicate);
    assert_eq!(second.canonical_record_id, first.canonical_record_id);
    assert_eq!(second.match_method, "high_confidence_fields_time");

    let records: i64 = sqlx::query_scalar("SELECT count(*) FROM records")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    let raw: i64 = sqlx::query_scalar("SELECT count(*) FROM raw_records")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    let links: i64 = sqlx::query_scalar("SELECT count(*) FROM record_source_links")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!((records, raw, links), (1, 2, 2));

    let provenance = repo::record_provenance(&db.pool, first.canonical_record_id)
        .await
        .unwrap();
    assert_eq!(provenance.len(), 2);
    assert_eq!(provenance[0].source_code, "kzgo");
    assert_eq!(provenance[1].source_code, "gokz_global");
    assert_eq!(
        provenance[1].match_method.as_deref(),
        Some("high_confidence_fields_time")
    );

    // A second equal-time run outside the matching window is not merged. This
    // protects repeated identical times from being silently collapsed.
    let later = recorded_at + chrono::Duration::days(10);
    let third = repo::ingest_external(&db.pool, &sample("gokz_global", "record:later", later))
        .await
        .expect("later run");
    assert!(!third.duplicate);
    let records: i64 = sqlx::query_scalar("SELECT count(*) FROM records")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(records, 2);
}

#[tokio::test]
async fn source_health_and_external_server_snapshot_are_persisted() {
    let Some(db) = fixture_db().await else {
        eprintln!("skipped: DATABASE_URL is not set");
        return;
    };
    let source = repo::find_source(&db.pool, "kzgo").await.unwrap().unwrap();
    let run_id = repo::start_run(&db.pool, source.id, "manual")
        .await
        .unwrap();
    repo::finish_run_success(
        &db.pool,
        source.id,
        run_id,
        &serde_json::json!({
            "cursor": {"maps_hash": 1, "maps_count": 990},
            "maps_seen": 990
        }),
    )
    .await
    .unwrap();
    let source = repo::find_source(&db.pool, "kzgo").await.unwrap().unwrap();
    assert_eq!(source.status, "healthy");
    assert_eq!(source.consecutive_failures, 0);
    assert!(source.last_success_at.is_some());

    let servers = vec![ExternalServerSnapshot {
        id: 0,
        source_code: "kzgo".to_owned(),
        external_id: "54.36.30.228:27015:HOC GOKZ".to_owned(),
        name: "House of Climb  EU GOKZ 1".to_owned(),
        label: Some("HOC GOKZ".to_owned()),
        host: "54.36.30.228".to_owned(),
        port: 27015,
        map_name: Some("kz_variety_fix".to_owned()),
        map_tier: Some(4),
        online_players: 2,
        max_players: 20,
        is_online: true,
        last_seen_at: Utc::now(),
    }];
    repo::replace_external_servers(&db.pool, source.id, &servers)
        .await
        .unwrap();
    assert!(repo::external_source_fresh(&db.pool, 900).await.unwrap());
    let listed = repo::list_external_servers(&db.pool, 900).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].online_players, 2);
    assert_eq!(listed[0].map_name.as_deref(), Some("kz_variety_fix"));

    let runs = repo::list_runs(&db.pool, source.id, 20, 0).await.unwrap();
    assert_eq!(runs.1, 1);
    assert_eq!(runs.0[0].status, "success");
}
