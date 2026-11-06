//! 后台批量拉取 Steam 玩家资料（昵称 + 头像）。
//!
//! ## 为什么走中转 Worker
//!
//! 生产服务器访问不到 `api.steampowered.com`（实测超时），而 `GET /profiles`
//! 由中转 Worker 代为请求，Worker 侧持有 `STEAM_API_KEY`，一次最多 100 个
//! SteamID，返回 persona 与三档头像。本地开发能直连时退回官方 Web API。
//!
//! ## 为什么是后台任务
//!
//! 头像属于「迟早会补齐」的数据，不应该让页面请求等待 Steam。worker 每
//! `AVATAR_SYNC_INTERVAL_SECS` 跑一轮，每轮最多 `AVATAR_SYNC_BATCHES × 100` 个
//! 玩家，优先处理从未同步过的账号，已经同步过的每 `AVATAR_REFRESH_DAYS` 天刷新一次。

use std::collections::HashMap;

use serde::Deserialize;
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::errors::AppError;
use crate::repositories::{auth as auth_repo, steam_profiles as repo};
use crate::services::auth::http_client;
use crate::state::AppState;

/// Steam `GetPlayerSummaries` 与中转 `/profiles` 的公共上限。
pub const BATCH_SIZE: usize = 100;

#[derive(Debug, Clone)]
pub struct SteamProfileData {
    pub steam_id64: String,
    pub persona_name: Option<String>,
    pub avatar_url: Option<String>,
}

/// 当前是否具备拉取资料的条件。
pub fn is_configured(config: &AppConfig) -> bool {
    config.steam_relay_url.is_some() || config.steam_api_key.is_some()
}

/// 批量拉取。中转优先，其次直连 Steam Web API。
pub async fn fetch_profiles(
    state: &AppState,
    steam_ids: &[String],
) -> Result<Vec<SteamProfileData>, AppError> {
    if steam_ids.is_empty() {
        return Ok(Vec::new());
    }
    let config = state.config();
    if config.steam_relay_url.is_some() {
        fetch_from_relay(config, steam_ids).await
    } else if config.steam_api_key.is_some() {
        fetch_from_steam_api(config, steam_ids).await
    } else {
        Err(AppError::internal(anyhow::anyhow!(
            "neither STEAM_RELAY_URL nor STEAM_API_KEY is configured"
        )))
    }
}

#[derive(Deserialize)]
struct RelayProfilesResponse {
    #[serde(default)]
    success: bool,
    #[serde(default)]
    players: Vec<RelayPlayer>,
}

#[derive(Deserialize)]
struct RelayPlayer {
    steamid: String,
    #[serde(default)]
    persona_name: Option<String>,
    #[serde(default)]
    avatar: Option<String>,
    #[serde(default)]
    avatar_medium: Option<String>,
    #[serde(default)]
    avatar_full: Option<String>,
}

async fn fetch_from_relay(
    config: &AppConfig,
    steam_ids: &[String],
) -> Result<Vec<SteamProfileData>, AppError> {
    let relay = config
        .steam_relay_url
        .as_deref()
        .expect("checked by caller");
    let mut url = reqwest::Url::parse(&format!("{relay}/profiles")).map_err(|error| {
        AppError::internal(anyhow::anyhow!(
            "STEAM_RELAY_URL is not a valid URL: {error}"
        ))
    })?;
    url.query_pairs_mut()
        .append_pair("steamids", &steam_ids.join(","));

    let response = http_client().get(url).send().await.map_err(|error| {
        warn!(error = ?error, "relay profile fetch failed");
        AppError::DependencyUnavailable {
            dependency: "steam_relay",
        }
    })?;

    if !response.status().is_success() {
        warn!(
            status = response.status().as_u16(),
            "relay profile fetch rejected"
        );
        return Err(AppError::DependencyUnavailable {
            dependency: "steam_relay",
        });
    }

    let body: RelayProfilesResponse = response.json().await.map_err(|error| {
        warn!(error = ?error, "relay profile response was not JSON");
        AppError::DependencyUnavailable {
            dependency: "steam_relay",
        }
    })?;

    if !body.success {
        return Err(AppError::DependencyUnavailable {
            dependency: "steam_relay",
        });
    }

    Ok(body
        .players
        .into_iter()
        .filter_map(|player| {
            if !is_steam_id64(&player.steamid) {
                return None;
            }
            let avatar_url = [player.avatar_medium, player.avatar_full, player.avatar]
                .into_iter()
                .flatten()
                .find(|url| is_avatar_url(url));
            Some(SteamProfileData {
                steam_id64: player.steamid,
                persona_name: sanitize_name(player.persona_name),
                avatar_url,
            })
        })
        .collect())
}

#[derive(Deserialize)]
struct SteamSummariesResponse {
    #[serde(default)]
    response: SteamSummaries,
}

#[derive(Deserialize, Default)]
struct SteamSummaries {
    #[serde(default)]
    players: Vec<SteamSummaryPlayer>,
}

#[derive(Deserialize)]
struct SteamSummaryPlayer {
    steamid: String,
    #[serde(default)]
    personaname: Option<String>,
    #[serde(default)]
    avatarmedium: Option<String>,
    #[serde(default)]
    avatarfull: Option<String>,
    #[serde(default)]
    avatar: Option<String>,
}

