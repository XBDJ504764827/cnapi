//! Lightweight per-client rate limiting for public read endpoints.
//!
//! A fixed-window counter keyed by client IP is kept in memory. This is a
//! single-process guard against accidental/abusive bursts; deployments with
//! multiple API replicas should use a shared Redis limiter or an edge proxy.
//!
//! The limiter is deliberately enabled only for `/api/v1/public/*` so health
//! probes, Swagger UI and system endpoints are unaffected.

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Request, StatusCode, header::HeaderValue},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::errors::AppError;

const DEFAULT_CAPACITY: u32 = 120;
const DEFAULT_WINDOW: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct RateLimiter {
    inner: Arc<Mutex<HashMap<String, Window>>>,
    capacity: u32,
    window: Duration,
}

#[derive(Debug, Clone, Copy)]
struct Window {
    started_at: Instant,
    count: u32,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY, DEFAULT_WINDOW)
    }
}

impl RateLimiter {
    pub fn new(capacity: u32, window: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            capacity: capacity.max(1),
            window,
        }
    }

    /// Returns `Ok(())` when the request is allowed, otherwise the number of
    /// seconds until the next window.
    pub fn check(&self, key: &str) -> Result<(), u64> {
        let now = Instant::now();
        let mut map = self
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Opportunistic cleanup prevents unbounded growth from one-off IPs.
        if map.len() > 4096 {
            map.retain(|_, window| now.duration_since(window.started_at) < self.window * 2);
        }

        let entry = map.entry(key.to_owned()).or_insert(Window {
            started_at: now,
            count: 0,
        });

        let elapsed = now.duration_since(entry.started_at);
        if elapsed >= self.window {
            entry.started_at = now;
            entry.count = 0;
        }

        if entry.count >= self.capacity {
            let remaining = self.window.saturating_sub(elapsed);
            return Err(remaining.as_secs().max(1));
        }

        entry.count += 1;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }
}

fn client_key(request: &Request<Body>) -> String {
    // Only the immediate peer address is trusted. X-Forwarded-For is
    // client-controlled and must not be used to bypass the limiter.
    if let Some(connect_info) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
        return connect_info.0.ip().to_string();
    }
    if let Some(value) = request
        .headers()
        .get("x-real-ip")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<IpAddr>().ok())
    {
        return value.to_string();
    }
    "unknown".to_owned()
}

pub async fn public_rate_limit(
    State(limiter): State<RateLimiter>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path();
    if !path.starts_with("/api/v1/public/") || path.starts_with("/api/v1/public/maintenance/") {
        // Protected maintenance endpoints are token-gated, not burst-limited.
        return next.run(request).await;
    }

    let key = client_key(&request);
    if let Err(retry_after_seconds) = limiter.check(&key) {
        let error = AppError::RateLimited {
            retry_after_seconds,
        };
        let mut response = error.into_response();
        // NotFound fallback body can pass through; only override when needed.
        if response.status() == StatusCode::TOO_MANY_REQUESTS
            && let Ok(value) = HeaderValue::from_str(&retry_after_seconds.to_string())
        {
            response.headers_mut().insert("retry-after", value);
        }
        return response;
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_capacity_then_rejects() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        assert!(limiter.check("1.2.3.4").is_ok());
        assert!(limiter.check("1.2.3.4").is_ok());
        assert!(limiter.check("1.2.3.4").is_err());
        assert!(
            limiter.check("5.6.7.8").is_ok(),
            "other clients are unaffected"
        );
    }

    #[test]
    fn window_resets() {
        let limiter = RateLimiter::new(1, Duration::from_millis(2));
        assert!(limiter.check("a").is_ok());
        assert!(limiter.check("a").is_err());
        std::thread::sleep(Duration::from_millis(5));
        assert!(limiter.check("a").is_ok());
    }

    #[test]
    fn key_cleanup_bounded() {
        let limiter = RateLimiter::new(1, Duration::from_millis(1));
        for index in 0..5000 {
            let _ = limiter.check(&format!("ip-{index}"));
            std::thread::yield_now();
        }
        assert!(limiter.len() <= 4097, "limiter map should be bounded");
    }
}
