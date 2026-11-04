//! SQL for the phase-7 CS:GO game-server ingestion API.
//!
//! No token is ever written to a log here. Token lookups use the database's
//! `sha256(convert_to(token, 'UTF8'))`, matching the phase-2 `servers` design.

use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};

use crate::dto::game_server::{
    AdminHeartbeatRow, AdminPlayerRow, AdminRecordRow, AdminReplayRow, AdminServerDetailResponse,
    AdminServerSummary, AdminSyncLogRow, AdminSyncTaskRow, ConfigCourse, ConfigMap, ConfigMode,
    ConfigStyle, ConfigTier,
};
use crate::dto::public::common::seconds_string_3;

const HEARTBEAT_FRESH_SECONDS: i64 = 120;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuthServer {
    pub id: i64,
    pub name: String,
    pub is_enabled: bool,
    pub api_key_revoked_at: Option<DateTime<Utc>>,
    pub plugin_version: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ServerAdminRow {
    pub id: i64,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub query_port: Option<i32>,
    pub game: String,
    pub region: Option<String>,
    pub server_type: String,
    pub max_players: i32,
    pub online_players: i32,
    pub current_map_id: Option<i64>,
    pub current_map_name: Option<String>,
    pub current_mode: Option<String>,
    pub is_online: bool,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub is_enabled: bool,
    pub plugin_version: Option<String>,
    pub api_key_prefix: String,
    pub api_key_created_at: DateTime<Utc>,
    pub api_key_revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl ServerAdminRow {
    pub fn status(&self) -> &'static str {
        match self.last_heartbeat_at {
            None => "unknown",
            Some(at) => {
                let fresh = (Utc::now() - at).num_seconds() <= HEARTBEAT_FRESH_SECONDS;
                if fresh && self.is_online {
                    "online"
                } else if fresh {
                    "offline"
                } else {
                    "expired"
                }
            }
        }
    }

    pub fn into_dto(self) -> AdminServerSummary {
        let status = self.status().to_owned();
        let heartbeat_age_seconds = self
            .last_heartbeat_at
            .map(|at| (Utc::now() - at).num_seconds().max(0));
        AdminServerSummary {
            id: self.id,
            name: self.name,
            ip: self.ip,
            port: self.port,
            query_port: self.query_port,
            game: self.game,
            region: self.region,
            server_type: self.server_type,
            max_players: self.max_players,
            online_players: self.online_players,
            current_map_id: self.current_map_id,
            current_map_name: self.current_map_name,
            current_mode: self.current_mode,
            status,
            is_enabled: self.is_enabled,
            is_online: self.is_online,
            plugin_version: self.plugin_version,
            last_heartbeat_at: self.last_heartbeat_at,
            heartbeat_age_seconds,
            api_key_prefix: self.api_key_prefix,
            api_key_created_at: self.api_key_created_at,
            api_key_revoked_at: self.api_key_revoked_at,
            created_at: self.created_at,
        }
    }
}

const ADMIN_SERVER_SELECT: &str = r#"
SELECT s.id, s.name, host(s.ip) AS ip, s.port, s.query_port, s.game, s.region,
       s.server_type, s.max_players, s.online_players,
       s.current_map_id, m.name AS current_map_name,
       mm.code AS current_mode,
       s.is_online, s.last_heartbeat_at, s.is_enabled, s.plugin_version,
       s.api_key_prefix, s.api_key_created_at, s.api_key_revoked_at, s.created_at
  FROM servers s
  LEFT JOIN maps m ON m.id = s.current_map_id
  LEFT JOIN movement_modes mm ON mm.id = s.current_mode_id
"#;

pub async fn authenticate_server(
    pool: &PgPool,
    token: &str,
) -> Result<Option<AuthServer>, sqlx::Error> {
    sqlx::query_as::<_, AuthServer>(
        r#"
        SELECT id, name, is_enabled, api_key_revoked_at, plugin_version
          FROM servers
         WHERE api_key_hash = sha256(convert_to($1, 'UTF8'))
           AND deleted_at IS NULL
           AND approved_at IS NOT NULL
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await
}

pub async fn log_sync(
    pool: &PgPool,
    server_id: Option<i64>,
    kind: &str,
    status: &str,
    message: Option<&str>,
    record_id: Option<i64>,
    metadata: serde_json::Value,
) {
    let _ = sqlx::query(
        "INSERT INTO server_sync_logs (server_id, kind, status, message, record_id, metadata)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(server_id)
    .bind(kind)
    .bind(status)
    .bind(message)
    .bind(record_id)
    .bind(metadata)
    .execute(pool)
    .await;
}

#[allow(clippy::too_many_arguments)]
pub async fn record_heartbeat(
    pool: &PgPool,
    server_id: i64,
    _server_name: Option<&str>,
    map_name: Option<&str>,
    mode_id: Option<i16>,
    online_players: Option<i64>,
    max_players: Option<i64>,
    game_version: Option<&str>,
    plugin_version: Option<&str>,
    reported_at: Option<DateTime<Utc>>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    let map_id: Option<i64> =
        match map_name.filter(|value| !value.trim().is_empty()) {
            Some(name) => sqlx::query_scalar(
                "SELECT id FROM maps WHERE lower(name) = lower($1) AND deleted_at IS NULL LIMIT 1",
            )
            .bind(name.trim())
            .fetch_optional(&mut *tx)
            .await?,
            None => None,
        };

    let mut effective_online = online_players;
    let mut effective_max = max_players;
    if effective_online.is_none() || effective_max.is_none() {
        let (online, max): (i64, i32) = sqlx::query_as(
            "SELECT s.online_players::bigint,
                    greatest(s.max_players, COALESCE((SELECT max_players FROM server_heartbeats WHERE server_id = s.id), s.max_players))
               FROM servers s WHERE s.id = $1",
        )
        .bind(server_id)
        .fetch_one(&mut *tx)
        .await?;
        effective_online = Some(online);
        effective_max = Some(i64::from(max));
    }
    let online_players = effective_online.unwrap_or(0).clamp(0, 128);
    let max_players = effective_max.unwrap_or(24).clamp(1, 128);

    sqlx::query(
        r#"
        UPDATE servers
           SET is_online = TRUE,
               last_heartbeat_at = now(),
               current_map_id = COALESCE($2, current_map_id),
               current_mode_id = COALESCE($3, current_mode_id),
               online_players = $4,
               max_players = $5,
               game_version = COALESCE($6, game_version),
               plugin_version = COALESCE($7, plugin_version)
         WHERE id = $1
        "#,
    )
    .bind(server_id)
    .bind(map_id)
    .bind(mode_id)
    .bind(online_players as i32)
    .bind(max_players as i32)
    .bind(game_version)
    .bind(plugin_version)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO server_heartbeats
            (server_id, reported_at, map_name, current_map_id, mode_code,
             online_players, max_players, plugin_version, game_version)
        VALUES ($1, $2, $3, $4, (SELECT code FROM movement_modes WHERE id = $5),
                $6, $7, $8, $9)
        "#,
    )
    .bind(server_id)
    .bind(reported_at)
    .bind(map_name)
    .bind(map_id)
    .bind(mode_id)
    .bind(online_players as i32)
    .bind(max_players as i32)
    .bind(plugin_version)
    .bind(game_version)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
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

async fn resolve_map_mode(
    tx: &mut Transaction<'_, Postgres>,
    map_name: Option<&str>,
    mode_code: Option<&str>,
) -> Result<(Option<i64>, Option<i16>), sqlx::Error> {
    let map_id =
        match map_name.filter(|value| !value.trim().is_empty()) {
            Some(name) => sqlx::query_scalar(
                "SELECT id FROM maps WHERE lower(name) = lower($1) AND deleted_at IS NULL LIMIT 1",
            )
            .bind(name.trim())
            .fetch_optional(&mut **tx)
            .await?,
            None => None,
        };
    let mode_id = match mode_code.filter(|value| !value.trim().is_empty()) {
        Some(code) => {
            sqlx::query_scalar(
                "SELECT mode_id FROM scoring_modes WHERE lower(code) = lower($1)
             UNION ALL
             SELECT id FROM movement_modes WHERE lower(code) = lower($1)
             LIMIT 1",
            )
            .bind(code.trim())
            .fetch_optional(&mut **tx)
            .await?
        }
        None => None,
    };
    Ok((map_id, mode_id))
}

#[allow(clippy::too_many_arguments)]
pub async fn connect_player(
    pool: &PgPool,
    server_id: i64,
    steam_id64: &str,
    display_name: &str,
    map_name: Option<&str>,
    mode_code: Option<&str>,
    status: &str,
    connected_at: Option<DateTime<Utc>>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let account_id = ensure_account(&mut tx, steam_id64, display_name).await?;
    let (map_id, mode_id) = resolve_map_mode(&mut tx, map_name, mode_code).await?;
    sqlx::query(
        r#"
        INSERT INTO server_player_sessions
            (server_id, account_id, display_name, current_map_id, current_mode_id,
             status, connected_at, last_seen_at)
        VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, now()), now())
        ON CONFLICT (server_id, account_id) WHERE disconnected_at IS NULL
        DO UPDATE SET display_name = EXCLUDED.display_name,
                      current_map_id = EXCLUDED.current_map_id,
                      current_mode_id = EXCLUDED.current_mode_id,
                      status = EXCLUDED.status,
                      last_seen_at = now()
        "#,
    )
    .bind(server_id)
    .bind(account_id)
    .bind(display_name)
    .bind(map_id)
    .bind(mode_id)
    .bind(status)
    .bind(connected_at)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn disconnect_player(
    pool: &PgPool,
    server_id: i64,
    steam_id64: &str,
    disconnected_at: Option<DateTime<Utc>>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE server_player_sessions s
           SET status = 'disconnected',
               disconnected_at = COALESCE($3, now()),
               last_seen_at = COALESCE($3, now())
          FROM accounts a
         WHERE a.id = s.account_id
           AND s.server_id = $1
           AND a.steam_id64 = $2::numeric
           AND s.disconnected_at IS NULL
        "#,
    )
    .bind(server_id)
    .bind(steam_id64)
    .bind(disconnected_at)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn sync_players(
    pool: &PgPool,
    server_id: i64,
    players: &[crate::dto::game_server::PlayerEventRequest],
    map_name: Option<&str>,
    mode_code: Option<&str>,
) -> Result<(i64, i64, i64), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let (map_id, mode_id) = resolve_map_mode(&mut tx, map_name, mode_code).await?;
    let mut account_ids = Vec::with_capacity(players.len());
    let mut connected = 0_i64;

    for player in players {
        let display_name = sanitize_display_name(&player.display_name);
        let account_id = ensure_account(&mut tx, &player.steam_id64, &display_name).await?;
        account_ids.push(account_id);
        let status = player.status.as_deref().unwrap_or("connected");
        let status = if matches!(status, "connected" | "spectating" | "afk") {
            status
        } else {
            "connected"
        };
        let player_map_id = match player.map.as_deref() {
            Some(map) => resolve_map_mode(&mut tx, Some(map), None).await?.0,
            None => map_id,
        };
        let player_mode_id = match player.mode.as_deref() {
            Some(mode) => resolve_map_mode(&mut tx, None, Some(mode)).await?.1,
            None => mode_id,
        };
        sqlx::query(
            r#"
            INSERT INTO server_player_sessions
                (server_id, account_id, display_name, current_map_id, current_mode_id,
                 status, connected_at, last_seen_at)
            VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, now()), now())
            ON CONFLICT (server_id, account_id) WHERE disconnected_at IS NULL
            DO UPDATE SET display_name = EXCLUDED.display_name,
                          current_map_id = EXCLUDED.current_map_id,
                          current_mode_id = EXCLUDED.current_mode_id,
                          status = EXCLUDED.status,
                          last_seen_at = now()
            "#,
        )
        .bind(server_id)
        .bind(account_id)
        .bind(&display_name)
        .bind(player_map_id)
        .bind(player_mode_id)
        .bind(status)
        .bind(player.connected_at)
        .execute(&mut *tx)
        .await?;
        connected += 1;
    }

    let disconnected = if account_ids.is_empty() {
        sqlx::query(
            "UPDATE server_player_sessions
                SET status = 'disconnected', disconnected_at = now(), last_seen_at = now()
              WHERE server_id = $1 AND disconnected_at IS NULL",
        )
        .bind(server_id)
        .execute(&mut *tx)
        .await?
        .rows_affected()
    } else {
        sqlx::query(
            "UPDATE server_player_sessions
                SET status = 'disconnected', disconnected_at = now(), last_seen_at = now()
              WHERE server_id = $1 AND disconnected_at IS NULL
                AND NOT (account_id = ANY($2))",
        )
        .bind(server_id)
        .bind(&account_ids)
        .execute(&mut *tx)
        .await?
        .rows_affected()
    };

    sqlx::query("UPDATE servers SET online_players = $2 WHERE id = $1")
        .bind(server_id)
        .bind(account_ids.len() as i32)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok((connected, disconnected as i64, account_ids.len() as i64))
}

