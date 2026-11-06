//! Points & ranking API contracts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::dto::public::common::{ModeRef, PageMeta, PageParams, PageQuery};
use crate::errors::AppError;
use crate::services::scoring::model::ScoringMode;

pub const MAX_PAGE_SIZE: u32 = 100;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModePointsDto {
    pub code: String,
    pub display_name: String,
    pub pro_points: i64,
    pub nub_points: i64,
    pub total_points: i64,
    pub filter_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerPointsResponse {
    pub account_id: i64,
    pub steam_id64: String,
    pub modes: Vec<ModePointsDto>,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PlayerFilterPointsQuery {
    /// KZT / SKZ / VNL. Omit for all modes.
    pub mode: Option<String>,
    /// nub / pro. Omit for both.
    pub record_type: Option<String>,
    /// 1..8.
    pub tier: Option<i16>,
    pub course_id: Option<i64>,
    pub map: Option<String>,
    /// rank / points / time / tier.
    pub sort: Option<String>,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerFilterPointsDto {
    pub filter_id: i64,
    pub map_id: i64,
    pub map_name: String,
    pub map_slug: String,
    pub course_id: i64,
    pub course_name: String,
    pub stage: i16,
    pub mode: ModeRef,
    pub record_type: String,
    pub tier: i16,
    /// Historical T9 label; the stored course tier is always T1-T8.
    pub t9_label: bool,
    pub pb_record_id: i64,
    /// PB duration in seconds.
    pub pb_time_seconds: String,
    pub wr_time_seconds: String,
    pub rank: i32,
    pub points: i32,
    pub is_world_record: bool,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerFilterPointsResponse {
    pub account_id: i64,
    pub items: Vec<PlayerFilterPointsDto>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PointsLeaderboardQuery {
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PointsLeaderboardEntry {
    pub rank: i64,
    pub account_id: i64,
    pub steam_id64: String,
    pub display_name: String,
    pub pro_points: i64,
    pub nub_points: i64,
    pub total_points: i64,
    pub filter_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PointsLeaderboardResponse {
    pub mode: ScoringModeDto,
    pub items: Vec<PointsLeaderboardEntry>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ScoringModeDto {
    pub code: String,
    pub display_name: String,
    pub mode: ModeRef,
    pub t9_enabled: bool,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct FilterLeaderboardQuery {
    pub map_id: i64,
    pub course_id: i64,
    /// KZT / SKZ / VNL.
    pub mode: String,
    /// nub / pro.
    pub record_type: String,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FilterLeaderboardEntry {
    pub rank: i32,
    pub account_id: i64,
    pub steam_id64: String,
    pub display_name: String,
    pub pb_record_id: i64,
    pub pb_time_seconds: String,
    pub points: i32,
    pub is_world_record: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FilterLeaderboardResponse {
    pub filter_id: i64,
    pub tier: i16,
    pub mode: ScoringModeDto,
    pub record_type: String,
    pub pb_count: i64,
    pub items: Vec<FilterLeaderboardEntry>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct PointsSystemStatus {
    pub pending: i64,
    pub running: i64,
    pub failed: i64,
    pub filters: i64,
    pub dirty_filters: i64,
    pub last_recalculated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct RecalcRunsQuery {
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct RecalcRunDto {
    pub id: i64,
    pub filter_id: i64,
    pub mode_id: i16,
    pub status: String,
    pub pb_count: i64,
    pub duration_ms: i32,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecalcRunsResponse {
    pub items: Vec<RecalcRunDto>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RecalcRequest {
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecalcResponse {
    pub queued_filters: i64,
    pub mode: String,
}

pub fn parse_mode(value: &str) -> Result<ScoringMode, AppError> {
    ScoringMode::parse(value)
        .ok_or_else(|| AppError::validation("`mode` must be one of KZT, SKZ, VNL"))
}

pub fn page_params(page: Option<u32>, page_size: Option<u32>) -> Result<PageParams, AppError> {
    PageQuery { page, page_size }.resolve()
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct AdminTierDto {
    pub map_id: i64,
    pub map_name: String,
    pub course_id: i64,
    pub course_name: String,
    pub stage: i16,
    pub mode_id: i16,
    pub mode_code: String,
    pub tier: i16,
    pub completed: bool,
    pub first_completion_record_id: Option<i64>,
    pub first_completion_account_id: Option<i64>,
    pub first_completion_player: String,
    pub first_completion_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct AdminTiersQuery {
    /// KZT / SKZ / VNL. Omit for all modes.
    pub mode: Option<String>,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminTiersResponse {
    pub items: Vec<AdminTierDto>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateTierRequest {
    pub tier: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateTierResponse {
    pub updated: bool,
    pub tier: i16,
}
