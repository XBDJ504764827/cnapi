//! System level events shown on the dashboard ("最近系统事件").
//!
//! Phase 1 only reads this table; writers are added together with the features
//! that produce events (deployments, ban sync, plugin registration, ...).

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct SystemEvent {
    pub id: Uuid,
    pub event_type: String,
    pub severity: String,
    pub message: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}
