//! Phase-8 external data source orchestration.
//!
//! Sources are intentionally isolated adapters around one canonical ingestion
//! function. External points/ranks are stored only inside the raw payload for
//! audit and are never read by the CNGOKZ scoring engine.

use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
};

use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::{info, warn};

use crate::dto::source::{ExternalServerSnapshot, SourceSyncResponse};
use crate::errors::AppError;
use crate::repositories::sources::{self as repo, ExternalRecordInput};
use crate::state::AppState;

const KZGO_MODES: [(&str, &str); 3] = [
    ("kz_timer", "kztimer"),
    ("kz_simple", "simple"),
    ("kz_vanilla", "vanilla"),
];
const GLOBALAPI_MODES: [(&str, &str); 3] = [
    ("kz_timer", "kztimer"),
    ("kz_simple", "simple"),
    ("kz_vanilla", "vanilla"),
];

#[derive(Debug, Deserialize, serde::Serialize)]
struct KzgoMap {
    id: i64,
    name: String,
    #[serde(default)]
    tier: i16,
    #[serde(rename = "workshopId")]
    workshop_id: Option<String>,
    #[serde(default)]
    bonuses: i16,
    #[serde(default)]
    sp: bool,
    #[serde(default)]
    vp: bool,
    #[serde(default)]
    date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct KzgoWr {
    #[serde(rename = "mapId")]
    map_id: i64,
    #[serde(rename = "mapName")]
    map_name: String,
    #[serde(rename = "playerName")]
    player_name: String,
    #[serde(rename = "steamId64")]
    steam_id64: String,
    #[serde(rename = "serverName")]
    server_name: Option<String>,
    #[serde(rename = "createdOn")]
    created_on: String,
    time: f64,
    #[serde(default)]
    tps: i32,
}

#[derive(Debug, Deserialize)]
struct KzgoServerResponse {
    #[serde(rename = "serverStates", default)]
    server_states: Vec<KzgoServer>,
}

#[derive(Debug, Deserialize)]
struct KzgoServer {
    #[serde(rename = "type", default)]
    kind: String,
    ip: String,
    port: i32,
    label: Option<String>,
    name: String,
    map: Option<KzgoServerMap>,
    #[serde(default)]
    players: Vec<Value>,
    #[serde(rename = "maxPlayers", default)]
    max_players_api: i32,
    #[serde(rename = "errBefore", default)]
    err_before_api: bool,
}

#[derive(Debug, Deserialize)]
struct KzgoServerMap {
    name: Option<String>,
    tier: Option<i16>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
struct GokzGlobalRecord {
    id: Option<Value>,
    steamid64: Option<String>,
    steam_id: Option<String>,
    player_name: Option<String>,
    map_id: Option<i64>,
    map_name: Option<String>,
    stage: Option<i16>,
    mode: Option<String>,
    teleports: Option<i32>,
    time: Option<f64>,
    created_on: Option<String>,
    server_name: Option<String>,
}

fn client(state: &AppState) -> Result<Client, AppError> {
    Client::builder()
        .timeout(state.config().source_http_timeout)
        .user_agent("CNGOKZAPI/0.1 (+https://github.com/XBDJ504764827/cnapi)")
        .build()
        .map_err(AppError::internal)
}

async fn fetch_json<T: for<'de> Deserialize<'de>>(
    client: &Client,
    url: &str,
) -> Result<T, anyhow::Error> {
    let response = client.get(url).send().await?;
    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("upstream HTTP {status} for {url}");
    }
    Ok(response.json::<T>().await?)
}

fn parse_kzgo_date(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.with_timezone(&Utc))
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S")
                .ok()
                .map(|value| value.and_utc())
        })
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .map(|value| value.and_utc())
        })
}

fn parse_global_date(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.with_timezone(&Utc))
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .map(|value| value.and_utc())
        })
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S")
                .ok()
                .map(|value| value.and_utc())
        })
}

fn time_ns(seconds: f64) -> i64 {
    (seconds * 1_000_000_000.0).round() as i64
}

fn payload_hash(value: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.to_string().hash(&mut hasher);
    hasher.finish()
}

