//! HTTP handlers. Handlers only extract input, call a service and wrap the
//! result in the unified response envelope.

pub mod auth;
pub mod bans;
pub mod game_servers;
pub mod health;
pub mod points;
pub mod public;
pub mod sources;
pub mod system;
