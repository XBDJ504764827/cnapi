//! Public leaderboard and player directory queries.
//!
//! Ranking metric: `completed_maps` (main-course completions), tie-broken by
//! world records and total records. Points are not computed because CNGOKZAPI
//! has no points algorithm yet.

use chrono::{DateTime, Utc};
use sqlx::{AssertSqlSafe, PgPool};

use crate::models::TimeType;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LeaderboardRow {
    pub account_id: i64,
    pub steam_id64: String,
    pub display_name: String,
    pub country: Option<String>,
    pub completed_maps: i64,
    pub wr_count: i64,
    pub total_records: i64,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub rank_position: i64,
}

fn mode_filter(alias: &str, mode_id: Option<i16>) -> String {
    match mode_id {
        Some(id) => format!(" AND {alias}.mode_id = {id}"),
        None => String::new(),
    }
}

fn time_type_filter(alias: &str, time_type: Option<TimeType>) -> String {
    match time_type {
        Some(value) => format!(" AND {alias}.time_type = '{}'", value.as_str()),
        None => String::new(),
    }
}

fn stats_cte(mode_id: Option<i16>, time_type: Option<TimeType>) -> String {
    let record_mode = mode_filter("r", mode_id);
    let record_tt = time_type_filter("r", time_type);
    let wr_mode = mode_filter("wr", mode_id);
    let wr_tt = time_type_filter("wr", time_type);

    format!(
        r#"
        WITH stats AS (
            SELECT a.id AS account_id,
                   a.steam_id64::text AS steam_id64,
                   COALESCE(p.display_name, 'Unknown player') AS display_name,
                   p.country,
                   (SELECT count(DISTINCT r.map_id)
                      FROM records r
                      JOIN map_courses c ON c.id = r.course_id AND c.stage = 0
                     WHERE r.account_id = a.id
                       AND r.status = 'approved'
                       AND r.deleted_at IS NULL{record_mode}{record_tt}) AS completed_maps,
                   (SELECT count(*)
                      FROM records r
                     WHERE r.account_id = a.id
                       AND r.status = 'approved'
                       AND r.deleted_at IS NULL{record_mode}{record_tt}) AS total_records,
                   (SELECT count(*)
                      FROM course_world_records w
                      JOIN records wr ON wr.id = w.record_id
                     WHERE wr.account_id = a.id
                       AND wr.status = 'approved'
                       AND wr.deleted_at IS NULL{wr_mode}{wr_tt}) AS wr_count,
                   (SELECT max(r.created_at)
                      FROM records r
                     WHERE r.account_id = a.id
                       AND r.status = 'approved'
                       AND r.deleted_at IS NULL{record_mode}{record_tt}) AS last_activity_at
              FROM accounts a
              LEFT JOIN player_profiles p ON p.account_id = a.id
             WHERE a.deleted_at IS NULL
        ),
        ranked AS (
            SELECT *,
                   RANK() OVER (
                       ORDER BY completed_maps DESC, wr_count DESC, total_records DESC, account_id
                   ) AS rank_position
              FROM stats
        )
        "#
    )
}

pub async fn leaderboard(
    pool: &PgPool,
    mode_id: i16,
    time_type: Option<TimeType>,
    limit: i64,
    offset: i64,
) -> Result<Vec<LeaderboardRow>, sqlx::Error> {
    let sql = format!(
        "{} SELECT * FROM ranked WHERE total_records > 0 ORDER BY rank_position, account_id LIMIT $1 OFFSET $2",
        stats_cte(Some(mode_id), time_type)
    );
    sqlx::query_as::<_, LeaderboardRow>(AssertSqlSafe(sql))
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
}

pub async fn leaderboard_total(
    pool: &PgPool,
    mode_id: i16,
    time_type: Option<TimeType>,
) -> Result<i64, sqlx::Error> {
    let tt_filter = match time_type {
        Some(value) => format!(" AND r.time_type = '{}'", value.as_str()),
        None => String::new(),
    };
    let sql = format!(
        "SELECT count(DISTINCT r.account_id)
           FROM records r
           JOIN accounts a ON a.id = r.account_id AND a.deleted_at IS NULL
          WHERE r.status = 'approved' AND r.deleted_at IS NULL
            AND r.mode_id = $1{tt_filter}"
    );
    sqlx::query_scalar::<_, i64>(AssertSqlSafe(sql))
        .bind(mode_id)
        .fetch_one(pool)
        .await
}

pub async fn player_directory(
    pool: &PgPool,
    search: Option<&str>,
    mode_id: Option<i16>,
    time_type: Option<TimeType>,
    sort: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<LeaderboardRow>, sqlx::Error> {
    let order = match sort {
        "wr" => "wr_count DESC, rank_position ASC, account_id ASC",
        "records" => "total_records DESC, rank_position ASC, account_id ASC",
        "recent" => "last_activity_at DESC NULLS LAST, rank_position ASC, account_id ASC",
        "name" => "display_name ASC, account_id ASC",
        _ => "rank_position ASC, account_id ASC",
    };
    let sql = format!(
        "{} SELECT * FROM ranked
         WHERE ($1::text IS NULL
                OR display_name ILIKE '%' || $1 || '%'
                OR steam_id64 LIKE $1 || '%')
         ORDER BY {order} LIMIT $2 OFFSET $3",
        stats_cte(mode_id, time_type)
    );
    sqlx::query_as::<_, LeaderboardRow>(AssertSqlSafe(sql))
        .bind(search)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
}

pub async fn player_directory_total(
    pool: &PgPool,
    search: Option<&str>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM accounts a
        LEFT JOIN player_profiles p ON p.account_id = a.id
        WHERE a.deleted_at IS NULL
          AND ($1::text IS NULL
               OR p.display_name ILIKE '%' || $1 || '%'
               OR a.steam_id64::text LIKE $1 || '%')
        "#,
    )
    .bind(search)
    .fetch_one(pool)
    .await
}

/// Global completion rank for a single account (same metric as the leaderboard).
pub async fn account_rank(
    pool: &PgPool,
    account_id: i64,
    mode_id: Option<i16>,
    time_type: Option<TimeType>,
) -> Result<Option<i64>, sqlx::Error> {
    let sql = format!(
        "{} SELECT rank_position FROM ranked WHERE account_id = $1",
        stats_cte(mode_id, time_type)
    );
    sqlx::query_scalar::<_, i64>(AssertSqlSafe(sql))
        .bind(account_id)
        .fetch_optional(pool)
        .await
}
