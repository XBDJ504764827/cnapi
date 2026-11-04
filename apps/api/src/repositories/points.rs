//! SQL queries for the points system. All multi-row work is set-based.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::dto::points::{
    AdminTierDto, FilterLeaderboardEntry, FilterLeaderboardQuery, PlayerFilterPointsDto,
    PlayerFilterPointsQuery, PlayerPointsResponse, PointsLeaderboardEntry, PointsSystemStatus,
    RecalcRunDto, ScoringModeDto,
};
use crate::dto::public::common::{ModeRef, map_slug};
use crate::services::scoring::model::ScoringMode;

#[derive(Debug, sqlx::FromRow)]
struct ModeRow {
    code: String,
    mode_id: i16,
    display_name: String,
}

#[derive(Debug, sqlx::FromRow)]
struct PlayerModeRow {
    pro_points: i32,
    nub_points: i32,
    total_points: i32,
    filter_count: i32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ScoringModeRow {
    pub mode_id: i16,
    pub code: String,
    pub display_name: String,
    pub mode_code: String,
    pub mode_name: String,
    pub t9_enabled: bool,
}

impl From<ScoringModeRow> for ScoringModeDto {
    fn from(row: ScoringModeRow) -> Self {
        Self {
            code: row.code,
            display_name: row.display_name,
            mode: ModeRef {
                id: row.mode_id,
                code: row.mode_code,
                display_name: row.mode_name,
            },
            t9_enabled: row.t9_enabled,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct PointsLeaderboardDbRow {
    rank: i64,
    account_id: i64,
    steam_id64: String,
    display_name: String,
    pro_points: i64,
    nub_points: i64,
    total_points: i64,
    filter_count: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct FilterLeaderboardDbRow {
    rank: i32,
    account_id: i64,
    steam_id64: String,
    display_name: String,
    pb_record_id: i64,
    pb_time_seconds: String,
    points: i32,
    is_world_record: bool,
}

pub struct FilterMetaRow {
    pub id: i64,
    pub tier: i16,
    pub pb_count: i64,
    pub mode_id: i16,
    pub code: String,
    pub display_name: String,
    pub mode_code: String,
    pub mode_name: String,
    pub t9_enabled: bool,
    pub record_type: String,
}

pub async fn scoring_mode_id(pool: &PgPool, code: &str) -> Result<Option<i16>, sqlx::Error> {
    sqlx::query_scalar::<_, i16>("SELECT mode_id FROM scoring_modes WHERE code = $1")
        .bind(code)
        .fetch_optional(pool)
        .await
}

pub async fn scoring_mode(
    pool: &PgPool,
    code: &str,
) -> Result<Option<ScoringModeRow>, sqlx::Error> {
    sqlx::query_as::<_, ScoringModeRow>(
        r#"
        SELECT sm.mode_id, sm.code, sm.display_name,
               mm.code AS mode_code, mm.display_name AS mode_name, sm.t9_enabled
          FROM scoring_modes sm JOIN movement_modes mm ON mm.id = sm.mode_id
         WHERE sm.code = $1
        "#,
    )
    .bind(code)
    .fetch_optional(pool)
    .await
}

pub async fn player_points(
    pool: &PgPool,
    account_id: i64,
) -> Result<PlayerPointsResponse, sqlx::Error> {
    let modes: Vec<ModeRow> = sqlx::query_as::<_, ModeRow>(
        "SELECT code, mode_id, display_name FROM scoring_modes ORDER BY sort_order",
    )
    .fetch_all(pool)
    .await?;

    let steam_id64 =
        sqlx::query_scalar::<_, String>("SELECT steam_id64::text FROM accounts WHERE id = $1")
            .bind(account_id)
            .fetch_one(pool)
            .await?;

    let mut result = PlayerPointsResponse {
        account_id,
        steam_id64,
        modes: Vec::with_capacity(modes.len()),
    };

    for mode in modes {
        let row: Option<PlayerModeRow> = sqlx::query_as::<_, PlayerModeRow>(
            "SELECT pro_points, nub_points, total_points, filter_count
               FROM player_mode_points WHERE mode_id = $1 AND account_id = $2",
        )
        .bind(mode.mode_id)
        .bind(account_id)
        .fetch_optional(pool)
        .await?;
        let row = row.unwrap_or(PlayerModeRow {
            pro_points: 0,
            nub_points: 0,
            total_points: 0,
            filter_count: 0,
        });
        result.modes.push(crate::dto::points::ModePointsDto {
            code: mode.code,
            display_name: mode.display_name,
            pro_points: row.pro_points as i64,
            nub_points: row.nub_points as i64,
            total_points: row.total_points as i64,
            filter_count: row.filter_count as i64,
        });
    }
    Ok(result)
}

pub async fn player_filter_points(
    pool: &PgPool,
    account_id: i64,
    query: &PlayerFilterPointsQuery,
    mode_id: Option<i16>,
    limit: i64,
    offset: i64,
) -> Result<Vec<PlayerFilterPointsDto>, sqlx::Error> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT pfp.filter_id, pfp.map_id, m.name AS map_name,
               pfp.course_id, c.name AS course_name, c.stage,
               mo.id AS mode_id, mo.display_name AS mode_name,
               sm.code AS scoring_mode_code,
               pfp.record_type, pfp.tier,
               (fc.record_id = pfp.pb_record_id) AS t9_label,
               pfp.pb_record_id,
               to_char(pfp.time_ns::numeric / 1000000000, 'FM9999999990.000000000') AS pb_time_seconds,
               to_char(pfp.wr_time_ns::numeric / 1000000000, 'FM9999999990.000000000') AS wr_time_seconds,
               pfp.rank, pfp.points, pfp.is_world_record, pfp.calculated_at
          FROM player_filter_points pfp
          JOIN maps m ON m.id = pfp.map_id
          JOIN map_courses c ON c.id = pfp.course_id
          JOIN movement_modes mo ON mo.id = pfp.mode_id
          JOIN scoring_modes sm ON sm.mode_id = pfp.mode_id
          LEFT JOIN filter_first_completions fc
            ON fc.map_id = pfp.map_id AND fc.course_id = pfp.course_id AND fc.mode_id = pfp.mode_id
         WHERE pfp.account_id = "#,
    );
    qb.push_bind(account_id);
    push_filter_conditions(&mut qb, query, mode_id);
    qb.push(" ORDER BY ");
    qb.push(match query.sort.as_deref().unwrap_or("points") {
        "rank" => "pfp.rank ASC, pfp.points DESC",
        "time" => "pfp.time_ns ASC",
        "tier" => "pfp.tier DESC, pfp.points DESC",
        _ => "pfp.points DESC, pfp.rank ASC",
    });
    qb.push(" LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);
    qb.build_query_as::<PlayerFilterPointsDbRow>()
        .fetch_all(pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
}

pub async fn player_filter_points_total(
    pool: &PgPool,
    account_id: i64,
    query: &PlayerFilterPointsQuery,
    mode_id: Option<i16>,
) -> Result<i64, sqlx::Error> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT count(*) FROM player_filter_points pfp JOIN maps m ON m.id = pfp.map_id WHERE pfp.account_id = ",
    );
    qb.push_bind(account_id);
    push_filter_conditions(&mut qb, query, mode_id);
    qb.build_query_scalar::<i64>().fetch_one(pool).await
}

fn push_filter_conditions(
    qb: &mut QueryBuilder<Postgres>,
    query: &PlayerFilterPointsQuery,
    mode_id: Option<i16>,
) {
    if let Some(mode_id) = mode_id {
        qb.push(" AND pfp.mode_id = ").push_bind(mode_id);
    } else if query.mode.is_some() {
        qb.push(" AND FALSE"); // invalid mode fails closed
    }
    if let Some(record_type) = query.record_type.as_deref() {
        qb.push(" AND pfp.record_type = ")
            .push_bind(record_type.to_ascii_lowercase());
    }
    if let Some(tier) = query.tier {
        qb.push(" AND pfp.tier = ").push_bind(tier);
    }
    if let Some(course_id) = query.course_id {
        qb.push(" AND pfp.course_id = ").push_bind(course_id);
    }
    if let Some(map) = query
        .map
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        qb.push(" AND m.name ILIKE ")
            .push_bind(format!("%{}%", map.trim()));
    }
}

#[derive(Debug, sqlx::FromRow)]
struct PlayerFilterPointsDbRow {
    filter_id: i64,
    map_id: i64,
    map_name: String,
    course_id: i64,
    course_name: String,
    stage: i16,
    mode_id: i16,
    mode_name: String,
    scoring_mode_code: String,
    record_type: String,
    tier: i16,
    t9_label: Option<bool>,
    pb_record_id: i64,
    pb_time_seconds: String,
    wr_time_seconds: String,
    rank: i32,
    points: i32,
    is_world_record: bool,
    calculated_at: DateTime<Utc>,
}

impl From<PlayerFilterPointsDbRow> for PlayerFilterPointsDto {
    fn from(row: PlayerFilterPointsDbRow) -> Self {
        Self {
            filter_id: row.filter_id,
            map_id: row.map_id,
            map_slug: map_slug(&row.map_name),
            map_name: row.map_name,
            course_id: row.course_id,
            course_name: row.course_name,
            stage: row.stage,
            mode: ModeRef {
                id: row.mode_id,
                code: row.scoring_mode_code,
                display_name: row.mode_name,
            },
            record_type: row.record_type,
            tier: row.tier,
            t9_label: row.t9_label.unwrap_or(false),
            pb_record_id: row.pb_record_id,
            pb_time_seconds: row.pb_time_seconds,
            wr_time_seconds: row.wr_time_seconds,
            rank: row.rank,
            points: row.points,
            is_world_record: row.is_world_record,
            calculated_at: row.calculated_at,
        }
    }
}

pub async fn mode_leaderboard(
    pool: &PgPool,
    mode_id: i16,
    limit: i64,
    offset: i64,
) -> Result<Vec<PointsLeaderboardEntry>, sqlx::Error> {
    let rows = sqlx::query_as::<_, PointsLeaderboardDbRow>(
        r#"
        SELECT row_number() OVER (ORDER BY pmp.total_points DESC, pmp.account_id)::bigint AS rank,
               a.id AS account_id, a.steam_id64::text AS steam_id64,
               COALESCE(p.display_name, 'Unknown player') AS display_name,
               pmp.pro_points::bigint AS pro_points,
               pmp.nub_points::bigint AS nub_points,
               pmp.total_points::bigint AS total_points,
               pmp.filter_count::bigint AS filter_count
          FROM player_mode_points pmp
          JOIN accounts a ON a.id = pmp.account_id AND a.deleted_at IS NULL
          LEFT JOIN player_profiles p ON p.account_id = a.id
         WHERE pmp.mode_id = $1
         ORDER BY pmp.total_points DESC, pmp.account_id
         LIMIT $2 OFFSET $3
        "#,
    )
    .bind(mode_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| PointsLeaderboardEntry {
            rank: row.rank,
            account_id: row.account_id,
            steam_id64: row.steam_id64,
            display_name: row.display_name,
            pro_points: row.pro_points,
            nub_points: row.nub_points,
            total_points: row.total_points,
            filter_count: row.filter_count,
        })
        .collect())
}

pub async fn mode_leaderboard_total(pool: &PgPool, mode_id: i16) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM player_mode_points pmp
          JOIN accounts a ON a.id = pmp.account_id AND a.deleted_at IS NULL
          WHERE pmp.mode_id = $1",
    )
    .bind(mode_id)
    .fetch_one(pool)
    .await
}

