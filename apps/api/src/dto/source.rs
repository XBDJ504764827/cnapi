//! Phase-8 data source administration and record provenance contracts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::dto::public::common::{PageMeta, PageParams, PageQuery};
use crate::errors::AppError;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct SourceSummary {
    pub id: i64,
    pub code: String,
    pub source_type: String,
    pub display_name: String,
    pub is_enabled: bool,
    pub priority: i32,
    pub status: String,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub consecutive_failures: i32,
    pub cursor: serde_json::Value,
    pub config: serde_json::Value,
    pub raw_records: i64,
    pub canonical_links: i64,
    pub latest_run_status: Option<String>,
    pub latest_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SourceListResponse {
    pub items: Vec<SourceSummary>,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct SourceRunsQuery {
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

impl SourceRunsQuery {
    pub fn page_params(&self) -> Result<PageParams, AppError> {
        PageQuery {
            page: self.page,
            page_size: self.page_size,
        }
        .resolve()
    }
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct SourceRunDto {
    pub id: i64,
    pub source_id: i64,
    pub run_type: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub stats: serde_json::Value,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SourceRunsResponse {
    pub items: Vec<SourceRunDto>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct UpdateSourceRequest {
    pub is_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SourceSyncResponse {
    pub source: String,
    pub run_id: Option<i64>,
    pub status: String,
    pub message: String,
    pub stats: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct RawRecordAuditRow {
    pub raw_record_id: i64,
    pub canonical_record_id: Option<i64>,
    pub source_code: String,
    pub source_type: String,
    pub external_record_id: String,
    pub processing_status: String,
    pub processing_error: Option<String>,
    pub match_method: Option<String>,
    pub match_confidence: Option<f64>,
    pub field_conflicts: serde_json::Value,
    pub source_recorded_at: Option<DateTime<Utc>>,
    pub received_at: DateTime<Utc>,
    pub raw_payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RecordProvenanceResponse {
    pub record_id: i64,
    pub sources: Vec<RawRecordAuditRow>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct ExternalServerSnapshot {
    pub id: i64,
    pub source_code: String,
    pub external_id: String,
    pub name: String,
    pub label: Option<String>,
    pub host: String,
    pub port: i32,
    pub map_name: Option<String>,
    pub map_tier: Option<i16>,
    pub online_players: i32,
    pub max_players: i32,
    pub is_online: bool,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ExternalServerListResponse {
    pub items: Vec<ExternalServerSnapshot>,
    pub source_healthy: bool,
    pub generated_at: DateTime<Utc>,
}
