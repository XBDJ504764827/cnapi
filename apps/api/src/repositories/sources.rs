//! PostgreSQL access for multi-source ingestion, provenance and source state.

use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};

use crate::dto::source::{ExternalServerSnapshot, RawRecordAuditRow, SourceRunDto, SourceSummary};

#[derive(Debug, thiserror::Error)]
pub enum SourceIngestError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("{0}")]
    Validation(String),
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SourceRow {
    pub id: i64,
    pub code: String,
    pub source_type: String,
    pub display_name: String,
    pub is_enabled: bool,
    pub priority: i32,
    pub status: String,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub consecutive_failures: i32,
    pub cursor: serde_json::Value,
    pub config: serde_json::Value,
}

pub async fn find_source(pool: &PgPool, code: &str) -> Result<Option<SourceRow>, sqlx::Error> {
    sqlx::query_as::<_, SourceRow>(
        r#"
        SELECT id, code, source_type, display_name, is_enabled, priority, status,
               last_attempt_at, last_success_at, last_failure_at, last_error,
               consecutive_failures, cursor, config
          FROM record_sources WHERE code = $1
        "#,
    )
    .bind(code)
    .fetch_optional(pool)
    .await
}

pub async fn list_sources(pool: &PgPool) -> Result<Vec<SourceSummary>, sqlx::Error> {
    sqlx::query_as::<_, SourceSummary>(
        r#"
        SELECT s.id, s.code, s.source_type, s.display_name, s.is_enabled, s.priority,
               s.status, s.last_attempt_at, s.last_success_at, s.last_failure_at,
               s.last_error, s.consecutive_failures, s.cursor, s.config,
               (SELECT count(*) FROM raw_records r WHERE r.source_id = s.id)::bigint AS raw_records,
               (SELECT count(*) FROM record_source_links l WHERE l.source_id = s.id)::bigint AS canonical_links,
               (SELECT run.status FROM source_sync_runs run
                 WHERE run.source_id = s.id ORDER BY run.started_at DESC LIMIT 1) AS latest_run_status,
               (SELECT run.started_at FROM source_sync_runs run
                 WHERE run.source_id = s.id ORDER BY run.started_at DESC LIMIT 1) AS latest_run_at
          FROM record_sources s
         ORDER BY s.priority, s.id
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn set_source_enabled(
    pool: &PgPool,
    code: &str,
    enabled: bool,
) -> Result<Option<SourceRow>, sqlx::Error> {
    sqlx::query(
        "UPDATE record_sources
            SET is_enabled = $2,
                status = CASE WHEN $2 THEN 'unknown' ELSE 'disabled' END
          WHERE code = $1",
    )
    .bind(code)
    .bind(enabled)
    .execute(pool)
    .await?;
    find_source(pool, code).await
}

pub async fn start_run(pool: &PgPool, source_id: i64, run_type: &str) -> Result<i64, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let run_id: i64 = sqlx::query_scalar(
        "INSERT INTO source_sync_runs (source_id, run_type, status)
         VALUES ($1, $2, 'running') RETURNING id",
    )
    .bind(source_id)
    .bind(run_type)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE record_sources
            SET last_attempt_at = now(), last_error = NULL
          WHERE id = $1",
    )
    .bind(source_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(run_id)
}

pub async fn finish_run_success(
    pool: &PgPool,
    source_id: i64,
    run_id: i64,
    stats: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "UPDATE source_sync_runs
            SET status = 'success', completed_at = now(), stats = $2
          WHERE id = $1",
    )
    .bind(run_id)
    .bind(stats)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE record_sources
            SET status = 'healthy',
                last_success_at = now(),
                last_failure_at = NULL,
                last_error = NULL,
                consecutive_failures = 0,
                cursor = COALESCE($2, cursor)
          WHERE id = $1",
    )
    .bind(source_id)
    .bind(stats.get("cursor"))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn finish_run_failure(
    pool: &PgPool,
    source_id: i64,
    run_id: i64,
    error: &str,
    stats: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        "UPDATE source_sync_runs
            SET status = 'failed', completed_at = now(), stats = $2, error = $3
          WHERE id = $1",
    )
    .bind(run_id)
    .bind(stats)
    .bind(error)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE record_sources
            SET status = CASE WHEN consecutive_failures + 1 >= 3 THEN 'offline' ELSE 'degraded' END,
                last_failure_at = now(),
                last_error = $2,
                consecutive_failures = consecutive_failures + 1
          WHERE id = $1",
    )
    .bind(source_id)
    .bind(error)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn list_runs(
    pool: &PgPool,
    source_id: i64,
    limit: i64,
    offset: i64,
) -> Result<(Vec<SourceRunDto>, i64), sqlx::Error> {
    let items = sqlx::query_as::<_, SourceRunDto>(
        "SELECT id, source_id, run_type, status, started_at, completed_at, stats, error
           FROM source_sync_runs WHERE source_id = $1
          ORDER BY started_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(source_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    let total =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM source_sync_runs WHERE source_id = $1")
            .bind(source_id)
            .fetch_one(pool)
            .await?;
    Ok((items, total))
}

#[derive(Debug, Clone)]
pub struct ExternalRecordInput {
    pub source_code: String,
    pub external_record_id: String,
    pub raw_payload: serde_json::Value,
    pub steam_id64: String,
    pub display_name: String,
    pub map_name: String,
    pub course_stage: i16,
    pub mode_code: String,
    pub style_code: String,
    pub time_ns: i64,
    pub teleports: i32,
    pub recorded_at: Option<DateTime<Utc>>,
    pub server_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExternalIngestOutcome {
    pub raw_record_id: i64,
    pub canonical_record_id: i64,
    pub duplicate: bool,
    pub is_pb: bool,
    pub is_world_record: bool,
    pub match_method: String,
}

async fn ensure_account(
    tx: &mut Transaction<'_, Postgres>,
    steam_id64: &str,
    display_name: &str,
) -> Result<i64, sqlx::Error> {
    let account_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO accounts (steam_id64) VALUES ($1::numeric)
        ON CONFLICT (steam_id64) WHERE deleted_at IS NULL
        DO UPDATE SET updated_at = now()
        RETURNING id
        "#,
    )
    .bind(steam_id64)
    .fetch_one(&mut **tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO player_profiles (account_id, display_name)
        VALUES ($1, $2)
        ON CONFLICT (account_id) DO UPDATE
           SET display_name = EXCLUDED.display_name, last_joined_at = now()
        "#,
    )
    .bind(account_id)
    .bind(display_name)
    .execute(&mut **tx)
    .await?;
    Ok(account_id)
}

pub async fn ingest_external(
    pool: &PgPool,
    input: &ExternalRecordInput,
) -> Result<ExternalIngestOutcome, SourceIngestError> {
    let mut tx = pool.begin().await?;
    let source: (i64, String) =
        sqlx::query_as("SELECT id, source_type FROM record_sources WHERE code = $1")
            .bind(&input.source_code)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| SourceIngestError::Validation("unknown record source".to_owned()))?;

    let raw: (i64, String, Option<i64>) = sqlx::query_as(
        r#"
        INSERT INTO raw_records (source_id, external_record_id, payload_hash, raw_payload)
        VALUES ($1, $2, sha256(convert_to($3, 'UTF8')), $4)
        ON CONFLICT (source_id, external_record_id) DO UPDATE
           SET received_at = raw_records.received_at
        RETURNING id, processing_status, canonical_record_id
        "#,
    )
    .bind(source.0)
    .bind(&input.external_record_id)
    .bind(input.raw_payload.to_string())
    .bind(&input.raw_payload)
    .fetch_one(&mut *tx)
    .await?;

    if let Some(canonical_id) = raw.2 {
        let (is_pb, is_world_record, method) = existing_link_flags(&mut tx, raw.0).await?;
        tx.commit().await?;
        return Ok(ExternalIngestOutcome {
            raw_record_id: raw.0,
            canonical_record_id: canonical_id,
            duplicate: true,
            is_pb,
            is_world_record,
            match_method: method,
        });
    }

    let context: Option<(i64, i64, i16, i16)> = sqlx::query_as(
        r#"
        SELECT m.id, c.id, mm.id, ms.id
          FROM maps m
          JOIN map_courses c ON c.map_id = m.id AND c.stage = $2
          JOIN movement_modes mm ON lower(mm.code) = lower($3)
          JOIN movement_styles ms ON lower(ms.code) = lower($4)
         WHERE lower(m.name) = lower($1) AND m.deleted_at IS NULL
         LIMIT 1
        "#,
    )
    .bind(&input.map_name)
    .bind(input.course_stage)
    .bind(&input.mode_code)
    .bind(&input.style_code)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((map_id, course_id, mode_id, style_id)) = context else {
        sqlx::query(
            "UPDATE raw_records SET processing_status = 'rejected', processing_error = $2 WHERE id = $1",
        )
        .bind(raw.0)
        .bind("map/course/mode/style not found")
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Err(SourceIngestError::Validation(
            "map/course/mode/style not found".to_owned(),
        ));
    };

    let display_name = truncate_name(&input.display_name);
    let account_id = ensure_account(&mut tx, &input.steam_id64, &display_name).await?;
    let time_type = if input.teleports == 0 { "pro" } else { "nub" };

    let matched: Option<i64> = if let Some(recorded_at) = input.recorded_at {
        sqlx::query_scalar(
            r#"
            SELECT r.id FROM records r
             WHERE r.account_id = $1 AND r.map_id = $2 AND r.course_id = $3
               AND r.mode_id = $4 AND r.style_id = $5 AND r.time_type = $6
               AND r.teleports = $7 AND r.time_ns = $8
               AND r.status = 'approved' AND r.deleted_at IS NULL
               AND abs(extract(epoch FROM (r.created_at - $9))) <= 600
             ORDER BY abs(extract(epoch FROM (r.created_at - $9))), r.id
             LIMIT 1
            "#,
        )
        .bind(account_id)
        .bind(map_id)
        .bind(course_id)
        .bind(mode_id)
        .bind(style_id)
        .bind(time_type)
        .bind(input.teleports)
        .bind(input.time_ns)
        .bind(recorded_at)
        .fetch_optional(&mut *tx)
        .await?
    } else {
        None
    };

    if let Some(canonical_id) = matched {
        let (is_pb, is_world_record) = record_flags(
            &mut tx,
            canonical_id,
            account_id,
            map_id,
            course_id,
            mode_id,
            style_id,
            time_type,
            input.time_ns,
        )
        .await?;
        sqlx::query(
            "INSERT INTO record_source_links
                (canonical_record_id, raw_record_id, source_id, external_record_id,
                 source_recorded_at, match_method, match_confidence, field_conflicts)
             VALUES ($1, $2, $3, $4, $5, 'high_confidence_fields_time', 0.95, '{}'::jsonb)
             ON CONFLICT (raw_record_id) DO NOTHING",
        )
        .bind(canonical_id)
        .bind(raw.0)
        .bind(source.0)
        .bind(&input.external_record_id)
        .bind(input.recorded_at)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE raw_records SET processing_status = 'duplicate', canonical_record_id = $2 WHERE id = $1",
        )
        .bind(raw.0)
        .bind(canonical_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Ok(ExternalIngestOutcome {
            raw_record_id: raw.0,
            canonical_record_id: canonical_id,
            duplicate: true,
            is_pb,
            is_world_record,
            match_method: "high_confidence_fields_time".to_owned(),
        });
    }

    let previous_pb: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT min(time_ns) FROM records
         WHERE account_id = $1 AND map_id = $2 AND course_id = $3
           AND mode_id = $4 AND style_id = $5 AND time_type = $6
           AND status = 'approved' AND deleted_at IS NULL
        "#,
    )
    .bind(account_id)
    .bind(map_id)
    .bind(course_id)
    .bind(mode_id)
    .bind(style_id)
    .bind(time_type)
    .fetch_one(&mut *tx)
    .await?;
    let is_pb = previous_pb.map(|time| input.time_ns < time).unwrap_or(true);

    let record_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO records
            (account_id, map_id, course_id, mode_id, style_id, tickrate, teleports,
             time_type, time_ns, status, source, external_ref, source_server_name, created_at)
        VALUES ($1, $2, $3, $4, $5, 128, $6, $7, $8, 'approved',
                'globalapi_import', $9, $10, COALESCE($11, now()))
        RETURNING id
        "#,
    )
    .bind(account_id)
    .bind(map_id)
    .bind(course_id)
    .bind(mode_id)
    .bind(style_id)
    .bind(input.teleports)
    .bind(time_type)
    .bind(input.time_ns)
    .bind(format!("raw:{}", raw.0))
    .bind(&input.server_name)
    .bind(input.recorded_at)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO record_history (record_id, action, new_status, metadata)
         VALUES ($1, 'created', 'approved', $2)",
    )
    .bind(record_id)
    .bind(json!({
        "source": input.source_code,
        "external_record_id": input.external_record_id,
        "raw_record_id": raw.0
    }))
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO course_world_records
            (course_id, mode_id, style_id, time_type, record_id, time_ns, achieved_at)
        VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, now()))
        ON CONFLICT (course_id, mode_id, style_id, time_type) DO UPDATE
           SET record_id = EXCLUDED.record_id,
               time_ns = EXCLUDED.time_ns,
               achieved_at = EXCLUDED.achieved_at
         WHERE EXCLUDED.time_ns < course_world_records.time_ns
        "#,
    )
    .bind(course_id)
    .bind(mode_id)
    .bind(style_id)
    .bind(time_type)
    .bind(record_id)
    .bind(input.time_ns)
    .bind(input.recorded_at)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO record_source_links
            (canonical_record_id, raw_record_id, source_id, external_record_id,
             source_recorded_at, match_method, match_confidence, field_conflicts)
         VALUES ($1, $2, $3, $4, $5, 'source_id', 1.0, '{}'::jsonb)",
    )
    .bind(record_id)
    .bind(raw.0)
    .bind(source.0)
    .bind(&input.external_record_id)
    .bind(input.recorded_at)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE raw_records SET processing_status = 'processed', canonical_record_id = $2 WHERE id = $1",
    )
    .bind(raw.0)
    .bind(record_id)
    .execute(&mut *tx)
    .await?;

    let is_world_record: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM course_world_records
          WHERE course_id = $1 AND mode_id = $2 AND style_id = $3
            AND time_type = $4 AND record_id = $5)",
    )
    .bind(course_id)
    .bind(mode_id)
    .bind(style_id)
    .bind(time_type)
    .bind(record_id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(ExternalIngestOutcome {
        raw_record_id: raw.0,
        canonical_record_id: record_id,
        duplicate: false,
        is_pb,
        is_world_record,
        match_method: "source_id".to_owned(),
    })
}

