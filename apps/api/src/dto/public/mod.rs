//! Public API contracts (`/api/v1/public/*`).

pub mod common;
pub mod leaderboards;
pub mod maps;
pub mod players;
pub mod records;
pub mod search;
pub mod servers;
pub mod stats;

pub use common::{
    CourseRef, MapRef, ModeRef, PageMeta, PageParams, PageQuery, Paginated, PlayerRef, StyleRef,
};
pub use leaderboards::{LeaderboardEntry, LeaderboardQuery, LeaderboardResponse};
pub use maps::{
    MapCourseDto, MapDetailQuery, MapDetailResponse, MapLeaderboardQuery, MapLeaderboardResponse,
    MapListQuery, MapListResponse, MapStatistics, MapSummary, ModeCount, TimeTypeCount,
};
pub use players::{
    CompareBest, CompareMapRow, ComparePlayerSide, CompareQuery, CompareResponse, CompareTier,
    PlayerDetailQuery, PlayerDetailResponse, PlayerListQuery, PlayerListResponse,
    PlayerMapProgress, PlayerMapsQuery, PlayerMapsResponse, PlayerModeStatsDto, PlayerOverallStats,
    PlayerProfileDto, PlayerRecordsQuery, PlayerRecordsResponse, PlayerSummary, TierProgressDto,
};
pub use records::{
    ActivePlayer, ActivePlayersQuery, ActivePlayersResponse, RecentRecordsQuery,
    RecentRecordsResponse, RecentWorldRecordsQuery, RecordDetailResponse, RecordListResponse,
    RecordQuery, RecordSummary, ServerRef, WorldRecordEntry,
};
pub use search::{SearchQuery, SearchResponse};
pub use servers::{ServerListResponse, ServerQuery, ServerSummary};
pub use stats::{PublicModesResponse, PublicStatistics};
