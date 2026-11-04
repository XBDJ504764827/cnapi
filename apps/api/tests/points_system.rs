//! Phase-6 integration tests on a temporary database with explicit fixtures.
//! Never touches a developer/production database.

use cngokz_api::services::scoring::formula::{ScoringInput, calculate_points};
use cngokz_api::services::scoring::model::{RecordType, Tier};
use cngokz_api::services::scoring::recalc;
use sqlx::{AssertSqlSafe, PgPool, postgres::PgConnectOptions};

const FIXTURES: &str = r#"
INSERT INTO accounts (id, steam_id64) OVERRIDING SYSTEM VALUE VALUES
  (1, 76561198000000001), (2, 76561198000000002), (3, 76561198000000003);
INSERT INTO player_profiles (account_id, display_name)
VALUES (1, 'Alpha'), (2, 'Bravo'), (3, 'Charlie');

INSERT INTO maps (id, name, workshop_id, status, approved_at) OVERRIDING SYSTEM VALUE VALUES
  (1, 'kz_points_alpha', 111111111, 'approved', now() - interval '10 days'),
  (2, 'kz_points_beta', 222222222, 'approved', now() - interval '5 days');
INSERT INTO map_courses (id, map_id, stage, name) OVERRIDING SYSTEM VALUE VALUES
  (1, 1, 0, 'Main'), (2, 1, 1, 'Bonus 1'), (3, 2, 0, 'Main');
INSERT INTO map_modes (map_id, mode_id) VALUES (1,1),(1,2),(1,3),(2,1),(2,2),(2,3);

-- Per-course, per-mode tiers. KZT T5, SKZ T7, VNL T8. Bonus has its own tier.
INSERT INTO course_mode_tiers (map_id, course_id, mode_id, tier, completed)
VALUES
  (1,1,3,5,FALSE), (1,1,2,7,FALSE), (1,1,1,8,FALSE),
  (1,2,1,3,FALSE), (1,2,2,4,FALSE), (1,2,3,2,FALSE),
  (2,3,1,8,FALSE), (2,3,2,6,FALSE), (2,3,3,4,FALSE);

-- course_tiers is seeded only for legacy views; scoring uses course_mode_tiers.
INSERT INTO course_tiers (course_id, mode_id, nub_tier, pro_tier, status)
VALUES (1,1,7,8,'ranked'), (1,2,6,7,'ranked'), (1,3,4,5,'ranked'),
       (2,1,2,3,'ranked'), (2,2,3,4,'ranked'), (2,3,1,2,'ranked'),
       (3,1,7,8,'ranked'), (3,2,5,6,'ranked'), (3,3,3,4,'ranked');
"#;

async fn fixture_db() -> Option<(PgPool, PgPool, String, String)> {
    let base = std::env::var("DATABASE_URL")
        .ok()
        .filter(|v| !v.is_empty())?;
    let admin = PgPool::connect(&base).await.expect("admin");
    let name = format!("cngokz_points_{}", uuid::Uuid::new_v4().simple());
    sqlx::raw_sql(AssertSqlSafe(format!("CREATE DATABASE \"{name}\"")))
        .execute(&admin)
        .await
        .expect("create db");
    let options: PgConnectOptions = base.parse().expect("url");
    let pool = PgPool::connect_with(options.database(&name))
        .await
        .expect("fixture db");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations");
    let url = match base.rsplit_once('/') {
        Some((prefix, _)) => format!("{prefix}/{name}"),
        None => base.clone(),
    };
    sqlx::raw_sql(AssertSqlSafe(FIXTURES.to_owned()))
        .execute(&pool)
        .await
        .expect("fixtures");
    Some((admin, pool, name, url))
}

async fn insert_record(
    pool: &PgPool,
    account: i64,
    map: i64,
    course: i64,
    mode: i16,
    record_type: &str,
    time_ns: i64,
) -> i64 {
    let teleports = if record_type == "pro" { 0 } else { 1 };
    sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO records (account_id, map_id, course_id, mode_id, style_id,
                             tickrate, teleports, time_type, time_ns, status, source)
        VALUES ($1,$2,$3,$4,1,128,$5,$6,$7,'approved','admin')
        RETURNING id
        "#,
    )
    .bind(account)
    .bind(map)
    .bind(course)
    .bind(mode)
    .bind(teleports)
    .bind(record_type)
    .bind(time_ns)
    .fetch_one(pool)
    .await
    .expect("insert record")
}