#[allow(clippy::too_many_arguments)]
async fn ingest(
    state: &AppState,
    source_code: &str,
    external_record_id: String,
    raw: Value,
    steam_id64: &str,
    display_name: &str,
    map_name: &str,
    course_stage: i16,
    mode_code: &str,
    time_seconds: f64,
    teleports: i32,
    recorded_at: Option<DateTime<Utc>>,
    server_name: Option<&str>,
) -> Result<bool, anyhow::Error> {
    let result = repo::ingest_external(
        state.db(),
        &ExternalRecordInput {
            source_code: source_code.to_owned(),
            external_record_id,
            raw_payload: raw,
            steam_id64: steam_id64.to_owned(),
            display_name: display_name.to_owned(),
            map_name: map_name.to_owned(),
            course_stage,
            mode_code: mode_code.to_owned(),
            style_code: "normal".to_owned(),
            time_ns: time_ns(time_seconds),
            teleports,
            recorded_at,
            server_name: server_name.map(ToOwned::to_owned),
        },
    )
    .await;
    match result {
        Ok(outcome) => Ok(!outcome.duplicate),
        Err(repo::SourceIngestError::Validation(message)) => {
            anyhow::bail!("{message}")
        }
        Err(repo::SourceIngestError::Database(error)) => Err(error.into()),
    }
}

async fn sync_kzgo_maps(
    state: &AppState,
    source_id: i64,
    maps: &[KzgoMap],
) -> Result<usize, anyhow::Error> {
    let mode_ids: HashMap<String, i16> =
        sqlx::query_as::<_, (String, i16)>("SELECT code, id FROM movement_modes")
            .fetch_all(state.db())
            .await?
            .into_iter()
            .collect();
    let scoring_modes: HashMap<String, i16> =
        sqlx::query_as::<_, (String, i16)>("SELECT code, mode_id FROM scoring_modes")
            .fetch_all(state.db())
            .await?
            .into_iter()
            .collect();

    let mut tx = state.db().begin().await?;
    let mut upserted = 0_usize;
    for map in maps {
        let workshop_id = map
            .workshop_id
            .as_deref()
            .and_then(|value| value.parse::<i64>().ok());
        let approved_at = map.date.as_deref().and_then(parse_kzgo_date);
        let existing: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM maps WHERE lower(name) = lower($1) AND deleted_at IS NULL",
        )
        .bind(&map.name)
        .fetch_optional(&mut *tx)
        .await?;
        let map_id = if let Some(id) = existing {
            sqlx::query(
                "UPDATE maps SET workshop_id = COALESCE($2, workshop_id),
                                 status = 'approved',
                                 approved_at = COALESCE(approved_at, $3)
                  WHERE id = $1",
            )
            .bind(id)
            .bind(workshop_id)
            .bind(approved_at)
            .execute(&mut *tx)
            .await?;
            id
        } else {
            sqlx::query_scalar(
                "INSERT INTO maps (name, workshop_id, status, approved_at)
                 VALUES ($1, $2, 'approved', $3) RETURNING id",
            )
            .bind(&map.name)
            .bind(workshop_id)
            .bind(approved_at)
            .fetch_one(&mut *tx)
            .await?
        };
        sqlx::query(
            "INSERT INTO map_source_links (source_id, external_map_id, map_id, external_map_name)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (source_id, external_map_id) DO UPDATE
                SET map_id = EXCLUDED.map_id, external_map_name = EXCLUDED.external_map_name",
        )
        .bind(source_id)
        .bind(map.id.to_string())
        .bind(map_id)
        .bind(&map.name)
        .execute(&mut *tx)
        .await?;

        let mut supported = vec!["kztimer"];
        if map.sp {
            supported.push("simple");
        }
        if map.vp {
            supported.push("vanilla");
        }
        for mode_code in &supported {
            let Some(mode_id) = mode_ids.get(*mode_code).copied() else {
                continue;
            };
            sqlx::query(
                "INSERT INTO map_modes (map_id, mode_id) VALUES ($1, $2)
                 ON CONFLICT (map_id, mode_id) DO NOTHING",
            )
            .bind(map_id)
            .bind(mode_id)
            .execute(&mut *tx)
            .await?;
        }

        for stage in 0..=map.bonuses.clamp(0, 99) {
            let course_name = if stage == 0 {
                "Main".to_owned()
            } else {
                format!("Bonus {stage}")
            };
            let course_id: i64 = sqlx::query_scalar(
                "INSERT INTO map_courses (map_id, stage, name) VALUES ($1, $2, $3)
                 ON CONFLICT (map_id, stage) DO UPDATE SET name = EXCLUDED.name
                 RETURNING id",
            )
            .bind(map_id)
            .bind(stage)
            .bind(&course_name)
            .fetch_one(&mut *tx)
            .await?;

            if map.tier > 0 && map.tier <= 10 {
                for mode_code in &supported {
                    let Some(mode_id) = mode_ids.get(*mode_code).copied() else {
                        continue;
                    };
                    sqlx::query(
                        "INSERT INTO course_tiers
                            (course_id, mode_id, nub_tier, pro_tier, status)
                         VALUES ($1, $2, $3, $3, 'ranked')
                         ON CONFLICT (course_id, mode_id) DO UPDATE
                            SET nub_tier = EXCLUDED.nub_tier,
                                pro_tier = EXCLUDED.pro_tier,
                                status = 'ranked'",
                    )
                    .bind(course_id)
                    .bind(mode_id)
                    .bind(map.tier)
                    .execute(&mut *tx)
                    .await?;
                }
                if map.tier <= 8 {
                    for scoring_code in ["KZT", "SKZ", "VNL"] {
                        let Some(mode_id) = scoring_modes.get(scoring_code).copied() else {
                            continue;
                        };
                        let movement_code = match scoring_code {
                            "KZT" => "kztimer",
                            "SKZ" => "simple",
                            "VNL" => "vanilla",
                            _ => "",
                        };
                        if !supported.contains(&movement_code) {
                            continue;
                        }
                        sqlx::query(
                            "INSERT INTO course_mode_tiers
                                (map_id, course_id, mode_id, tier)
                             VALUES ($1, $2, $3, $4)
                             ON CONFLICT (map_id, course_id, mode_id) DO UPDATE
                                SET tier = EXCLUDED.tier",
                        )
                        .bind(map_id)
                        .bind(course_id)
                        .bind(mode_id)
                        .bind(map.tier)
                        .execute(&mut *tx)
                        .await?;
                    }
                }
            }
        }
        upserted += 1;
    }
    tx.commit().await?;
    Ok(upserted)
}

