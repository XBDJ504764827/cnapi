//! SQL for the admin ban console (list / create / revoke).

use sqlx::PgPool;

use crate::errors::AppError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BanListRow {
    pub id: i64,
    pub account_id: i64,
    pub steam_id64: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub ban_type: String,
    pub reason: String,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
    pub source: String,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
    pub revoke_reason: Option<String>,
}

/// Active/expired status is derived at read time: a ban whose `expires_at` is
/// in the past is reported as `expired` even though the stored status stays
/// `active` (the schema deliberately has no background status job).
pub async fn list(
    db: &PgPool,
    search: Option<&str>,
    status: &str,
    limit: i64,
    offset: i64,
) -> Result<(Vec<BanListRow>, i64), AppError> {
    let rows = sqlx::query_as::<_, BanListRow>(
        r#"
        SELECT b.id, b.account_id, a.steam_id64::text AS steam_id64,
               p.display_name, p.avatar_url,
               b.ban_type, b.reason, b.issued_at, b.expires_at,
               b.status, b.source,
               br.revoked_at, br.reason AS revoke_reason
          FROM bans b
          JOIN accounts a ON a.id = b.account_id
          LEFT JOIN player_profiles p ON p.account_id = b.account_id
          LEFT JOIN ban_revocations br ON br.ban_id = b.id
         WHERE ($1::text IS NULL
                OR p.display_name ILIKE '%' || $1 || '%'
                OR a.steam_id64::text LIKE $1 || '%')
           AND (
                 $2::text = 'all'
                 OR ($2 = 'active'  AND b.status = 'active'  AND (b.expires_at IS NULL OR b.expires_at > now()))
                 OR ($2 = 'expired' AND b.status = 'active'  AND b.expires_at IS NOT NULL AND b.expires_at <= now())
                 OR ($2 = 'revoked' AND b.status = 'revoked')
           )
         ORDER BY b.issued_at DESC, b.id DESC
         LIMIT $3 OFFSET $4
        "#,
    )
    .bind(search)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await?;

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)
          FROM bans b
          JOIN accounts a ON a.id = b.account_id
          LEFT JOIN player_profiles p ON p.account_id = b.account_id
         WHERE ($1::text IS NULL
                OR p.display_name ILIKE '%' || $1 || '%'
                OR a.steam_id64::text LIKE $1 || '%')
           AND (
                 $2::text = 'all'
                 OR ($2 = 'active'  AND b.status = 'active'  AND (b.expires_at IS NULL OR b.expires_at > now()))
                 OR ($2 = 'expired' AND b.status = 'active'  AND b.expires_at IS NOT NULL AND b.expires_at <= now())
                 OR ($2 = 'revoked' AND b.status = 'revoked')
           )
        "#,
    )
    .bind(search)
    .bind(status)
    .fetch_one(db)
    .await?;

    Ok((rows, total))
}

pub async fn create(
    db: &PgPool,
    account_id: i64,
    issuer_account_id: i64,
    ban_type: &str,
    reason: &str,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<i64, AppError> {
    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO bans
            (account_id, ban_type, reason, issued_by_account_id, expires_at, status, source)
        VALUES ($1, $2, $3, $4, $5, 'active', 'admin')
        RETURNING id
        "#,
    )
    .bind(account_id)
    .bind(ban_type)
    .bind(reason)
    .bind(issuer_account_id)
    .bind(expires_at)
    .fetch_one(db)
    .await?;
    Ok(id)
}

pub async fn revoke(
    db: &PgPool,
    ban_id: i64,
    revoked_by_account_id: i64,
    reason: &str,
) -> Result<bool, AppError> {
    let mut tx = db.begin().await?;
    let updated = sqlx::query(
        r#"
        UPDATE bans SET status = 'revoked', updated_at = now()
         WHERE id = $1 AND status = 'active'
        "#,
    )
    .bind(ban_id)
    .execute(&mut *tx)
    .await?;
    if updated.rows_affected() == 0 {
        return Ok(false);
    }
    sqlx::query(
        r#"
        INSERT INTO ban_revocations (ban_id, revoked_by_account_id, reason)
        VALUES ($1, $2, $3)
        ON CONFLICT (ban_id) DO NOTHING
        "#,
    )
    .bind(ban_id)
    .bind(revoked_by_account_id)
    .bind(reason)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(true)
}

pub async fn active_ban_exists(db: &PgPool, account_id: i64) -> Result<bool, AppError> {
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM bans
             WHERE account_id = $1
               AND status = 'active'
               AND (expires_at IS NULL OR expires_at > now())
        )
        "#,
    )
    .bind(account_id)
    .fetch_one(db)
    .await?;
    Ok(exists)
}
