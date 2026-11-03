//! System information, status and event contracts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::dto::health::DependencyStatusDto;
use crate::models::SystemEvent;

/// Response of `GET /api/v1/system/info`.
#[derive(Debug, Serialize, ToSchema)]
pub struct SystemInfoResponse {
    pub name: String,
    pub version: String,
    /// `development`, `test` or `production`.
    pub environment: String,
    /// API version prefix currently served, e.g. `v1`.
    pub api_version: String,
    pub started_at: DateTime<Utc>,
    pub uptime_seconds: u64,
}

/// Response of `GET /api/v1/system/status`.
///
/// Contains the real state of every dependency, probed on each request.
#[derive(Debug, Serialize, ToSchema)]
pub struct SystemStatusResponse {
    /// `healthy` when every dependency is up, otherwise `degraded`.
    pub status: String,
    pub version: String,
    pub environment: String,
    pub started_at: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub checks: Vec<DependencyStatusDto>,
    pub checked_at: DateTime<Utc>,
}

/// Query parameters of `GET /api/v1/system/events`.
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct RecentEventsQuery {
    /// Maximum number of events to return (1-100, default 20).
    #[param(minimum = 1, maximum = 100, default = 20)]
    pub limit: Option<u32>,
}

impl RecentEventsQuery {
    pub const DEFAULT_LIMIT: u32 = 20;
    pub const MAX_LIMIT: u32 = 100;

    pub fn resolved_limit(&self) -> u32 {
        self.limit.unwrap_or(Self::DEFAULT_LIMIT)
    }
}

/// A single system event.
#[derive(Debug, Serialize, ToSchema)]
pub struct SystemEventResponse {
    pub id: Uuid,
    /// Dotted event name, e.g. `api.started`.
    pub event_type: String,
    /// `info`, `warning` or `error`.
    pub severity: String,
    pub message: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl From<SystemEvent> for SystemEventResponse {
    fn from(event: SystemEvent) -> Self {
        Self {
            id: event.id,
            event_type: event.event_type,
            severity: event.severity,
            message: event.message,
            metadata: event.metadata,
            created_at: event.created_at,
        }
    }
}

/// Response of `GET /api/v1/system/events`.
#[derive(Debug, Serialize, ToSchema)]
pub struct SystemEventsResponse {
    pub items: Vec<SystemEventResponse>,
    pub count: usize,
    pub limit: u32,
}
