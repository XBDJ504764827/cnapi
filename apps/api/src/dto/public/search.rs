//! Public global search.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::common::{MapRef, PlayerRef};

#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct SearchQuery {
    /// Display name, SteamID64 or map name (2-64 characters).
    pub q: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchResponse {
    pub query: String,
    pub players: Vec<PlayerRef>,
    pub maps: Vec<MapRef>,
}
