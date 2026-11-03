//! Redis client management.
//!
//! A single [`RedisClient`] is created at startup and cloned per request. The
//! multiplexed connection is created lazily on first use and re-created once if
//! it breaks, so a temporary Redis outage does not require an API restart.

use std::{fmt, sync::Arc};

use redis::{Client, aio::ConnectionManager};
use tokio::sync::Mutex;
use tracing::warn;

#[derive(Clone)]
pub struct RedisClient {
    client: Client,
    connection: Arc<Mutex<Option<ConnectionManager>>>,
    key_prefix: Arc<str>,
}

impl RedisClient {
    /// Parses the URL but does not open a connection yet.
    pub fn new(url: &str, key_prefix: impl Into<String>) -> Result<Self, redis::RedisError> {
        let client = Client::open(url)?;
        Ok(Self {
            client,
            connection: Arc::new(Mutex::new(None)),
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

    /// Drops the cached connection, forcing a reconnect on the next command.
    pub async fn reset(&self) {
        self.invalidate().await;
    }

    /// Currently cached connection, if any. Used by diagnostics.
    pub async fn is_connected(&self) -> bool {
        self.connection.lock().await.is_some()
    }

    async fn connection(&self) -> Result<ConnectionManager, redis::RedisError> {
        let mut guard = self.connection.lock().await;
        if let Some(manager) = guard.as_ref() {
            return Ok(manager.clone());
        }
        let manager = ConnectionManager::new(self.client.clone()).await?;
        *guard = Some(manager.clone());
        Ok(manager)
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
