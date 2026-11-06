//! HTTP routing assembly.

pub mod openapi;

use axum::{Router, middleware::from_fn_with_state};
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
/// - `/api/v1/auth/*` (Steam OpenID login, session, logout)
/// - `/api/v1/admin/sources*` (maintenance-token / admin-session administration)
/// - `/api/v1/game-servers/*` (server-token ingestion)
/// - `/api/v1/admin/game-servers/*` (maintenance-token administration)
/// - `/swagger-ui`, `/api-docs/openapi.json`
pub fn build_router(state: AppState) -> Router {
    let swagger =
        SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi::ApiDoc::openapi());
    let cors = middleware::cors_layer(&state.config().cors_allowed_origins);
    let public_rate_limiter = middleware::RateLimiter::new(
        state.config().public_rate_limit_per_minute,
        std::time::Duration::from_secs(60),
    );

    Router::new()
        .merge(handlers::health::router())
        .nest("/api/v1", handlers::system::router())
        .nest("/api/v1", handlers::auth::router())
        .nest("/api/v1", handlers::points::router())
        .nest("/api/v1/game-servers", handlers::game_servers::router())
        .nest(
            "/api/v1/admin/game-servers",
            handlers::game_servers::admin_router(),
        )
        // Phase 8 的数据源管理（/sources、/records/{id}/sources、/external-servers）。
        // 之前只写了 handler 与 OpenAPI 条目，却漏了挂载，导致管理后台「数据源」页
        // 拿到的是 404 而不是 401/403。
        .nest("/api/v1/admin", handlers::sources::router())
        .nest("/api/v1/public", handlers::public::router())
        .merge(swagger)
        .fallback(not_found)
        .layer(from_fn_with_state(
            public_rate_limiter,
            middleware::rate_limit::public_rate_limit,
        ))
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
