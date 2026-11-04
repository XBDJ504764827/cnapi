//! Public server queries.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, QueryBuilder};

use crate::dto::public::common::map_slug;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ServerRow {
    pub id: i64,
    pub name: String,
    pub community: Option<String>,
    pub ip: String,
    pub port: i32,
    pub online: bool,
    pub current_map_id: Option<i64>,
    pub current_map_name: Option<String>,
    pub plugin_version: Option<String>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub online_players: i32,
    pub max_players: i32,
    pub current_mode: Option<String>,
    pub region: Option<String>,
    pub server_type: String,
    pub game: String,
}

impl ServerRow {
    pub fn map_slug(&self) -> Option<String> {
        self.current_map_name.as_deref().map(map_slug)
    }
}

fn push_filters(qb: &mut QueryBuilder<Postgres>, search: Option<&str>, online: Option<bool>) {
    qb.push(" WHERE s.deleted_at IS NULL");
    if let Some(search) = search {
        qb.push(" AND (s.name ILIKE ");
        qb.push_bind(format!("%{search}%"));
        qb.push(" OR s.ip::text LIKE ");
        qb.push_bind(format!("%{search}%"));
        qb.push(")");
    }
    match online {
        Some(true) => {
            qb.push(" AND s.is_online AND s.last_heartbeat_at IS NOT NULL AND s.last_heartbeat_at > now() - interval '2 minutes'");
        }
        Some(false) => {
            qb.push(" AND (NOT s.is_online OR s.last_heartbeat_at IS NULL OR s.last_heartbeat_at <= now() - interval '2 minutes')");
        }
        None => {}
    }
}

const SERVER_SELECT: &str = r#"
SELECT s.id,
       s.name,
       p.display_name AS community,
       host(s.ip) AS ip,
       s.port,
       (s.is_online
        AND s.last_heartbeat_at IS NOT NULL
        AND s.last_heartbeat_at > now() - interval '2 minutes') AS online,
       s.current_map_id,
       m.name AS current_map_name,
       s.plugin_version,
       s.last_heartbeat_at,
       s.online_players,
       s.max_players,
       mm.code AS current_mode,
       s.region,
       s.server_type,
       s.game
FROM servers s
LEFT JOIN accounts a ON a.id = s.owner_account_id
LEFT JOIN player_profiles p ON p.account_id = a.id
LEFT JOIN maps m ON m.id = s.current_map_id
LEFT JOIN movement_modes mm ON mm.id = s.current_mode_id
"#;

pub async fn list_servers(
    pool: &PgPool,
    search: Option<&str>,
    online: Option<bool>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ServerRow>, sqlx::Error> {
    let mut qb = QueryBuilder::<Postgres>::new(SERVER_SELECT);
    push_filters(&mut qb, search, online);
    qb.push(" ORDER BY online DESC, s.name ASC LIMIT ");
    qb.push_bind(limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);
    qb.build_query_as::<ServerRow>().fetch_all(pool).await
}

pub async fn count_servers(
    pool: &PgPool,
    search: Option<&str>,
    online: Option<bool>,
) -> Result<i64, sqlx::Error> {
    let mut qb = QueryBuilder::<Postgres>::new("SELECT count(*) FROM servers s");
    push_filters(&mut qb, search, online);
    qb.build_query_scalar::<i64>().fetch_one(pool).await
}

pub async fn find_server(pool: &PgPool, server_id: i64) -> Result<Option<ServerRow>, sqlx::Error> {
    let mut qb = QueryBuilder::<Postgres>::new(SERVER_SELECT);
    qb.push(" WHERE s.deleted_at IS NULL AND s.id = ");
    qb.push_bind(server_id);
    qb.build_query_as::<ServerRow>().fetch_optional(pool).await
}