fn sanitize_display_name(name: &str) -> String {
    let trimmed: String = name.trim().chars().take(64).collect();
    if trimmed.is_empty() {
        "Unknown player".to_owned()
    } else {
        trimmed
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RecordContext {
    pub map_id: i64,
    pub course_id: i64,
    pub mode_id: i16,
    pub style_id: i16,
    pub tier_configured: bool,
}

pub async fn resolve_record_context(
    pool: &PgPool,
    map_name: &str,
    course_stage: i16,
    mode_code: &str,
    style_code: &str,
) -> Result<Option<RecordContext>, sqlx::Error> {
    sqlx::query_as::<_, RecordContext>(
        r#"
        SELECT m.id AS map_id, c.id AS course_id, mm.id AS mode_id, ms.id AS style_id,
               EXISTS (
                   SELECT 1 FROM course_mode_tiers cmt
                    WHERE cmt.map_id = m.id AND cmt.course_id = c.id AND cmt.mode_id = mm.id
               ) AS tier_configured
          FROM maps m
          JOIN map_courses c ON c.map_id = m.id AND c.stage = $2
          JOIN movement_modes mm ON lower(mm.code) = lower($3)
          JOIN movement_styles ms ON lower(ms.code) = lower($4)
         WHERE lower(m.name) = lower($1) AND m.deleted_at IS NULL
         LIMIT 1
        "#,
    )
    .bind(map_name.trim())
    .bind(course_stage)
    .bind(mode_code)
    .bind(style_code)
    .fetch_optional(pool)
    .await
}

#[derive(Debug, Clone)]
pub struct RecordInput {
    pub server_id: i64,
    pub server_plugin_version: Option<String>,
    pub steam_id64: String,
    pub player_name: Option<String>,
    pub map_id: i64,
    pub course_id: i64,
    pub mode_id: i16,
    pub style_id: i16,
    pub tickrate: i16,
    pub teleports: i32,
    pub time_ns: i64,
    pub record_identifier: String,
    pub created_at: Option<DateTime<Utc>>,
    pub replay_path: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ExistingRecord {
    pub id: i64,
    pub time_ns: i64,
    pub time_type: String,
}

#[derive(Debug, Clone)]
pub struct IngestResult {
    pub record_id: i64,
    pub duplicate: bool,
    pub time_ns: i64,
    pub time_type: String,
    pub is_pb: bool,
    pub is_world_record: bool,
    pub points_status: String,
    pub replay_status: String,
}

pub async fn find_record_by_key(
    pool: &PgPool,
    server_id: i64,
    record_identifier: &str,
) -> Result<Option<ExistingRecord>, sqlx::Error> {
    sqlx::query_as::<_, ExistingRecord>(
        "SELECT id, time_ns, time_type FROM records
          WHERE server_id = $1 AND submission_key = $2",
    )
    .bind(server_id)
    .bind(record_identifier)
    .fetch_optional(pool)
    .await
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
        "SELECT EXISTS (
            SELECT 1 FROM course_world_records
             WHERE course_id = $1 AND mode_id = $2 AND style_id = $3
               AND time_type = $4 AND record_id = $5
         )",
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

async fn replay_status(
    tx: &mut Transaction<'_, Postgres>,
    record_id: i64,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT status FROM replays WHERE record_id = $1 AND deleted_at IS NULL
          ORDER BY created_at DESC LIMIT 1",
    )
    .bind(record_id)
    .fetch_optional(&mut **tx)
    .await
    .map(|value| value.flatten().unwrap_or_else(|| "not_reported".to_owned()))
}

pub async fn ingest_record(
    pool: &PgPool,
    input: &RecordInput,
) -> Result<IngestResult, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let display_name = input
        .player_name
        .as_deref()
        .map(sanitize_display_name)
        .unwrap_or_else(|| "Unknown player".to_owned());
    let account_id = ensure_account(&mut tx, &input.steam_id64, &display_name).await?;

    let existing = sqlx::query_as::<_, ExistingRecord>(
        "SELECT id, time_ns, time_type FROM records
          WHERE server_id = $1 AND submission_key = $2",
    )
    .bind(input.server_id)
    .bind(&input.record_identifier)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(existing) = existing {
        let (is_pb, is_world_record) = record_flags(
            &mut tx,
            existing.id,
            account_id,
            input.map_id,
            input.course_id,
            input.mode_id,
            input.style_id,
            &existing.time_type,
            existing.time_ns,
        )
        .await?;
        let replay_status = replay_status(&mut tx, existing.id).await?;
        sqlx::query(
            "INSERT INTO server_sync_logs (server_id, kind, status, message, record_id, metadata)
             VALUES ($1, 'record', 'duplicate', 'duplicate record_identifier', $2, $3)",
        )
        .bind(input.server_id)
        .bind(existing.id)
        .bind(json!({ "record_identifier": input.record_identifier }))
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Ok(IngestResult {
            record_id: existing.id,
            duplicate: true,
            time_ns: existing.time_ns,
            time_type: existing.time_type,
            is_pb,
            is_world_record,
            points_status: "queued".to_owned(),
            replay_status,
        });
    }

    let time_type = if input.teleports == 0 { "pro" } else { "nub" };
    let previous_pb: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT min(time_ns) FROM records
         WHERE account_id = $1 AND map_id = $2 AND course_id = $3
           AND mode_id = $4 AND style_id = $5 AND time_type = $6
           AND status = 'approved' AND deleted_at IS NULL
        "#,
    )
    .bind(account_id)
    .bind(input.map_id)
    .bind(input.course_id)
    .bind(input.mode_id)
    .bind(input.style_id)
    .bind(time_type)
    .fetch_one(&mut *tx)
    .await?;
    let is_pb =
        previous_pb.map(|time| input.time_ns < time).unwrap_or(true) || previous_pb.is_none();

    let inserted: Option<i64> = sqlx::query_scalar(
        r#"
        INSERT INTO records
            (account_id, map_id, course_id, mode_id, style_id, server_id,
             server_plugin_version, tickrate, teleports, time_type, time_ns,
             status, source, submission_key, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                'approved', 'plugin', $12, COALESCE($13, now()))
        ON CONFLICT (server_id, submission_key)
            WHERE submission_key IS NOT NULL AND server_id IS NOT NULL
        DO NOTHING
        RETURNING id
        "#,
    )
    .bind(account_id)
    .bind(input.map_id)
    .bind(input.course_id)
    .bind(input.mode_id)
    .bind(input.style_id)
    .bind(input.server_id)
    .bind(&input.server_plugin_version)
    .bind(input.tickrate)
    .bind(input.teleports)
    .bind(time_type)
    .bind(input.time_ns)
    .bind(&input.record_identifier)
    .bind(input.created_at)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(record_id) = inserted else {
        // A concurrent retry won the unique-index race. Return the existing row.
        let existing = sqlx::query_as::<_, ExistingRecord>(
            "SELECT id, time_ns, time_type FROM records
              WHERE server_id = $1 AND submission_key = $2",
        )
        .bind(input.server_id)
        .bind(&input.record_identifier)
        .fetch_one(&mut *tx)
        .await?;
        let (is_pb, is_world_record) = record_flags(
            &mut tx,
            existing.id,
            account_id,
            input.map_id,
            input.course_id,
            input.mode_id,
            input.style_id,
            &existing.time_type,
            existing.time_ns,
        )
        .await?;
        let replay_status = replay_status(&mut tx, existing.id).await?;
        tx.commit().await?;
        return Ok(IngestResult {
            record_id: existing.id,
            duplicate: true,
            time_ns: existing.time_ns,
            time_type: existing.time_type,
            is_pb,
            is_world_record,
            points_status: "queued".to_owned(),
            replay_status,
        });
    };

    sqlx::query(
        "INSERT INTO record_history (record_id, action, new_status, metadata)
         VALUES ($1, 'created', 'approved', $2)",
    )
    .bind(record_id)
    .bind(json!({
        "server_id": input.server_id,
        "record_identifier": input.record_identifier,
        "source": "cngokz_sync"
    }))
    .execute(&mut *tx)
    .await?;

    // Phase 8: plugin submissions also enter the raw/provenance pipeline. The
    // canonical row stays in `records`; this is not a second scoring path.
    if let Some(source_id) =
        sqlx::query_scalar::<_, i64>("SELECT id FROM record_sources WHERE code = 'cngokz_plugin'")
            .fetch_optional(&mut *tx)
            .await?
    {
        let raw_payload = json!({
            "server_id": input.server_id,
            "record_identifier": input.record_identifier,
            "steam_id64": input.steam_id64,
            "map_id": input.map_id,
            "course_id": input.course_id,
            "mode_id": input.mode_id,
            "style_id": input.style_id,
            "time_ns": input.time_ns,
            "teleports": input.teleports,
            "record_type": time_type,
            "plugin_version": input.server_plugin_version
        });
        let raw_id: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO raw_records (source_id, external_record_id, payload_hash, raw_payload)
            VALUES ($1, $2, sha256(convert_to($3, 'UTF8')), $4)
            ON CONFLICT (source_id, external_record_id) DO UPDATE
               SET received_at = raw_records.received_at
            RETURNING id
            "#,
        )
        .bind(source_id)
        .bind(&input.record_identifier)
        .bind(raw_payload.to_string())
        .bind(&raw_payload)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query(
            "INSERT INTO record_source_links
                (canonical_record_id, raw_record_id, source_id, external_record_id,
                 source_recorded_at, match_method, match_confidence, field_conflicts)
             VALUES ($1, $2, $3, $4, $5, 'plugin_submission', 1.0, '{}'::jsonb)
             ON CONFLICT (raw_record_id) DO NOTHING",
        )
        .bind(record_id)
        .bind(raw_id)
        .bind(source_id)
        .bind(&input.record_identifier)
        .bind(input.created_at)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE raw_records SET processing_status = 'processed', canonical_record_id = $2 WHERE id = $1",
        )
        .bind(raw_id)
        .bind(record_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE record_sources
                SET status = 'healthy', last_attempt_at = now(), last_success_at = now(),
                    last_error = NULL, consecutive_failures = 0
              WHERE code = 'cngokz_plugin'",
        )
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query(
        r#"
        INSERT INTO course_world_records
            (course_id, mode_id, style_id, time_type, record_id, time_ns, achieved_at)
        VALUES ($1, $2, $3, $4, $5, $6, now())
        ON CONFLICT (course_id, mode_id, style_id, time_type) DO UPDATE
           SET record_id = EXCLUDED.record_id,
               time_ns = EXCLUDED.time_ns,
               achieved_at = EXCLUDED.achieved_at
         WHERE EXCLUDED.time_ns < course_world_records.time_ns
        "#,
    )
    .bind(input.course_id)
    .bind(input.mode_id)
    .bind(input.style_id)
    .bind(time_type)
    .bind(record_id)
    .bind(input.time_ns)
    .execute(&mut *tx)
    .await?;

    let is_world_record: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM course_world_records
             WHERE course_id = $1 AND mode_id = $2 AND style_id = $3
               AND time_type = $4 AND record_id = $5
         )",
    )
    .bind(input.course_id)
    .bind(input.mode_id)
    .bind(input.style_id)
    .bind(time_type)
    .bind(record_id)
    .fetch_one(&mut *tx)
    .await?;

    let filter_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM scoring_filters
             WHERE map_id = $1 AND course_id = $2 AND mode_id = $3 AND record_type = $4
         )",
    )
    .bind(input.map_id)
    .bind(input.course_id)
    .bind(input.mode_id)
    .bind(time_type)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO server_sync_tasks (server_id, record_id, task_type, status, payload)
         VALUES ($1, $2, 'replay_upload', 'pending', $3)",
    )
    .bind(input.server_id)
    .bind(record_id)
    .bind(json!({ "replay_path": input.replay_path }))
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO server_sync_logs (server_id, kind, status, message, record_id, metadata)
         VALUES ($1, 'record', 'success', 'record accepted', $2, $3)",
    )
    .bind(input.server_id)
    .bind(record_id)
    .bind(json!({
        "record_identifier": input.record_identifier,
        "time_ns": input.time_ns,
        "time_type": time_type,
        "is_pb": is_pb,
        "is_world_record": is_world_record
    }))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(IngestResult {
        record_id,
        duplicate: false,
        time_ns: input.time_ns,
        time_type: time_type.to_owned(),
        is_pb,
        is_world_record,
        points_status: if filter_exists {
            "queued".to_owned()
        } else {
            "not_scored".to_owned()
        },
        replay_status: "pending".to_owned(),
    })
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StatusRecordRow {
    pub id: i64,
    pub account_id: i64,
    pub map_id: i64,
    pub course_id: i64,
    pub mode_id: i16,
    pub style_id: i16,
    pub time_type: String,
    pub time_ns: i64,
    pub status: String,
    pub replay_id: Option<i64>,
    pub replay_status: Option<String>,
}

