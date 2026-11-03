//! OpenAPI document. Every HTTP handler is registered here; the document is
//! served at `/api-docs/openapi.json` and rendered by Swagger UI.

use utoipa::OpenApi;

use crate::dto::{
    ApiErrorBody, ApiErrorDetail, ErrorCode,
    health::{DependencyStatusDto, HealthResponse, LivenessResponse, ReadinessResponse},
    response::ApiResponse,
    system::{
        RecentEventsQuery, SystemEventResponse, SystemEventsResponse, SystemInfoResponse,
        SystemStatusResponse,
    },
};
use crate::handlers::{health, system};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "CNGOKZAPI",
        version = env!("CARGO_PKG_VERSION"),
        description = "API of the CNGOKZAPI platform for the CS:GO / CS2 KZ community.\n\nAll endpoints are versioned under `/api/v1` (except health probes and documentation). Successful responses use the `{ success, data, message }` envelope, failures use `{ success, error, message }` with a proper HTTP status code.",
        license(name = "MIT")
    ),
    paths(
        health::health,
        health::liveness,
        health::readiness,
        system::info,
        system::status,
        system::events,
    ),
    components(schemas(
        ApiResponse<HealthResponse>,
        ApiResponse<LivenessResponse>,
        ApiResponse<ReadinessResponse>,
        ApiResponse<SystemInfoResponse>,
        ApiResponse<SystemStatusResponse>,
        ApiResponse<SystemEventsResponse>,
        ApiErrorBody,
        ApiErrorDetail,
        ErrorCode,
        HealthResponse,
        LivenessResponse,
        ReadinessResponse,
        DependencyStatusDto,
        SystemInfoResponse,
        SystemStatusResponse,
        SystemEventResponse,
        SystemEventsResponse,
        RecentEventsQuery,
    )),
    tags(
        (name = "health", description = "Liveness and readiness probes"),
        (name = "system", description = "System information, status and events")
    )
)]
pub struct ApiDoc;
