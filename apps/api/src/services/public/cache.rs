//! Redis cache for public read endpoints.
//!
//! Design:
//! - Keys are namespaced with a version counter (`public:version`); one `INCR`
//!   invalidates every public cache entry without `SCAN`/`DEL`.
//! - Key suffixes are hashed to a fixed length so arbitrary user input cannot
//!   create unbounded, long Redis keys.
//! - TTLs are jittered to avoid synchronized expiry (cache avalanche).
//! - Concurrent misses for the same key share a single PostgreSQL load
//!   (in-process singleflight), reducing cache stampedes.
//! - Cache failures never fail a request: the loader runs against PostgreSQL.
//!
//! See `docs/public-portal/api-requirements.md` for TTL policy.

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
    time::Duration,
};

use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::Mutex as AsyncMutex;
use tracing::warn;

use crate::errors::AppError;
use crate::state::{AppState, RedisClient};

const VERSION_KEY: &str = "public:version";
const MODE_VERSION_PREFIX: &str = "points:mode";

/// Creates the version counter at startup with `SET NX` so the first explicit
/// invalidation is observable from `"1"` to `"2"`. Safe on every replica.
pub async fn initialize(state: &AppState) {
    let key = state.redis().key(VERSION_KEY);
    if let Err(error) = state.redis().set_nx(&key, "1").await {
        warn!(error = %error, key, "failed to initialize public cache namespace");
    }
}

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

/// Per-key async locks used to coalesce concurrent cache misses. Entries are
/// kept bounded; the locks are cheap and do not hold cached values.
#[derive(Default)]
struct FlightMap {
    locks: Mutex<HashMap<String, Arc<AsyncMutex<()>>>>,
}

impl FlightMap {
    fn lock_for(&self, key: &str) -> Arc<AsyncMutex<()>> {
        let mut locks = self
            .locks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if locks.len() > 1024 {
            locks.retain(|_, lock| Arc::strong_count(lock) > 1);
        }
        locks
            .entry(key.to_owned())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    }
}

static FLIGHTS: std::sync::OnceLock<FlightMap> = std::sync::OnceLock::new();

fn flights() -> &'static FlightMap {
    FLIGHTS.get_or_init(FlightMap::default)
}

/// Stable 64-bit FNV-1a hash. Used only for key shortening, not security.
fn stable_hash(value: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Deterministic TTL jitter (0-15%) derived from the key. Avoids all keys with
/// the same base TTL being populated and expiring simultaneously.
fn jittered_ttl(ttl: Duration, key: &str) -> u64 {
    let seconds = ttl.as_secs().max(1);
    let jitter = stable_hash(key) % (seconds.max(1) * 15 / 100 + 1);
    seconds + jitter
}

fn short_key(key_suffix: &str) -> String {
    // Keep a readable prefix for debugging, hash the variable-length tail.
    let prefix = key_suffix.split(':').next().unwrap_or("entry");
    format!("{prefix}:{:016x}", stable_hash(key_suffix))
}

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
///
/// On a miss, only one request per key executes the loader; other requests wait
/// for the lock and then re-read Redis. If the first loader fails or Redis is
/// unavailable, the waiting request still runs its own loader so the endpoint
/// degrades to PostgreSQL rather than failing.
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
        .key(&format!("public:v{version}:{}", short_key(key_suffix)));

    if let Ok(Some(raw)) = state.redis().get(&key).await {
        match serde_json::from_str::<T>(&raw) {
            Ok(value) => return Ok(value),
            Err(error) => {
                warn!(error = %error, key, "invalid public cache entry; reloading");
            }
        }
    }

    let guard = flights().lock_for(&key);
    let _flight = guard.lock().await;

    // Another request may have populated the cache while we waited.
    if let Ok(Some(raw)) = state.redis().get(&key).await
        && let Ok(value) = serde_json::from_str::<T>(&raw)
    {
        return Ok(value);
    }

    let value = load().await?;

    if let Ok(raw) = serde_json::to_string(&value)
        && let Err(error) = state
            .redis()
            .set_ex(&key, &raw, jittered_ttl(ttl, &key))
            .await
    {
        warn!(error = %error, key, "failed to write public cache entry");
    }

    Ok(value)
}

/// Invalidates all public caches by bumping the namespace version.
///
/// Phase 5 write endpoints (record review, ingestion, server updates) and the
/// protected maintenance endpoint call this after committing changes.
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

/// Warms a single loader into the cache. Used by the protected maintenance
/// endpoint for hot data after a deployment or cache invalidation.
pub async fn warm<T, F, Fut>(
    state: &AppState,
    key_suffix: &str,
    ttl: Duration,
    load: F,
) -> Result<(), AppError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
{
    get_or_load(state, key_suffix, ttl, load).await.map(|_| ())
}

pub async fn cache_version_value(state: &AppState) -> String {
    cache_version(state).await
}

/// Invalidate only one scoring mode's points cache namespace. A KZT recalc
/// must never clear SKZ or VNL points data.
pub async fn invalidate_mode(redis: &RedisClient, mode_id: i16) -> Result<(), redis::RedisError> {
    let key = redis.key(&format!("{MODE_VERSION_PREFIX}:{mode_id}:version"));
    redis.incr(&key).await.map(|_| ())
}

pub async fn mode_version(redis: &RedisClient, mode_id: i16) -> String {
    let key = redis.key(&format!("{MODE_VERSION_PREFIX}:{mode_id}:version"));
    match redis.get(&key).await {
        Ok(Some(value)) => value,
        _ => "1".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_short_and_stable() {
        let first = short_key("search:very-long-user-input-with-special-characters-一二三");
        let second = short_key("search:very-long-user-input-with-special-characters-一二三");
        assert_eq!(first, second);
        assert!(first.len() < 40, "unexpected key length: {first}");
        assert!(!first.contains("special"));
        assert_ne!(first, short_key("search:another-input"));
    }

    #[test]
    fn jitter_is_bounded() {
        let ttl = Duration::from_secs(30);
        for index in 0..50 {
            let value = jittered_ttl(ttl, &format!("key-{index}"));
            assert!(
                (30..=34).contains(&value),
                "ttl out of expected range: {value}"
            );
        }
    }
}
