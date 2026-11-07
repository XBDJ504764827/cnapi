//! Admin ban console: list, create and revoke.
//!
//! 所有端点都经 `require_admin`；创建/撤销时以当前会话账号作为签发/撤销者，
//! 满足 `ck_bans_issuer` / `ck_ban_revocations_actor` 约束。

use chrono::Utc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
};

use crate::dto::ban::*;
use crate::dto::response::{ApiErrorBody, ApiResponse};
use crate::errors::AppError;
use crate::repositories::bans as repo;
use crate::services::auth::require_admin;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/bans", get(list_bans).post(create_ban))
        .route("/bans/{id}/revoke", post(revoke_ban))
}

#[utoipa::path(
    get, path = "/api/v1/admin/bans", tag = "bans",
    summary = "List bans",
    params(BanListQuery),
    responses((status = 200, body = ApiResponse<BanListResponse>), (status = 401, body = ApiErrorBody))
)]
pub async fn list_bans(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<BanListQuery>,
) -> Result<Json<ApiResponse<BanListResponse>>, AppError> {
    require_admin(&state, &headers).await?;
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let status = query.status.as_deref().unwrap_or("all");
    if !matches!(status, "all" | "active" | "expired" | "revoked") {
        return Err(AppError::validation(
            "`status` must be all|active|expired|revoked",
        ));
    }
    let search = query
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let (rows, total) = repo::list(
        state.db(),
        search,
        status,
        page_size,
        (page - 1) * page_size,
    )
    .await?;

    let items = rows
        .into_iter()
        .map(|row| {
            let status = if row.status == "revoked" {
                "revoked"
            } else if row.expires_at.is_some_and(|at| at <= Utc::now()) {
                "expired"
            } else {
                "active"
            };
            BanSummary {
                id: row.id,
                account_id: row.account_id,
                steam_id64: row.steam_id64.clone(),
                display_name: row
                    .display_name
                    .unwrap_or_else(|| format!("Player {}", row.steam_id64)),
                avatar_url: row.avatar_url,
                ban_type: row.ban_type,
                reason: row.reason,
                issued_at: row.issued_at,
                expires_at: row.expires_at,
                status: status.to_owned(),
                source: row.source,
                revoked_at: row.revoked_at,
                revoke_reason: row.revoke_reason,
            }
        })
        .collect();

    Ok(Json(ApiResponse::ok(BanListResponse {
        items,
        total,
        page,
        page_size,
    })))
}

#[utoipa::path(
    post, path = "/api/v1/admin/bans", tag = "bans",
    summary = "Create a ban",
    request_body = CreateBanRequest,
    responses((status = 200, body = ApiResponse<BanMutationResponse>), (status = 400, body = ApiErrorBody), (status = 404, body = ApiErrorBody))
)]
pub async fn create_ban(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateBanRequest>,
) -> Result<Json<ApiResponse<BanMutationResponse>>, AppError> {
    let principal = require_admin(&state, &headers).await?;
    let issuer = match principal {
        crate::models::auth::AdminPrincipal::Session(user) => user.account_id,
        crate::models::auth::AdminPrincipal::MaintenanceToken => {
            return Err(AppError::Forbidden {
                message: "ban creation requires an admin session (for audit)".to_owned(),
            });
        }
    };

    let steam_id64 = request.steam_id64.trim();
    if !(17..=20).contains(&steam_id64.len())
        || !steam_id64.chars().all(|c| c.is_ascii_digit())
        || steam_id64.chars().all(|c| c == '0')
    {
        return Err(AppError::validation(
            "`steam_id64` must be a decimal SteamID64",
        ));
    }
    let reason = request.reason.trim();
    if reason.is_empty() || reason.chars().count() > 1024 {
        return Err(AppError::validation("`reason` must be 1–1024 characters"));
    }
    let ban_type = request
        .ban_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("global");
    if ban_type.chars().count() > 64 {
        return Err(AppError::validation("`ban_type` is too long"));
    }
    if let Some(expires_at) = request.expires_at
        && expires_at <= Utc::now()
    {
        return Err(AppError::validation("`expires_at` must be in the future"));
    }

    // 目标账号必须已存在（不允许凭空给一个未登记 SteamID 建封禁记录）。
    let account_id: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM accounts WHERE steam_id64 = $1::numeric AND deleted_at IS NULL",
    )
    .bind(steam_id64)
    .fetch_optional(state.db())
    .await?;
    let account_id = account_id.ok_or_else(|| AppError::not_found("account"))?;

    if repo::active_ban_exists(state.db(), account_id).await? {
        return Err(AppError::validation("该玩家已有生效中的封禁"));
    }

    let ban_id = repo::create(
        state.db(),
        account_id,
        issuer,
        ban_type,
        reason,
        request.expires_at,
    )
    .await?;
    Ok(Json(ApiResponse::ok(BanMutationResponse { ban_id })))
}

#[utoipa::path(
    post, path = "/api/v1/admin/bans/{id}/revoke", tag = "bans",
    summary = "Revoke a ban",
    params(("id" = i64, Path)),
    request_body = RevokeBanRequest,
    responses((status = 200, body = ApiResponse<BanMutationResponse>), (status = 404, body = ApiErrorBody))
)]
pub async fn revoke_ban(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(request): Json<RevokeBanRequest>,
) -> Result<Json<ApiResponse<BanMutationResponse>>, AppError> {
    let principal = require_admin(&state, &headers).await?;
    let revoker = match principal {
        crate::models::auth::AdminPrincipal::Session(user) => user.account_id,
        crate::models::auth::AdminPrincipal::MaintenanceToken => {
            return Err(AppError::Forbidden {
                message: "ban revocation requires an admin session (for audit)".to_owned(),
            });
        }
    };
    let reason = request.reason.trim();
    if reason.is_empty() || reason.chars().count() > 1024 {
        return Err(AppError::validation("`reason` must be 1–1024 characters"));
    }
    if !repo::revoke(state.db(), id, revoker, reason).await? {
        return Err(AppError::not_found("active ban"));
    }
    Ok(Json(ApiResponse::ok(BanMutationResponse { ban_id: id })))
}
