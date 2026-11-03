//! Schema-level integration tests for the phase 2 KZ data model.
//!
//! Every test opens a transaction and rolls it back, so the database is never
//! polluted and the tests can run repeatedly. Set `DATABASE_URL` to run them;
//! without it the tests print a skip message (consistent with the phase 1
//! integration tests).

use chrono::Utc;
use sqlx::types::BigDecimal;
use sqlx::{PgPool, Postgres, Transaction};
use std::str::FromStr as _;

use cngokz_api::models::{
    Account, BanSource, BanStatus, IngestionSource, MapStatus, RecordHistoryAction, RecordStatus,
    ReplayStatus, StorageProvider, TimeType,
};

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL")
        .ok()
        .filter(|value| !value.is_empty())
}

/// Ensures the schema tests never run against an uninitialized database.
async fn pool() -> Option<PgPool> {
    let Some(url) = database_url() else {
        eprintln!("skipped: DATABASE_URL is not set");
        return None;
    };
    let pool = PgPool::connect(&url)
        .await
        .expect("connect to test database");

    let has_migrations: bool = sqlx::query_scalar(
        "SELECT to_regclass('public.records') IS NOT NULL AND to_regclass('public.accounts') IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("schema probe");
    assert!(
        has_migrations,
        "phase 2 migrations are not applied; start the API once or run sqlx migrate"
    );

    Some(pool)
}

#[tokio::test]
async fn all_phase_two_tables_exist() {
    let Some(pool) = pool().await else { return };

    let expected = [
        "accounts",
        "player_profiles",
        "player_mode_stats",
        "player_identities",
        "maps",
        "map_versions",
        "map_courses",
        "map_modes",
        "course_tiers",
        "servers",
        "records",
        "record_history",
        "course_world_records",
        "jumpstats",
        "jump_types",
        "movement_modes",
        "movement_styles",
        "replays",
        "bans",
        "ban_revocations",
    ];

    let tables: Vec<String> =
        sqlx::query_scalar("SELECT tablename::text FROM pg_tables WHERE schemaname = 'public'")
            .fetch_all(&pool)
            .await
            .expect("table list");

    for table in expected {
        assert!(
            tables.iter().any(|name| name == table),
            "missing table: {table}"
        );
    }
}

#[tokio::test]
async fn steam_id64_column_has_no_overflow_risk() {
    let Some(pool) = pool().await else { return };

    let (data_type, precision, scale): (String, Option<i32>, Option<i32>) = sqlx::query_as(
        "SELECT data_type::text, numeric_precision, numeric_scale
         FROM information_schema.columns
         WHERE table_schema = 'public' AND table_name = 'accounts' AND column_name = 'steam_id64'",
    )
    .fetch_one(&pool)
    .await
    .expect("column metadata");

    assert_eq!(data_type, "numeric");
    assert_eq!(precision, Some(20));
    assert_eq!(scale, Some(0));

    // The CHECK constraint must reject values above uint64::MAX.
    let mut tx = pool.begin().await.expect("tx");
    sqlx::query("SAVEPOINT s_overflow")
        .execute(&mut *tx)
        .await
        .expect("savepoint");
    let error = sqlx::query("INSERT INTO accounts (steam_id64) VALUES (18446744073709551616)")
        .execute(&mut *tx)
        .await
        .expect_err("uint64 overflow must be rejected");
    assert!(
        error.to_string().contains("ck_accounts_steam_id64_range"),
        "unexpected error: {error}"
    );
    sqlx::query("ROLLBACK TO SAVEPOINT s_overflow")
        .execute(&mut *tx)
        .await
        .expect("rollback to savepoint");
    tx.rollback().await.expect("rollback");
}

#[tokio::test]
async fn reference_data_is_seeded_from_upstream_sources() {
    let Some(pool) = pool().await else { return };

    let modes: Vec<String> = sqlx::query_scalar("SELECT code FROM movement_modes ORDER BY code")
        .fetch_all(&pool)
        .await
        .expect("modes");
    assert_eq!(modes, vec!["classic", "kztimer", "simple", "vanilla"]);

    let styles: Vec<String> = sqlx::query_scalar("SELECT code FROM movement_styles ORDER BY code")
        .fetch_all(&pool)
        .await
        .expect("styles");
    assert_eq!(styles, vec!["normal"]);

    let jump_types: Vec<(String, Option<i16>, Option<i16>)> =
        sqlx::query_as("SELECT code, gokz_code, cs2kz_code FROM jump_types ORDER BY code")
            .fetch_all(&pool)
            .await
            .expect("jump types");
    assert_eq!(jump_types.len(), 12);
    assert!(
        jump_types
            .iter()
            .any(|(code, gokz, cs2kz)| code == "long-jump"
                && *gokz == Some(0)
                && *cs2kz == Some(0))
    );
    assert!(
        jump_types
            .iter()
            .any(|(code, gokz, cs2kz)| code == "fall" && *gokz == Some(9) && *cs2kz == Some(7))
    );
    assert!(
        jump_types
            .iter()
            .any(|(code, gokz, cs2kz)| code == "lowpre-bhop"
                && *gokz == Some(7)
                && cs2kz.is_none())
    );
}

#[tokio::test]
async fn account_roundtrip_preserves_uint64_steam_id() {
    let Some(pool) = pool().await else { return };

    let mut tx = pool.begin().await.expect("tx");
    let steam_id = BigDecimal::from_str("18446744073709551615").expect("decimal");
    let account: Account = sqlx::query_as(
        "INSERT INTO accounts (steam_id64, steam_id2)
         VALUES ($1, 'STEAM_1:1:12345678')
         RETURNING *",
    )
    .bind(&steam_id)
    .fetch_one(&mut *tx)
    .await
    .expect("insert account");

    assert_eq!(account.steam_id64_string(), "18446744073709551615");

    // Numeric comparison must still work, not only text comparison.
    let found: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM accounts WHERE steam_id64 = $1)")
            .bind(&steam_id)
            .fetch_one(&mut *tx)
            .await
            .expect("lookup");
    assert!(found, "account must be found by numeric steam_id64");

    tx.rollback().await.expect("rollback");
}

