//! Unified error handling.
//!
//! [`AppError`] is the only error type returned by HTTP handlers. It maps to a
//! correct HTTP status code and to the unified error envelope. Internal details
//! (SQL statements, connection strings, stack traces) are logged server side
//! and are **never** serialized into the response.

pub mod startup;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::{error, warn};

use crate::dto::{ApiErrorBody, ApiErrorDetail, ErrorCode};

pub use startup::StartupError;

/// Domain level error returned by handlers and services.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation failed: {message}")]
    Validation { message: String },

    /// Reserved for phase 2 (API keys, sessions). No protected route exists yet.
    #[error("unauthorized: {message}")]
    Unauthorized { message: String },

    /// Reserved for phase 2 (role based access control).
    #[error("forbidden: {message}")]
    Forbidden { message: String },

    #[error("not found: {resource}")]
    NotFound { resource: String },

    #[error("database error")]
    Database(#[source] sqlx::Error),

    #[error("redis error")]
    Redis(#[source] redis::RedisError),

    #[error("dependency `{dependency}` is unavailable")]
    DependencyUnavailable { dependency: &'static str },

    #[error("service is not ready")]
    NotReady { details: serde_json::Value },

    #[error("too many requests")]
    RateLimited { retry_after_seconds: u64 },

    #[error("internal error")]
    Internal(#[source] anyhow::Error),
}

impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    pub fn internal(source: impl Into<anyhow::Error>) -> Self {
        Self::Internal(source.into())
    }

    pub const fn status_code(&self) -> StatusCode {
        match self {
            Self::Validation { .. } => StatusCode::BAD_REQUEST,
            Self::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            Self::Forbidden { .. } => StatusCode::FORBIDDEN,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Redis(_) | Self::DependencyUnavailable { .. } | Self::NotReady { .. } => {
                StatusCode::SERVICE_UNAVAILABLE
            }
        }
    }

    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::Validation { .. } => ErrorCode::ValidationError,
            Self::Unauthorized { .. } => ErrorCode::Unauthorized,
            Self::Forbidden { .. } => ErrorCode::Forbidden,
            Self::NotFound { .. } => ErrorCode::NotFound,
            Self::Database(_) => ErrorCode::DatabaseError,
            Self::Redis(_) => ErrorCode::RedisError,
            Self::DependencyUnavailable { .. } => ErrorCode::DependencyUnavailable,
            Self::NotReady { .. } => ErrorCode::NotReady,
            Self::Internal(_) => ErrorCode::InternalError,
            Self::RateLimited { .. } => ErrorCode::RateLimited,
        }
    }

    /// Client facing message. Never derived from `sqlx`/`redis`/`anyhow` errors.
    pub fn public_message(&self) -> String {
        match self {
            Self::Validation { message } => message.clone(),
            Self::Unauthorized { .. } => "Authentication is required".to_owned(),
            Self::Forbidden { .. } => {
                "You do not have permission to access this resource".to_owned()
            }
            Self::NotFound { .. } => "The requested resource was not found".to_owned(),
            Self::Database(_) => "A database error occurred".to_owned(),
            Self::Redis(_) => "The cache service is unavailable".to_owned(),
            Self::DependencyUnavailable { dependency } => {
                format!("Dependency `{dependency}` is unavailable")
            }
            Self::NotReady { .. } => "Service is not ready to accept traffic".to_owned(),
            Self::Internal(_) => "Internal server error".to_owned(),
            Self::RateLimited { .. } => "Too many requests, please retry later".to_owned(),
        }
    }

    /// Optional machine readable details attached to the error envelope.
    pub fn details(&self) -> Option<serde_json::Value> {
        match self {
            Self::NotReady { details } => Some(details.clone()),
            Self::RateLimited {
                retry_after_seconds,
            } => Some(serde_json::json!({ "retry_after_seconds": retry_after_seconds })),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => Self::not_found("record"),
            other => Self::Database(other),
        }
    }
}

impl From<redis::RedisError> for AppError {
    fn from(error: redis::RedisError) -> Self {
        Self::Redis(error)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        Self::Internal(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.code();

        // Log the full cause; the client only sees the sanitized message.
        if status.is_server_error() {
            error!(error = ?self, error_code = code.as_str(), status = status.as_u16(), "request failed");
        } else {
            warn!(error = ?self, error_code = code.as_str(), status = status.as_u16(), "request rejected");
        }

        let body = ApiErrorBody {
            success: false,
            error: ApiErrorDetail {
                code,
                details: self.details(),
            },
            message: self.public_message(),
        };

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::response::ErrorCode;

    #[test]
    fn maps_status_codes() {
        assert_eq!(
            AppError::validation("bad").status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::not_found("route").status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            AppError::DependencyUnavailable {
                dependency: "redis"
            }
            .status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            AppError::internal(anyhow::anyhow!("boom")).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            AppError::Unauthorized {
                message: "no token".into()
            }
            .status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            AppError::Forbidden {
                message: "nope".into()
            }
            .status_code(),
            StatusCode::FORBIDDEN
        );
    }

    #[test]
    fn database_errors_do_not_leak_internals() {
        let error = AppError::from(sqlx::Error::RowNotFound);
        assert_eq!(error.status_code(), StatusCode::NOT_FOUND);

        let error = AppError::Database(sqlx::Error::Protocol("SELECT * FROM secret_table".into()));
        let message = error.public_message();
        assert!(!message.contains("SELECT"), "leaked SQL: {message}");
        assert_eq!(message, "A database error occurred");
        assert_eq!(error.code(), ErrorCode::DatabaseError);
    }

    #[test]
    fn internal_errors_do_not_leak_sources() {
        let error = AppError::internal(anyhow::anyhow!(
            "connection string postgres://user:pw@host/db"
        ));
        assert_eq!(error.public_message(), "Internal server error");
        assert!(error.details().is_none());
    }

    #[test]
    fn not_ready_keeps_sanitized_details() {
        let error = AppError::NotReady {
            details: serde_json::json!({ "checks": [] }),
        };
        assert_eq!(error.status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            error.public_message(),
            "Service is not ready to accept traffic"
        );
        assert!(error.details().is_some());
    }
}
