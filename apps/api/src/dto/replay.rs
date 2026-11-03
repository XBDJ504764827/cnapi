//! Phase 3 read contract for replay metadata.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::models::replay::ReplayStatus;
use crate::models::replay::StorageProvider;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ReplaySummary {
    pub id: i64,
    pub record_id: i64,
    pub storage_provider: StorageProvider,
    pub bucket: String,
    pub object_key: String,
    pub size_bytes: i64,
    pub status: ReplayStatus,
    pub uploaded_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}
