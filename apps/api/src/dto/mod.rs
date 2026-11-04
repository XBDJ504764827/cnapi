//! API contracts (request/response bodies). Every type in this module is
//! documented in the generated OpenAPI schema.

pub mod account;
pub mod health;
pub mod jumpstat;
pub mod map;
pub mod moderation;
pub mod points;
pub mod public;
pub mod record;
pub mod replay;
pub mod response;
pub mod server;
pub mod system;

pub use response::{ApiErrorBody, ApiErrorDetail, ApiResponse, ErrorCode};