async fn existing_link_flags(
    tx: &mut Transaction<'_, Postgres>,
    raw_record_id: i64,
) -> Result<(bool, bool, String), sqlx::Error> {
    let row: (i64, i64, i64, i64, i16, i16, String, i64, String) = sqlx::query_as(
        r#"
        SELECT r.id, r.account_id, r.map_id, r.course_id, r.mode_id, r.style_id,
               r.time_type, r.time_ns, l.match_method
          FROM raw_records raw
          JOIN record_source_links l ON l.raw_record_id = raw.id
          JOIN records r ON r.id = l.canonical_record_id
         WHERE raw.id = $1
        "#,
    )
    .bind(raw_record_id)
    .fetch_one(&mut **tx)
    .await?;
    let (is_pb, is_world_record) =
        record_flags(tx, row.0, row.1, row.2, row.3, row.4, row.5, &row.6, row.7).await?;
    Ok((is_pb, is_world_record, row.8))
}

#[allow(clippy::too_many_arguments)]
async fn record_flags(
    tx: &mut Transaction<'_, Postgres>,
    record_id: i64,
    account_id: i64,
    map_id: i64,
    course_id: i64,
    mode_id: i16,
    style_id: i16,
    time_type: &str,
    time_ns: i64,
) -> Result<(bool, bool), sqlx::Error> {
    let is_pb: bool = sqlx::query_scalar(
        r#"
        SELECT NOT EXISTS (
            SELECT 1 FROM records r
             WHERE r.account_id = $1 AND r.map_id = $2 AND r.course_id = $3
               AND r.mode_id = $4 AND r.style_id = $5 AND r.time_type = $6
               AND r.status = 'approved' AND r.deleted_at IS NULL
               AND (r.time_ns < $7 OR (r.time_ns = $7 AND r.id < $8))
        )
        "#,
    )
    .bind(account_id)
    .bind(map_id)
    .bind(course_id)
    .bind(mode_id)
    .bind(style_id)
    .bind(time_type)
    .bind(time_ns)
    .bind(record_id)
    .fetch_one(&mut **tx)
    .await?;
    let is_world_record: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM course_world_records
          WHERE course_id = $1 AND mode_id = $2 AND style_id = $3
            AND time_type = $4 AND record_id = $5)",
    )
    .bind(course_id)
    .bind(mode_id)
    .bind(style_id)
    .bind(time_type)
    .bind(record_id)
    .fetch_one(&mut **tx)
    .await?;
    Ok((is_pb, is_world_record))
}

