//! Internal domain models for the KZ modules.
//!
//! These structs mirror the phase 2 schema (`migrations/20261103000002` ...
//! `0009`). They are intentionally free of HTTP concerns; the `dto` module
//! defines the public contracts that will be exposed in phase 3.
//!
//! Status/reference enums are stored as `TEXT` with CHECK constraints in the
//! database and mapped through `sqlx::Type` (rename_all = snake_case), so the
//! Rust values and the SQL constraint values cannot drift unnoticed.

pub mod account;
pub mod dependency;
pub mod jumpstat;
pub mod map;
pub mod moderation;
pub mod movement;
pub mod record;
pub mod replay;
pub mod server;
pub mod system_event;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use account::{Account, PlayerIdentity, PlayerModeStats, PlayerProfile};
pub use dependency::{DependencyCheck, DependencyName, DependencyStatus};
pub use jumpstat::Jumpstat;
pub use map::{CourseTier, CourseTierStatus, Map, MapCourse, MapStatus, MapVersion};
pub use moderation::{Ban, BanRevocation, BanSource, BanStatus};
pub use movement::{JumpType, MovementMode, MovementStyle, TimeType};
pub use record::{CourseWorldRecord, Record, RecordHistory, RecordHistoryAction, RecordStatus};
pub use replay::{Replay, ReplayStatus, StorageProvider};
pub use server::Server;
pub use system_event::SystemEvent;

/// Where a record/jumpstat came from.
///
/// Mirrors the `source` CHECK constraints in `records` and `jumpstats`.
/// `plugin` is the native CNGOKZ protocol; the `*_import` values are used by the
/// GlobalAPI / cs2kz import adapters and by the legacy importer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum IngestionSource {
    Plugin,
    Admin,
    GlobalapiImport,
    Cs2kzImport,
    LegacyImport,
}

impl IngestionSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Plugin => "plugin",
            Self::Admin => "admin",
            Self::GlobalapiImport => "globalapi_import",
            Self::Cs2kzImport => "cs2kz_import",
            Self::LegacyImport => "legacy_import",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IngestionSource;

    #[test]
    fn ingestion_source_codes_match_sql_constraints() {
        assert_eq!(IngestionSource::Plugin.as_str(), "plugin");
        assert_eq!(
            IngestionSource::GlobalapiImport.as_str(),
            "globalapi_import"
        );
        assert_eq!(IngestionSource::Cs2kzImport.as_str(), "cs2kz_import");
        assert_eq!(IngestionSource::LegacyImport.as_str(), "legacy_import");
    }
}