pub async fn filter_leaderboard(
    pool: &PgPool,
    query: &FilterLeaderboardQuery,
    mode_id: i16,
    limit: i64,
    offset: i64,
) -> Result<Vec<FilterLeaderboardEntry>, sqlx::Error> {
    let rows = sqlx::query_as::<_, FilterLeaderboardDbRow>(
        r#"
        SELECT pfp.rank, a.id AS account_id, a.steam_id64::text AS steam_id64,
               COALESCE(p.display_name, 'Unknown player') AS display_name,
               pfp.pb_record_id,
               to_char(pfp.time_ns::numeric / 1000000000, 'FM9999999990.000000000') AS pb_time_seconds,
               pfp.points, pfp.is_world_record
          FROM player_filter_points pfp
          JOIN scoring_filters f ON f.id = pfp.filter_id
          JOIN accounts a ON a.id = pfp.account_id AND a.deleted_at IS NULL
          LEFT JOIN player_profiles p ON p.account_id = a.id
         WHERE f.map_id = $1 AND f.course_id = $2 AND f.mode_id = $3 AND f.record_type = $4
         ORDER BY pfp.rank, pfp.account_id
         LIMIT $5 OFFSET $6
        "#,
    )
    .bind(query.map_id)
    .bind(query.course_id)
    .bind(mode_id)
    .bind(query.record_type.to_ascii_lowercase())
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| FilterLeaderboardEntry {
            rank: row.rank,
            account_id: row.account_id,
            steam_id64: row.steam_id64,
            display_name: row.display_name,
            pb_record_id: row.pb_record_id,
            pb_time_seconds: row.pb_time_seconds,
            points: row.points,
            is_world_record: row.is_world_record,
        })
        .collect())
}

