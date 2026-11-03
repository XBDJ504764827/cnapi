//! Tracing / logging setup.

use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AppConfig;

/// Initializes the global tracing subscriber.
///
/// Production emits JSON so logs can be shipped to a collector; development
/// keeps the human readable format.
pub fn init(config: &AppConfig) {
    let filter = EnvFilter::try_new(&config.log_level).unwrap_or_else(|_| EnvFilter::new("info"));

    let registry = tracing_subscriber::registry().with(filter);

    if config.env.is_production() {
        registry
            .with(
                fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_span_list(false),
            )
            .init();
    } else {
        registry
            .with(fmt::layer().compact().with_target(true).with_level(true))
            .init();
    }
}
