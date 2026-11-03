//! CORS policy built from `CORS_ALLOWED_ORIGINS`.

use std::time::Duration;

use axum::http::{HeaderName, HeaderValue, Method, header};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

/// Builds the CORS layer.
///
/// `*` is only meant for local development and is rejected by the deployment
/// documentation for production. Credentials are intentionally not allowed,
/// authentication will be header based.
pub fn layer(allowed_origins: &[String]) -> CorsLayer {
    let layer = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::ACCEPT,
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            HeaderName::from_static(request_id_header()),
        ])
        .expose_headers([HeaderName::from_static(request_id_header())])
        .max_age(Duration::from_secs(600));

    if allowed_origins.iter().any(|origin| origin == "*") {
        return layer.allow_origin(Any);
    }

    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();

    if origins.is_empty() {
        // No configured origin: browsers get no CORS headers, requests are still
        // served for server-side clients.
        layer
    } else {
        layer.allow_origin(AllowOrigin::list(origins))
    }
}

const fn request_id_header() -> &'static str {
    super::request_id::REQUEST_ID_HEADER
}
