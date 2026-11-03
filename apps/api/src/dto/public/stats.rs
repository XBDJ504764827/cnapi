//! Public statistics and reference modes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Aggregate counts shown on the public home page.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PublicStatistics {
    pub players: i64,
    pub maps: i64,
    pub courses: i64,
    pub records: i64,
    pub world_records: i64,
    pub servers_online: i64,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PublicMode {
    pub id: i16,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PublicModesResponse {
    pub items: Vec<PublicMode>,
}