struct Fixtures {
    account_id: i64,
    map_id: i64,
    course_id: i64,
    mode_id: i16,
    style_id: i16,
    server_id: i64,
}

/// Creates a minimal account/map/course/server graph inside the transaction.
async fn fixtures(tx: &mut Transaction<'_, Postgres>) -> Fixtures {
    let account_id: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (steam_id64) VALUES (76561198000000001) RETURNING id",
    )
    .fetch_one(&mut **tx)
    .await
    .expect("account");

    let map_id: i64 = sqlx::query_scalar(
        "INSERT INTO maps (name, status, approved_at) VALUES ('kz_schema_test', 'approved', now()) RETURNING id",
    )
    .fetch_one(&mut **tx)
    .await
    .expect("map");

    let course_id: i64 = sqlx::query_scalar(
        "INSERT INTO map_courses (map_id, stage, name) VALUES ($1, 0, 'main') RETURNING id",
    )
    .bind(map_id)
    .fetch_one(&mut **tx)
    .await
    .expect("course");

    let mode_id: i16 = sqlx::query_scalar("SELECT id FROM movement_modes WHERE code = 'vanilla'")
        .fetch_one(&mut **tx)
        .await
        .expect("mode");
    let style_id: i16 = sqlx::query_scalar("SELECT id FROM movement_styles WHERE code = 'normal'")
        .fetch_one(&mut **tx)
        .await
        .expect("style");

    let server_id: i64 = sqlx::query_scalar(
        "INSERT INTO servers (name, ip, port, owner_account_id, api_key_hash, api_key_prefix)
         VALUES ('schema-test-server', '127.0.0.1', 27015, $1, sha256('key'), 'abcd1234')
         RETURNING id",
    )
    .bind(account_id)
    .fetch_one(&mut **tx)
    .await
    .expect("server");

    Fixtures {
        account_id,
        map_id,
        course_id,
        mode_id,
        style_id,
        server_id,
    }
}