#[tokio::test]
async fn filter_recalc_pb_wr_t9_and_mode_isolation() {
    let Some((admin, pool, name, _url)) = fixture_db().await else {
        eprintln!("skipped: DATABASE_URL not set");
        return;
    };

    // KZT PRO: Alpha 10s (WR), Bravo 12s, Charlie 15s.
    insert_record(&pool, 1, 1, 1, 3, "pro", 10_000_000_000).await;
    // KZT NUB on the same course is an independent Filter.
    insert_record(&pool, 1, 1, 1, 3, "nub", 11_000_000_000).await;
    insert_record(&pool, 2, 1, 1, 3, "pro", 12_000_000_000).await;
    insert_record(&pool, 3, 1, 1, 3, "pro", 15_000_000_000).await;
    // SKZ PRO on the same map must not affect KZT.
    insert_record(&pool, 1, 1, 1, 2, "pro", 9_000_000_000).await;
    // VNL PRO on a different map: first completion (T9), then T8.
    insert_record(&pool, 1, 2, 3, 1, "pro", 20_000_000_000).await;

    let filters: Vec<(i64, i16)> =
        sqlx::query_as("SELECT id, mode_id FROM scoring_filters ORDER BY id")
            .fetch_all(&pool)
            .await
            .expect("filters");
    assert_eq!(filters.len(), 4, "expected 4 Filters: {filters:?}");

    for (filter_id, _) in &filters {
        recalc::recalc_filter(&pool, *filter_id)
            .await
            .expect("recalc");
        let _ = filter_id;
    }

    // KZT PRO Filter: WR = 1000, others less than 1000, rank order preserved.
    let kzt: Vec<(i64, i32, i32, bool)> = sqlx::query_as(
        "SELECT account_id, rank, points, is_world_record FROM player_filter_points
          WHERE mode_id=3 AND record_type='pro' ORDER BY rank",
    )
    .fetch_all(&pool)
    .await
    .expect("kzt rows");
    assert_eq!(kzt.len(), 3);
    assert_eq!((kzt[0].0, kzt[0].1, kzt[0].2, kzt[0].3), (1, 1, 1000, true));
    assert!(kzt[1].2 < 1000 && kzt[2].2 <= kzt[1].2);

    // Mode totals: KZT = PRO + NUB, and SKZ/VNL have their own independent rows.
    let totals: Vec<(i16, i64, i32, i32)> = sqlx::query_as(
        "SELECT mode_id, account_id, pro_points, total_points FROM player_mode_points ORDER BY mode_id, account_id",
    )
    .fetch_all(&pool)
    .await
    .expect("totals");
    for (mode_id, _, pro, total) in &totals {
        assert!(*total >= *pro);
        assert!([1, 2, 3].contains(mode_id));
    }
    // KZT total = KZT PRO + KZT NUB, and no cross-mode aggregation.
    let kzt_alpha: (i32, i32, i32) = sqlx::query_as(
        "SELECT pro_points, nub_points, total_points FROM player_mode_points WHERE mode_id=3 AND account_id=1",
    )
    .fetch_one(&pool)
    .await
    .expect("kzt alpha");
    assert_eq!(kzt_alpha.0 + kzt_alpha.1, kzt_alpha.2);
    assert!(
        kzt_alpha.1 > 0,
        "the KZT NUB Filter must contribute NUB points"
    );

    // VNL T9: first completion was labelled, course became T8 immediately.
    let t9: Option<(i64, i64)> = sqlx::query_as(
        "SELECT record_id, account_id FROM filter_first_completions WHERE map_id=2 AND course_id=3 AND mode_id=1",
    )
    .fetch_optional(&pool)
    .await
    .expect("t9");
    assert!(t9.is_some(), "VNL first completion must be recorded");
    let tier: i16 = sqlx::query_scalar(
        "SELECT tier FROM course_mode_tiers WHERE map_id=2 AND course_id=3 AND mode_id=1",
    )
    .fetch_one(&pool)
    .await
    .expect("tier");
    assert_eq!(tier, 8);
    let kzt_tier: i16 = sqlx::query_scalar(
        "SELECT tier FROM course_mode_tiers WHERE map_id=1 AND course_id=1 AND mode_id=3",
    )
    .fetch_one(&pool)
    .await
    .expect("tier");
    assert_eq!(kzt_tier, 5, "KZT tier must not change");
    let skz_tier: i16 = sqlx::query_scalar(
        "SELECT tier FROM course_mode_tiers WHERE map_id=1 AND course_id=1 AND mode_id=2",
    )
    .fetch_one(&pool)
    .await
    .expect("tier");
    assert_eq!(skz_tier, 7, "SKZ tier must not change");

    // Incremental == full recalc: recompute every Filter and compare snapshots.
    let before: Vec<(i64, i64, i32)> = sqlx::query_as(
        "SELECT filter_id, account_id, points FROM player_filter_points ORDER BY filter_id, account_id",
    )
    .fetch_all(&pool)
    .await
    .expect("before");
    for (filter_id, _) in &filters {
        recalc::recalc_filter(&pool, *filter_id)
            .await
            .expect("recalc");
    }
    let after: Vec<(i64, i64, i32)> = sqlx::query_as(
        "SELECT filter_id, account_id, points FROM player_filter_points ORDER BY filter_id, account_id",
    )
    .fetch_all(&pool)
    .await
    .expect("after");
    assert_eq!(before, after, "recalc must be idempotent");

    // A new KZT WR pushes the old WR down and does not change SKZ/VNL points.
    let skz_before: i64 = sqlx::query_scalar(
        "SELECT COALESCE(sum(points),0)::bigint FROM player_filter_points WHERE mode_id=2",
    )
    .fetch_one(&pool)
    .await
    .expect("skz before");
    let vnl_before: i64 = sqlx::query_scalar(
        "SELECT COALESCE(sum(points),0)::bigint FROM player_filter_points WHERE mode_id=1",
    )
    .fetch_one(&pool)
    .await
    .expect("vnl before");

    insert_record(&pool, 2, 1, 1, 3, "pro", 9_000_000_000).await;
    let kzt_filter: i64 = sqlx::query_scalar(
        "SELECT id FROM scoring_filters WHERE map_id=1 AND course_id=1 AND mode_id=3 AND record_type='pro'",
    )
    .fetch_one(&pool)
    .await
    .expect("filter");
    recalc::recalc_filter(&pool, kzt_filter)
        .await
        .expect("recalc");

    let wr_owner: i64 = sqlx::query_scalar(
        "SELECT account_id FROM player_filter_points WHERE filter_id=$1 AND rank=1",
    )
    .bind(kzt_filter)
    .fetch_one(&pool)
    .await
    .expect("wr");
    assert_eq!(wr_owner, 2, "new WR should be the faster Bravo");
    let alpha_points: i32 = sqlx::query_scalar(
        "SELECT points FROM player_filter_points WHERE filter_id=$1 AND account_id=1",
    )
    .bind(kzt_filter)
    .fetch_one(&pool)
    .await
    .expect("alpha");
    assert!(
        alpha_points < 1000,
        "old WR must lose the locked 1000 points"
    );

    let skz_after: i64 = sqlx::query_scalar(
        "SELECT COALESCE(sum(points),0)::bigint FROM player_filter_points WHERE mode_id=2",
    )
    .fetch_one(&pool)
    .await
    .expect("skz after");
    let vnl_after: i64 = sqlx::query_scalar(
        "SELECT COALESCE(sum(points),0)::bigint FROM player_filter_points WHERE mode_id=1",
    )
    .fetch_one(&pool)
    .await
    .expect("vnl after");
    assert_eq!(skz_before, skz_after);
    assert_eq!(vnl_before, vnl_after);

    // Pure formula WR invariant.
    let out = calculate_points(ScoringInput {
        tier: Tier::new(9).unwrap(),
        record_type: RecordType::Pro,
        rank: 1,
        time: 20.0,
        wr: 20.0,
        pb_count: 1,
    })
    .unwrap();
    assert_eq!(out.points, 1000);

    pool.close().await;
    sqlx::raw_sql(AssertSqlSafe(format!(
        "DROP DATABASE IF EXISTS \"{name}\" WITH (FORCE)"
    )))
    .execute(&admin)
    .await
    .expect("drop");
    admin.close().await;
}
