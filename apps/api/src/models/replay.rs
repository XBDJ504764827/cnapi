//! Replay object metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Object storage backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum StorageProvider {
    /// Cloudflare R2.
    R2,
    /// Generic S3 compatible storage.
    S3,
    /// Local disk (development only).
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum ReplayStatus {
    Pending,
    Available,
    Failed,
    Deleted,
}

#[derive(Debug, Clone, FromRow)]
pub struct Replay {
    pub id: i64,
    pub record_id: i64,
    pub account_id: i64,
    pub map_id: i64,
    pub storage_provider: StorageProvider,
    pub bucket: String,
    pub object_key: String,
    pub content_type: String,
    pub size_bytes: i64,
    /// SHA-256 digest (32 bytes).
    pub checksum_sha256: Vec<u8>,
    pub replay_format_version: Option<i16>,
    pub status: ReplayStatus,
    pub uploaded_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