pub async fn record_status(
    pool: &PgPool,
    server_id: i64,
    record_identifier: &str,
) -> Result<Option<StatusRecordRow>, sqlx::Error> {
    sqlx::query_as::<_, StatusRecordRow>(
        r#"
        SELECT r.id, r.account_id, r.map_id, r.course_id, r.mode_id, r.style_id,
               r.time_type, r.time_ns, r.status,
               replay.id AS replay_id, replay.status AS replay_status
          FROM records r
          LEFT JOIN LATERAL (
              SELECT rp.id, rp.status FROM replays rp
               WHERE rp.record_id = r.id AND rp.deleted_at IS NULL
               ORDER BY rp.created_at DESC LIMIT 1
          ) replay ON TRUE
         WHERE r.server_id = $1 AND r.submission_key = $2
        "#,
    )
    .bind(server_id)
    .bind(record_identifier)
    .fetch_optional(pool)
    .await
}

pub async fn status_flags(
    pool: &PgPool,
    row: &StatusRecordRow,
) -> Result<(bool, bool, String), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let (is_pb, is_world_record) = record_flags(
        &mut tx,
        row.id,
        row.account_id,
        row.map_id,
        row.course_id,
        row.mode_id,
        row.style_id,
        &row.time_type,
        row.time_ns,
    )
    .await?;
    let dirty: Option<bool> = sqlx::query_scalar(
        "SELECT dirty FROM scoring_filters
          WHERE map_id = $1 AND course_id = $2 AND mode_id = $3 AND record_type = $4",
    )
    .bind(row.map_id)
    .bind(row.course_id)
    .bind(row.mode_id)
    .bind(&row.time_type)
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;
    let points_status = match dirty {
        Some(true) => "queued",
        Some(false) => "calculated",
        None => "not_scored",
    };
    Ok((is_pb, is_world_record, points_status.to_owned()))
}

