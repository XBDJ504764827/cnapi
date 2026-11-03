//! API contracts (request/response bodies). Every type in this module is
//! documented in the generated OpenAPI schema.

pub mod health;
pub mod response;
pub mod system;

pub use response::{ApiErrorBody, ApiErrorDetail, ApiResponse, ErrorCode};
