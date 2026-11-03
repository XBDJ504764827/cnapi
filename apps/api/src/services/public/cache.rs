//! Redis cache for public read endpoints.
//!
//! Design:
//! - Cache keys are namespaced with a version counter (`public:version`), so a
//!   single `INCR` invalidates every public cache entry without SCAN/DEL.
//! - Cache failures never fail a request: the loader runs against PostgreSQL.
//! - TTLs are deliberately short; see `docs/public-portal/api-requirements.md`.

use std::time::Duration;

use serde::{Serialize, de::DeserializeOwned};
use tracing::warn;

use crate::errors::AppError;
use crate::state::AppState;

const VERSION_KEY: &str = "public:version";

pub const TTL_STATISTICS: Duration = Duration::from_secs(30);
pub const TTL_MODES: Duration = Duration::from_secs(300);
pub const TTL_LEADERBOARD: Duration = Duration::from_secs(60);
pub const TTL_PLAYER_LIST: Duration = Duration::from_secs(30);
pub const TTL_PLAYER_DETAIL: Duration = Duration::from_secs(30);
pub const TTL_RECORD_LIST: Duration = Duration::from_secs(20);
pub const TTL_RECENT_RECORDS: Duration = Duration::from_secs(15);
pub const TTL_WORLD_RECORDS: Duration = Duration::from_secs(30);
pub const TTL_ACTIVE_PLAYERS: Duration = Duration::from_secs(60);
pub const TTL_MAP_LIST: Duration = Duration::from_secs(60);
pub const TTL_MAP_DETAIL: Duration = Duration::from_secs(120);
pub const TTL_SERVER_LIST: Duration = Duration::from_secs(15);
pub const TTL_SEARCH: Duration = Duration::from_secs(30);

async fn cache_version(state: &AppState) -> String {
    match state.redis().get(&state.redis().key(VERSION_KEY)).await {
        Ok(Some(version)) => version,
        Ok(None) => "1".to_owned(),
        Err(error) => {
            warn!(error = %error, "public cache version lookup failed");
            "1".to_owned()
        }
    }
}

/// Returns the cached JSON value or runs `load` and stores the result.
pub async fn get_or_load<T, F, Fut>(
    state: &AppState,
    key_suffix: &str,
    ttl: Duration,
    load: F,
) -> Result<T, AppError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
{
    let version = cache_version(state).await;
    let key = state
        .redis()
        .key(&format!("public:v{version}:{key_suffix}"));

    if let Ok(Some(raw)) = state.redis().get(&key).await {
        match serde_json::from_str::<T>(&raw) {
            Ok(value) => return Ok(value),
            Err(error) => {
                warn!(error = %error, key, "invalid public cache entry; reloading");
            }
        }
    }

    let value = load().await?;

    if let Ok(raw) = serde_json::to_string(&value)
        && let Err(error) = state.redis().set_ex(&key, &raw, ttl.as_secs().max(1)).await
    {
        warn!(error = %error, key, "failed to write public cache entry");
    }

    Ok(value)
}

/// Invalidates all public caches by bumping the namespace version.
///
/// Phase 5 write endpoints (record review, ingestion, server updates) must call
/// this after committing changes.
pub async fn invalidate_all(state: &AppState) -> Result<(), AppError> {
    state
        .redis()
        .incr(&state.redis().key(VERSION_KEY))
        .await
        .map(|_| ())
        .map_err(|error| {
            warn!(error = %error, "failed to invalidate public cache");
            AppError::Redis(error)
        })
}

/// Warms nothing today; exposed for post-deploy cache priming in phase 5.
pub async fn cache_version_value(state: &AppState) -> String {
    cache_version(state).await
}
