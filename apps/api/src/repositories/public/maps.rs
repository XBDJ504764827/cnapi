//! Public map listing, detail and statistics queries.

use chrono::{DateTime, Utc};
use sqlx::{AssertSqlSafe, PgPool};

use crate::models::TimeType;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MapRow {
    pub id: i64,
    pub name: String,
    pub workshop_id: Option<i64>,
    pub description: Option<String>,
    pub tier: Option<i16>,
    pub courses: i64,
    pub completed_players: i64,
    pub records_count: i64,
    pub wr_time_ns: Option<i64>,
    pub wr_player_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
}

fn tt(alias: &str, time_type: Option<TimeType>) -> String {
    match time_type {
        Some(value) => format!(" AND {alias}.time_type = '{}'", value.as_str()),
        None => String::new(),
    }
}

fn map_select(tier_column: &str, time_type: Option<TimeType>) -> String {
    let r_tt = tt("r", time_type);
    let w_tt = tt("w", time_type);
    format!(
        r#"
        WITH map_stats AS (
            SELECT m.id,
                   m.name,
                   m.workshop_id,
                   m.description,
                   m.created_at,
                   m.approved_at,
                   {tier_column} AS tier,
                   (SELECT count(*) FROM map_courses c WHERE c.map_id = m.id)::bigint AS courses,
                   (SELECT count(DISTINCT r.account_id)
                      FROM records r
                      JOIN map_courses c2 ON c2.id = r.course_id AND c2.stage = 0
                     WHERE r.map_id = m.id
                       AND r.mode_id = $1
                       AND r.status = 'approved'
                       AND r.deleted_at IS NULL{r_tt}) AS completed_players,
                   (SELECT count(*)
                      FROM records r
                     WHERE r.map_id = m.id
                       AND r.mode_id = $1
                       AND r.status = 'approved'
                       AND r.deleted_at IS NULL{r_tt}) AS records_count,
                   (SELECT w.time_ns
                      FROM course_world_records w
                      JOIN map_courses cw ON cw.id = w.course_id AND cw.stage = 0
                     WHERE cw.map_id = m.id
                       AND w.mode_id = $1{w_tt}
                     ORDER BY w.time_ns ASC
                     LIMIT 1) AS wr_time_ns,
                   (SELECT p.display_name
                      FROM course_world_records w
                      JOIN map_courses cw ON cw.id = w.course_id AND cw.stage = 0
                      JOIN records wr ON wr.id = w.record_id
                      JOIN accounts a ON a.id = wr.account_id
                      LEFT JOIN player_profiles p ON p.account_id = a.id
                     WHERE cw.map_id = m.id
                       AND w.mode_id = $1{w_tt}
                     ORDER BY w.time_ns ASC
                     LIMIT 1) AS wr_player_name
              FROM maps m
              JOIN map_modes mm ON mm.map_id = m.id AND mm.mode_id = $1
              JOIN map_courses mc ON mc.map_id = m.id AND mc.stage = 0
              LEFT JOIN course_tiers ct ON ct.course_id = mc.id AND ct.mode_id = $1
             WHERE m.status = 'approved' AND m.deleted_at IS NULL
        )
        "#
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn list_maps(
    pool: &PgPool,
    mode_id: i16,
    time_type: Option<TimeType>,
    search: Option<&str>,
    tier: Option<i16>,
    sort: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<MapRow>, sqlx::Error> {
    let tier_column = match time_type {
        Some(TimeType::Pro) => "ct.pro_tier",
        _ => "ct.nub_tier",
    };
    let order = match sort {
        "recent" => "approved_at DESC NULLS LAST, created_at DESC",
        "name" => "name ASC",
        "records" => "records_count DESC, completed_players DESC, name ASC",
        _ => "completed_players DESC, records_count DESC, name ASC",
    };
    let sql = format!(
        r#"
        {select}
        SELECT * FROM map_stats
         WHERE ($2::text IS NULL OR name ILIKE '%' || $2 || '%')
           AND ($3::smallint IS NULL OR ($3 = 0 AND tier IS NULL) OR tier = $3)
         ORDER BY {order}
         LIMIT $4 OFFSET $5
        "#,
        select = map_select(tier_column, time_type),
    );
    sqlx::query_as::<_, MapRow>(AssertSqlSafe(sql))
        .bind(mode_id)
        .bind(search)
        .bind(tier)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
}

pub async fn count_maps(
    pool: &PgPool,
    mode_id: i16,
    time_type: Option<TimeType>,
    search: Option<&str>,
    tier: Option<i16>,
) -> Result<i64, sqlx::Error> {
    let tier_column = match time_type {
        Some(TimeType::Pro) => "ct.pro_tier",
        _ => "ct.nub_tier",
    };
    let sql = format!(
        r#"
        {select}
        SELECT count(*) FROM map_stats
         WHERE ($2::text IS NULL OR name ILIKE '%' || $2 || '%')
           AND ($3::smallint IS NULL OR ($3 = 0 AND tier IS NULL) OR tier = $3)
        "#,
        select = map_select(tier_column, time_type),
    );
    sqlx::query_scalar::<_, i64>(AssertSqlSafe(sql))
        .bind(mode_id)
        .bind(search)
        .bind(tier)
        .fetch_one(pool)
        .await
}

pub async fn find_map_row(
    pool: &PgPool,
    map_id: i64,
    mode_id: i16,
    time_type: Option<TimeType>,
) -> Result<Option<MapRow>, sqlx::Error> {
    let tier_column = match time_type {
        Some(TimeType::Pro) => "ct.pro_tier",
        _ => "ct.nub_tier",
    };
    let sql = format!(
        "{} SELECT * FROM map_stats WHERE id = $2",
        map_select(tier_column, time_type)
    );
    sqlx::query_as::<_, MapRow>(AssertSqlSafe(sql))
        .bind(mode_id)
        .bind(map_id)
        .fetch_optional(pool)
        .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CourseRow {
    pub id: i64,
    pub stage: i16,
    pub name: String,
    pub nub_tier: Option<i16>,
    pub pro_tier: Option<i16>,
    pub completed_players: i64,
    pub records_count: i64,
}

pub async fn map_courses(
    pool: &PgPool,
    map_id: i64,
    mode_id: i16,
    time_type: Option<TimeType>,
) -> Result<Vec<CourseRow>, sqlx::Error> {
    let tt_filter = match time_type {
        Some(value) => format!(" AND r.time_type = '{}'", value.as_str()),
        None => String::new(),
    };
    let sql = format!(
        r#"
        SELECT c.id,
               c.stage,
               c.name,
               ct.nub_tier,
               ct.pro_tier,
               (SELECT count(DISTINCT r.account_id)
                  FROM records r
                 WHERE r.course_id = c.id
                   AND r.mode_id = $2
                   AND r.status = 'approved'
                   AND r.deleted_at IS NULL{tt_filter})::bigint AS completed_players,
               (SELECT count(*)
                  FROM records r
                 WHERE r.course_id = c.id
                   AND r.mode_id = $2
                   AND r.status = 'approved'
                   AND r.deleted_at IS NULL{tt_filter})::bigint AS records_count
          FROM map_courses c
          LEFT JOIN course_tiers ct ON ct.course_id = c.id AND ct.mode_id = $2
         WHERE c.map_id = $1
         ORDER BY c.stage
        "#
    );
    sqlx::query_as::<_, CourseRow>(AssertSqlSafe(sql))
        .bind(map_id)
        .bind(mode_id)
        .fetch_all(pool)
        .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ModeCountRow {
    pub mode_id: i16,
    pub code: String,
    pub display_name: String,
    pub records: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TimeTypeCountRow {
    pub time_type: String,
    pub records: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MapStatsRow {
    pub total_players: i64,
    pub completed_players: i64,
    pub pro_completed_players: i64,
    pub nub_completed_players: i64,
    pub record_count: i64,
}

pub async fn map_statistics(
    pool: &PgPool,
    map_id: i64,
    mode_id: i16,
) -> Result<(MapStatsRow, Vec<ModeCountRow>, Vec<TimeTypeCountRow>), sqlx::Error> {
    let stats = sqlx::query_as::<_, MapStatsRow>(
        r#"
        SELECT
            (SELECT count(*) FROM accounts WHERE deleted_at IS NULL)::bigint AS total_players,
            (SELECT count(DISTINCT r.account_id)
               FROM records r
               JOIN map_courses c ON c.id = r.course_id AND c.stage = 0
              WHERE r.map_id = $1
                AND r.mode_id = $2
                AND r.status = 'approved'
                AND r.deleted_at IS NULL)::bigint AS completed_players,
            (SELECT count(DISTINCT r.account_id)
               FROM records r
               JOIN map_courses c ON c.id = r.course_id AND c.stage = 0
              WHERE r.map_id = $1
                AND r.mode_id = $2
                AND r.time_type = 'pro'
                AND r.status = 'approved'
                AND r.deleted_at IS NULL)::bigint AS pro_completed_players,
            (SELECT count(DISTINCT r.account_id)
               FROM records r
               JOIN map_courses c ON c.id = r.course_id AND c.stage = 0
              WHERE r.map_id = $1
                AND r.mode_id = $2
                AND r.time_type = 'nub'
                AND r.status = 'approved'
                AND r.deleted_at IS NULL)::bigint AS nub_completed_players,
            (SELECT count(*)
               FROM records r
              WHERE r.map_id = $1
                AND r.mode_id = $2
                AND r.status = 'approved'
                AND r.deleted_at IS NULL)::bigint AS record_count
        "#,
    )
    .bind(map_id)
    .bind(mode_id)
    .fetch_one(pool)
    .await?;

    let modes = sqlx::query_as::<_, ModeCountRow>(
        r#"
        SELECT mo.id AS mode_id,
               mo.code,
               mo.display_name,
               count(r.id)::bigint AS records
          FROM movement_modes mo
          LEFT JOIN records r ON r.mode_id = mo.id
                             AND r.map_id = $1
                             AND r.status = 'approved'
                             AND r.deleted_at IS NULL
         GROUP BY mo.id
         ORDER BY mo.id
        "#,
    )
    .bind(map_id)
    .fetch_all(pool)
    .await?;

    let time_types = sqlx::query_as::<_, TimeTypeCountRow>(
        r#"
        SELECT r.time_type::text AS time_type,
               count(*)::bigint AS records
          FROM records r
         WHERE r.map_id = $1
           AND r.mode_id = $2
           AND r.status = 'approved'
           AND r.deleted_at IS NULL
         GROUP BY r.time_type
         ORDER BY r.time_type
        "#,
    )
    .bind(map_id)
    .bind(mode_id)
    .fetch_all(pool)
    .await?;

    Ok((stats, modes, time_types))
}

pub async fn supported_modes(
    pool: &PgPool,
    map_id: i64,
) -> Result<Vec<super::directory::ModeRow>, sqlx::Error> {
    sqlx::query_as::<_, super::directory::ModeRow>(
        r#"
        SELECT mo.id, mo.code, mo.display_name, mo.description
          FROM map_modes mm
          JOIN movement_modes mo ON mo.id = mm.mode_id
         WHERE mm.map_id = $1
         ORDER BY mo.id
        "#,
    )
    .bind(map_id)
    .fetch_all(pool)
    .await
}
