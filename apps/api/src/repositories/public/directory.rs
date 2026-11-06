//! Lookup helpers shared by the public repositories.

use sqlx::PgPool;

use crate::dto::public::common::{MapRef, ModeRef, PlayerRef, StyleRef, map_slug};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ModeRow {
    pub id: i16,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
}

impl ModeRow {
    pub fn to_ref(&self) -> ModeRef {
        ModeRef {
            id: self.id,
            code: self.code.clone(),
            display_name: self.display_name.clone(),
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StyleRow {
    pub id: i16,
    pub code: String,
    pub display_name: String,
}

impl StyleRow {
    pub fn to_ref(&self) -> StyleRef {
        StyleRef {
            id: self.id,
            code: self.code.clone(),
            display_name: self.display_name.clone(),
        }
    }
}

pub async fn list_modes(pool: &PgPool) -> Result<Vec<ModeRow>, sqlx::Error> {
    sqlx::query_as::<_, ModeRow>(
        "SELECT id, code, display_name, description FROM movement_modes ORDER BY id",
    )
    .fetch_all(pool)
    .await
}

pub async fn find_mode(pool: &PgPool, code: &str) -> Result<Option<ModeRow>, sqlx::Error> {
    sqlx::query_as::<_, ModeRow>(
        "SELECT id, code, display_name, description FROM movement_modes WHERE code = $1",
    )
    .bind(code)
    .fetch_optional(pool)
    .await
}

pub async fn default_mode(pool: &PgPool) -> Result<Option<ModeRow>, sqlx::Error> {
    sqlx::query_as::<_, ModeRow>(
        "SELECT id, code, display_name, description FROM movement_modes ORDER BY id LIMIT 1",
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_style(pool: &PgPool, code: &str) -> Result<Option<StyleRow>, sqlx::Error> {
    sqlx::query_as::<_, StyleRow>(
        "SELECT id, code, display_name FROM movement_styles WHERE code = $1",
    )
    .bind(code)
    .fetch_optional(pool)
    .await
}

pub async fn default_style(pool: &PgPool) -> Result<Option<StyleRow>, sqlx::Error> {
    sqlx::query_as::<_, StyleRow>(
        "SELECT id, code, display_name FROM movement_styles ORDER BY id LIMIT 1",
    )
    .fetch_optional(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PlayerLookupRow {
    pub account_id: i64,
    pub steam_id64: String,
    pub display_name: Option<String>,
    pub country: Option<String>,
}

impl PlayerLookupRow {
    pub fn display_name(&self) -> String {
        self.display_name
            .clone()
            .unwrap_or_else(|| format!("Player {}", self.steam_id64))
    }

    pub fn to_ref(&self) -> PlayerRef {
        PlayerRef {
            account_id: self.account_id,
            steam_id64: self.steam_id64.clone(),
            display_name: self.display_name(),
        }
    }
}

pub async fn find_player(
    pool: &PgPool,
    steam_id64: &str,
) -> Result<Option<PlayerLookupRow>, sqlx::Error> {
    sqlx::query_as::<_, PlayerLookupRow>(
        r#"
        SELECT a.id AS account_id,
               a.steam_id64::text AS steam_id64,
               p.display_name,
               p.country
        FROM accounts a
        LEFT JOIN player_profiles p ON p.account_id = a.id
        WHERE a.deleted_at IS NULL
          AND (a.steam_id64::text = $1 OR lower(p.display_name) = lower($2))
        ORDER BY (a.steam_id64::text = $1) DESC
        LIMIT 1
        "#,
    )
    .bind(steam_id64)
    .bind(steam_id64)
    .fetch_optional(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MapLookupRow {
    pub id: i64,
    pub name: String,
    pub workshop_id: Option<i64>,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl MapLookupRow {
    pub fn slug(&self) -> String {
        map_slug(&self.name)
    }

    pub fn to_ref(&self) -> MapRef {
        MapRef {
            id: self.id,
            name: self.name.clone(),
            slug: self.slug(),
        }
    }
}

pub async fn find_map_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<MapLookupRow>, sqlx::Error> {
    sqlx::query_as::<_, MapLookupRow>(
        r#"
        SELECT id, name, workshop_id, description, created_at, approved_at
        FROM maps
        WHERE deleted_at IS NULL
          AND status = 'approved'
          AND (lower(name) = lower($1) OR lower(replace(name, ' ', '-')) = lower($1))
        LIMIT 1
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CourseRow {
    pub id: i64,
    pub stage: i16,
    pub name: String,
    pub map_id: i64,
}

pub async fn find_course(
    pool: &PgPool,
    map_id: i64,
    stage: i16,
) -> Result<Option<CourseRow>, sqlx::Error> {
    sqlx::query_as::<_, CourseRow>(
        "SELECT id, stage, name, map_id FROM map_courses WHERE map_id = $1 AND stage = $2",
    )
    .bind(map_id)
    .bind(stage)
    .fetch_optional(pool)
    .await
}

pub async fn search_players(
    pool: &PgPool,
    query: &str,
    limit: i64,
) -> Result<Vec<PlayerLookupRow>, sqlx::Error> {
    sqlx::query_as::<_, PlayerLookupRow>(
        r#"
        SELECT a.id AS account_id,
               a.steam_id64::text AS steam_id64,
               p.display_name,
               p.country
        FROM accounts a
        LEFT JOIN player_profiles p ON p.account_id = a.id
        WHERE a.deleted_at IS NULL
          AND (
            a.steam_id64::text LIKE $1
            OR p.display_name ILIKE $2
          )
        ORDER BY (a.steam_id64::text = $3) DESC, p.display_name NULLS LAST
        LIMIT $4
        "#,
    )
    .bind(format!("{query}%"))
    .bind(format!("%{query}%"))
    .bind(query)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn search_maps(
    pool: &PgPool,
    query: &str,
    limit: i64,
) -> Result<Vec<MapLookupRow>, sqlx::Error> {
    sqlx::query_as::<_, MapLookupRow>(
        r#"
        SELECT id, name, workshop_id, description, created_at, approved_at
        FROM maps
        WHERE deleted_at IS NULL
          AND status = 'approved'
          AND name ILIKE $1
        ORDER BY (lower(name) = lower($2)) DESC, name
        LIMIT $3
        "#,
    )
    .bind(format!("%{query}%"))
    .bind(query)
    .bind(limit)
    .fetch_all(pool)
    .await
}