async fn sync_kzgo_wrs(state: &AppState) -> Result<(Value, usize, usize, usize), anyhow::Error> {
    let client = client(state)?;
    let base = state.config().kzgo_base_url.as_str();
    let mut raw = 0_usize;
    let mut created = 0_usize;
    let mut duplicate = 0_usize;
    let mut failed = 0_usize;
    let mut cursor = serde_json::Map::new();

    for (kz_mode, mode_code) in KZGO_MODES {
        let wrs: Vec<KzgoWr> = fetch_json(&client, &format!("{base}/api/wrs/{kz_mode}")).await?;
        raw += wrs.len();
        let mut max_created: Option<DateTime<Utc>> = None;
        for wr in wrs {
            let Some(recorded_at) = parse_kzgo_date(&wr.created_on) else {
                failed += 1;
                continue;
            };
            if max_created.is_none() || Some(recorded_at) > max_created {
                max_created = Some(recorded_at);
            }
            let time_type = if wr.tps == 0 { "pro" } else { "nub" };
            let external_id = format!(
                "wr:{mode_code}:{}:{time_type}:{}:{}:{}",
                wr.map_id,
                wr.steam_id64,
                recorded_at.timestamp(),
                time_ns(wr.time)
            );
            let payload = json!({
                "source": "kzgo",
                "kind": "world_record",
                "kz_mode": kz_mode,
                "map_id": wr.map_id,
                "map_name": wr.map_name,
                "player_name": wr.player_name,
                "steam_id64": wr.steam_id64,
                "server_name": wr.server_name,
                "created_on": wr.created_on,
                "time": wr.time,
                "teleports": wr.tps
            });
            match ingest(
                state,
                "kzgo",
                external_id,
                payload,
                &wr.steam_id64,
                &wr.player_name,
                &wr.map_name,
                0,
                mode_code,
                wr.time,
                wr.tps,
                Some(recorded_at),
                wr.server_name.as_deref(),
            )
            .await
            {
                Ok(true) => created += 1,
                Ok(false) => duplicate += 1,
                Err(error) => {
                    failed += 1;
                    warn!(error = %error, map = %wr.map_name, "failed to ingest kzgo WR");
                }
            }
        }
        if let Some(max_created) = max_created {
            cursor.insert(
                format!("last_wr_{mode_code}"),
                Value::String(max_created.to_rfc3339()),
            );
        }
    }
    Ok((Value::Object(cursor), raw, created, duplicate + failed))
}

