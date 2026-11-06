//! Public player statistics, map progress and comparison queries.

use chrono::{DateTime, Utc};
use sqlx::{AssertSqlSafe, PgPool};

use crate::models::TimeType;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OverallRow {
    pub total_records: i64,
    pub pro_count: i64,
    pub tp_count: i64,
    pub completed_maps: i64,
    pub total_maps: i64,
    pub wr_count: i64,
    pub first_record_at: Option<DateTime<Utc>>,
    pub last_record_at: Option<DateTime<Utc>>,
}

pub async fn overall(pool: &PgPool, account_id: i64) -> Result<OverallRow, sqlx::Error> {
    sqlx::query_as::<_, OverallRow>(
        r#"
        SELECT
            (SELECT count(*) FROM records r
              WHERE r.account_id = $1 AND r.status = 'approved' AND r.deleted_at IS NULL) AS total_records,
            (SELECT count(*) FROM records r
              WHERE r.account_id = $1 AND r.status = 'approved' AND r.deleted_at IS NULL
                AND r.time_type = 'pro') AS pro_count,
            (SELECT count(*) FROM records r
              WHERE r.account_id = $1 AND r.status = 'approved' AND r.deleted_at IS NULL
                AND r.time_type = 'nub') AS tp_count,
            (SELECT count(DISTINCT r.map_id) FROM records r
              JOIN map_courses c ON c.id = r.course_id AND c.stage = 0
              WHERE r.account_id = $1 AND r.status = 'approved' AND r.deleted_at IS NULL) AS completed_maps,
            (SELECT count(*) FROM maps m
              WHERE m.status = 'approved' AND m.deleted_at IS NULL) AS total_maps,
            (SELECT count(*) FROM course_world_records w
              JOIN records wr ON wr.id = w.record_id
              WHERE wr.account_id = $1 AND wr.status = 'approved' AND wr.deleted_at IS NULL) AS wr_count,
            (SELECT min(r.created_at) FROM records r
              WHERE r.account_id = $1 AND r.status = 'approved' AND r.deleted_at IS NULL) AS first_record_at,
            (SELECT max(r.created_at) FROM records r
              WHERE r.account_id = $1 AND r.status = 'approved' AND r.deleted_at IS NULL) AS last_record_at
        "#,
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
}

/// Overall stats for a player under one explicit scope.
///
/// `None` filters mean "all modes / all time types / all styles". `completed_maps`
/// always means: distinct approved maps with a main-course (stage=0) record.
pub async fn scoped_overall(
    pool: &PgPool,
    account_id: i64,
    mode_id: Option<i16>,
    time_type: Option<TimeType>,
    style_id: Option<i16>,
) -> Result<OverallRow, sqlx::Error> {
    // Whitelisted, parameterized SQL fragments built only from enum/id values.
    let mode_filter = match mode_id {
        Some(_) => " AND r.mode_id = $2",
        None => "",
    };
    let tt_filter = match time_type {
        Some(TimeType::Pro) => " AND r.time_type = 'pro'",
        Some(TimeType::Nub) => " AND r.time_type = 'nub'",
        None => "",
    };
    let style_filter = match style_id {
        Some(_) => " AND r.style_id = $3",
        None => "",
    };
    let wr_mode = match mode_id {
        Some(_) => " AND wr.mode_id = $2",
        None => "",
    };
    let wr_tt = match time_type {
        Some(TimeType::Pro) => " AND wr.time_type = 'pro'",
        Some(TimeType::Nub) => " AND wr.time_type = 'nub'",
        None => "",
    };
    let wr_style = match style_id {
        Some(_) => " AND wr.style_id = $3",
        None => "",
    };

    let sql = format!(
        r#"
        SELECT
            (SELECT count(*) FROM records r
              WHERE r.account_id = $1 AND r.status = 'approved' AND r.deleted_at IS NULL
                {mode_filter}{tt_filter}{style_filter}) AS total_records,
            (SELECT count(*) FROM records r
              WHERE r.account_id = $1 AND r.status = 'approved' AND r.deleted_at IS NULL
                AND r.time_type = 'pro'{mode_filter}{tt_filter}{style_filter}) AS pro_count,
            (SELECT count(*) FROM records r
              WHERE r.account_id = $1 AND r.status = 'approved' AND r.deleted_at IS NULL
                AND r.time_type = 'nub'{mode_filter}{tt_filter}{style_filter}) AS tp_count,
            (SELECT count(DISTINCT r.map_id) FROM records r
              JOIN map_courses c ON c.id = r.course_id AND c.stage = 0
              WHERE r.account_id = $1 AND r.status = 'approved' AND r.deleted_at IS NULL
                {mode_filter}{tt_filter}{style_filter}) AS completed_maps,
            (SELECT count(*) FROM maps m
              WHERE m.status = 'approved' AND m.deleted_at IS NULL
                AND ($2::smallint IS NULL OR EXISTS (
                    SELECT 1 FROM map_modes mm WHERE mm.map_id = m.id AND mm.mode_id = $2
                ))) AS total_maps,
            (SELECT count(*) FROM course_world_records w
              JOIN records wr ON wr.id = w.record_id
              WHERE wr.account_id = $1 AND wr.status = 'approved' AND wr.deleted_at IS NULL
                {wr_mode}{wr_tt}{wr_style}) AS wr_count,
            (SELECT min(r.created_at) FROM records r
              WHERE r.account_id = $1 AND r.status = 'approved' AND r.deleted_at IS NULL
                {mode_filter}{tt_filter}{style_filter}) AS first_record_at,
            (SELECT max(r.created_at) FROM records r
              WHERE r.account_id = $1 AND r.status = 'approved' AND r.deleted_at IS NULL
                {mode_filter}{tt_filter}{style_filter}) AS last_record_at
        "#
    );

    let mut query = sqlx::query_as::<_, OverallRow>(AssertSqlSafe(sql)).bind(account_id);
    query = query.bind(mode_id);
    query = query.bind(style_id);
    query.fetch_one(pool).await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ModeStatsRow {
    pub mode_id: i16,
    pub code: String,
    pub display_name: String,
    pub total_records: i64,
    pub pro_count: i64,
    pub tp_count: i64,
    pub completed_maps: i64,
    pub total_maps: i64,
    pub wr_count: i64,
}

pub async fn mode_stats(pool: &PgPool, account_id: i64) -> Result<Vec<ModeStatsRow>, sqlx::Error> {
    sqlx::query_as::<_, ModeStatsRow>(
        r#"
        SELECT mo.id AS mode_id,
               mo.code,
               mo.display_name,
               count(r.id)::bigint AS total_records,
               count(*) FILTER (WHERE r.time_type = 'pro')::bigint AS pro_count,
               count(*) FILTER (WHERE r.time_type = 'nub')::bigint AS tp_count,
               count(DISTINCT r.map_id) FILTER (WHERE c.stage = 0)::bigint AS completed_maps,
               (SELECT count(*) FROM map_modes mm
                 JOIN maps m ON m.id = mm.map_id
                WHERE mm.mode_id = mo.id
                  AND m.status = 'approved' AND m.deleted_at IS NULL) AS total_maps,
               (SELECT count(*) FROM course_world_records w
                 JOIN records wr ON wr.id = w.record_id
                WHERE wr.account_id = $1
                  AND wr.mode_id = mo.id
                  AND wr.status = 'approved' AND wr.deleted_at IS NULL) AS wr_count
        FROM movement_modes mo
        LEFT JOIN records r ON r.mode_id = mo.id
                           AND r.account_id = $1
                           AND r.status = 'approved'
                           AND r.deleted_at IS NULL
        LEFT JOIN map_courses c ON c.id = r.course_id
        GROUP BY mo.id
        ORDER BY mo.id
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TierRow {
    pub tier: Option<i16>,
    pub map_count: i64,
    pub completed_count: i64,
}

#[allow(clippy::too_many_arguments)]
pub async fn tier_progress(
    pool: &PgPool,
    account_id: i64,
    mode_id: i16,
    time_type: Option<TimeType>,
    style_id: Option<i16>,
) -> Result<Vec<TierRow>, sqlx::Error> {
    let tier_column = match time_type {
        Some(TimeType::Pro) => "ct.pro_tier",
        _ => "ct.nub_tier",
    };
    let tt_filter = match time_type {
        Some(value) => format!(" AND r.time_type = '{}'", value.as_str()),
        None => String::new(),
    };
    let style_filter = match style_id {
        Some(_) => " AND r.style_id = $3",
        None => "",
    };

    let sql = format!(
        r#"
        WITH map_tiers AS (
            SELECT m.id AS map_id, {tier_column} AS tier
              FROM maps m
              JOIN map_courses c ON c.map_id = m.id AND c.stage = 0
              JOIN map_modes mm ON mm.map_id = m.id AND mm.mode_id = $2
              LEFT JOIN course_tiers ct ON ct.course_id = c.id AND ct.mode_id = $2
             WHERE m.status = 'approved' AND m.deleted_at IS NULL
        ),
        player_maps AS (
            SELECT DISTINCT r.map_id
              FROM records r
              JOIN map_courses c ON c.id = r.course_id AND c.stage = 0
             WHERE r.account_id = $1
               AND r.mode_id = $2
               AND r.status = 'approved'
               AND r.deleted_at IS NULL{tt_filter}{style_filter}
        )
        SELECT mt.tier,
               count(*)::bigint AS map_count,
               count(pm.map_id)::bigint AS completed_count
          FROM map_tiers mt
          LEFT JOIN player_maps pm ON pm.map_id = mt.map_id
         GROUP BY mt.tier
         ORDER BY mt.tier NULLS LAST
        "#
    );

    sqlx::query_as::<_, TierRow>(AssertSqlSafe(sql))
        .bind(account_id)
        .bind(mode_id)
        .bind(style_id)
        .fetch_all(pool)
        .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MapProgressRow {
    pub map_id: i64,
    pub name: String,
    pub tier: Option<i16>,
    pub completed: bool,
    pub time_ns: Option<i64>,
    pub best_rank: Option<i64>,
    pub best_time_type: Option<String>,
    pub last_record_at: Option<DateTime<Utc>>,
    pub bonus_total: i64,
    pub bonus_completed: i64,
}

fn map_base_cte(tier_column: &str, mode_param: &str) -> String {
    format!(
        r#"
        WITH map_base AS (
            SELECT m.id AS map_id,
                   m.name,
                   {tier_column} AS tier,
                   (SELECT count(*) FROM map_courses bc WHERE bc.map_id = m.id AND bc.stage > 0)::bigint AS bonus_total
              FROM maps m
              JOIN map_modes mm ON mm.map_id = m.id AND mm.mode_id = {mode_param}
              JOIN map_courses mc ON mc.map_id = m.id AND mc.stage = 0
              LEFT JOIN course_tiers ct ON ct.course_id = mc.id AND ct.mode_id = {mode_param}
             WHERE m.status = 'approved' AND m.deleted_at IS NULL
        )
        "#
    )
}

fn best_lateral(
    alias: &str,
    account_param: &str,
    mode_param: &str,
    style_param: &str,
    tt_filter: &str,
) -> String {
    format!(
        r#"
        LEFT JOIN LATERAL (
            SELECT r.id,
                   r.time_ns,
                   r.created_at,
                   r.time_type,
                   (SELECT count(*) + 1 FROM records f
                     WHERE f.course_id = r.course_id
                       AND f.mode_id = r.mode_id
                       AND f.style_id = r.style_id
                       AND f.time_type = r.time_type
                       AND f.status = 'approved'
                       AND f.deleted_at IS NULL
                       AND f.time_ns < r.time_ns) AS best_rank
              FROM records r
              JOIN map_courses c2 ON c2.id = r.course_id AND c2.stage = 0
             WHERE r.account_id = {account_param}
               AND r.map_id = mb.map_id
               AND r.mode_id = {mode_param}
               AND r.style_id = {style_param}
               AND r.status = 'approved'
               AND r.deleted_at IS NULL{tt_filter}
             ORDER BY r.time_ns ASC, r.id ASC
             LIMIT 1
        ) {alias} ON TRUE
        "#
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn map_progress(
    pool: &PgPool,
    account_id: i64,
    mode_id: i16,
    style_id: i16,
    time_type: Option<TimeType>,
    status: &str,
    search: Option<&str>,
    tier: Option<i16>,
    sort: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<MapProgressRow>, sqlx::Error> {
    let tier_column = match time_type {
        Some(TimeType::Pro) => "ct.pro_tier",
        _ => "ct.nub_tier",
    };
    let tt_filter = match time_type {
        Some(value) => format!(" AND r.time_type = '{}'", value.as_str()),
        None => String::new(),
    };
    let tt_filter_br = tt_filter.replace("r.", "br.");
    let status_filter = match status {
        "completed" => " AND b.id IS NOT NULL",
        "uncompleted" => " AND b.id IS NULL",
        _ => "",
    };
    let order = match sort {
        "name" => "mb.name ASC",
        "time" => "b.time_ns ASC NULLS LAST, mb.name ASC",
        "recent" => "b.created_at DESC NULLS LAST, mb.name ASC",
        _ => "mb.tier ASC NULLS LAST, mb.name ASC",
    };

    let sql = format!(
        r#"
        {base}
        SELECT mb.map_id,
               mb.name,
               mb.tier,
               (b.id IS NOT NULL) AS completed,
               b.time_ns,
               b.best_rank,
               b.time_type::text AS best_time_type,
               b.created_at AS last_record_at,
               mb.bonus_total,
               (SELECT count(*) FROM records br
                  JOIN map_courses bc ON bc.id = br.course_id AND bc.stage > 0
                 WHERE br.account_id = $1
                   AND br.map_id = mb.map_id
                   AND br.mode_id = $2
                   AND br.style_id = $3
                   AND br.status = 'approved'
                   AND br.deleted_at IS NULL{tt_filter_br})::bigint AS bonus_completed
          FROM map_base mb
          {best}
         WHERE ($4::text IS NULL OR mb.name ILIKE '%' || $4 || '%')
           AND ($5::smallint IS NULL OR ($5 = 0 AND mb.tier IS NULL) OR mb.tier = $5)
           {status_filter}
         ORDER BY {order}
         LIMIT $6 OFFSET $7
        "#,
        base = map_base_cte(tier_column, "$2"),
        best = best_lateral("b", "$1", "$2", "$3", &tt_filter),
        tt_filter_br = tt_filter_br,
        status_filter = status_filter,
        order = order,
    );

    sqlx::query_as::<_, MapProgressRow>(AssertSqlSafe(sql))
        .bind(account_id)
        .bind(mode_id)
        .bind(style_id)
        .bind(search)
        .bind(tier)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
}

#[allow(clippy::too_many_arguments)]
pub async fn map_progress_total(
    pool: &PgPool,
    account_id: i64,
    mode_id: i16,
    style_id: i16,
    time_type: Option<TimeType>,
    status: &str,
    search: Option<&str>,
    tier: Option<i16>,
) -> Result<i64, sqlx::Error> {
    let tier_column = match time_type {
        Some(TimeType::Pro) => "ct.pro_tier",
        _ => "ct.nub_tier",
    };
    let tt_filter = match time_type {
        Some(value) => format!(" AND r.time_type = '{}'", value.as_str()),
        None => String::new(),
    };
    let status_filter = match status {
        "completed" => " AND b.id IS NOT NULL",
        "uncompleted" => " AND b.id IS NULL",
        _ => "",
    };

    let sql = format!(
        r#"
        {base}
        SELECT count(*)
          FROM map_base mb
          {best}
         WHERE ($4::text IS NULL OR mb.name ILIKE '%' || $4 || '%')
           AND ($5::smallint IS NULL OR ($5 = 0 AND mb.tier IS NULL) OR mb.tier = $5)
           {status_filter}
        "#,
        base = map_base_cte(tier_column, "$2"),
        best = best_lateral("b", "$1", "$2", "$3", &tt_filter),
        status_filter = status_filter,
    );

    sqlx::query_scalar::<_, i64>(AssertSqlSafe(sql))
        .bind(account_id)
        .bind(mode_id)
        .bind(style_id)
        .bind(search)
        .bind(tier)
        .fetch_one(pool)
        .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CompareMapRow {
    pub map_id: i64,
    pub name: String,
    pub tier: Option<i16>,
    pub left_time_ns: Option<i64>,
    pub left_rank: Option<i64>,
    pub left_record_at: Option<DateTime<Utc>>,
    pub right_time_ns: Option<i64>,
    pub right_rank: Option<i64>,
    pub right_record_at: Option<DateTime<Utc>>,
}

#[allow(clippy::too_many_arguments)]
pub async fn compare_maps(
    pool: &PgPool,
    left_account: i64,
    right_account: i64,
    mode_id: i16,
    style_id: i16,
    time_type: Option<TimeType>,
    filter: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<CompareMapRow>, sqlx::Error> {
    let tier_column = match time_type {
        Some(TimeType::Pro) => "ct.pro_tier",
        _ => "ct.nub_tier",
    };
    let tt_filter = match time_type {
        Some(value) => format!(" AND r.time_type = '{}'", value.as_str()),
        None => String::new(),
    };
    let filter_sql = match filter {
        "both" => " AND l.id IS NOT NULL AND rr.id IS NOT NULL",
        "left_only" => " AND l.id IS NOT NULL AND rr.id IS NULL",
        "right_only" => " AND l.id IS NULL AND rr.id IS NOT NULL",
        "neither" => " AND l.id IS NULL AND rr.id IS NULL",
        "left_faster" => " AND l.time_ns < rr.time_ns",
        "right_faster" => " AND l.time_ns > rr.time_ns",
        _ => "",
    };

    let left_best = best_lateral("l", "$1", "$3", "$4", &tt_filter);
    let right_best = best_lateral("rr", "$2", "$3", "$4", &tt_filter);

    let sql = format!(
        r#"
        {base}
        SELECT mb.map_id,
               mb.name,
               mb.tier,
               l.time_ns AS left_time_ns,
               l.best_rank AS left_rank,
               l.created_at AS left_record_at,
               rr.time_ns AS right_time_ns,
               rr.best_rank AS right_rank,
               rr.created_at AS right_record_at
          FROM map_base mb
          {left_best}
          {right_best}
         WHERE true{filter_sql}
         ORDER BY mb.tier ASC NULLS LAST, mb.name ASC
         LIMIT $5 OFFSET $6
        "#,
        base = map_base_cte(tier_column, "$3"),
        filter_sql = filter_sql,
    );

    sqlx::query_as::<_, CompareMapRow>(AssertSqlSafe(sql))
        .bind(left_account)
        .bind(right_account)
        .bind(mode_id)
        .bind(style_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
}

pub async fn compare_maps_total(
    pool: &PgPool,
    left_account: i64,
    right_account: i64,
    mode_id: i16,
    style_id: i16,
    time_type: Option<TimeType>,
    filter: &str,
) -> Result<i64, sqlx::Error> {
    let tier_column = match time_type {
        Some(TimeType::Pro) => "ct.pro_tier",
        _ => "ct.nub_tier",
    };
    let tt_filter = match time_type {
        Some(value) => format!(" AND r.time_type = '{}'", value.as_str()),
        None => String::new(),
    };
    let filter_sql = match filter {
        "both" => " AND l.id IS NOT NULL AND rr.id IS NOT NULL",
        "left_only" => " AND l.id IS NOT NULL AND rr.id IS NULL",
        "right_only" => " AND l.id IS NULL AND rr.id IS NOT NULL",
        "neither" => " AND l.id IS NULL AND rr.id IS NULL",
        "left_faster" => " AND l.time_ns < rr.time_ns",
        "right_faster" => " AND l.time_ns > rr.time_ns",
        _ => "",
    };

    let sql = format!(
        r#"
        {base}
        SELECT count(*)
          FROM map_base mb
          {left_best}
          {right_best}
         WHERE true{filter_sql}
        "#,
        base = map_base_cte(tier_column, "$3"),
        left_best = best_lateral("l", "$1", "$3", "$4", &tt_filter),
        right_best = best_lateral("rr", "$2", "$3", "$4", &tt_filter),
        filter_sql = filter_sql,
    );

    sqlx::query_scalar::<_, i64>(AssertSqlSafe(sql))
        .bind(left_account)
        .bind(right_account)
        .bind(mode_id)
        .bind(style_id)
        .fetch_one(pool)
        .await
}