fn truncate_name(value: &str) -> String {
    let value: String = value.trim().chars().take(64).collect();
    if value.is_empty() {
        "Unknown player".to_owned()
    } else {
        value
    }
}

pub async fn record_provenance(
    pool: &PgPool,
    record_id: i64,
) -> Result<Vec<RawRecordAuditRow>, sqlx::Error> {
    sqlx::query_as::<_, RawRecordAuditRow>(
        r#"
        SELECT raw.id AS raw_record_id, raw.canonical_record_id,
               s.code AS source_code, s.source_type,
               raw.external_record_id, raw.processing_status,
               raw.processing_error,
               l.match_method, l.match_confidence::float8 AS match_confidence,
               COALESCE(l.field_conflicts, '{}'::jsonb) AS field_conflicts,
               l.source_recorded_at, raw.received_at, raw.raw_payload
          FROM raw_records raw
          JOIN record_sources s ON s.id = raw.source_id
          LEFT JOIN record_source_links l ON l.raw_record_id = raw.id
         WHERE raw.canonical_record_id = $1
         ORDER BY raw.received_at ASC, raw.id ASC
        "#,
    )
    .bind(record_id)
    .fetch_all(pool)
    .await
}

pub async fn replace_external_servers(
    pool: &PgPool,
    source_id: i64,
    servers: &[ExternalServerSnapshot],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE external_servers SET is_online = FALSE WHERE source_id = $1")
        .bind(source_id)
        .execute(&mut *tx)
        .await?;
    for server in servers {
        sqlx::query(
            r#"
            INSERT INTO external_servers
                (source_id, external_id, name, label, host, port, map_name, map_tier,
                 online_players, max_players, is_online, tags, raw_payload, last_seen_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, TRUE, '[]'::jsonb, '{}'::jsonb, now())
            ON CONFLICT (source_id, external_id) DO UPDATE
               SET name = EXCLUDED.name, label = EXCLUDED.label,
                   host = EXCLUDED.host, port = EXCLUDED.port,
                   map_name = EXCLUDED.map_name, map_tier = EXCLUDED.map_tier,
                   online_players = EXCLUDED.online_players,
                   max_players = EXCLUDED.max_players,
                   is_online = TRUE, last_seen_at = now()
            "#,
        )
        .bind(source_id)
        .bind(&server.external_id)
        .bind(&server.name)
        .bind(&server.label)
        .bind(&server.host)
        .bind(server.port)
        .bind(&server.map_name)
        .bind(server.map_tier)
        .bind(server.online_players)
        .bind(server.max_players)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn list_external_servers(
    pool: &PgPool,
    max_age_seconds: i64,
) -> Result<Vec<ExternalServerSnapshot>, sqlx::Error> {
    sqlx::query_as::<_, ExternalServerSnapshot>(
        r#"
        SELECT e.id, s.code AS source_code, e.external_id, e.name, e.label, e.host, e.port,
               e.map_name, e.map_tier, e.online_players, e.max_players,
               e.is_online, e.last_seen_at
          FROM external_servers e
          JOIN record_sources s ON s.id = e.source_id
         WHERE s.code = 'kzgo'
           AND s.is_enabled
           AND e.is_online
           AND e.last_seen_at > now() - make_interval(secs => $1)
         ORDER BY e.name
        "#,
    )
    .bind(max_age_seconds as f64)
    .fetch_all(pool)
    .await
}

pub async fn external_source_fresh(
    pool: &PgPool,
    max_age_seconds: i64,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM record_sources
             WHERE code = 'kzgo' AND is_enabled
               AND last_success_at IS NOT NULL
               AND last_success_at > now() - make_interval(secs => $1)
        )
        "#,
    )
    .bind(max_age_seconds as f64)
    .fetch_one(pool)
    .await
}