async fn insert_record(
    tx: &mut Transaction<'_, Postgres>,
    fixtures: &Fixtures,
    teleports: i32,
    time_type: &str,
    time_ns: i64,
    submission_id: Option<uuid::Uuid>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO records
           (account_id, map_id, course_id, mode_id, style_id, server_id, tickrate,
            teleports, time_type, time_ns, status, source, submission_id)
         VALUES ($1, $2, $3, $4, $5, $6, 128, $7, $8, $9, 'approved', 'plugin', $10)
         RETURNING id",
    )
    .bind(fixtures.account_id)
    .bind(fixtures.map_id)
    .bind(fixtures.course_id)
    .bind(fixtures.mode_id)
    .bind(fixtures.style_id)
    .bind(fixtures.server_id)
    .bind(teleports)
    .bind(time_type)
    .bind(time_ns)
    .bind(submission_id)
    .fetch_one(&mut **tx)
    .await
}

#[tokio::test]
async fn record_invariants_and_enum_mapping_are_enforced() {
    let Some(pool) = pool().await else { return };

    let mut tx = pool.begin().await.expect("tx");
    let fixtures = fixtures(&mut tx).await;

    let record_id = insert_record(&mut tx, &fixtures, 0, "pro", 12_345_678_901, None)
        .await
        .expect("valid pro record");

    // Enum columns decode into Rust enums directly from TEXT.
    let (time_type, status, source): (TimeType, RecordStatus, IngestionSource) =
        sqlx::query_as("SELECT time_type, status, source FROM records WHERE id = $1")
            .bind(record_id)
            .fetch_one(&mut *tx)
            .await
            .expect("decode enums");
    assert_eq!(time_type, TimeType::Pro);
    assert_eq!(status, RecordStatus::Approved);
    assert_eq!(source, IngestionSource::Plugin);

    // Trailing zeros of the nanosecond value must be preserved.
    let time_ns: i64 = sqlx::query_scalar("SELECT time_ns FROM records WHERE id = $1")
        .bind(record_id)
        .fetch_one(&mut *tx)
        .await
        .expect("time");
    assert_eq!(time_ns, 12_345_678_901);

    // teleports > 0 must be NUB; a PRO run with teleports is rejected.
    sqlx::query("SAVEPOINT s_invariant")
        .execute(&mut *tx)
        .await
        .expect("savepoint");
    let error = insert_record(&mut tx, &fixtures, 3, "pro", 1_000, None)
        .await
        .expect_err("teleports > 0 with time_type=pro must fail");
    assert!(
        error
            .to_string()
            .contains("ck_records_time_type_matches_teleports"),
        "unexpected error: {error}"
    );
    sqlx::query("ROLLBACK TO SAVEPOINT s_invariant")
        .execute(&mut *tx)
        .await
        .expect("rollback to savepoint");

    // Duplicate submission ids for the same server are rejected (idempotency).
    let submission_id = uuid::Uuid::new_v4();
    insert_record(&mut tx, &fixtures, 0, "pro", 5_000, Some(submission_id))
        .await
        .expect("first submission");
    sqlx::query("SAVEPOINT s_duplicate")
        .execute(&mut *tx)
        .await
        .expect("savepoint");
    let error = insert_record(&mut tx, &fixtures, 0, "pro", 6_000, Some(submission_id))
        .await
        .expect_err("duplicate submission must fail");
    assert!(
        error.to_string().contains("uq_records_submission"),
        "unexpected error: {error}"
    );
    sqlx::query("ROLLBACK TO SAVEPOINT s_duplicate")
        .execute(&mut *tx)
        .await
        .expect("rollback to savepoint");

    tx.rollback().await.expect("rollback");
}

