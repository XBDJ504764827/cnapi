//! Public record queries (recent, filtered, detail, world records, activity).

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::dto::public::common::{
    CourseRef, MapRef, ModeRef, PlayerRef, StyleRef, map_slug, seconds_string,
};
use crate::dto::public::records::RecordSummary;
use crate::models::TimeType;

#[derive(Debug, Clone, Default)]
pub struct RecordFilters {
    pub account_id: Option<i64>,
    pub player_query: Option<String>,
    pub map_query: Option<String>,
    pub map_id: Option<i64>,
    pub course_stage: Option<i16>,
    pub mode_id: Option<i16>,
    pub style_id: Option<i16>,
    pub time_type: Option<TimeType>,
    pub tier: Option<i16>,
    pub world_record: Option<bool>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RecordRow {
    pub id: i64,
    pub account_id: i64,
    pub steam_id64: String,
    pub display_name: Option<String>,
    pub map_id: i64,
    pub map_name: String,
    pub course_id: i64,
    pub stage: i16,
    pub course_name: String,
    pub mode_id: i16,
    pub mode_code: String,
    pub mode_name: String,
    pub style_id: i16,
    pub style_code: String,
    pub style_name: String,
    pub tier: Option<i16>,
    pub time_type: TimeType,
    pub teleports: i32,
    pub time_ns: i64,
    pub created_at: DateTime<Utc>,
    pub rank_position: i64,
    pub is_world_record: bool,
    pub replay_available: bool,
    pub replay_id: Option<i64>,
    pub server_id: Option<i64>,
    pub server_name: Option<String>,
    pub achieved_at: Option<DateTime<Utc>>,
}

impl RecordRow {
    pub fn into_summary(self) -> RecordSummary {
        RecordSummary {
            id: self.id,
            player: PlayerRef {
                account_id: self.account_id,
                steam_id64: self.steam_id64,
                display_name: self
                    .display_name
                    .unwrap_or_else(|| "Unknown player".to_owned()),
            },
            map: MapRef {
                id: self.map_id,
                slug: map_slug(&self.map_name),
                name: self.map_name,
            },
            course: CourseRef {
                id: self.course_id,
                stage: self.stage,
                name: self.course_name,
            },
            mode: ModeRef {
                id: self.mode_id,
                code: self.mode_code,
                display_name: self.mode_name,
            },
            style: StyleRef {
                id: self.style_id,
                code: self.style_code,
                display_name: self.style_name,
            },
            tier: self.tier,
            time_type: self.time_type,
            teleports: self.teleports,
            time_seconds: seconds_string(self.time_ns),
            time_ns: self.time_ns,
            rank: self.rank_position,
            is_world_record: self.is_world_record,
            replay_available: self.replay_available,
            points: None,
            created_at: self.created_at,
        }
    }
}

pub const RECORD_SELECT: &str = r#"
SELECT r.id,
       a.id AS account_id,
       a.steam_id64::text AS steam_id64,
       p.display_name,
       m.id AS map_id,
       m.name AS map_name,
       c.id AS course_id,
       c.stage,
       c.name AS course_name,
       mo.id AS mode_id,
       mo.code AS mode_code,
       mo.display_name AS mode_name,
       st.id AS style_id,
       st.code AS style_code,
       st.display_name AS style_name,
       (CASE WHEN r.time_type = 'pro' THEN ct.pro_tier ELSE ct.nub_tier END) AS tier,
       r.time_type,
       r.teleports,
       r.time_ns,
       r.created_at,
       (SELECT count(*) + 1
          FROM records f
         WHERE f.status = 'approved'
           AND f.deleted_at IS NULL
           AND f.course_id = r.course_id
           AND f.mode_id = r.mode_id
           AND f.style_id = r.style_id
           AND f.time_type = r.time_type
           AND f.time_ns < r.time_ns) AS rank_position,
       EXISTS (
           SELECT 1 FROM course_world_records w
            WHERE w.course_id = r.course_id
              AND w.mode_id = r.mode_id
              AND w.style_id = r.style_id
              AND w.time_type = r.time_type
              AND w.record_id = r.id
       ) AS is_world_record,
       (replay.id IS NOT NULL) AS replay_available,
       replay.id AS replay_id,
       sv.id AS server_id,
       sv.name AS server_name,
       NULL::timestamptz AS achieved_at
FROM records r
JOIN accounts a ON a.id = r.account_id AND a.deleted_at IS NULL
LEFT JOIN player_profiles p ON p.account_id = a.id
JOIN maps m ON m.id = r.map_id AND m.deleted_at IS NULL
JOIN map_courses c ON c.id = r.course_id
JOIN movement_modes mo ON mo.id = r.mode_id
JOIN movement_styles st ON st.id = r.style_id
LEFT JOIN course_tiers ct ON ct.course_id = r.course_id AND ct.mode_id = r.mode_id
LEFT JOIN servers sv ON sv.id = r.server_id
LEFT JOIN LATERAL (
    SELECT rp.id
      FROM replays rp
     WHERE rp.record_id = r.id
       AND rp.status = 'available'
       AND rp.deleted_at IS NULL
     ORDER BY rp.created_at DESC
     LIMIT 1
) replay ON TRUE
"#;

pub const RECORD_COUNT_FROM: &str = r#"
FROM records r
JOIN accounts a ON a.id = r.account_id AND a.deleted_at IS NULL
LEFT JOIN player_profiles p ON p.account_id = a.id
JOIN maps m ON m.id = r.map_id AND m.deleted_at IS NULL
JOIN map_courses c ON c.id = r.course_id
JOIN movement_modes mo ON mo.id = r.mode_id
JOIN movement_styles st ON st.id = r.style_id
LEFT JOIN course_tiers ct ON ct.course_id = r.course_id AND ct.mode_id = r.mode_id
"#;

/// Adds the common WHERE clauses. Assumes `RECORD_SELECT` / `RECORD_COUNT_FROM`
/// aliases are in scope. Binds are pushed in a fixed order.
pub fn push_record_filters(qb: &mut QueryBuilder<Postgres>, filters: &RecordFilters) {
    qb.push(" WHERE r.status = 'approved' AND r.deleted_at IS NULL");

    if let Some(account_id) = filters.account_id {
        qb.push(" AND r.account_id = ");
        qb.push_bind(account_id);
    }
    if let Some(player) = filters.player_query.as_deref() {
        let numeric = player.chars().all(|character| character.is_ascii_digit());
        if numeric {
            qb.push(" AND (a.steam_id64::text = ");
            qb.push_bind(player.to_owned());
            qb.push(" OR a.steam_id64::text LIKE ");
            qb.push_bind(format!("{player}%"));
            qb.push(")");
        } else {
            qb.push(" AND p.display_name ILIKE ");
            qb.push_bind(format!("%{player}%"));
        }
    }
    if let Some(map) = filters.map_query.as_deref() {
        qb.push(" AND m.name ILIKE ");
        qb.push_bind(format!("%{map}%"));
    }
    if let Some(map_id) = filters.map_id {
        qb.push(" AND r.map_id = ");
        qb.push_bind(map_id);
    }
    if let Some(stage) = filters.course_stage {
        qb.push(" AND c.stage = ");
        qb.push_bind(stage);
    }
    if let Some(mode_id) = filters.mode_id {
        qb.push(" AND r.mode_id = ");
        qb.push_bind(mode_id);
    }
    if let Some(style_id) = filters.style_id {
        qb.push(" AND r.style_id = ");
        qb.push_bind(style_id);
    }
    if let Some(time_type) = filters.time_type {
        qb.push(" AND r.time_type = ");
        qb.push_bind(time_type.as_str());
    }
    if let Some(tier) = filters.tier {
        if tier == 0 {
            qb.push(" AND (CASE WHEN r.time_type = 'pro' THEN ct.pro_tier ELSE ct.nub_tier END) IS NULL");
        } else {
            qb.push(
                " AND (CASE WHEN r.time_type = 'pro' THEN ct.pro_tier ELSE ct.nub_tier END) = ",
            );
            qb.push_bind(tier);
        }
    }
    match filters.world_record {
        Some(true) => {
            qb.push(
                " AND EXISTS (
                SELECT 1 FROM course_world_records w
                 WHERE w.course_id = r.course_id
                   AND w.mode_id = r.mode_id
                   AND w.style_id = r.style_id
                   AND w.time_type = r.time_type
                   AND w.record_id = r.id
              )",
            );
        }
        Some(false) => {
            qb.push(
                " AND NOT EXISTS (
                SELECT 1 FROM course_world_records w
                 WHERE w.course_id = r.course_id
                   AND w.mode_id = r.mode_id
                   AND w.style_id = r.style_id
                   AND w.time_type = r.time_type
                   AND w.record_id = r.id
              )",
            );
        }
        None => {}
    }
    if let Some(from) = filters.from {
        qb.push(" AND r.created_at >= ");
        qb.push_bind(from);
    }
    if let Some(to) = filters.to {
        qb.push(" AND r.created_at <= ");
        qb.push_bind(to);
    }
}

