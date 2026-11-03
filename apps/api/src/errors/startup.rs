//! Errors raised while bootstrapping the process (before the HTTP server runs).

use std::net::SocketAddr;

use thiserror::Error;

use crate::config::ConfigError;

#[derive(Debug, Error)]
pub enum StartupError {
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("failed to connect to PostgreSQL")]
    Database(#[source] sqlx::Error),

    #[error("failed to run database migrations")]
    Migration(#[source] sqlx::migrate::MigrateError),

    #[error("invalid Redis configuration or connection failure")]
    Redis(#[source] redis::RedisError),

    #[error("failed to bind {addr}")]
    Bind {
        addr: SocketAddr,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to serve HTTP: {0}")]
    Serve(#[source] std::io::Error),
}
