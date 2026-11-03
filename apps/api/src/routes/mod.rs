//! HTTP routing assembly.

pub mod openapi;

use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::errors::AppError;
use crate::handlers;
use crate::middleware;
use crate::state::AppState;

/// Builds the complete application router.
///
/// Path map:
/// - `/health`, `/health/live`, `/health/ready`
/// - `/api/v1/system/{info,status,events}`
/// - `/swagger-ui`, `/api-docs/openapi.json`
pub fn build_router(state: AppState) -> Router {
    let swagger =
        SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi::ApiDoc::openapi());
    let cors = middleware::cors_layer(&state.config().cors_allowed_origins);

    Router::new()
        .merge(handlers::health::router())
        .nest("/api/v1", handlers::system::router())
        .nest("/api/v1/public", handlers::public::router())
        .merge(swagger)
        .fallback(not_found)
        .layer(middleware::trace_layer())
        .layer(middleware::request_id::propagate_layer())
        .layer(middleware::request_id::set_layer())
        .layer(cors)
        .with_state(state)
}

/// Unknown routes answer with the unified error envelope instead of an empty 404.
async fn not_found() -> AppError {
    AppError::not_found("route")
}