async fn sync_kzgo_servers(state: &AppState, source_id: i64) -> Result<usize, anyhow::Error> {
    let client = client(state)?;
    let response: KzgoServerResponse = fetch_json(
        &client,
        &format!("{}/api/servers", state.config().kzgo_base_url),
    )
    .await?;
    let mut snapshots = Vec::new();
    for server in response.server_states {
        let error_state = server.err_before_api;
        let max_players = server.max_players_api;
        if server.kind != "PUB"
            || error_state
            || max_players <= 0
            || server.name == "Server not responding"
        {
            continue;
        }
        let external_id = format!(
            "{}:{}:{}",
            server.ip,
            server.port,
            server.label.as_deref().unwrap_or_default()
        );
        snapshots.push(ExternalServerSnapshot {
            id: 0,
            source_code: "kzgo".to_owned(),
            external_id,
            name: server.name,
            label: server.label,
            host: server.ip.clone(),
            port: server.port,
            map_name: server.map.as_ref().and_then(|map| map.name.clone()),
            map_tier: server.map.as_ref().and_then(|map| map.tier),
            online_players: server.players.len() as i32,
            max_players,
            is_online: true,
            last_seen_at: Utc::now(),
        });
    }
    let count = snapshots.len();
    repo::replace_external_servers(state.db(), source_id, &snapshots).await?;
    Ok(count)
}

pub async fn sync_kzgo(state: &AppState, run_type: &str) -> Result<SourceSyncResponse, AppError> {
    let source = repo::find_source(state.db(), "kzgo")
        .await?
        .ok_or_else(|| AppError::not_found("source"))?;
    if !source.is_enabled && run_type != "manual" {
        return Ok(SourceSyncResponse {
            source: "kzgo".to_owned(),
            run_id: None,
            status: "disabled".to_owned(),
            message: "source is disabled".to_owned(),
            stats: json!({}),
        });
    }
    let run_id = repo::start_run(state.db(), source.id, run_type).await?;
    let result = async {
        let client = client(state)?;
        let base = state.config().kzgo_base_url.as_str();
        let maps: Vec<KzgoMap> = fetch_json(&client, &format!("{base}/api/maps")).await?;
        let maps_hash = payload_hash(&serde_json::to_value(&maps)?);
        let previous_hash = source
            .cursor
            .get("maps_hash")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        let map_count = if maps.len()
            != source
                .cursor
                .get("maps_count")
                .and_then(Value::as_u64)
                .unwrap_or_default() as usize
            || maps_hash != previous_hash
        {
            sync_kzgo_maps(state, source.id, &maps).await?
        } else {
            0
        };
        let (wr_cursor, wr_raw, wr_created, wr_skipped) = sync_kzgo_wrs(state).await?;
        let server_count = sync_kzgo_servers(state, source.id).await?;
        let mut cursor = wr_cursor;
        if let Value::Object(ref mut object) = cursor {
            object.insert("maps_hash".to_owned(), json!(maps_hash));
            object.insert("maps_count".to_owned(), json!(maps.len()));
        }
        Ok::<Value, anyhow::Error>(json!({
            "maps_seen": maps.len(),
            "maps_upserted": map_count,
            "wrs_seen": wr_raw,
            "wrs_created": wr_created,
            "wrs_skipped": wr_skipped,
            "servers_seen": server_count,
            "cursor": cursor
        }))
    }
    .await;

    match result {
        Ok(stats) => {
            repo::finish_run_success(state.db(), source.id, run_id, &stats).await?;
            info!(?stats, "kzgo sync completed");
            Ok(SourceSyncResponse {
                source: "kzgo".to_owned(),
                run_id: Some(run_id),
                status: "success".to_owned(),
                message: "kzgo sync completed".to_owned(),
                stats,
            })
        }
        Err(error) => {
            let message = format!("{error:#}");
            repo::finish_run_failure(state.db(), source.id, run_id, &message, &json!({})).await?;
            warn!(error = %message, "kzgo sync failed");
            Err(AppError::internal(anyhow::anyhow!(message)))
        }
    }
}

