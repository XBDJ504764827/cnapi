//! CNGOKZAPI backend library.
//!
//! The binary (`cngokz-api`) only bootstraps this library: configuration,
//! logging, connections and the HTTP server. All application code lives in the
//! modules below.
//!
//! Layering (modular monolith):
//!
//! ```text
//! routes    -> HTTP routing / OpenAPI wiring
//! handlers  -> extract + validate input, call services, build responses
//! services  -> orchestration, business rules, dependency checks
//! repositories -> SQL / Redis access, no business rules
//! models    -> internal domain types
//! dto       -> API contracts (serde + utoipa schemas)
//! errors    -> single error type mapped to HTTP responses
//! ```

pub mod config;
pub mod dto;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod state;
pub mod utils;

pub use routes::build_router;
