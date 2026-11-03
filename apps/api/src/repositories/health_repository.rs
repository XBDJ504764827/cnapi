//! Connectivity probes against PostgreSQL.

use sqlx::PgPool;

/// Cheapest possible round trip used by health checks.
pub async fn ping(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .map(|_| ())
}

/// Number of applied migrations. Useful for diagnostics and integration tests.
pub async fn applied_migration_count(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>("SELECT count(*) FROM _sqlx_migrations WHERE success")
        .fetch_one(pool)
        .await
}
