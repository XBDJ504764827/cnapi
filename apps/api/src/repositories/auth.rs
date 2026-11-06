//! SQL for Steam OpenID authentication, sessions and roles.
//!
//! Two invariants are enforced here rather than in the service layer:
//!   * a session token is only ever compared through its SHA-256 digest;
//!   * a login state (CSRF nonce) is consumed by a single atomic `UPDATE`, so
//!     replaying a callback cannot succeed twice.

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::auth::AuthUser;

/// Returns the account id for a SteamID64, creating the account when needed.
///
/// The `ON CONFLICT` target mirrors the partial unique index on active
/// (`deleted_at IS NULL`) rows, so a soft-deleted account is never revived.
pub async fn ensure_account(db: &PgPool, steam_id64: &str) -> Result<i64, AppError> {
    let account_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO accounts (steam_id64) VALUES ($1::numeric)
        ON CONFLICT (steam_id64) WHERE deleted_at IS NULL
        DO UPDATE SET updated_at = now()
        RETURNING id
        "#,
    )
    .bind(steam_id64)
    .fetch_one(db)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO player_profiles (account_id, display_name)
        VALUES ($1, $2)
        ON CONFLICT (account_id) DO NOTHING
        "#,
    )
    .bind(account_id)
    .bind(fallback_display_name(steam_id64))
    .execute(db)
    .await?;

    Ok(account_id)
}

/// Last four SteamID64 digits, matching [`crate::models::auth::AuthUser`].
pub fn fallback_display_name(steam_id64: &str) -> String {
    let suffix: String = steam_id64
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("Player {suffix}")
}

/// Refreshes the profile with data from the Steam Web API.
///
/// `None` values keep the previous column value, so a failed lookup never
/// wipes an existing persona name or avatar.
pub async fn sync_profile(
    db: &PgPool,
    account_id: i64,
    display_name: Option<&str>,
    avatar_url: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE player_profiles
           SET display_name = COALESCE($2, display_name),
               avatar_url = COALESCE($3, avatar_url),
               last_joined_at = now()
         WHERE account_id = $1
        "#,
    )
    .bind(account_id)
    .bind(display_name)
    .bind(avatar_url)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn grant_role(db: &PgPool, account_id: i64, role: &str) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO account_roles (account_id, role) VALUES ($1, $2)
        ON CONFLICT (account_id, role) DO NOTHING
        "#,
    )
    .bind(account_id)
    .bind(role)
    .execute(db)
    .await?;
    Ok(())
}

/// Inserts a login state and opportunistically drops expired rows.
pub async fn create_login_state(
    db: &PgPool,
    state: &str,
    redirect_path: &str,
    ttl: Duration,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO steam_login_states (state, redirect_path, expires_at)
        VALUES ($1, $2, now() + $3 * interval '1 second')
        "#,
    )
    .bind(state)
    .bind(redirect_path)
    .bind(ttl.num_seconds())
    .execute(db)
    .await?;

    sqlx::query("DELETE FROM steam_login_states WHERE expires_at < now() - interval '1 day'")
        .execute(db)
        .await?;

    Ok(())
}

/// Atomically consumes a login state and returns the redirect path it stored.
///
/// Returns `Ok(None)` for unknown, already consumed and expired states alike:
/// the caller must not be able to tell them apart.
pub async fn consume_login_state(db: &PgPool, state: &str) -> Result<Option<String>, AppError> {
    let redirect_path: Option<String> = sqlx::query_scalar(
        r#"
        UPDATE steam_login_states
           SET consumed_at = now()
         WHERE state = $1
           AND consumed_at IS NULL
           AND expires_at > now()
        RETURNING redirect_path
        "#,
    )
    .bind(state)
    .fetch_optional(db)
    .await?;
    Ok(redirect_path)
}

pub async fn create_session(
    db: &PgPool,
    account_id: i64,
    token_hash: &[u8],
    expires_at: DateTime<Utc>,
    user_agent: Option<&str>,
    ip: Option<&str>,
) -> Result<uuid::Uuid, AppError> {
    let session_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO auth_sessions (account_id, token_hash, expires_at, user_agent, ip)
        VALUES ($1, $2, $3, $4, $5::inet)
        RETURNING id
        "#,
    )
    .bind(account_id)
    .bind(token_hash)
    .bind(expires_at)
    .bind(user_agent)
    .bind(ip)
    .fetch_one(db)
    .await?;
    Ok(session_id)
}

/// Resolves an active session by token digest.
pub async fn find_session(db: &PgPool, token_hash: &[u8]) -> Result<Option<AuthUser>, AppError> {
    let user = sqlx::query_as::<_, AuthUser>(
        r#"
        SELECT s.id AS session_id,
               a.id AS account_id,
               a.steam_id64,
               p.display_name,
               p.avatar_url,
               s.expires_at,
               EXISTS (
                   SELECT 1 FROM account_roles r
                    WHERE r.account_id = a.id AND r.role = 'admin'
               ) AS is_admin,
               COALESCE(
                   (SELECT array_agg(r.role ORDER BY r.role)
                      FROM account_roles r
                     WHERE r.account_id = a.id),
                   '{}'::text[]
               ) AS roles
          FROM auth_sessions s
          JOIN accounts a ON a.id = s.account_id AND a.deleted_at IS NULL
     LEFT JOIN player_profiles p ON p.account_id = a.id
         WHERE s.token_hash = $1
           AND s.revoked_at IS NULL
           AND s.expires_at > now()
        "#,
    )
    .bind(token_hash)
    .fetch_optional(db)
    .await?;
    Ok(user)
}

/// Refreshes `last_seen_at`, at most once per five minutes per session.
pub async fn touch_session(db: &PgPool, session_id: uuid::Uuid) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE auth_sessions
           SET last_seen_at = now()
         WHERE id = $1
           AND last_seen_at < now() - interval '5 minutes'
        "#,
    )
    .bind(session_id)
    .execute(db)
    .await?;
    Ok(())
}

/// Revokes a session. Returns whether an active session was revoked.
pub async fn revoke_session(db: &PgPool, token_hash: &[u8]) -> Result<bool, AppError> {
    let result = sqlx::query(
        r#"
        UPDATE auth_sessions
           SET revoked_at = now()
         WHERE token_hash = $1
           AND revoked_at IS NULL
        "#,
    )
    .bind(token_hash)
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Drops sessions that expired more than a week ago; safe to call on login.
pub async fn purge_expired_sessions(db: &PgPool) -> Result<u64, AppError> {
    let result =
        sqlx::query("DELETE FROM auth_sessions WHERE expires_at < now() - interval '7 days'")
            .execute(db)
            .await?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_name_uses_the_last_four_digits() {
        assert_eq!(fallback_display_name("76561198000001234"), "Player 1234");
        assert_eq!(fallback_display_name("12"), "Player 12");
    }
}
