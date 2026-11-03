//! Business logic layer. Services orchestrate repositories and turn failures
//! into [`crate::errors::AppError`].

pub mod health;
pub mod public;
pub mod system;