pub async fn sync_gokz_global(
    state: &AppState,
    run_type: &str,
) -> Result<SourceSyncResponse, AppError> {
    let source = repo::find_source(state.db(), "gokz_global")
        .await?
        .ok_or_else(|| AppError::not_found("source"))?;
    if !source.is_enabled && run_type != "manual" {
        return Ok(SourceSyncResponse {
            source: "gokz_global".to_owned(),
            run_id: None,
            status: "disabled".to_owned(),
            message: "source is disabled".to_owned(),
            stats: json!({}),
        });
    }
    let run_id = repo::start_run(state.db(), source.id, run_type).await?;
    let result = async {
        let client = client(state)?;
        let base = state.config().gokz_global_base_url.as_str();
        let created_since = source
            .cursor
            .get("last_created_on")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let mut seen = 0_usize;
        let mut created = 0_usize;
        let mut skipped = 0_usize;
        let mut max_created: Option<String> = created_since.clone();
        for (api_mode, mode_code) in GLOBALAPI_MODES {
            let mut url = format!(
                "{base}/records/top/recent?tickrate=128&limit=500&modes_list_string={api_mode}"
            );
            if let Some(since) = created_since.as_deref() {
                url.push_str("&created_since=");
                url.push_str(&urlencoding(since));
            }
            let records: Vec<GokzGlobalRecord> = match fetch_json(&client, &url).await {
                Ok(records) => records,
                Err(error) => {
                    warn!(error = %error, url = %url, "GlobalAPI recent records request failed");
                    return Err(error);
                }
            };
            seen += records.len();
            for record in records {
                let Some(steam_id64) = record
                    .steamid64
                    .clone()
                    .or_else(|| record.steam_id.as_deref().and_then(steamid64_from_steam2))
                else {
                    skipped += 1;
                    continue;
                };
                let Some(map_name) = record.map_name.clone() else {
                    skipped += 1;
                    continue;
                };
                let Some(time) = record.time else {
                    skipped += 1;
                    continue;
                };
                let Some(created_on) = record.created_on.clone() else {
                    skipped += 1;
                    continue;
                };
                let Some(recorded_at) = parse_global_date(&created_on) else {
                    skipped += 1;
                    continue;
                };
                let record_mode = record
                    .mode
                    .as_deref()
                    .and_then(normalize_api_mode)
                    .unwrap_or(mode_code);
                let external_id = record
                    .id
                    .as_ref()
                    .map(|value| match value {
                        Value::String(value) => format!("record:{value}"),
                        other => format!("record:{other}"),
                    })
                    .unwrap_or_else(|| {
                        format!(
                            "record:{}:{}:{}:{}:{}",
                            map_name,
                            record.stage.unwrap_or(0),
                            steam_id64,
                            time_ns(time),
                            recorded_at.timestamp()
                        )
                    });
                let payload = serde_json::to_value(&record)?;
                match ingest(
                    state,
                    "gokz_global",
                    external_id,
                    payload,
                    &steam_id64,
                    record.player_name.as_deref().unwrap_or("Unknown player"),
                    &map_name,
                    record.stage.unwrap_or(0),
                    record_mode,
                    time,
                    record.teleports.unwrap_or(0),
                    Some(recorded_at),
                    record.server_name.as_deref(),
                )
                .await
                {
                    Ok(true) => created += 1,
                    Ok(false) => skipped += 1,
                    Err(error) => {
                        skipped += 1;
                        warn!(error = %error, map = %map_name, "failed to ingest GlobalAPI record");
                    }
                }
                if max_created
                    .as_deref()
                    .map(|value| created_on.as_str() > value)
                    .unwrap_or(true)
                {
                    max_created = Some(created_on);
                }
            }
        }
        Ok::<Value, anyhow::Error>(json!({
            "records_seen": seen,
            "records_created": created,
            "records_skipped": skipped,
            "cursor": {"last_created_on": max_created}
        }))
    }
    .await;

    match result {
        Ok(stats) => {
            repo::finish_run_success(state.db(), source.id, run_id, &stats).await?;
            info!(?stats, "gokz-global sync completed");
            Ok(SourceSyncResponse {
                source: "gokz_global".to_owned(),
                run_id: Some(run_id),
                status: "success".to_owned(),
                message: "gokz-global sync completed".to_owned(),
                stats,
            })
        }
        Err(error) => {
            let message = format!("{error:#}");
            repo::finish_run_failure(state.db(), source.id, run_id, &message, &json!({})).await?;
            warn!(error = %message, "gokz-global sync failed");
            Err(AppError::internal(anyhow::anyhow!(message)))
        }
    }
}

