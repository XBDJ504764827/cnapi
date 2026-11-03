//! Internal domain models. These types are not part of the public API contract.

pub mod dependency;
pub mod system_event;

pub use dependency::{DependencyCheck, DependencyName, DependencyStatus};
pub use system_event::SystemEvent;
