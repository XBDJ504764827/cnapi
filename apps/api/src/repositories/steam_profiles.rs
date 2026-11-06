//! SQL for the background Steam profile (avatar) synchronisation.

use sqlx::PgPool;

use crate::errors::AppError;

/// Accounts whose avatar was never fetched or is older than `refresh_days`.
///
/// Ordered so that never-synced players come first: with a limited number of
/// batches per tick, new players get an avatar before old ones are refreshed.
pub async fn pending(
    db: &PgPool,
    limit: i64,
    refresh_days: i32,
) -> Result<Vec<(i64, String)>, AppError> {
    let rows = sqlx::query_as::<_, (i64, String)>(
        r#"
        SELECT a.id, a.steam_id64::text
          FROM accounts a
     LEFT JOIN player_profiles p ON p.account_id = a.id
         WHERE a.deleted_at IS NULL
           AND (
                p.avatar_synced_at IS NULL
                OR p.avatar_synced_at < now() - make_interval(days => $2)
           )
         ORDER BY p.avatar_synced_at NULLS FIRST, a.id
         LIMIT $1
        "#,
    )
    .bind(limit)
    .bind(refresh_days)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

/// Stores a fetched profile.
///
/// The persona name is only used when the profile row has to be created;
/// existing display names come from the game server plugin and must not be
/// overwritten by a background job.
pub async fn store(
    db: &PgPool,
    account_id: i64,
    fallback_name: &str,
    persona_name: Option<&str>,
    avatar_url: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO player_profiles (account_id, display_name, avatar_url, avatar_synced_at)
        VALUES ($1, $2, $3, now())
        ON CONFLICT (account_id) DO UPDATE
           SET avatar_url = COALESCE(EXCLUDED.avatar_url, player_profiles.avatar_url),
               avatar_synced_at = now()
        "#,
    )
    .bind(account_id)
    .bind(persona_name.unwrap_or(fallback_name))
    .bind(avatar_url)
    .execute(db)
    .await?;
    Ok(())
}

/// Marks a batch as attempted even when Steam returned nothing for it.
///
/// Without this, a SteamID that Steam no longer knows would be retried on every
/// tick and could starve the queue.
pub async fn mark_synced(db: &PgPool, account_id: i64) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE player_profiles SET avatar_synced_at = now() WHERE account_id = $1
        "#,
    )
    .bind(account_id)
    .execute(db)
    .await?;
    Ok(())
}