#[tokio::test]
async fn world_record_upsert_cannot_be_overwritten_by_slower_runs() {
    let Some(pool) = pool().await else { return };

    let mut tx = pool.begin().await.expect("tx");
    let fixtures = fixtures(&mut tx).await;

    let slow = insert_record(&mut tx, &fixtures, 0, "pro", 9_000_000_000, None)
        .await
        .expect("slow record");
    let fast = insert_record(&mut tx, &fixtures, 0, "pro", 7_000_000_000, None)
        .await
        .expect("fast record");

    let upsert = "INSERT INTO course_world_records
                    (course_id, mode_id, style_id, time_type, record_id, time_ns, achieved_at)
                  VALUES ($1, $2, $3, 'pro', $4, $5, now())
                  ON CONFLICT (course_id, mode_id, style_id, time_type) DO UPDATE
                  SET record_id = EXCLUDED.record_id,
                      time_ns = EXCLUDED.time_ns,
                      achieved_at = EXCLUDED.achieved_at
                  WHERE EXCLUDED.time_ns < course_world_records.time_ns";

    sqlx::query(upsert)
        .bind(fixtures.course_id)
        .bind(fixtures.mode_id)
        .bind(fixtures.style_id)
        .bind(slow)
        .bind(9_000_000_000_i64)
        .execute(&mut *tx)
        .await
        .expect("seed world record");

    // A slower run must not replace the current world record.
    sqlx::query(upsert)
        .bind(fixtures.course_id)
        .bind(fixtures.mode_id)
        .bind(fixtures.style_id)
        .bind(slow)
        .bind(9_500_000_000_i64)
        .execute(&mut *tx)
        .await
        .expect("slower upsert");
    let current: i64 =
        sqlx::query_scalar("SELECT record_id FROM course_world_records WHERE course_id = $1 AND mode_id = $2 AND style_id = $3 AND time_type = 'pro'")
            .bind(fixtures.course_id)
            .bind(fixtures.mode_id)
            .bind(fixtures.style_id)
            .fetch_one(&mut *tx)
            .await
            .expect("current wr");
    assert_eq!(current, slow, "slower run overwrote the world record");

    // A faster run replaces it.
    sqlx::query(upsert)
        .bind(fixtures.course_id)
        .bind(fixtures.mode_id)
        .bind(fixtures.style_id)
        .bind(fast)
        .bind(7_000_000_000_i64)
        .execute(&mut *tx)
        .await
        .expect("faster upsert");
    let (current, time_ns): (i64, i64) = sqlx::query_as(
        "SELECT record_id, time_ns FROM course_world_records WHERE course_id = $1 AND mode_id = $2 AND style_id = $3 AND time_type = 'pro'",
    )
    .bind(fixtures.course_id)
    .bind(fixtures.mode_id)
    .bind(fixtures.style_id)
    .fetch_one(&mut *tx)
    .await
    .expect("updated wr");
    assert_eq!((current, time_ns), (fast, 7_000_000_000));

    tx.rollback().await.expect("rollback");
}

#[tokio::test]
async fn soft_delete_releases_unique_steam_id() {
    let Some(pool) = pool().await else { return };

    let mut tx = pool.begin().await.expect("tx");
    let steam_id = BigDecimal::from_str("76561198000000002").expect("decimal");

    sqlx::query("INSERT INTO accounts (steam_id64) VALUES ($1)")
        .bind(&steam_id)
        .execute(&mut *tx)
        .await
        .expect("first account");

    // While active, the same SteamID64 is rejected.
    sqlx::query("SAVEPOINT s_dup_account")
        .execute(&mut *tx)
        .await
        .expect("savepoint");
    let error = sqlx::query("INSERT INTO accounts (steam_id64) VALUES ($1)")
        .bind(&steam_id)
        .execute(&mut *tx)
        .await
        .expect_err("duplicate active account must fail");
    assert!(
        error.to_string().contains("uq_accounts_steam_id64_active"),
        "unexpected error: {error}"
    );
    sqlx::query("ROLLBACK TO SAVEPOINT s_dup_account")
        .execute(&mut *tx)
        .await
        .expect("rollback to savepoint");

    // After a soft delete a new row is allowed (partial unique index).
    sqlx::query("UPDATE accounts SET deleted_at = now() WHERE steam_id64 = $1")
        .bind(&steam_id)
        .execute(&mut *tx)
        .await
        .expect("soft delete");
    sqlx::query("INSERT INTO accounts (steam_id64) VALUES ($1)")
        .bind(&steam_id)
        .execute(&mut *tx)
        .await
        .expect("re-register after soft delete");

    tx.rollback().await.expect("rollback");
}