#[allow(clippy::too_many_arguments)]
pub async fn register_replay(
    pool: &PgPool,
    server_id: i64,
    record_identifier: &str,
    bucket: &str,
    object_key: &str,
    storage_provider: &str,
    size_bytes: i64,
    checksum: &[u8],
    replay_format_version: Option<i16>,
    status: &str,
    uploaded_at: Option<DateTime<Utc>>,
) -> Result<Option<(i64, i64, String, bool)>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let record = sqlx::query_as::<_, (i64, i64, i64)>(
        "SELECT r.id, r.account_id, r.map_id FROM records r
          WHERE r.server_id = $1 AND r.submission_key = $2",
    )
    .bind(server_id)
    .bind(record_identifier)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((record_id, account_id, map_id)) = record else {
        tx.rollback().await?;
        return Ok(None);
    };

    let inserted: Option<(i64, String)> = sqlx::query_as(
        r#"
        INSERT INTO replays
            (record_id, account_id, map_id, storage_provider, bucket, object_key,
             size_bytes, checksum_sha256, replay_format_version, status, uploaded_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (bucket, object_key) DO NOTHING
        RETURNING id, status
        "#,
    )
    .bind(record_id)
    .bind(account_id)
    .bind(map_id)
    .bind(storage_provider)
    .bind(bucket)
    .bind(object_key)
    .bind(size_bytes)
    .bind(checksum)
    .bind(replay_format_version)
    .bind(status)
    .bind(uploaded_at)
    .fetch_optional(&mut *tx)
    .await?;

    let (replay_id, replay_status, duplicate) = match inserted {
        Some((id, status)) => (id, status, false),
        None => {
            let existing: (i64, String) = sqlx::query_as(
                "SELECT id, status FROM replays WHERE bucket = $1 AND object_key = $2",
            )
            .bind(bucket)
            .bind(object_key)
            .fetch_one(&mut *tx)
            .await?;
            (existing.0, existing.1, true)
        }
    };

    if replay_status == "available" {
        sqlx::query(
            "UPDATE server_sync_tasks
                SET status = 'done', completed_at = now(), last_error = NULL
              WHERE server_id = $1 AND record_id = $2 AND task_type = 'replay_upload'",
        )
        .bind(server_id)
        .bind(record_id)
        .execute(&mut *tx)
        .await?;
    } else if replay_status == "failed" {
        sqlx::query(
            "UPDATE server_sync_tasks
                SET status = 'failed', attempts = attempts + 1, last_error = 'replay upload failed'
              WHERE server_id = $1 AND record_id = $2 AND task_type = 'replay_upload'",
        )
        .bind(server_id)
        .bind(record_id)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query(
        "INSERT INTO server_sync_logs (server_id, kind, status, message, record_id, metadata)
         VALUES ($1, 'replay', $2, $3, $4, $5)",
    )
    .bind(server_id)
    .bind(if duplicate { "duplicate" } else { "success" })
    .bind(if duplicate {
        "replay already registered"
    } else {
        "replay metadata registered"
    })
    .bind(record_id)
    .bind(json!({ "record_identifier": record_identifier, "object_key": object_key }))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Some((replay_id, record_id, replay_status, duplicate)))
}

