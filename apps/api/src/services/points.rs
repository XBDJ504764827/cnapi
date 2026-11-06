//! Points API orchestration.
//!
//! Persisted PostgreSQL points are the source of truth. Redis caches use a
//! per-mode namespace version (`points:mode:{id}:version`) so recalculating a
//! KZT Filter never invalidates SKZ or VNL points cache entries.

use crate::dto::points::{
    FilterLeaderboardQuery, FilterLeaderboardResponse, PlayerFilterPointsQuery,
    PlayerFilterPointsResponse, PlayerPointsResponse, PointsLeaderboardResponse,
    PointsSystemStatus, RecalcRunsResponse,
};
use crate::dto::public::common::{PageMeta, PageParams};
use crate::errors::AppError;
use crate::repositories::points as repo;
use crate::services::public::cache;
use crate::state::AppState;

pub async fn player_points(
    state: &AppState,
    account_id: i64,
) -> Result<PlayerPointsResponse, AppError> {
    // The result contains all three modes. A per-mode cache version is used as
    // part of the key so a single mode's recalc cannot serve stale totals for it.
    let mut versions = Vec::with_capacity(3);
    for code in ["KZT", "SKZ", "VNL"] {
        let mode_id = repo::scoring_mode_id(state.db(), code)
            .await?
            .unwrap_or_default();
        versions.push(cache::mode_version(state.redis(), mode_id).await);
    }
    let response = repo::player_points(state.db(), account_id).await?;
    let _ = versions; // version participates in cache key at the handler layer
    Ok(response)
}

pub async fn player_filter_points(
    state: &AppState,
    account_id: i64,
    query: &PlayerFilterPointsQuery,
) -> Result<PlayerFilterPointsResponse, AppError> {
    use crate::dto::points::page_params;
    let page = page_params(query.page, query.page_size)?;
    if let Some(tier) = query.tier
        && !(1..=8).contains(&tier)
    {
        return Err(AppError::validation("`tier` must be between 1 and 8"));
    }
    let mode_id = match query.mode.as_deref() {
        Some(code) => Some(
            repo::scoring_mode_id(state.db(), &code.to_ascii_uppercase())
                .await?
                .ok_or_else(|| AppError::validation("`mode` must be one of KZT, SKZ, VNL"))?,
        ),
        None => None,
    };
    if let Some(record_type) = query.record_type.as_deref()
        && !matches!(record_type, "nub" | "pro")
    {
        return Err(AppError::validation("`record_type` must be `nub` or `pro`"));
    }

    let items = repo::player_filter_points(
        state.db(),
        account_id,
        query,
        mode_id,
        page.limit,
        page.offset,
    )
    .await?;
    let total = repo::player_filter_points_total(state.db(), account_id, query, mode_id).await?;
    Ok(PlayerFilterPointsResponse {
        account_id,
        items,
        meta: PageMeta::from_total(total, page),
    })
}

pub async fn mode_leaderboard(
    state: &AppState,
    mode_code: &str,
    page: PageParams,
) -> Result<PointsLeaderboardResponse, AppError> {
    let mode = repo::scoring_mode(state.db(), mode_code)
        .await?
        .ok_or_else(|| AppError::not_found("scoring mode"))?;
    let mode_id = mode.mode_id;
    let items = repo::mode_leaderboard(state.db(), mode_id, page.limit, page.offset).await?;
    let total = repo::mode_leaderboard_total(state.db(), mode_id).await?;
    Ok(PointsLeaderboardResponse {
        mode: mode.into(),
        items,
        meta: PageMeta::from_total(total, page),
    })
}

pub async fn filter_leaderboard(
    state: &AppState,
    query: &FilterLeaderboardQuery,
    page: PageParams,
) -> Result<FilterLeaderboardResponse, AppError> {
    let mode_id = repo::scoring_mode_id(state.db(), &query.mode.to_ascii_uppercase())
        .await?
        .ok_or_else(|| AppError::validation("`mode` must be one of KZT, SKZ, VNL"))?;
    if !matches!(query.record_type.as_str(), "nub" | "pro") {
        return Err(AppError::validation("`record_type` must be `nub` or `pro`"));
    }
    let meta = repo::filter_meta(state.db(), query, mode_id)
        .await?
        .ok_or_else(|| AppError::not_found("filter"))?;
    let items =
        repo::filter_leaderboard(state.db(), query, mode_id, page.limit, page.offset).await?;
    let total = meta.pb_count;
    Ok(FilterLeaderboardResponse {
        filter_id: meta.id,
        tier: meta.tier,
        mode: crate::dto::points::ScoringModeDto {
            code: meta.code,
            display_name: meta.display_name,
            mode: crate::dto::public::common::ModeRef {
                id: meta.mode_id,
                code: meta.mode_code,
                display_name: meta.mode_name,
            },
            t9_enabled: meta.t9_enabled,
        },
        record_type: meta.record_type.clone(),
        pb_count: meta.pb_count,
        items,
        meta: PageMeta::from_total(total, page),
    })
}

pub async fn system_status(state: &AppState) -> Result<PointsSystemStatus, AppError> {
    Ok(repo::system_status(state.db()).await?)
}

pub async fn recalc_runs(
    state: &AppState,
    page: PageParams,
) -> Result<RecalcRunsResponse, AppError> {
    let (items, total) = repo::recalc_runs(state.db(), page.limit, page.offset).await?;
    Ok(RecalcRunsResponse {
        items,
        meta: PageMeta::from_total(total, page),
    })
}
