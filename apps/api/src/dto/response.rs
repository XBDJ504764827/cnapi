//! Unified response envelope.
//!
//! Success: `{ "success": true, "data": ..., "message": "..." }`
//! Failure: `{ "success": false, "error": { "code": ..., "details": ... }, "message": "..." }`

use serde::Serialize;
use utoipa::ToSchema;

/// Envelope used by every successful response.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data,
            message: "ok".to_owned(),
        }
    }

    pub fn with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data,
            message: message.into(),
        }
    }
}

/// Envelope used by every failed response.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiErrorBody {
    pub success: bool,
    pub error: ApiErrorDetail,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiErrorDetail {
    pub code: ErrorCode,
    pub details: Option<serde_json::Value>,
}

/// Stable, machine readable error codes. Clients should branch on these instead
/// of parsing `message`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    ValidationError,
    Unauthorized,
    Forbidden,
    NotFound,
    DatabaseError,
    RedisError,
    DependencyUnavailable,
    NotReady,
    InternalError,
    RateLimited,
}

impl ErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ValidationError => "validation_error",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound => "not_found",
            Self::DatabaseError => "database_error",
            Self::RedisError => "redis_error",
            Self::DependencyUnavailable => "dependency_unavailable",
            Self::NotReady => "not_ready",
            Self::InternalError => "internal_error",
            Self::RateLimited => "rate_limited",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_envelope_shape_is_stable() {
        let body = ApiResponse::ok(serde_json::json!({ "a": 1 }));
        let json = serde_json::to_value(&body).expect("serializable");

        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "ok");
        assert_eq!(json["data"]["a"], 1);
    }

    #[test]
    fn error_codes_are_snake_case() {
        let json = serde_json::to_value(ApiErrorBody {
            success: false,
            error: ApiErrorDetail {
                code: ErrorCode::DependencyUnavailable,
                details: None,
            },
            message: "down".to_owned(),
        })
        .expect("serializable");

        assert_eq!(json["error"]["code"], "dependency_unavailable");
        assert_eq!(json["success"], false);
    }
}