pub async fn sync_source(
    state: &AppState,
    code: &str,
    run_type: &str,
) -> Result<SourceSyncResponse, AppError> {
    match code {
        "kzgo" => sync_kzgo(state, run_type).await,
        "gokz_global" => sync_gokz_global(state, run_type).await,
        "cngokz_plugin" => Ok(SourceSyncResponse {
            source: code.to_owned(),
            run_id: None,
            status: "push".to_owned(),
            message: "plugin source is push-only; no polling endpoint".to_owned(),
            stats: json!({}),
        }),
        _ => Err(AppError::not_found("source")),
    }
}

pub async fn run_worker(state: AppState, mut shutdown: tokio::sync::watch::Receiver<bool>) {
    let interval = state.config().source_sync_interval;
    info!(
        interval_secs = interval.as_secs(),
        "source sync worker started"
    );
    loop {
        if *shutdown.borrow() {
            break;
        }
        if state.config().kzgo_sync_enabled {
            let _ = sync_kzgo(&state, "incremental").await;
        }
        if state.config().gokz_global_sync_enabled {
            let _ = sync_gokz_global(&state, "incremental").await;
        }
        tokio::select! {
            _ = tokio::time::sleep(interval) => {}
            _ = shutdown.changed() => break,
        }
    }
}

fn normalize_api_mode(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "kz_timer" | "kztimer" | "kzt" => Some("kztimer"),
        "kz_simple" | "simple" | "skz" => Some("simple"),
        "kz_vanilla" | "vanilla" | "vnl" => Some("vanilla"),
        _ => None,
    }
}

fn steamid64_from_steam2(value: &str) -> Option<String> {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 3 || !parts[0].starts_with("STEAM_") {
        return None;
    }
    let y: u64 = parts[1].parse().ok()?;
    let z: u64 = parts[2].parse().ok()?;
    Some((z * 2 + y + 76_561_197_960_265_728).to_string())
}

fn urlencoding(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                output.push(char::from(byte));
            }
            b' ' => output.push_str("%20"),
            b':' => output.push_str("%3A"),
            b'+' => output.push_str("%2B"),
            other => output.push_str(&format!("%{other:02X}")),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_kzgo_timestamp_without_timezone() {
        let value = parse_kzgo_date("2021-11-18T21:11:34").expect("timestamp");
        assert_eq!(value.to_rfc3339(), "2021-11-18T21:11:34+00:00");
    }

    #[test]
    fn converts_real_steam2_sample_to_steamid64() {
        assert_eq!(
            steamid64_from_steam2("STEAM_1:0:182656610").as_deref(),
            Some("76561198325578948")
        );
    }

    #[test]
    fn normalizes_global_api_mode_names() {
        assert_eq!(normalize_api_mode("kz_timer"), Some("kztimer"));
        assert_eq!(normalize_api_mode("kz_simple"), Some("simple"));
        assert_eq!(normalize_api_mode("kz_vanilla"), Some("vanilla"));
        assert_eq!(normalize_api_mode("unknown"), None);
    }
}