pub async fn config_version(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT GREATEST(
            COALESCE(EXTRACT(EPOCH FROM (SELECT max(updated_at) FROM maps))::bigint, 0),
            COALESCE(EXTRACT(EPOCH FROM (SELECT max(updated_at) FROM map_courses))::bigint, 0),
            COALESCE(EXTRACT(EPOCH FROM (SELECT max(updated_at) FROM course_mode_tiers))::bigint, 0),
            COALESCE(EXTRACT(EPOCH FROM (SELECT max(created_at) FROM movement_modes))::bigint, 0),
            COALESCE(EXTRACT(EPOCH FROM (SELECT max(created_at) FROM movement_styles))::bigint, 0),
            1
        )::bigint
        "#,
    )
    .fetch_one(pool)
    .await
}

pub async fn config_modes(pool: &PgPool) -> Result<Vec<ConfigMode>, sqlx::Error> {
    sqlx::query_as::<_, ConfigMode>("SELECT id, code, display_name FROM movement_modes ORDER BY id")
        .fetch_all(pool)
        .await
}

pub async fn config_styles(pool: &PgPool) -> Result<Vec<ConfigStyle>, sqlx::Error> {
    sqlx::query_as::<_, ConfigStyle>(
        "SELECT id, code, display_name FROM movement_styles ORDER BY id",
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, sqlx::FromRow)]
struct ConfigMapRow {
    id: i64,
    name: String,
    status: String,
    modes: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct ConfigTierRow {
    course_id: i64,
    mode_code: String,
    tier: i16,
}

pub async fn config_maps(pool: &PgPool) -> Result<Vec<ConfigMap>, sqlx::Error> {
    let maps: Vec<ConfigMapRow> = sqlx::query_as(
        r#"
        SELECT m.id, m.name, m.status,
               (SELECT string_agg(mm.code, ',' ORDER BY mm.code)
                  FROM map_modes mmd JOIN movement_modes mm ON mm.id = mmd.mode_id
                 WHERE mmd.map_id = m.id) AS modes
          FROM maps m
         WHERE m.deleted_at IS NULL
         ORDER BY m.name
        "#,
    )
    .fetch_all(pool)
    .await?;
    let mut result = Vec::with_capacity(maps.len());
    for map in maps {
        let courses: Vec<ConfigCourse> = sqlx::query_as(
            "SELECT id, stage, name FROM map_courses WHERE map_id = $1 ORDER BY stage",
        )
        .bind(map.id)
        .fetch_all(pool)
        .await?;
        let tiers: Vec<ConfigTierRow> = sqlx::query_as(
            r#"
            SELECT cmt.course_id, sm.code AS mode_code, cmt.tier
              FROM course_mode_tiers cmt
              JOIN scoring_modes sm ON sm.mode_id = cmt.mode_id
             WHERE cmt.map_id = $1
             ORDER BY cmt.course_id, sm.sort_order
            "#,
        )
        .bind(map.id)
        .fetch_all(pool)
        .await?;
        result.push(ConfigMap {
            id: map.id,
            name: map.name,
            status: map.status,
            modes: map
                .modes
                .unwrap_or_default()
                .split(',')
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
            courses,
            tiers: tiers
                .into_iter()
                .map(|row| ConfigTier {
                    course_id: row.course_id,
                    mode: row.mode_code,
                    tier: row.tier,
                })
                .collect(),
        });
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Admin server management
// ---------------------------------------------------------------------------

pub async fn admin_list_servers(
    pool: &PgPool,
    search: Option<&str>,
    enabled: Option<bool>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ServerAdminRow>, sqlx::Error> {
    let mut qb = sqlx::QueryBuilder::<Postgres>::new(ADMIN_SERVER_SELECT);
    qb.push(" WHERE s.deleted_at IS NULL");
    if let Some(search) = search.filter(|value| !value.trim().is_empty()) {
        qb.push(" AND (s.name ILIKE ");
        qb.push_bind(format!("%{}%", search.trim()));
        qb.push(" OR s.ip::text LIKE ");
        qb.push_bind(format!("%{}%", search.trim()));
        qb.push(")");
    }
    if let Some(enabled) = enabled {
        qb.push(" AND s.is_enabled = ");
        qb.push_bind(enabled);
    }
    qb.push(" ORDER BY s.name ASC LIMIT ");
    qb.push_bind(limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);
    qb.build_query_as::<ServerAdminRow>().fetch_all(pool).await
}

pub async fn admin_count_servers(
    pool: &PgPool,
    search: Option<&str>,
    enabled: Option<bool>,
) -> Result<i64, sqlx::Error> {
    let mut qb = sqlx::QueryBuilder::<Postgres>::new(
        "SELECT count(*) FROM servers s WHERE s.deleted_at IS NULL",
    );
    if let Some(search) = search.filter(|value| !value.trim().is_empty()) {
        qb.push(" AND (s.name ILIKE ");
        qb.push_bind(format!("%{}%", search.trim()));
        qb.push(" OR s.ip::text LIKE ");
        qb.push_bind(format!("%{}%", search.trim()));
        qb.push(")");
    }
    if let Some(enabled) = enabled {
        qb.push(" AND s.is_enabled = ");
        qb.push_bind(enabled);
    }
    qb.build_query_scalar::<i64>().fetch_one(pool).await
}

pub async fn admin_find_server(
    pool: &PgPool,
    server_id: i64,
) -> Result<Option<ServerAdminRow>, sqlx::Error> {
    let mut qb = sqlx::QueryBuilder::<Postgres>::new(ADMIN_SERVER_SELECT);
    qb.push(" WHERE s.deleted_at IS NULL AND s.id = ");
    qb.push_bind(server_id);
    qb.build_query_as::<ServerAdminRow>()
        .fetch_optional(pool)
        .await
}

#[allow(clippy::too_many_arguments)]
pub async fn admin_create_server(
    pool: &PgPool,
    name: &str,
    ip: &str,
    port: i32,
    query_port: Option<i32>,
    region: Option<&str>,
    server_type: &str,
    max_players: i32,
    owner_steam_id64: &str,
    owner_display_name: &str,
    plugin_version: Option<&str>,
    token: &str,
) -> Result<i64, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let owner_account_id = ensure_account(&mut tx, owner_steam_id64, owner_display_name).await?;
    let prefix: String = token.chars().take(12).collect();
    let server_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO servers
            (name, ip, port, query_port, region, server_type, max_players,
             owner_account_id, api_key_hash, api_key_hash_algorithm, api_key_prefix,
             plugin_version, approved_at, is_enabled, game)
        VALUES ($1, $2::inet, $3, $4, $5, $6, $7, $8,
                sha256(convert_to($9, 'UTF8')), 'sha256', $10, $11, now(), TRUE, 'csgo')
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(ip)
    .bind(port)
    .bind(query_port)
    .bind(region)
    .bind(server_type)
    .bind(max_players)
    .bind(owner_account_id)
    .bind(token)
    .bind(prefix)
    .bind(plugin_version)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO server_sync_logs (server_id, kind, status, message, metadata)
         VALUES ($1, 'other', 'success', 'server created', $2)",
    )
    .bind(server_id)
    .bind(json!({ "owner_account_id": owner_account_id }))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(server_id)
}

#[allow(clippy::too_many_arguments)]
pub async fn admin_update_server(
    pool: &PgPool,
    server_id: i64,
    name: Option<&str>,
    region: Option<&str>,
    server_type: Option<&str>,
    max_players: Option<i32>,
    query_port: Option<i32>,
    plugin_version: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE servers
           SET name = COALESCE($2, name),
               region = COALESCE($3, region),
               server_type = COALESCE($4, server_type),
               max_players = COALESCE($5, max_players),
               query_port = COALESCE($6, query_port),
               plugin_version = COALESCE($7, plugin_version),
               online_players = LEAST(online_players, COALESCE($5, max_players))
         WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(server_id)
    .bind(name)
    .bind(region)
    .bind(server_type)
    .bind(max_players)
    .bind(query_port)
    .bind(plugin_version)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn admin_set_enabled(
    pool: &PgPool,
    server_id: i64,
    enabled: bool,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let result = sqlx::query(
        "UPDATE servers SET is_enabled = $2, is_online = CASE WHEN $2 THEN is_online ELSE FALSE END
          WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(server_id)
    .bind(enabled)
    .execute(&mut *tx)
    .await?;
    if !enabled && result.rows_affected() > 0 {
        sqlx::query(
            "UPDATE server_player_sessions
                SET status = 'disconnected', disconnected_at = now(), last_seen_at = now()
              WHERE server_id = $1 AND disconnected_at IS NULL",
        )
        .bind(server_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE servers SET online_players = 0 WHERE id = $1")
            .bind(server_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(result.rows_affected() > 0)
}

pub async fn admin_rotate_token(
    pool: &PgPool,
    server_id: i64,
    token: &str,
) -> Result<Option<(String, DateTime<Utc>)>, sqlx::Error> {
    let prefix: String = token.chars().take(12).collect();
    let created_at: Option<DateTime<Utc>> = sqlx::query_scalar(
        r#"
        UPDATE servers
           SET api_key_hash = sha256(convert_to($2, 'UTF8')),
               api_key_hash_algorithm = 'sha256',
               api_key_prefix = $3,
               api_key_created_at = now(),
               api_key_revoked_at = NULL
         WHERE id = $1 AND deleted_at IS NULL
        RETURNING api_key_created_at
        "#,
    )
    .bind(server_id)
    .bind(token)
    .bind(&prefix)
    .fetch_optional(pool)
    .await?;
    Ok(created_at.map(|at| (prefix, at)))
}

pub async fn admin_revoke_token(pool: &PgPool, server_id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE servers SET api_key_revoked_at = now()
          WHERE id = $1 AND deleted_at IS NULL AND api_key_revoked_at IS NULL",
    )
    .bind(server_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn admin_server_detail(
    pool: &PgPool,
    server: ServerAdminRow,
) -> Result<AdminServerDetailResponse, sqlx::Error> {
    let server_id = server.id;
    let heartbeats = sqlx::query_as::<_, AdminHeartbeatRow>(
        r#"
        SELECT id, received_at, map_name, mode_code, online_players, max_players,
               plugin_version, game_version
          FROM server_heartbeats WHERE server_id = $1
         ORDER BY received_at DESC LIMIT 20
        "#,
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;
    let players = sqlx::query_as::<_, AdminPlayerRow>(
        r#"
        SELECT a.id AS account_id, a.steam_id64::text AS steam_id64, s.display_name,
               s.status, s.connected_at, s.last_seen_at
          FROM server_player_sessions s JOIN accounts a ON a.id = s.account_id
         WHERE s.server_id = $1 AND s.disconnected_at IS NULL
         ORDER BY s.connected_at ASC
        "#,
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;
    let records = sqlx::query_as::<_, AdminRecordRow>(
        r#"
        SELECT r.id, a.id AS account_id, a.steam_id64::text AS steam_id64,
               p.display_name, m.name AS map_name, c.stage AS course_stage,
               mm.code AS mode_code, r.time_type, r.time_ns,
               ''::text AS time_seconds,
               r.status,
               EXISTS (SELECT 1 FROM course_world_records w WHERE w.record_id = r.id) AS is_world_record,
               (SELECT rp.status FROM replays rp WHERE rp.record_id = r.id
                 ORDER BY rp.created_at DESC LIMIT 1) AS replay_status,
               r.created_at
          FROM records r
          JOIN accounts a ON a.id = r.account_id
          LEFT JOIN player_profiles p ON p.account_id = a.id
          JOIN maps m ON m.id = r.map_id
          JOIN map_courses c ON c.id = r.course_id
          JOIN movement_modes mm ON mm.id = r.mode_id
         WHERE r.server_id = $1 AND r.deleted_at IS NULL
         ORDER BY r.created_at DESC LIMIT 20
        "#,
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;
    let records = records
        .into_iter()
        .map(|mut row| {
            row.time_seconds = seconds_string_3(row.time_ns);
            row
        })
        .collect();
    let replays = sqlx::query_as::<_, AdminReplayRow>(
        r#"
        SELECT rp.id, rp.record_id, rp.bucket, rp.object_key, rp.size_bytes,
               rp.status, rp.uploaded_at, rp.created_at
          FROM replays rp JOIN records r ON r.id = rp.record_id
         WHERE r.server_id = $1 AND rp.deleted_at IS NULL
         ORDER BY rp.created_at DESC LIMIT 20
        "#,
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;
    let logs = sqlx::query_as::<_, AdminSyncLogRow>(
        r#"
        SELECT id, kind, status, message, record_id, created_at
          FROM server_sync_logs WHERE server_id = $1
         ORDER BY created_at DESC LIMIT 50
        "#,
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;
    let tasks = sqlx::query_as::<_, AdminSyncTaskRow>(
        r#"
        SELECT id, record_id, task_type, status, attempts, last_error, created_at, updated_at
          FROM server_sync_tasks WHERE server_id = $1
         ORDER BY created_at DESC LIMIT 50
        "#,
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;

    Ok(AdminServerDetailResponse {
        server: server.into_dto(),
        recent_heartbeats: heartbeats,
        online_players: players,
        recent_records: records,
        recent_replays: replays,
        sync_logs: logs,
        sync_tasks: tasks,
    })
}

pub async fn admin_retry_task(pool: &PgPool, task_id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        "UPDATE server_sync_tasks
            SET status = 'pending', last_error = NULL, completed_at = NULL
          WHERE id = $1 AND status IN ('failed', 'pending')
        RETURNING status",
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await
}

// ---------------------------------------------------------------------------
// Public online players
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PublicOnlinePlayerRow {
    pub account_id: i64,
    pub steam_id64: String,
    pub display_name: String,
    pub server_id: i64,
    pub server_name: String,
    pub map_name: Option<String>,
    pub mode_code: Option<String>,
    pub status: String,
    pub connected_at: DateTime<Utc>,
}

pub async fn public_online_players(
    pool: &PgPool,
    server_id: Option<i64>,
    limit: i64,
) -> Result<Vec<PublicOnlinePlayerRow>, sqlx::Error> {
    let mut qb = sqlx::QueryBuilder::<Postgres>::new(
        r#"
        SELECT a.id AS account_id, a.steam_id64::text AS steam_id64, s.display_name,
               sv.id AS server_id, sv.name AS server_name, m.name AS map_name,
               mm.code AS mode_code, s.status, s.connected_at
          FROM server_player_sessions s
          JOIN accounts a ON a.id = s.account_id AND a.deleted_at IS NULL
          JOIN servers sv ON sv.id = s.server_id AND sv.deleted_at IS NULL AND sv.is_enabled
          LEFT JOIN maps m ON m.id = s.current_map_id
          LEFT JOIN movement_modes mm ON mm.id = s.current_mode_id
         WHERE s.disconnected_at IS NULL
           AND sv.is_online
           AND sv.last_heartbeat_at IS NOT NULL
           AND sv.last_heartbeat_at > now() - interval '2 minutes'
        "#,
    );
    if let Some(server_id) = server_id {
        qb.push(" AND s.server_id = ");
        qb.push_bind(server_id);
    }
    qb.push(" ORDER BY s.connected_at ASC LIMIT ");
    qb.push_bind(limit);
    qb.build_query_as::<PublicOnlinePlayerRow>()
        .fetch_all(pool)
        .await
}