async fn fetch_from_steam_api(
    config: &AppConfig,
    steam_ids: &[String],
) -> Result<Vec<SteamProfileData>, AppError> {
    let key = config.steam_api_key.as_deref().expect("checked by caller");
    let mut url = reqwest::Url::parse(&format!(
        "{}/ISteamUser/GetPlayerSummaries/v0002/",
        config.steam_api_base_url
    ))
    .map_err(|error| AppError::internal(anyhow::anyhow!("invalid STEAM_API_BASE_URL: {error}")))?;
    url.query_pairs_mut()
        .append_pair("key", key)
        .append_pair("steamids", &steam_ids.join(","));

    let response = http_client().get(url).send().await.map_err(|error| {
        warn!(error = ?error, "steam profile fetch failed");
        AppError::DependencyUnavailable {
            dependency: "steam_api",
        }
    })?;

    if !response.status().is_success() {
        return Err(AppError::DependencyUnavailable {
            dependency: "steam_api",
        });
    }

    let body: SteamSummariesResponse = response.json().await.map_err(|error| {
        warn!(error = ?error, "steam profile response was not JSON");
        AppError::DependencyUnavailable {
            dependency: "steam_api",
        }
    })?;

    Ok(body
        .response
        .players
        .into_iter()
        .map(|player| SteamProfileData {
            avatar_url: [player.avatarmedium, player.avatarfull, player.avatar]
                .into_iter()
                .flatten()
                .find(|url| is_avatar_url(url)),
            persona_name: sanitize_name(player.personaname),
            steam_id64: player.steamid,
        })
        .collect())
}

fn is_steam_id64(value: &str) -> bool {
    (17..=20).contains(&value.len())
        && value.chars().all(|c| c.is_ascii_digit())
        && value.chars().any(|c| c != '0')
}

fn is_avatar_url(value: &str) -> bool {
    value.starts_with("https://") && value.len() <= 512
}

fn sanitize_name(value: Option<String>) -> Option<String> {
    value.and_then(|name| {
        let name = name.trim().chars().take(64).collect::<String>();
        (!name.is_empty()).then_some(name)
    })
}

/// 跑一轮同步，返回本轮处理的账号数。
pub async fn sync_once(state: &AppState) -> Result<usize, AppError> {
    let config = state.config();
    let mut handled = 0usize;

    for batch in 0..config.avatar_sync_batches {
        let pending =
            repo::pending(state.db(), BATCH_SIZE as i64, config.avatar_refresh_days).await?;
        if pending.is_empty() {
            break;
        }

        let ids: Vec<String> = pending.iter().map(|(_, id)| id.clone()).collect();
        let fetched = fetch_profiles(state, &ids).await?;
        let by_id: HashMap<&str, &SteamProfileData> = fetched
            .iter()
            .map(|profile| (profile.steam_id64.as_str(), profile))
            .collect();

        for (account_id, steam_id) in &pending {
            match by_id.get(steam_id.as_str()) {
                Some(profile) => {
                    repo::store(
                        state.db(),
                        *account_id,
                        &auth_repo::fallback_display_name(steam_id),
                        profile.persona_name.as_deref(),
                        profile.avatar_url.as_deref(),
                    )
                    .await?;
                }
                // Steam 不认识这个 SteamID：记录时间戳，避免每轮重试同一批。
                None => repo::mark_synced(state.db(), *account_id).await?,
            }
            handled += 1;
        }

        if batch + 1 < config.avatar_sync_batches {
            info!(
                batch = batch + 1,
                count = pending.len(),
                "avatar batch synced"
            );
        }
    }

    Ok(handled)
}

pub async fn run_worker(state: AppState, mut shutdown: tokio::sync::watch::Receiver<bool>) {
    if !state.config().avatar_sync_enabled {
        info!("avatar sync disabled");
        return;
    }
    if !is_configured(state.config()) {
        warn!("avatar sync skipped: neither STEAM_RELAY_URL nor STEAM_API_KEY is configured");
        return;
    }

    let interval = state.config().avatar_sync_interval;
    info!(
        interval_secs = interval.as_secs(),
        "avatar sync worker started"
    );

    loop {
        if *shutdown.borrow() {
            break;
        }
        match sync_once(&state).await {
            Ok(0) => {}
            Ok(count) => info!(count, "avatar sync completed"),
            Err(error) => warn!(error = ?error, "avatar sync failed"),
        }
        tokio::select! {
            _ = tokio::time::sleep(interval) => {}
            _ = shutdown.changed() => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avatar_urls_must_be_https_and_bounded() {
        assert!(is_avatar_url("https://avatars.steamstatic.com/abc.jpg"));
        assert!(!is_avatar_url("http://avatars.steamstatic.com/abc.jpg"));
        assert!(!is_avatar_url("javascript:alert(1)"));
        assert!(!is_avatar_url(&format!("https://x/{}", "a".repeat(600))));
    }

    #[test]
    fn persona_names_are_trimmed_and_bounded() {
        assert_eq!(
            sanitize_name(Some("  Runner  ".into())),
            Some("Runner".into())
        );
        assert_eq!(sanitize_name(Some("   ".into())), None);
        assert_eq!(sanitize_name(None), None);
        assert_eq!(
            sanitize_name(Some("a".repeat(120))).map(|n| n.len()),
            Some(64)
        );
    }
}
