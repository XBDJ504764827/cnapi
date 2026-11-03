//! Read access to `system_events`.

use sqlx::PgPool;

use crate::models::SystemEvent;

pub async fn list_recent(pool: &PgPool, limit: i64) -> Result<Vec<SystemEvent>, sqlx::Error> {
    sqlx::query_as::<_, SystemEvent>(
        r#"
        SELECT id, event_type, severity, message, metadata, created_at
        FROM system_events
        ORDER BY created_at DESC, id DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}
