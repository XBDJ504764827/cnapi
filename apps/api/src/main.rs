//! Entry point: configuration -> logging -> connections -> HTTP server.
//!
//! Nothing but bootstrap lives here; all behaviour is implemented in the
//! library modules.

use anyhow::Context;
use cngokz_api::config::AppConfig;
use cngokz_api::errors::StartupError;
use cngokz_api::routes;
use cngokz_api::services::scoring::recalc;
use cngokz_api::state;
use cngokz_api::utils::telemetry;
use tokio::signal;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // `.env` is optional: in containers the variables come from the environment.
    let _ = dotenvy::dotenv();

    let config = AppConfig::from_env().context("failed to load configuration")?;
    telemetry::init(&config);

    info!(
        environment = %config.env,
        version = %config.version,
        "starting CNGOKZAPI"
    );

    let state = state::connect(&config)
        .await
        .context("failed to initialize application state")?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let worker_state = state.clone();
    let worker = tokio::spawn(async move { recalc::run_worker(worker_state, shutdown_rx).await });
    let source_shutdown = shutdown_tx.subscribe();
    let source_state = state.clone();
    let source_worker = tokio::spawn(async move {
        cngokz_api::services::sources::run_worker(source_state, source_shutdown).await
    });
    let avatar_shutdown = shutdown_tx.subscribe();
    let avatar_state = state.clone();
    let avatar_worker = tokio::spawn(async move {
        cngokz_api::services::steam_profiles::run_worker(avatar_state, avatar_shutdown).await
    });
    let router = routes::build_router(state.clone());
    let addr = config.socket_addr()?;

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|source| StartupError::Bind { addr, source })
        .context("failed to bind HTTP listener")?;

    info!(
        %addr,
        swagger_ui = %format!("http://{addr}/swagger-ui"),
        "HTTP server listening"
    );

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .map_err(StartupError::Serve)
    .context("HTTP server error")?;

    info!("shutdown signal received, closing resources");
    let _ = shutdown_tx.send(true);
    let _ = tokio::time::timeout(std::time::Duration::from_secs(5), worker).await;
    let _ = tokio::time::timeout(std::time::Duration::from_secs(5), source_worker).await;
    let _ = tokio::time::timeout(std::time::Duration::from_secs(5), avatar_worker).await;
    state.db().close().await;
    info!("bye");

    Ok(())
}

/// Resolves on Ctrl-C or SIGTERM and lets axum drain in-flight requests.
async fn shutdown_signal() {
    let ctrl_c = async {
        if signal::ctrl_c().await.is_err() {
            warn!("failed to install Ctrl-C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(error) => {
                warn!(error = %error, "failed to install SIGTERM handler");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("received Ctrl-C"),
        _ = terminate => info!("received SIGTERM"),
    }
}
