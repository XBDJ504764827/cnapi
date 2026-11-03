//! System information, status and event services.

use chrono::Utc;

use crate::dto::health::DependencyStatusDto;
use crate::dto::system::{
    RecentEventsQuery, SystemEventResponse, SystemEventsResponse, SystemInfoResponse,
    SystemStatusResponse,
};
use crate::errors::AppError;
use crate::repositories::system_event_repository;
use crate::services::health;
use crate::state::AppState;

pub const SERVICE_NAME: &str = "CNGOKZAPI";
pub const API_VERSION: &str = "v1";

/// Static build/runtime information about the service.
pub fn info(state: &AppState) -> SystemInfoResponse {
    let config = state.config();
    SystemInfoResponse {
        name: SERVICE_NAME.to_owned(),
        version: config.version.clone(),
        environment: config.env.as_str().to_owned(),
        api_version: API_VERSION.to_owned(),
        started_at: state.started_at(),
        uptime_seconds: state.uptime_seconds(),
    }
}

/// Live status of the service and its dependencies.
pub async fn status(state: &AppState) -> SystemStatusResponse {
    let config = state.config();
    let report = health::readiness(state).await;

    SystemStatusResponse {
        status: if report.ready { "healthy" } else { "degraded" }.to_owned(),
        version: config.version.clone(),
        environment: config.env.as_str().to_owned(),
        started_at: state.started_at(),
        uptime_seconds: state.uptime_seconds(),
        checks: report
            .checks
            .iter()
            .map(DependencyStatusDto::from)
            .collect(),
        checked_at: Utc::now(),
    }
}

/// Most recent system events (empty in a fresh installation).
pub async fn recent_events(
    state: &AppState,
    query: &RecentEventsQuery,
) -> Result<SystemEventsResponse, AppError> {
    let limit = validate_events_limit(query.resolved_limit())?;
    let events = system_event_repository::list_recent(state.db(), i64::from(limit)).await?;

    let items: Vec<SystemEventResponse> =
        events.into_iter().map(SystemEventResponse::from).collect();
    Ok(SystemEventsResponse {
        count: items.len(),
        items,
        limit,
    })
}

pub fn validate_events_limit(limit: u32) -> Result<u32, AppError> {
    if (1..=RecentEventsQuery::MAX_LIMIT).contains(&limit) {
        Ok(limit)
    } else {
        Err(AppError::validation(format!(
            "`limit` must be between 1 and {}",
            RecentEventsQuery::MAX_LIMIT
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_limits_within_range() {
        assert_eq!(validate_events_limit(1).expect("valid"), 1);
        assert_eq!(validate_events_limit(100).expect("valid"), 100);
    }

    #[test]
    fn rejects_limits_outside_range() {
        for invalid in [0, 101, 1_000] {
            let error = validate_events_limit(invalid).expect_err("must be rejected");
            assert_eq!(error.status_code(), axum::http::StatusCode::BAD_REQUEST);
            assert!(error.public_message().contains("limit"));
        }
    }
}
