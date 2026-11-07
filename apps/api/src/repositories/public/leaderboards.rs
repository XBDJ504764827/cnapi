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
    pub avatar_url: Option<String>,
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
    let board_mode = mode_filter("b", mode_id);
    let board_tt = time_type_filter("b", time_type);
    let wr_mode = mode_filter("w", mode_id);
    let wr_tt = time_type_filter("w", time_type);

    // 读数来自物化视图 mv_player_board / mv_player_wr（每人每模式每类型一行，
    // 几百行），不再扫 records 的 160 万行。视图由 refresh_board_views 在记录
    // 变化后刷新（积分重算 worker 队列排空时触发）。
    format!(
        r#"
        WITH rec AS (
            SELECT b.account_id,
                   sum(b.total_records)::bigint AS total_records,
                   sum(b.completed_maps)::bigint AS completed_maps,
                   max(b.last_activity_at) AS last_activity_at
              FROM mv_player_board b
             WHERE TRUE{board_mode}{board_tt}
             GROUP BY b.account_id
        ),
        wr AS (
            SELECT w.account_id, sum(w.wr_count)::bigint AS wr_count
              FROM mv_player_wr w
             WHERE TRUE{wr_mode}{wr_tt}
             GROUP BY w.account_id
        ),
        stats AS (
            SELECT a.id AS account_id,
                   a.steam_id64::text AS steam_id64,
                   COALESCE(p.display_name, 'Unknown player') AS display_name,
                   p.avatar_url,
                   p.country,
                   COALESCE(rec.completed_maps, 0) AS completed_maps,
                   COALESCE(rec.total_records, 0) AS total_records,
                   COALESCE(wr.wr_count, 0) AS wr_count,
                   rec.last_activity_at
              FROM accounts a
              LEFT JOIN player_profiles p ON p.account_id = a.id
              LEFT JOIN rec ON rec.account_id = a.id
              LEFT JOIN wr ON wr.account_id = a.id
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

/// 刷新排行榜物化视图。只在记录发生变化后调用。
pub async fn refresh_board_views(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY mv_player_board")
        .execute(pool)
        .await?;
    sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY mv_player_wr")
        .execute(pool)
        .await?;
    Ok(())
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