#[derive(Debug, sqlx::FromRow)]
struct FilterMetaDbRow {
    id: i64,
    tier: i16,
    pb_count: i64,
    mode_id: i16,
    code: String,
    display_name: String,
    mode_code: String,
    mode_name: String,
    t9_enabled: bool,
    record_type: String,
}

pub async fn filter_meta(
    pool: &PgPool,
    query: &FilterLeaderboardQuery,
    mode_id: i16,
) -> Result<Option<FilterMetaRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, FilterMetaDbRow>(
        r#"
        SELECT f.id, f.tier,
               (SELECT count(*)::bigint FROM player_filter_points pfp WHERE pfp.filter_id = f.id) AS pb_count,
               sm.mode_id, sm.code, sm.display_name,
               mm.code AS mode_code, mm.display_name AS mode_name,
               sm.t9_enabled, f.record_type
          FROM scoring_filters f
          JOIN scoring_modes sm ON sm.mode_id = f.mode_id
          JOIN movement_modes mm ON mm.id = f.mode_id
         WHERE f.map_id = $1 AND f.course_id = $2 AND f.mode_id = $3 AND f.record_type = $4
        "#,
    )
    .bind(query.map_id)
    .bind(query.course_id)
    .bind(mode_id)
    .bind(query.record_type.to_ascii_lowercase())
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| FilterMetaRow {
        id: row.id,
        tier: row.tier,
        pb_count: row.pb_count,
        mode_id: row.mode_id,
        code: row.code,
        display_name: row.display_name,
        mode_code: row.mode_code,
        mode_name: row.mode_name,
        t9_enabled: row.t9_enabled,
        record_type: row.record_type,
    }))
}

