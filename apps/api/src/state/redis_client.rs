//! Redis client management.
//!
//! A single [`RedisClient`] is created at startup and cloned per request. The
//! multiplexed connection is created lazily on first use and re-created once if
//! it breaks, so a temporary Redis outage does not require an API restart.

use std::{
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};

use redis::{Client, ErrorKind, aio::ConnectionManager, aio::ConnectionManagerConfig};
use tokio::sync::Mutex;
use tracing::warn;

#[derive(Clone)]
pub struct RedisClient {
    client: Client,
    connection: Arc<Mutex<Option<ConnectionManager>>>,
    /// Fast-fail window after a connection failure, so a Redis outage does not
    /// add a connection timeout to every request.
    failed_until: Arc<Mutex<Option<Instant>>>,
    key_prefix: Arc<str>,
}

impl RedisClient {
    /// Parses the URL but does not open a connection yet.
    pub fn new(url: &str, key_prefix: impl Into<String>) -> Result<Self, redis::RedisError> {
        let client = Client::open(url)?;
        Ok(Self {
            client,
            connection: Arc::new(Mutex::new(None)),
            failed_until: Arc::new(Mutex::new(None)),
            key_prefix: Arc::from(key_prefix.into()),
        })
    }

    pub fn key_prefix(&self) -> &str {
        &self.key_prefix
    }

    /// Builds a namespaced key, e.g. `cngokz:health`.
    pub fn key(&self, suffix: &str) -> String {
        format!("{}:{}", self.key_prefix, suffix)
    }

    /// Issues a `PING`. The cached connection is dropped when the command fails
    /// so the next call reconnects.
    pub async fn ping(&self) -> Result<(), redis::RedisError> {
        let mut connection = self.connection().await?;
        match redis::cmd("PING")
            .query_async::<String>(&mut connection)
            .await
        {
            Ok(_) => Ok(()),
            Err(error) => {
                warn!(error = %error, "redis PING failed; dropping pooled connection");
                self.invalidate().await;
                Err(error)
            }
        }
    }

    /// Drops the cached connection and the failure window, forcing a reconnect
    /// on the next command.
    pub async fn reset(&self) {
        self.invalidate().await;
        *self.failed_until.lock().await = None;
    }

    /// Reads a UTF-8 string value.
    pub async fn get(&self, key: &str) -> Result<Option<String>, redis::RedisError> {
        let mut connection = self.connection().await?;
        match redis::cmd("GET")
            .arg(key)
            .query_async::<Option<String>>(&mut connection)
            .await
        {
            Ok(value) => Ok(value),
            Err(error) => {
                warn!(error = %error, key, "redis GET failed; dropping pooled connection");
                self.invalidate().await;
                Err(error)
            }
        }
    }

    /// Writes a UTF-8 string value with a TTL (seconds).
    pub async fn set_ex(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: u64,
    ) -> Result<(), redis::RedisError> {
        let mut connection = self.connection().await?;
        match redis::cmd("SET")
            .arg(key)
            .arg(value)
            .arg("EX")
            .arg(ttl_seconds)
            .query_async::<()>(&mut connection)
            .await
        {
            Ok(()) => Ok(()),
            Err(error) => {
                warn!(error = %error, key, "redis SETEX failed; dropping pooled connection");
                self.invalidate().await;
                Err(error)
            }
        }
    }

    /// Atomically increments a counter and returns the new value.
    pub async fn incr(&self, key: &str) -> Result<i64, redis::RedisError> {
        let mut connection = self.connection().await?;
        match redis::cmd("INCR")
            .arg(key)
            .query_async::<i64>(&mut connection)
            .await
        {
            Ok(value) => Ok(value),
            Err(error) => {
                warn!(error = %error, key, "redis INCR failed; dropping pooled connection");
                self.invalidate().await;
                Err(error)
            }
        }
    }

    /// Currently cached connection, if any. Used by diagnostics.
    pub async fn is_connected(&self) -> bool {
        self.connection.lock().await.is_some()
    }

    async fn connection(&self) -> Result<ConnectionManager, redis::RedisError> {
        {
            let failed_until = self.failed_until.lock().await;
            if let Some(until) = *failed_until
                && until > Instant::now()
            {
                return Err(redis::RedisError::from((
                    ErrorKind::Io,
                    "redis recently unavailable",
                )));
            }
        }

        let mut guard = self.connection.lock().await;
        if let Some(manager) = guard.as_ref() {
            return Ok(manager.clone());
        }

        let config = ConnectionManagerConfig::new()
            .set_number_of_retries(0)
            .set_connection_timeout(Some(Duration::from_millis(300)))
            .set_response_timeout(Some(Duration::from_millis(300)));

        match ConnectionManager::new_with_config(self.client.clone(), config).await {
            Ok(manager) => {
                *guard = Some(manager.clone());
                *self.failed_until.lock().await = None;
                Ok(manager)
            }
            Err(error) => {
                *self.failed_until.lock().await = Some(Instant::now() + Duration::from_secs(5));
                Err(error)
            }
        }
    }

    async fn invalidate(&self) {
        *self.connection.lock().await = None;
    }
}

/// Prevents credentials from being printed through `Debug`.
impl fmt::Debug for RedisClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RedisClient")
            .field("key_prefix", &self.key_prefix)
            .finish_non_exhaustive()
    }
}
