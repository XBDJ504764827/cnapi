//! Health / readiness contracts.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::models::DependencyCheck;

/// Response of `GET /health`.
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// `ok` when the process is able to serve HTTP requests.
    pub status: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
}

/// Response of `GET /health/live`.
#[derive(Debug, Serialize, ToSchema)]
pub struct LivenessResponse {
    /// Always `alive` when the process responds.
    pub status: String,
    pub uptime_seconds: u64,
}

/// Response of `GET /health/ready`.
#[derive(Debug, Serialize, ToSchema)]
pub struct ReadinessResponse {
    /// `ready` when every dependency is reachable, otherwise `not_ready`.
    pub status: String,
    pub version: String,
    pub checks: Vec<DependencyStatusDto>,
}

/// Sanitized state of a single dependency.
#[derive(Debug, Serialize, ToSchema)]
pub struct DependencyStatusDto {
    /// `postgresql` or `redis`.
    pub name: String,
    /// `up` or `down`.
    pub status: String,
    pub latency_ms: u64,
    /// Stable reason when the dependency is down (`timeout`, `connection_failed`, ...).
    pub detail: Option<String>,
}

impl From<DependencyCheck> for DependencyStatusDto {
    fn from(check: DependencyCheck) -> Self {
        Self {
            name: check.name.as_str().to_owned(),
            status: check.status.as_str().to_owned(),
            latency_ms: check.latency_ms,
            detail: check.detail,
        }
    }
}

impl From<&DependencyCheck> for DependencyStatusDto {
    fn from(check: &DependencyCheck) -> Self {
        Self {
            name: check.name.as_str().to_owned(),
            status: check.status.as_str().to_owned(),
            latency_ms: check.latency_ms,
            detail: check.detail.clone(),
        }
    }
}
