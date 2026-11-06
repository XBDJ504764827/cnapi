//! Startup wiring for PostgreSQL and Redis.

use sqlx::postgres::PgPoolOptions;
use tokio::time::timeout;
use tracing::{info, warn};

use super::{AppState, RedisClient};
use crate::config::AppConfig;
use crate::errors::StartupError;

/// Connects to PostgreSQL, runs pending migrations, prepares the Redis client
/// and bundles everything into [`AppState`].
///
/// PostgreSQL is required: without it the service cannot answer any real query,
/// so startup fails fast. Redis is prepared lazily; if it is unreachable the API
/// still starts and `/health/ready` reports the dependency as down.
pub async fn connect(config: &AppConfig) -> Result<AppState, StartupError> {
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .min_connections(config.database_min_connections)
        .acquire_timeout(config.database_acquire_timeout)
        .connect(&config.database_url)
        .await
        .map_err(StartupError::Database)?;

    info!(
        max_connections = config.database_max_connections,
        min_connections = config.database_min_connections,
        "postgresql connection pool ready"
    );

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(StartupError::Migration)?;
    info!("database migrations applied");

    let redis = RedisClient::new(&config.redis_url, config.redis_key_prefix.clone())
        .map_err(StartupError::Redis)?;

    // Best effort probe: report the state without blocking startup.
    match timeout(config.health_check_timeout, redis.ping()).await {
        Ok(Ok(())) => info!("redis connection ready"),
        Ok(Err(error)) => {
            warn!(error = %error, "redis is not reachable at startup; /health/ready will report it as down")
        }
        Err(_) => warn!("redis probe timed out at startup; /health/ready will report it as down"),
    }

    let state = AppState::new(config.clone(), pool, redis);
    crate::services::public::cache::initialize(&state).await;
    Ok(state)
}