pub async fn system_status(pool: &PgPool) -> Result<PointsSystemStatus, sqlx::Error> {
    sqlx::query_as::<_, PointsSystemStatus>(
        r#"
        SELECT count(*) FILTER (WHERE status='pending')::bigint AS pending,
               count(*) FILTER (WHERE status='running')::bigint AS running,
               count(*) FILTER (WHERE status='failed')::bigint AS failed,
               (SELECT count(*)::bigint FROM scoring_filters) AS filters,
               (SELECT count(*)::bigint FROM scoring_filters WHERE dirty) AS dirty_filters,
               (SELECT max(last_recalculated_at) FROM scoring_filters) AS last_recalculated_at
          FROM filter_recalc_tasks
        "#,
    )
    .fetch_one(pool)
    .await
}

pub async fn recalc_runs(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<RecalcRunDto>, i64), sqlx::Error> {
    let items = sqlx::query_as::<_, RecalcRunDto>(
        r#"
        SELECT id, filter_id, mode_id, status, pb_count, duration_ms, error, created_at
          FROM filter_recalc_runs ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    let total = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM filter_recalc_runs")
        .fetch_one(pool)
        .await?;
    Ok((items, total))
}

pub async fn tiers_for_admin(
    pool: &PgPool,
    mode_id: Option<i16>,
    limit: i64,
    offset: i64,
) -> Result<Vec<AdminTierDto>, sqlx::Error> {
    sqlx::query_as::<_, AdminTierDto>(
        r#"
        SELECT cmt.map_id, m.name AS map_name, cmt.course_id, c.stage, c.name AS course_name,
               cmt.mode_id, sm.code AS mode_code, cmt.tier, cmt.completed,
               cmt.first_completion_record_id, fc.account_id AS first_completion_account_id,
               COALESCE(p.display_name, '') AS first_completion_player,
               cmt.first_completion_at, cmt.updated_at
          FROM course_mode_tiers cmt
          JOIN maps m ON m.id = cmt.map_id
          JOIN map_courses c ON c.id = cmt.course_id
          JOIN scoring_modes sm ON sm.mode_id = cmt.mode_id
          LEFT JOIN filter_first_completions fc
            ON fc.map_id = cmt.map_id AND fc.course_id = cmt.course_id AND fc.mode_id = cmt.mode_id
          LEFT JOIN accounts a ON a.id = fc.account_id
          LEFT JOIN player_profiles p ON p.account_id = a.id
         WHERE ($1::smallint IS NULL OR cmt.mode_id = $1)
         ORDER BY sm.sort_order, m.name, c.stage
         LIMIT $2 OFFSET $3
        "#,
    )
    .bind(mode_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn tiers_total(pool: &PgPool, mode_id: Option<i16>) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM course_mode_tiers WHERE ($1::smallint IS NULL OR mode_id = $1)",
    )
    .bind(mode_id)
    .fetch_one(pool)
    .await
}

pub async fn update_tier(
    pool: &PgPool,
    map_id: i64,
    course_id: i64,
    mode_id: i16,
    tier: i16,
) -> Result<bool, sqlx::Error> {
    let updated = sqlx::query(
        "UPDATE course_mode_tiers SET tier = $4, updated_at = now()
          WHERE map_id = $1 AND course_id = $2 AND mode_id = $3",
    )
    .bind(map_id)
    .bind(course_id)
    .bind(mode_id)
    .bind(tier)
    .execute(pool)
    .await?;
    Ok(updated.rows_affected() > 0)
}

pub fn mode_code(code: &str) -> Option<&'static str> {
    ScoringMode::parse(code).map(ScoringMode::mode_code)
}
