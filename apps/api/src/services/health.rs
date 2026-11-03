//! Dependency health checks.

use std::time::Instant;

use tokio::time::timeout;
use tracing::warn;

use crate::models::{DependencyCheck, DependencyName};
use crate::repositories::health_repository;
use crate::state::AppState;

/// Aggregate readiness result.
pub struct ReadinessReport {
    pub ready: bool,
    pub checks: Vec<DependencyCheck>,
}

/// Probes PostgreSQL and Redis concurrently.
///
/// Checks are bounded by `HEALTH_CHECK_TIMEOUT_MS` so a hanging dependency
/// cannot make the readiness endpoint hang.
pub async fn readiness(state: &AppState) -> ReadinessReport {
    let (database, redis) = tokio::join!(check_database(state), check_redis(state));
    let checks = vec![database, redis];
    ReadinessReport {
        ready: checks.iter().all(|check| check.status.is_up()),
        checks,
    }
}

pub async fn check_database(state: &AppState) -> DependencyCheck {
    let started = Instant::now();
    let result = timeout(
        state.config().health_check_timeout,
        health_repository::ping(state.db()),
    )
    .await;
    let latency_ms = elapsed_ms(started);

    match result {
        Ok(Ok(())) => DependencyCheck::up(DependencyName::Database, latency_ms),
        Ok(Err(error)) => {
            warn!(error = %error, "postgresql health check failed");
            DependencyCheck::down(DependencyName::Database, latency_ms, "query_failed")
        }
        Err(_) => {
            warn!("postgresql health check timed out");
            DependencyCheck::down(DependencyName::Database, latency_ms, "timeout")
        }
    }
}

pub async fn check_redis(state: &AppState) -> DependencyCheck {
    let started = Instant::now();
    let result = timeout(state.config().health_check_timeout, state.redis().ping()).await;
    let latency_ms = elapsed_ms(started);

    match result {
        Ok(Ok(())) => DependencyCheck::up(DependencyName::Redis, latency_ms),
        Ok(Err(error)) => {
            warn!(error = %error, "redis health check failed");
            DependencyCheck::down(DependencyName::Redis, latency_ms, "connection_failed")
        }
        Err(_) => {
            // The cached connection may be stuck; force a reconnect next time.
            state.redis().reset().await;
            warn!("redis health check timed out");
            DependencyCheck::down(DependencyName::Redis, latency_ms, "timeout")
        }
    }
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::config::AppConfig;
    use crate::state::RedisClient;
    use sqlx::postgres::PgPoolOptions;

    fn offline_state() -> AppState {
        let config = AppConfig::from_pairs([
            ("APP_ENV", "test"),
            (
                "DATABASE_URL",
                "postgres://user:password@127.0.0.1:1/cngokzapi",
            ),
            ("REDIS_URL", "redis://127.0.0.1:1"),
            ("HEALTH_CHECK_TIMEOUT_MS", "300"),
        ])
        .expect("test config");

        let db = PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(300))
            .max_connections(1)
            .connect_lazy(&config.database_url)
            .expect("lazy pool");

        let redis = RedisClient::new(&config.redis_url, "test").expect("redis client");
        AppState::new(config, db, redis)
    }

    #[tokio::test]
    async fn reports_unreachable_dependencies_as_down() {
        let state = offline_state();
        let report = readiness(&state).await;

        assert!(!report.ready);
        assert_eq!(report.checks.len(), 2);
        for check in &report.checks {
            assert!(!check.status.is_up(), "{:?} should be down", check.name);
            assert!(check.detail.is_some());
        }
    }

    #[test]
    fn latency_conversion_is_monotonic() {
        let started = Instant::now();
        std::thread::sleep(Duration::from_millis(2));
        assert!(elapsed_ms(started) >= 1);
    }
}