#[tokio::test]
async fn enum_values_match_sql_constraints() {
    let Some(pool) = pool().await else { return };

    // Unknown status values are rejected by CHECK constraints.
    let mut tx = pool.begin().await.expect("tx");

    let checks: Vec<(String, String)> = sqlx::query_as(
        "SELECT conrelid::regclass::text, pg_get_constraintdef(oid)
         FROM pg_constraint
         WHERE contype = 'c' AND conrelid::regclass::text IN ('records', 'maps', 'replays', 'bans')
         ORDER BY 1, 2",
    )
    .fetch_all(&mut *tx)
    .await
    .expect("constraints");

    assert!(
        checks
            .iter()
            .any(|(table, def)| table == "records" && def.contains("approved"))
    );
    assert!(
        checks
            .iter()
            .any(|(table, def)| table == "maps" && def.contains("in_testing"))
    );
    assert!(
        checks
            .iter()
            .any(|(table, def)| table == "replays" && def.contains("available"))
    );
    assert!(
        checks
            .iter()
            .any(|(table, def)| table == "bans" && def.contains("revoked"))
    );

    tx.rollback().await.expect("rollback");
}

#[tokio::test]
async fn enum_rust_values_round_trip_through_text_columns() {
    let Some(pool) = pool().await else { return };

    let mut tx = pool.begin().await.expect("tx");
    let fixtures = fixtures(&mut tx).await;

    // Insert a rejected record plus history and read the enums back.
    let record_id: i64 = sqlx::query_scalar(
        "INSERT INTO records
           (account_id, map_id, course_id, mode_id, style_id, tickrate, teleports, time_type, time_ns, status)
         VALUES ($1, $2, $3, $4, $5, 128, 2, 'nub', 1500000000, 'rejected')
         RETURNING id",
    )
    .bind(fixtures.account_id)
    .bind(fixtures.map_id)
    .bind(fixtures.course_id)
    .bind(fixtures.mode_id)
    .bind(fixtures.style_id)
    .fetch_one(&mut *tx)
    .await
    .expect("rejected record");

    let action: RecordHistoryAction = sqlx::query_scalar(
        "INSERT INTO record_history (record_id, action, old_status, new_status, reason)
         VALUES ($1, 'status_changed', 'pending', 'rejected', 'schema test')
         RETURNING action",
    )
    .bind(record_id)
    .fetch_one(&mut *tx)
    .await
    .expect("history action");
    assert_eq!(action, RecordHistoryAction::StatusChanged);

    let status: RecordStatus = sqlx::query_scalar("SELECT status FROM records WHERE id = $1")
        .bind(record_id)
        .fetch_one(&mut *tx)
        .await
        .expect("status");
    assert_eq!(status, RecordStatus::Rejected);

    // Reference enum values decode too.
    let map_status: MapStatus = sqlx::query_scalar("SELECT status FROM maps WHERE id = $1")
        .bind(fixtures.map_id)
        .fetch_one(&mut *tx)
        .await
        .expect("map status");
    assert_eq!(map_status, MapStatus::Approved);

    let ban_source: BanSource = sqlx::query_scalar(
        "INSERT INTO bans (account_id, reason, issued_by_account_id, source)
         VALUES ($1, 'test', $1, 'admin') RETURNING source",
    )
    .bind(fixtures.account_id)
    .fetch_one(&mut *tx)
    .await
    .expect("ban source");
    assert_eq!(ban_source, BanSource::Admin);

    let ban_status: BanStatus = sqlx::query_scalar("SELECT status FROM bans WHERE account_id = $1")
        .bind(fixtures.account_id)
        .fetch_one(&mut *tx)
        .await
        .expect("ban status");
    assert_eq!(ban_status, BanStatus::Active);

    // Replay metadata with checksum + storage provider.
    let replay_status: ReplayStatus = sqlx::query_scalar(
        "INSERT INTO replays
           (record_id, account_id, map_id, storage_provider, bucket, object_key, size_bytes, checksum_sha256, status, uploaded_at)
         VALUES ($1, $2, $3, 'r2', 'replays', 'test/object.replay', 1024, sha256('payload'), 'available', now())
         RETURNING status",
    )
    .bind(record_id)
    .bind(fixtures.account_id)
    .bind(fixtures.map_id)
    .fetch_one(&mut *tx)
    .await
    .expect("replay status");
    assert_eq!(replay_status, ReplayStatus::Available);

    let provider: StorageProvider =
        sqlx::query_scalar("SELECT storage_provider FROM replays WHERE record_id = $1")
            .bind(record_id)
            .fetch_one(&mut *tx)
            .await
            .expect("storage provider");
    assert_eq!(provider, StorageProvider::R2);

    tx.rollback().await.expect("rollback");
}

