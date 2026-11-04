//! Filter-level points recalculation.
//!
//! All PB/WR/rank aggregation is done in PostgreSQL in one transaction. The
//! pure formula is applied once per PB row in Rust; results are upserted in a
//! single batch and stale rows are removed atomically. Recalculation is
//! idempotent: running it twice on unchanged records produces identical rows.

use std::time::{Duration, Instant};

use sqlx::{PgPool, Postgres, Transaction};
use tracing::{info, warn};
use uuid::Uuid;

use super::formula::{ScoringInput, calculate_points};
use super::model::{RecordType, Tier};
use crate::state::AppState;

#[derive(Debug, sqlx::FromRow)]
struct TaskRow {
    id: i64,
    filter_id: i64,
    mode_id: i16,
}

#[derive(Debug, sqlx::FromRow)]
struct PbRow {
    account_id: i64,
    pb_record_id: i64,
    time_ns: i64,
    rank_position: i64,
    wr_time_ns: i64,
    pb_count: i64,
    tier: i16,
    record_type: String,
    is_first_completion: bool,
}

#[derive(Debug, sqlx::FromRow)]
pub struct RecalcStatus {
    pub pending: i64,
    pub running: i64,
    pub failed: i64,
    pub last_recalculated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Claims one pending task with `FOR UPDATE SKIP LOCKED`, preventing duplicate
/// concurrent recomputation of the same Filter.
async fn claim_task(pool: &PgPool, worker_id: &str) -> Result<Option<TaskRow>, sqlx::Error> {
    sqlx::query_as::<_, TaskRow>(
        r#"
        WITH claimed AS (
            SELECT id, filter_id, mode_id
              FROM filter_recalc_tasks
             WHERE status = 'pending'
             ORDER BY requested_at, id
             FOR UPDATE SKIP LOCKED
             LIMIT 1
        )
        UPDATE filter_recalc_tasks t
           SET status = 'running', started_at = now(), attempts = t.attempts + 1, worker_id = $1
          FROM claimed c
         WHERE t.id = c.id
        RETURNING t.id, t.filter_id, t.mode_id
        "#,
    )
    .bind(worker_id)
    .fetch_optional(pool)
    .await
}

pub async fn recalc_status(pool: &PgPool) -> Result<RecalcStatus, sqlx::Error> {
    sqlx::query_as::<_, RecalcStatus>(
        r#"
        SELECT
            count(*) FILTER (WHERE status = 'pending')::bigint AS pending,
            count(*) FILTER (WHERE status = 'running')::bigint AS running,
            count(*) FILTER (WHERE status = 'failed')::bigint AS failed,
            (SELECT max(last_recalculated_at)::timestamptz FROM scoring_filters) AS last_recalculated_at
        FROM filter_recalc_tasks
        "#,
    )
    .fetch_one(pool)
    .await
}

/// Recalculates one Filter inside a transaction. Returns the number of PBs.
pub async fn recalc_filter(pool: &PgPool, filter_id: i64) -> Result<i64, anyhow::Error> {
    let mut tx = pool.begin().await?;

    let filter: Option<(i64, i64, i16, String, i16)> = sqlx::query_as(
        r#"
        SELECT map_id, course_id, mode_id, record_type, tier
          FROM scoring_filters WHERE id = $1 FOR UPDATE
        "#,
    )
    .bind(filter_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some((map_id, course_id, mode_id, record_type, tier)) = filter else {
        tx.rollback().await?;
        anyhow::bail!("scoring filter {filter_id} not found");
    };

    // One row per player: their fastest valid record in this exact Filter.
    // Style is not part of a Filter; all styles are aggregated by min(time_ns).
    let pbs: Vec<PbRow> = sqlx::query_as::<_, PbRow>(
        r#"
        WITH pbs AS (
            SELECT r.account_id,
                   (array_agg(r.id ORDER BY r.time_ns ASC, r.id ASC))[1] AS pb_record_id,
                   min(r.time_ns) AS time_ns,
                   row_number() OVER (ORDER BY min(r.time_ns) ASC, min(r.id) ASC) AS rank_position
              FROM records r
             WHERE r.status = 'approved' AND r.deleted_at IS NULL
               AND r.map_id = $1 AND r.course_id = $2 AND r.mode_id = $3 AND r.time_type = $4
             GROUP BY r.account_id
        ),
        stats AS (SELECT count(*)::bigint AS n, min(time_ns) AS wr FROM pbs)
        SELECT p.account_id, p.pb_record_id, p.time_ns, p.rank_position::bigint AS rank_position,
               s.wr::bigint AS wr_time_ns, s.n AS pb_count, $5::smallint AS tier,
               $4::text AS record_type,
               EXISTS (
                   SELECT 1 FROM filter_first_completions fc
                    WHERE fc.record_id = p.pb_record_id
               ) AS is_first_completion
          FROM pbs p CROSS JOIN stats s
         ORDER BY p.rank_position
        "#,
    )
    .bind(map_id)
    .bind(course_id)
    .bind(mode_id)
    .bind(&record_type)
    .bind(tier)
    .fetch_all(&mut *tx)
    .await?;

    // All writes happen in this transaction. Stale player scores are removed
    // first, then the complete new set is inserted in one statement.
    sqlx::query("DELETE FROM player_filter_points WHERE filter_id = $1")
        .bind(filter_id)
        .execute(&mut *tx)
        .await?;

    let mut calculated = Vec::with_capacity(pbs.len());
    for pb in &pbs {
        let scoring_tier = if pb.is_first_completion {
            Tier::new(9)?
        } else {
            Tier::new(pb.tier as u8)?
        };
        let record_type_enum = RecordType::parse(&pb.record_type)
            .ok_or_else(|| anyhow::anyhow!("invalid record_type {}", pb.record_type))?;

        let output = calculate_points(ScoringInput {
            tier: scoring_tier,
            record_type: record_type_enum,
            rank: pb.rank_position as u32,
            time: pb.time_ns as f64 / 1_000_000_000.0,
            wr: pb.wr_time_ns as f64 / 1_000_000_000.0,
            pb_count: pb.pb_count as u64,
        })?;

        calculated.push((
            pb.account_id,
            pb.pb_record_id,
            pb.time_ns,
            pb.rank_position as i32,
            pb.tier,
            pb.wr_time_ns,
            pb.pb_count,
            output.points as i32,
            pb.rank_position == 1,
        ));
    }

    if !calculated.is_empty() {
        let mut builder: sqlx::QueryBuilder<Postgres> = sqlx::QueryBuilder::new(
            "INSERT INTO player_filter_points
             (filter_id, account_id, map_id, course_id, mode_id, record_type, pb_record_id,
              time_ns, rank, tier, wr_time_ns, pb_count, points, is_world_record, calculated_at) ",
        );
        builder.push_values(calculated.iter(), |mut b, row| {
            b.push_bind(filter_id)
                .push_bind(row.0)
                .push_bind(map_id)
                .push_bind(course_id)
                .push_bind(mode_id)
                .push_bind(&record_type)
                .push_bind(row.1)
                .push_bind(row.2)
                .push_bind(row.3)
                .push_bind(row.4)
                .push_bind(row.5)
                .push_bind(row.6)
                .push_bind(row.7)
                .push_bind(row.8)
                .push_bind(chrono::Utc::now());
        });

        builder.build().execute(&mut *tx).await?;
    }

    rebuild_mode_totals(&mut tx, mode_id).await?;

    sqlx::query(
        "UPDATE scoring_filters
            SET dirty = FALSE, version = version + 1, last_recalculated_at = now()
          WHERE id = $1",
    )
    .bind(filter_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(pbs.len() as i64)
}

/// Rebuilds PRO/NUB/total for every player of one mode, from persisted Filter
/// points. A KZT recalc can never alter SKZ or VNL rows.
async fn rebuild_mode_totals(
    tx: &mut Transaction<'_, Postgres>,
    mode_id: i16,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        WITH totals AS (
            SELECT account_id,
                   COALESCE(sum(points) FILTER (WHERE record_type = 'pro'), 0)::integer AS pro_points,
                   COALESCE(sum(points) FILTER (WHERE record_type = 'nub'), 0)::integer AS nub_points,
                   count(*)::integer AS filter_count
              FROM player_filter_points
             WHERE mode_id = $1
             GROUP BY account_id
        )
        INSERT INTO player_mode_points AS pmp
            (mode_id, account_id, pro_points, nub_points, total_points, filter_count)
        SELECT $1, t.account_id, t.pro_points, t.nub_points,
               t.pro_points + t.nub_points, t.filter_count
          FROM totals t
        ON CONFLICT (mode_id, account_id) DO UPDATE
           SET pro_points = EXCLUDED.pro_points,
               nub_points = EXCLUDED.nub_points,
               total_points = EXCLUDED.total_points,
               filter_count = EXCLUDED.filter_count,
               updated_at = now()
        "#,
    )
    .bind(mode_id)
    .execute(&mut **tx)
    .await?;

    // Remove players with no remaining PB in this mode.
    sqlx::query(
        "DELETE FROM player_mode_points pmp
          WHERE pmp.mode_id = $1
            AND NOT EXISTS (
                SELECT 1 FROM player_filter_points pfp
                 WHERE pfp.mode_id = pmp.mode_id AND pfp.account_id = pmp.account_id
            )",
    )
    .bind(mode_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Marks all Filters of one mode dirty (used only by the protected admin
/// full-recalc endpoint). Never crosses into another mode.
pub async fn mark_all_dirty(pool: &PgPool, mode_id: i16) -> Result<i64, sqlx::Error> {
    let affected = sqlx::query_scalar::<_, i64>(
        r#"
        WITH marked AS (
            UPDATE scoring_filters SET dirty = TRUE WHERE mode_id = $1 RETURNING id
        ),
        inserted AS (
            INSERT INTO filter_recalc_tasks (filter_id, mode_id)
            SELECT id, $1 FROM marked
            ON CONFLICT (filter_id, status) DO NOTHING
            RETURNING id
        )
        SELECT count(*)::bigint FROM marked
        "#,
    )
    .bind(mode_id)
    .fetch_one(pool)
    .await?;
    Ok(affected)
}

/// Enqueues one Filter if it exists. Used by the protected admin endpoint.
pub async fn mark_filter_dirty(pool: &PgPool, filter_id: i64) -> Result<bool, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i16>(
        "UPDATE scoring_filters SET dirty = TRUE WHERE id = $1 RETURNING mode_id",
    )
    .bind(filter_id)
    .fetch_optional(pool)
    .await?;

    if let Some(mode_id) = row {
        sqlx::query(
            "INSERT INTO filter_recalc_tasks (filter_id, mode_id)
             VALUES ($1, $2)
             ON CONFLICT (filter_id, status) DO NOTHING",
        )
        .bind(filter_id)
        .bind(mode_id)
        .execute(pool)
        .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Worker loop: claims tasks one at a time, recomputes, records success/failure
/// and invalidates only the affected mode's cache namespace.
pub async fn run_worker(state: AppState, mut shutdown: tokio::sync::watch::Receiver<bool>) {
    let worker_id = Uuid::new_v4().simple().to_string();
    info!(worker = %worker_id, "points recalc worker started");

    loop {
        if *shutdown.borrow() {
            break;
        }

        let task = match claim_task(state.db(), &worker_id).await {
            Ok(Some(task)) => task,
            Ok(None) => {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_millis(750)) => continue,
                    _ = shutdown.changed() => break,
                }
            }
            Err(error) => {
                warn!(error = %error, "failed to claim recalc task");
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(2)) => continue,
                    _ = shutdown.changed() => break,
                }
            }
        };

        let started = Instant::now();
        let result = recalc_filter(state.db(), task.filter_id).await;
        let duration_ms = started.elapsed().as_millis().min(i32::MAX as u128) as i32;

        match result {
            Ok(pb_count) => {
                if let Err(error) = finish_task(
                    state.db(),
                    task.id,
                    task.filter_id,
                    task.mode_id,
                    true,
                    pb_count,
                    duration_ms,
                    None,
                )
                .await
                {
                    warn!(error = %error, task_id = task.id, "failed to finalize successful recalc");
                }
                if let Err(error) =
                    crate::services::public::cache::invalidate_mode(state.redis(), task.mode_id)
                        .await
                {
                    warn!(error = %error, mode_id = task.mode_id, "failed to invalidate mode cache");
                }
                info!(
                    filter_id = task.filter_id,
                    mode_id = task.mode_id,
                    pb_count,
                    duration_ms,
                    "filter recalculated"
                );
            }
            Err(error) => {
                let message = format!("{error:#}");
                if let Err(db_error) = finish_task(
                    state.db(),
                    task.id,
                    task.filter_id,
                    task.mode_id,
                    false,
                    0,
                    duration_ms,
                    Some(&message),
                )
                .await
                {
                    warn!(error = %db_error, "failed to record recalc failure");
                }
                warn!(error = %message, filter_id = task.filter_id, "filter recalc failed");
            }
        }
    }

    info!(worker = %worker_id, "points recalc worker stopped");
}

#[allow(clippy::too_many_arguments)]
async fn finish_task(
    pool: &PgPool,
    task_id: i64,
    filter_id: i64,
    mode_id: i16,
    success: bool,
    pb_count: i64,
    duration_ms: i32,
    error: Option<&str>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let status = if success { "done" } else { "failed" };
    sqlx::query(
        "UPDATE filter_recalc_tasks
            SET status = $2, completed_at = now(), last_error = $3, worker_id = NULL
          WHERE id = $1",
    )
    .bind(task_id)
    .bind(status)
    .bind(error)
    .execute(&mut *tx)
    .await?;

    // Keep a bounded audit trail: done rows are not needed after completion.
    sqlx::query("DELETE FROM filter_recalc_tasks WHERE id = $1 AND status = 'done'")
        .bind(task_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO filter_recalc_runs
            (filter_id, mode_id, status, pb_count, duration_ms, error)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(filter_id)
    .bind(mode_id)
    .bind(if success { "success" } else { "failed" })
    .bind(pb_count)
    .bind(duration_ms)
    .bind(error)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

/// Idempotent initialization for deployments with records inserted before the
/// scoring system was enabled. It never fabricates T9 history: existing
/// completions only set `completed=TRUE` and an audit anchor. Tier comes from
/// the configured `course_tiers` table, and every discovered Filter is queued.
pub async fn initialize_existing(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO course_mode_tiers
            (map_id, course_id, mode_id, tier, completed,
             first_completion_record_id, first_completion_account_id, first_completion_at)
        SELECT c.map_id, c.id, sm.mode_id, COALESCE(ct.pro_tier, ct.nub_tier),
               first.id IS NOT NULL, first.id, first.account_id, first.created_at
          FROM map_courses c
          JOIN scoring_modes sm ON TRUE
          JOIN course_tiers ct ON ct.course_id = c.id AND ct.mode_id = sm.mode_id
          LEFT JOIN LATERAL (
              SELECT r.id, r.account_id, r.created_at
                FROM records r
               WHERE r.course_id = c.id AND r.map_id = c.map_id AND r.mode_id = sm.mode_id
                 AND r.status = 'approved' AND r.deleted_at IS NULL
               ORDER BY r.created_at ASC, r.id ASC LIMIT 1
          ) first ON TRUE
         WHERE COALESCE(ct.pro_tier, ct.nub_tier) BETWEEN 1 AND 8
        ON CONFLICT (map_id, course_id, mode_id) DO NOTHING
        "#,
    )
    .execute(&mut *tx)
    .await?;

    let inserted = sqlx::query_scalar::<_, i64>(
        r#"
        WITH inserted AS (
            INSERT INTO scoring_filters (map_id, course_id, mode_id, record_type, tier, dirty)
            SELECT DISTINCT r.map_id, r.course_id, r.mode_id, r.time_type, cmt.tier, TRUE
              FROM records r
              JOIN course_mode_tiers cmt
                ON cmt.map_id = r.map_id AND cmt.course_id = r.course_id AND cmt.mode_id = r.mode_id
             WHERE r.status = 'approved' AND r.deleted_at IS NULL
            ON CONFLICT (map_id, course_id, mode_id, record_type) DO UPDATE SET dirty = TRUE
            RETURNING id, mode_id
        ),
        queued AS (
            INSERT INTO filter_recalc_tasks (filter_id, mode_id)
            SELECT id, mode_id FROM inserted
            ON CONFLICT (filter_id, status) DO NOTHING
            RETURNING id
        )
        SELECT count(*)::bigint FROM inserted
        "#,
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(inserted)
}
