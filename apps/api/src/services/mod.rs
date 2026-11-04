//! Business logic layer. Services orchestrate repositories and turn failures
//! into [`crate::errors::AppError`].

pub mod health;
pub mod points;
pub mod public;
pub mod scoring;
pub mod system;