fn push_order(qb: &mut QueryBuilder<Postgres>, sort: &str) {
    match sort {
        "time" => qb.push(" ORDER BY r.time_ns ASC, r.id ASC"),
        "rank" => qb.push(" ORDER BY rank_position ASC, r.time_ns ASC"),
        _ => qb.push(" ORDER BY r.created_at DESC, r.id DESC"),
    };
}

pub async fn list_records(
    pool: &PgPool,
    filters: &RecordFilters,
    sort: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<RecordRow>, sqlx::Error> {
    let mut qb = QueryBuilder::<Postgres>::new(RECORD_SELECT);
    push_record_filters(&mut qb, filters);
    push_order(&mut qb, sort);
    qb.push(" LIMIT ");
    qb.push_bind(limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);
    qb.build_query_as::<RecordRow>().fetch_all(pool).await
}

pub async fn count_records(pool: &PgPool, filters: &RecordFilters) -> Result<i64, sqlx::Error> {
    let mut qb = QueryBuilder::<Postgres>::new("SELECT count(*)");
    qb.push(RECORD_COUNT_FROM);
    push_record_filters(&mut qb, filters);
    qb.build_query_scalar::<i64>().fetch_one(pool).await
}

pub async fn recent_records(
    pool: &PgPool,
    filters: &RecordFilters,
    limit: i64,
) -> Result<Vec<RecordRow>, sqlx::Error> {
    list_records(pool, filters, "uploaded", limit, 0).await
}

pub async fn find_record(pool: &PgPool, record_id: i64) -> Result<Option<RecordRow>, sqlx::Error> {
    let mut qb = QueryBuilder::<Postgres>::new(RECORD_SELECT);
    qb.push(" WHERE r.id = ");
    qb.push_bind(record_id);
    qb.push(" AND r.status = 'approved' AND r.deleted_at IS NULL");
    qb.build_query_as::<RecordRow>().fetch_optional(pool).await
}

pub async fn recent_world_records(
    pool: &PgPool,
    mode_id: Option<i16>,
    time_type: Option<TimeType>,
    limit: i64,
) -> Result<Vec<RecordRow>, sqlx::Error> {
    let mut qb = QueryBuilder::<Postgres>::new(
        r#"
        SELECT r.id,
               a.id AS account_id,
               a.steam_id64::text AS steam_id64,
               p.display_name,
               m.id AS map_id,
               m.name AS map_name,
               c.id AS course_id,
               c.stage,
               c.name AS course_name,
               mo.id AS mode_id,
               mo.code AS mode_code,
               mo.display_name AS mode_name,
               st.id AS style_id,
               st.code AS style_code,
               st.display_name AS style_name,
               (CASE WHEN r.time_type = 'pro' THEN ct.pro_tier ELSE ct.nub_tier END) AS tier,
               r.time_type,
               r.teleports,
               r.time_ns,
               r.created_at,
               (SELECT count(*) + 1
                  FROM records f
                 WHERE f.status = 'approved'
                   AND f.deleted_at IS NULL
                   AND f.course_id = r.course_id
                   AND f.mode_id = r.mode_id
                   AND f.style_id = r.style_id
                   AND f.time_type = r.time_type
                   AND f.time_ns < r.time_ns) AS rank_position,
               true AS is_world_record,
               (replay.id IS NOT NULL) AS replay_available,
               replay.id AS replay_id,
               sv.id AS server_id,
               sv.name AS server_name,
               w.achieved_at AS achieved_at
        FROM course_world_records w
        JOIN records r ON r.id = w.record_id
        JOIN accounts a ON a.id = r.account_id AND a.deleted_at IS NULL
        LEFT JOIN player_profiles p ON p.account_id = a.id
        JOIN maps m ON m.id = r.map_id AND m.deleted_at IS NULL
        JOIN map_courses c ON c.id = r.course_id
        JOIN movement_modes mo ON mo.id = r.mode_id
        JOIN movement_styles st ON st.id = r.style_id
        LEFT JOIN course_tiers ct ON ct.course_id = r.course_id AND ct.mode_id = r.mode_id
        LEFT JOIN servers sv ON sv.id = r.server_id
        LEFT JOIN LATERAL (
            SELECT rp.id
              FROM replays rp
             WHERE rp.record_id = r.id
               AND rp.status = 'available'
               AND rp.deleted_at IS NULL
             ORDER BY rp.created_at DESC
             LIMIT 1
        ) replay ON TRUE
        WHERE r.status = 'approved' AND r.deleted_at IS NULL
        "#,
    );
    if let Some(mode_id) = mode_id {
        qb.push(" AND w.mode_id = ");
        qb.push_bind(mode_id);
    }
    if let Some(time_type) = time_type {
        qb.push(" AND w.time_type = ");
        qb.push_bind(time_type.as_str());
    }
    qb.push(" ORDER BY w.achieved_at DESC, w.updated_at DESC LIMIT ");
    qb.push_bind(limit);
    qb.build_query_as::<RecordRow>().fetch_all(pool).await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ActivePlayerRow {
    pub account_id: i64,
    pub steam_id64: String,
    pub display_name: Option<String>,
    pub records: i64,
    pub last_record_at: Option<DateTime<Utc>>,
}

pub async fn active_players(
    pool: &PgPool,
    days: u32,
    limit: i64,
) -> Result<Vec<ActivePlayerRow>, sqlx::Error> {
    sqlx::query_as::<_, ActivePlayerRow>(
        r#"
        SELECT a.id AS account_id,
               a.steam_id64::text AS steam_id64,
               p.display_name,
               count(*)::bigint AS records,
               max(r.created_at) AS last_record_at
        FROM records r
        JOIN accounts a ON a.id = r.account_id AND a.deleted_at IS NULL
        LEFT JOIN player_profiles p ON p.account_id = a.id
        WHERE r.status = 'approved'
          AND r.deleted_at IS NULL
          AND r.created_at >= now() - make_interval(days => $1)
        GROUP BY a.id, a.steam_id64, p.display_name
        ORDER BY records DESC, last_record_at DESC
        LIMIT $2
        "#,
    )
    .bind(days as i32)
    .bind(limit)
    .fetch_all(pool)
    .await
}
