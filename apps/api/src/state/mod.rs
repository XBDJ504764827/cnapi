//! Shared application state handed to every handler.

pub mod connections;
pub mod redis_client;

use std::{fmt, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub use connections::connect;
pub use redis_client::RedisClient;

use crate::config::AppConfig;

/// Cheap to clone (`Arc` inside) and shared by every request.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    config: AppConfig,
    db: PgPool,
    redis: RedisClient,
    started_at: DateTime<Utc>,
}

impl AppState {
    pub fn new(config: AppConfig, db: PgPool, redis: RedisClient) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                config,
                db,
                redis,
                started_at: Utc::now(),
            }),
        }
    }

    pub fn config(&self) -> &AppConfig {
        &self.inner.config
    }

    pub fn db(&self) -> &PgPool {
        &self.inner.db
    }

    pub fn redis(&self) -> &RedisClient {
        &self.inner.redis
    }

    pub fn started_at(&self) -> DateTime<Utc> {
        self.inner.started_at
    }

    pub fn uptime(&self) -> Duration {
        let elapsed = Utc::now().signed_duration_since(self.inner.started_at);
        elapsed.to_std().unwrap_or(Duration::ZERO)
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.uptime().as_secs()
    }
}

impl fmt::Debug for AppState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppState")
            .field("config", &self.inner.config)
            .field("started_at", &self.inner.started_at)
            .finish_non_exhaustive()
    }
}
