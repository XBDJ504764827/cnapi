//! HTTP level tests that do not require PostgreSQL or Redis.
//!
//! Dependencies point at closed ports, which also verifies that the readiness
//! endpoint reports a real (unhealthy) state instead of a hardcoded one.

use std::time::Duration;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use cngokz_api::{config::AppConfig, routes, state::AppState, state::RedisClient};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

fn offline_state() -> AppState {
    let config = AppConfig::from_pairs([
        ("APP_ENV", "test"),
        ("APP_VERSION", "0.0.0-test"),
        (
            "DATABASE_URL",
            "postgres://user:password@127.0.0.1:1/cngokzapi",
        ),
        ("REDIS_URL", "redis://127.0.0.1:1"),
        ("HEALTH_CHECK_TIMEOUT_MS", "300"),
        ("CORS_ALLOWED_ORIGINS", "http://localhost:5173"),
        ("DATABASE_ACQUIRE_TIMEOUT_SECS", "1"),
    ])
    .expect("test configuration must be valid");

    let db = PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(300))
        .max_connections(1)
        .connect_lazy(&config.database_url)
        .expect("lazy pool");

    let redis = RedisClient::new(&config.redis_url, "test").expect("redis client");
    AppState::new(config, db, redis)
}

fn app() -> Router {
    routes::build_router(offline_state())
}

async fn send(request: Request<Body>) -> (StatusCode, Value, axum::http::HeaderMap) {
    let response = app().oneshot(request).await.expect("request must succeed");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let json: Value = serde_json::from_slice(&bytes).expect("response must be JSON");
    (status, json, headers)
}

fn get(path: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .body(Body::empty())
        .expect("request")
}

#[tokio::test]
async fn health_returns_unified_success_envelope() {
    let (status, body, headers) = send(get("/health")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "ok");
    assert_eq!(body["data"]["version"], "0.0.0-test");
    assert!(body["data"]["timestamp"].is_string());
    assert!(
        headers.contains_key("x-request-id"),
        "request id must be returned"
    );
}

#[tokio::test]
async fn liveness_reports_uptime() {
    let (status, body, _) = send(get("/health/live")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "alive");
    assert!(body["data"]["uptime_seconds"].is_number());
}

#[tokio::test]
async fn readiness_reports_503_when_dependencies_are_unreachable() {
    let (status, body, _) = send(get("/health/ready")).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "not_ready");

    let checks = body["error"]["details"]["checks"]
        .as_array()
        .expect("checks array");
    assert_eq!(checks.len(), 2);
    for check in checks {
        assert_eq!(check["status"], "down", "unexpected check: {check}");
    }

    // Internals must not leak into the payload.
    let serialized = body.to_string();
    assert!(
        !serialized.contains("127.0.0.1"),
        "response leaked an address: {serialized}"
    );
    assert!(
        !serialized.contains("SELECT"),
        "response leaked SQL: {serialized}"
    );
}

#[tokio::test]
async fn system_info_exposes_runtime_metadata() {
    let (status, body, _) = send(get("/api/v1/system/info")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["name"], "CNGOKZAPI");
    assert_eq!(body["data"]["environment"], "test");
    assert_eq!(body["data"]["api_version"], "v1");
    assert!(body["data"]["started_at"].is_string());
}

#[tokio::test]
async fn system_status_reports_unhealthy_dependencies() {
    let (status, body, _) = send(get("/api/v1/system/status")).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "status endpoint always answers 200 with details"
    );
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "degraded");

    let checks = body["data"]["checks"].as_array().expect("checks array");
    assert_eq!(checks.len(), 2);
    assert!(checks.iter().all(|check| check["status"] == "down"));
    assert!(checks.iter().any(|check| check["name"] == "postgresql"));
    assert!(checks.iter().any(|check| check["name"] == "redis"));
}

#[tokio::test]
async fn events_limit_is_validated_before_querying_the_database() {
    let (status, body, _) = send(get("/api/v1/system/events?limit=0")).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "validation_error");
    assert!(
        body["message"]
            .as_str()
            .unwrap_or_default()
            .contains("limit")
    );
}

#[tokio::test]
async fn unknown_route_returns_404_envelope() {
    let (status, body, _) = send(get("/api/v1/does-not-exist")).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "not_found");
}

#[tokio::test]
async fn openapi_document_is_served() {
    let (status, body, _) = send(get("/api-docs/openapi.json")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["info"]["title"], "CNGOKZAPI");
    assert!(body["paths"]["/health"].is_object());
    assert!(body["paths"]["/api/v1/system/status"].is_object());
}

#[tokio::test]
async fn swagger_ui_is_served() {
    // `/swagger-ui` redirects to `/swagger-ui/`, which serves the HTML shell.
    let redirect = app()
        .oneshot(
            Request::builder()
                .uri("/swagger-ui")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("request must succeed");
    assert!(
        redirect.status().is_redirection(),
        "unexpected status: {}",
        redirect.status()
    );

    let response = app()
        .oneshot(
            Request::builder()
                .uri("/swagger-ui/")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("request must succeed");

    assert!(
        response.status().is_success(),
        "unexpected status: {}",
        response.status()
    );
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    assert!(
        content_type.contains("text/html"),
        "unexpected content type: {content_type}"
    );
}

#[tokio::test]
async fn cors_preflight_is_handled_for_allowed_origin() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/v1/system/info")
                .header(header::ORIGIN, "http://localhost:5173")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("request must succeed");

    assert!(response.status().is_success());
    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok()),
        Some("http://localhost:5173")
    );
}
