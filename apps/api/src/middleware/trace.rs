//! Structured request tracing.

use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnResponse, HttpMakeClassifier, TraceLayer,
};
use tracing::Level;

/// HTTP trace layer with sane production defaults.
pub fn layer() -> TraceLayer<HttpMakeClassifier> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(tower_http::LatencyUnit::Millis),
        )
        .on_failure(DefaultOnFailure::new().level(Level::ERROR))
}
