//! Public aggregate statistics.

use sqlx::PgPool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StatisticsRow {
    pub players: i64,
    pub maps: i64,
    pub courses: i64,
    pub records: i64,
    pub world_records: i64,
    pub servers_online: i64,
}

pub async fn statistics(pool: &PgPool) -> Result<StatisticsRow, sqlx::Error> {
    sqlx::query_as::<_, StatisticsRow>(
        r#"
        SELECT
            (SELECT count(*) FROM accounts WHERE deleted_at IS NULL) AS players,
            (SELECT count(*) FROM maps WHERE status = 'approved' AND deleted_at IS NULL) AS maps,
            (SELECT count(*) FROM map_courses c
              JOIN maps m ON m.id = c.map_id
             WHERE m.status = 'approved' AND m.deleted_at IS NULL) AS courses,
            (SELECT count(*) FROM records r
              JOIN accounts a ON a.id = r.account_id AND a.deleted_at IS NULL
              JOIN maps m ON m.id = r.map_id AND m.deleted_at IS NULL
             WHERE r.status = 'approved' AND r.deleted_at IS NULL) AS records,
            (SELECT count(*) FROM course_world_records) AS world_records,
            (SELECT count(*) FROM servers
              WHERE deleted_at IS NULL
                AND is_online
                AND last_heartbeat_at IS NOT NULL
                AND last_heartbeat_at > now() - interval '2 minutes') AS servers_online
        "#,
    )
    .fetch_one(pool)
    .await
}
