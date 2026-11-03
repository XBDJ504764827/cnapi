//! HTTP handlers. Handlers only extract input, call a service and wrap the
//! result in the unified response envelope.

pub mod health;
pub mod public;
pub mod system;