#[tokio::test]
async fn indexes_for_leaderboards_and_retention_exist() {
    let Some(pool) = pool().await else { return };

    let indexes: Vec<String> = sqlx::query_scalar(
        "SELECT indexname::text FROM pg_indexes WHERE schemaname = 'public' ORDER BY indexname",
    )
    .fetch_all(&pool)
    .await
    .expect("indexes");

    for expected in [
        "idx_records_leaderboard",
        "uq_records_submission",
        "uq_records_external_ref",
        "idx_jumpstats_top",
        "idx_replays_retention",
        "idx_servers_online",
        "idx_bans_account_active",
        "uq_accounts_steam_id64_active",
    ] {
        assert!(
            indexes.iter().any(|name| name == expected),
            "missing index: {expected}"
        );
    }
}

#[tokio::test]
async fn updated_at_trigger_is_installed() {
    let Some(pool) = pool().await else { return };

    let triggers: Vec<String> = sqlx::query_scalar(
        "SELECT tgname::text FROM pg_trigger WHERE NOT tgisinternal ORDER BY tgname",
    )
    .fetch_all(&pool)
    .await
    .expect("triggers");

    for expected in [
        "trg_accounts_updated_at",
        "trg_records_updated_at",
        "trg_replays_updated_at",
    ] {
        assert!(
            triggers.iter().any(|name| name == expected),
            "missing trigger: {expected}"
        );
    }

    // Trigger smoke test. `now()` is transaction time, so the update must run in
    // its own committed transaction; a dedicated row keeps this repeatable.
    let name = format!("kz_trigger_test_{}", uuid::Uuid::new_v4().simple());
    let map_id: i64 = sqlx::query_scalar("INSERT INTO maps (name) VALUES ($1) RETURNING id")
        .bind(&name)
        .fetch_one(&pool)
        .await
        .expect("insert map");

    let before: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT updated_at FROM maps WHERE id = $1")
            .bind(map_id)
            .fetch_one(&pool)
            .await
            .expect("before");

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    sqlx::query("UPDATE maps SET description = 'touched' WHERE id = $1")
        .bind(map_id)
        .execute(&pool)
        .await
        .expect("update");

    let after: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT updated_at FROM maps WHERE id = $1")
            .bind(map_id)
            .fetch_one(&pool)
            .await
            .expect("after");

    sqlx::query("DELETE FROM maps WHERE id = $1")
        .bind(map_id)
        .execute(&pool)
        .await
        .expect("cleanup");

    assert!(
        after > before,
        "updated_at trigger did not fire: {before} -> {after}"
    );
}
