//! Public read-only services.
//!
//! All queries return pages of at most 100 rows; aggregations are done in SQL.
//! Points/rating are intentionally not computed (no algorithm exists yet).

pub mod cache;

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use crate::dto::public::common::{
    PageMeta, PlayerRef, StyleRef, map_slug, parse_time_type_filter, seconds_string_3,
    validate_code, validate_search, validate_sort,
};
use crate::dto::public::leaderboards::{LeaderboardEntry, LeaderboardQuery, LeaderboardResponse};
use crate::dto::public::maps::{
    MapCourseDto, MapDetailQuery, MapDetailResponse, MapLeaderboardQuery, MapLeaderboardResponse,
    MapListQuery, MapListResponse, MapStatistics, MapSummary, ModeCount, TimeTypeCount,
};
use crate::dto::public::players::{
    CompareBest, CompareMapRow, ComparePlayerSide, CompareQuery, CompareResponse, CompareTier,
    PlayerDetailQuery, PlayerDetailResponse, PlayerListQuery, PlayerListResponse,
    PlayerMapProgress, PlayerMapsQuery, PlayerMapsResponse, PlayerModeStatsDto, PlayerOverallStats,
    PlayerProfileDto, PlayerRecordsQuery, PlayerRecordsResponse, PlayerStatScope, PlayerSummary,
    TierProgressDto,
};
use crate::dto::public::records::{
    ActivePlayer, ActivePlayersQuery, ActivePlayersResponse, RecentRecordsQuery,
    RecentRecordsResponse, RecentWorldRecordsQuery, RecordDetailResponse, RecordListResponse,
    RecordQuery, RecordSummary, ServerRef, WorldRecordEntry,
};
use crate::dto::public::search::{SearchQuery, SearchResponse};
use crate::dto::public::servers::{
    OnlinePlayerSummary, OnlinePlayersQuery, OnlinePlayersResponse, ServerListResponse,
    ServerQuery, ServerSummary,
};
use crate::dto::public::stats::{PublicModesResponse, PublicStatistics};
use crate::errors::AppError;
use crate::models::TimeType;
use crate::repositories::game_servers as game_server_repo;
use crate::repositories::public::directory::{self, ModeRow, StyleRow};
use crate::repositories::public::leaderboards;
use crate::repositories::public::maps as map_repo;
use crate::repositories::public::players as player_repo;
use crate::repositories::public::records::{self as record_repo, RecordFilters, RecordRow};
use crate::repositories::public::servers as server_repo;
use crate::repositories::public::stats as stats_repo;
use crate::repositories::sources as source_repo;
use crate::state::AppState;

pub const RANKING_METRIC: &str = "completed_maps";

const PLAYER_SORTS: &[&str] = &["completion", "wr", "records", "recent", "name"];
const RECORD_SORTS: &[&str] = &["uploaded", "time", "rank"];
const MAP_SORTS: &[&str] = &["completions", "recent", "name", "records"];
const PLAYER_MAP_SORTS: &[&str] = &["tier", "name", "time", "recent"];
const COMPARE_FILTERS: &[&str] = &[
    "all",
    "both",
    "left_only",
    "right_only",
    "neither",
    "left_faster",
    "right_faster",
];

async fn mode_or_default(state: &AppState, code: Option<&str>) -> Result<ModeRow, AppError> {
    let code = validate_code(code)?.unwrap_or_else(|| "vanilla".to_owned());
    if let Some(mode) = directory::find_mode(state.db(), &code).await? {
        return Ok(mode);
    }
    if code == "vanilla"
        && let Some(mode) = directory::default_mode(state.db()).await?
    {
        return Ok(mode);
    }
    Err(AppError::not_found(format!("movement mode `{code}`")))
}

async fn style_or_default(state: &AppState, code: Option<&str>) -> Result<StyleRow, AppError> {
    let code = validate_code(code)?.unwrap_or_else(|| "normal".to_owned());
    if let Some(style) = directory::find_style(state.db(), &code).await? {
        return Ok(style);
    }
    if code == "normal"
        && let Some(style) = directory::default_style(state.db()).await?
    {
        return Ok(style);
    }
    Err(AppError::not_found(format!("movement style `{code}`")))
}

fn time_type_label(time_type: Option<TimeType>) -> &'static str {
    match time_type {
        None => "all",
        Some(TimeType::Nub) => "nub",
        Some(TimeType::Pro) => "pro",
    }
}

fn parse_date(value: Option<&str>, field: &str) -> Result<Option<DateTime<Utc>>, AppError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    DateTime::parse_from_rfc3339(value)
        .map(|value| Some(value.with_timezone(&Utc)))
        .map_err(|_| AppError::validation(format!("`{field}` must be an RFC3339 timestamp")))
}

fn player_ref(
    account_id: i64,
    steam_id64: String,
    display_name: Option<String>,
    avatar_url: Option<String>,
) -> PlayerRef {
    PlayerRef {
        account_id,
        steam_id64,
        display_name: display_name.unwrap_or_else(|| "Unknown player".to_owned()),
        avatar_url,
    }
}

fn record_summary_from_row(row: RecordRow) -> RecordSummary {
    row.into_summary()
}

pub async fn statistics(state: &AppState) -> Result<PublicStatistics, AppError> {
    cache::get_or_load(state, "statistics", cache::TTL_STATISTICS, || async {
        let row = stats_repo::statistics(state.db()).await?;
        Ok(PublicStatistics {
            players: row.players,
            maps: row.maps,
            courses: row.courses,
            records: row.records,
            world_records: row.world_records,
            servers_online: row.servers_online,
            generated_at: Utc::now(),
        })
    })
    .await
}

pub async fn modes(state: &AppState) -> Result<PublicModesResponse, AppError> {
    cache::get_or_load(state, "modes", cache::TTL_MODES, || async {
        let rows = directory::list_modes(state.db()).await?;
        Ok(PublicModesResponse {
            items: rows
                .into_iter()
                .map(|row| crate::dto::public::stats::PublicMode {
                    id: row.id,
                    code: row.code,
                    display_name: row.display_name,
                    description: row.description,
                })
                .collect(),
        })
    })
    .await
}

pub async fn leaderboard(
    state: &AppState,
    query: &LeaderboardQuery,
) -> Result<LeaderboardResponse, AppError> {
    let mode = mode_or_default(state, query.mode.as_deref()).await?;
    let time_type = parse_time_type_filter(query.time_type.as_deref())?;
    let page = query.page_params()?;

    let key = format!(
        "leaderboard:{}:{}:{}:{}",
        mode.code,
        time_type_label(time_type),
        page.page,
        page.page_size
    );

    cache::get_or_load(state, &key, cache::TTL_LEADERBOARD, || async {
        let rows =
            leaderboards::leaderboard(state.db(), mode.id, time_type, page.limit, page.offset)
                .await?;
        let total = leaderboards::leaderboard_total(state.db(), mode.id, time_type).await?;
        let items = rows
            .into_iter()
            .map(|row| LeaderboardEntry {
                rank: row.rank_position,
                account_id: row.account_id,
                steam_id64: row.steam_id64,
                display_name: row.display_name,
                avatar_url: row.avatar_url,
                country: row.country,
                completed_maps: row.completed_maps,
                wr_count: row.wr_count,
                total_records: row.total_records,
                last_activity_at: row.last_activity_at,
                points: None,
            })
            .collect();

        Ok(LeaderboardResponse {
            mode: mode.to_ref(),
            time_type: time_type_label(time_type).to_owned(),
            ranking_metric: RANKING_METRIC.to_owned(),
            points_available: false,
            items,
            meta: PageMeta::from_total(total, page),
        })
    })
    .await
}

pub async fn players(
    state: &AppState,
    query: &PlayerListQuery,
) -> Result<PlayerListResponse, AppError> {
    let search = validate_search(query.search.as_deref())?;
    let sort = validate_sort(query.sort.as_deref(), PLAYER_SORTS, "completion")?;
    let page = query.page_params()?;
    let mode = match validate_code(query.mode.as_deref())? {
        Some(code) => Some(mode_or_default(state, Some(&code)).await?),
        None => None,
    };
    let time_type = None;

    let key = format!(
        "players:{}:{}:{}:{}:{}:{}",
        search.as_deref().unwrap_or(""),
        mode.as_ref()
            .map(|mode| mode.code.as_str())
            .unwrap_or("all"),
        sort,
        page.page,
        page.page_size,
        time_type_label(time_type)
    );

    cache::get_or_load(state, &key, cache::TTL_PLAYER_LIST, || async {
        let rows = leaderboards::player_directory(
            state.db(),
            search.as_deref(),
            mode.as_ref().map(|mode| mode.id),
            time_type,
            sort,
            page.limit,
            page.offset,
        )
        .await?;
        let total = leaderboards::player_directory_total(state.db(), search.as_deref()).await?;

        let items = rows
            .into_iter()
            .map(|row| PlayerSummary {
                rank: if row.total_records > 0 {
                    Some(row.rank_position)
                } else {
                    None
                },
                account_id: row.account_id,
                steam_id64: row.steam_id64,
                display_name: row.display_name,
                avatar_url: row.avatar_url,
                country: row.country,
                completed_maps: row.completed_maps,
                wr_count: row.wr_count,
                total_records: row.total_records,
                last_activity_at: row.last_activity_at,
                points: None,
            })
            .collect();

        Ok(PlayerListResponse {
            mode: mode.as_ref().map(ModeRow::to_ref),
            ranking_metric: RANKING_METRIC.to_owned(),
            points_available: false,
            items,
            meta: PageMeta::from_total(total, page),
        })
    })
    .await
}

fn profile_dto(
    player: &directory::PlayerLookupRow,
    first_record_at: Option<DateTime<Utc>>,
    last_record_at: Option<DateTime<Utc>>,
) -> PlayerProfileDto {
    PlayerProfileDto {
        account_id: player.account_id,
        steam_id64: player.steam_id64.clone(),
        display_name: player.display_name(),
        avatar_url: player.avatar_url.clone(),
        country: player.country.clone(),
        steam_profile_url: format!("https://steamcommunity.com/profiles/{}", player.steam_id64),
        first_record_at,
        last_record_at,
    }
}

async fn overall_stats(
    state: &AppState,
    account_id: i64,
    mode_id: Option<i16>,
    time_type: Option<TimeType>,
    style_id: Option<i16>,
    scoped: bool,
) -> Result<
    (
        PlayerOverallStats,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
    ),
    AppError,
> {
    let row =
        player_repo::scoped_overall(state.db(), account_id, mode_id, time_type, style_id).await?;
    let rank = if row.total_records > 0 {
        leaderboards::account_rank(state.db(), account_id, mode_id, time_type).await?
    } else {
        None
    };
    let completion_rate = if row.total_maps > 0 {
        row.completed_maps as f64 / row.total_maps as f64
    } else {
        0.0
    };

    Ok((
        PlayerOverallStats {
            scoped,
            global_rank: rank,
            completed_maps: row.completed_maps,
            total_maps: row.total_maps,
            completion_rate,
            wr_count: row.wr_count,
            pro_count: row.pro_count,
            tp_count: row.tp_count,
            total_records: row.total_records,
            points: None,
            rating: None,
        },
        row.first_record_at,
        row.last_record_at,
    ))
}

async fn mode_stats_dtos(
    state: &AppState,
    account_id: i64,
) -> Result<Vec<PlayerModeStatsDto>, AppError> {
    let rows = player_repo::mode_stats(state.db(), account_id).await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let rank = if row.total_records > 0 {
            leaderboards::account_rank(state.db(), account_id, Some(row.mode_id), None).await?
        } else {
            None
        };
        let completion_rate = if row.total_maps > 0 {
            row.completed_maps as f64 / row.total_maps as f64
        } else {
            0.0
        };
        items.push(PlayerModeStatsDto {
            mode: ModeRow {
                id: row.mode_id,
                code: row.code,
                display_name: row.display_name,
                description: None,
            }
            .to_ref(),
            rank,
            completed_maps: row.completed_maps,
            total_maps: row.total_maps,
            completion_rate,
            wr_count: row.wr_count,
            pro_count: row.pro_count,
            tp_count: row.tp_count,
            total_records: row.total_records,
            points: None,
        });
    }
    Ok(items)
}

pub async fn player_detail(
    state: &AppState,
    steam_id64: &str,
    query: &PlayerDetailQuery,
) -> Result<PlayerDetailResponse, AppError> {
    let player = directory::find_player(state.db(), steam_id64)
        .await?
        .ok_or_else(|| AppError::not_found("player"))?;
    let mode = mode_or_default(state, query.mode.as_deref()).await?;
    let time_type = parse_time_type_filter(query.time_type.as_deref())?;

    let key = format!(
        "player:{}:{}:{}",
        player.steam_id64,
        mode.code,
        time_type_label(time_type)
    );

    cache::get_or_load(state, &key, cache::TTL_PLAYER_DETAIL, || async {
        // `overall` is an explicitly unscoped lifetime summary; `scoped`, tiers
        // and map progress use the same mode/time_type/style scope.
        let (overall, first_record_at, last_record_at) =
            overall_stats(state, player.account_id, None, None, None, false).await?;
        let (scoped, _, _) = overall_stats(
            state,
            player.account_id,
            Some(mode.id),
            time_type,
            None,
            true,
        )
        .await?;
        let modes = mode_stats_dtos(state, player.account_id).await?;
        let tier_rows =
            player_repo::tier_progress(state.db(), player.account_id, mode.id, time_type, None)
                .await?;
        let mut tiers: Vec<TierProgressDto> = tier_rows
            .into_iter()
            .map(|row| {
                let uncompleted = (row.map_count - row.completed_count).max(0);
                TierProgressDto {
                    tier: row.tier,
                    map_count: row.map_count,
                    completed_count: row.completed_count,
                    uncompleted_count: uncompleted,
                    completion_rate: if row.map_count > 0 {
                        row.completed_count as f64 / row.map_count as f64
                    } else {
                        0.0
                    },
                    points: None,
                    average_points: None,
                }
            })
            .collect();
        tiers.sort_by_key(|tier| (tier.tier.is_none(), tier.tier));

        Ok(PlayerDetailResponse {
            profile: profile_dto(&player, first_record_at, last_record_at),
            overall,
            modes,
            tiers,
            scope: PlayerStatScope {
                mode: mode.to_ref(),
                style: StyleRef {
                    id: 0,
                    code: "all".to_owned(),
                    display_name: "全部 Style".to_owned(),
                },
                time_type: time_type_label(time_type).to_owned(),
                completion_definition: "主线课程（stage 0）存在已通过成绩；Bonus 成绩不计入完成数"
                    .to_owned(),
            },
            scoped,
            ranking_metric: RANKING_METRIC.to_owned(),
            points_available: false,
        })
    })
    .await
}

pub async fn player_records(
    state: &AppState,
    steam_id64: &str,
    query: &PlayerRecordsQuery,
) -> Result<PlayerRecordsResponse, AppError> {
    let player = directory::find_player(state.db(), steam_id64)
        .await?
        .ok_or_else(|| AppError::not_found("player"))?;
    let mode = match validate_code(query.mode.as_deref())? {
        Some(code) => Some(mode_or_default(state, Some(&code)).await?),
        None => None,
    };
    let style = match validate_code(query.style.as_deref())? {
        Some(code) => Some(style_or_default(state, Some(&code)).await?),
        None => None,
    };
    let time_type = parse_time_type_filter(query.time_type.as_deref())?;
    let page = query.page_params()?;

    let filters = RecordFilters {
        account_id: Some(player.account_id),
        mode_id: mode.as_ref().map(|mode| mode.id),
        style_id: style.as_ref().map(|style| style.id),
        time_type,
        ..Default::default()
    };

    let key = format!(
        "player-records:{}:{}:{}:{}:{}:{}",
        player.steam_id64,
        mode.as_ref()
            .map(|mode| mode.code.as_str())
            .unwrap_or("all"),
        style
            .as_ref()
            .map(|style| style.code.as_str())
            .unwrap_or("all"),
        time_type_label(time_type),
        page.page,
        page.page_size
    );

    cache::get_or_load(state, &key, cache::TTL_RECORD_LIST, || async {
        let rows =
            record_repo::list_records(state.db(), &filters, "uploaded", page.limit, page.offset)
                .await?;
        let total = record_repo::count_records(state.db(), &filters).await?;
        Ok(PlayerRecordsResponse {
            account: player_ref(
                player.account_id,
                player.steam_id64.clone(),
                Some(player.display_name()),
                player.avatar_url.clone(),
            ),
            items: rows.into_iter().map(record_summary_from_row).collect(),
            meta: PageMeta::from_total(total, page),
        })
    })
    .await
}

pub async fn player_maps(
    state: &AppState,
    steam_id64: &str,
    query: &PlayerMapsQuery,
) -> Result<PlayerMapsResponse, AppError> {
    let player = directory::find_player(state.db(), steam_id64)
        .await?
        .ok_or_else(|| AppError::not_found("player"))?;
    let mode = mode_or_default(state, query.mode.as_deref()).await?;
    let style = style_or_default(state, query.style.as_deref()).await?;
    let time_type = parse_time_type_filter(query.time_type.as_deref())?;
    let status = query.status.as_deref().unwrap_or("all");
    if !matches!(status, "all" | "completed" | "uncompleted") {
        return Err(AppError::validation(
            "`status` must be one of `all`, `completed`, `uncompleted`",
        ));
    }
    let sort = validate_sort(query.sort.as_deref(), PLAYER_MAP_SORTS, "tier")?;
    let search = validate_search(query.search.as_deref())?;
    let tier = match query.tier {
        Some(value) if !(0..=10).contains(&value) => {
            return Err(AppError::validation("`tier` must be between 0 and 10"));
        }
        value => value,
    };
    let page = query.page_params()?;

    let key = format!(
        "player-maps:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        player.steam_id64,
        mode.code,
        style.code,
        time_type_label(time_type),
        status,
        search.as_deref().unwrap_or(""),
        tier.map(|value| value.to_string())
            .unwrap_or_else(|| "all".to_owned()),
        sort,
        page.page,
        page.page_size
    );

    cache::get_or_load(state, &key, cache::TTL_PLAYER_DETAIL, || async {
        let rows = player_repo::map_progress(
            state.db(),
            player.account_id,
            mode.id,
            style.id,
            time_type,
            status,
            search.as_deref(),
            tier,
            sort,
            page.limit,
            page.offset,
        )
        .await?;
        let total = player_repo::map_progress_total(
            state.db(),
            player.account_id,
            mode.id,
            style.id,
            time_type,
            status,
            search.as_deref(),
            tier,
        )
        .await?;

        let items = rows
            .into_iter()
            .map(|row| PlayerMapProgress {
                map_id: row.map_id,
                slug: map_slug(&row.name),
                name: row.name,
                tier: row.tier,
                completed: row.completed,
                best_time_ns: row.time_ns,
                best_time_seconds: row.time_ns.map(seconds_string_3),
                best_rank: row.best_rank,
                best_time_type: row.best_time_type,
                last_record_at: row.last_record_at,
                bonus_total: row.bonus_total,
                bonus_completed: row.bonus_completed,
                points: None,
            })
            .collect();

        Ok(PlayerMapsResponse {
            account: player_ref(
                player.account_id,
                player.steam_id64.clone(),
                Some(player.display_name()),
                player.avatar_url.clone(),
            ),
            mode: mode.to_ref(),
            style: style.to_ref(),
            time_type: time_type_label(time_type).to_owned(),
            items,
            meta: PageMeta::from_total(total, page),
        })
    })
    .await
}

async fn compare_side(
    state: &AppState,
    account_id: i64,
    steam_id64: &str,
    display_name: Option<String>,
    country: Option<String>,
    avatar_url: Option<String>,
) -> Result<ComparePlayerSide, AppError> {
    let player = directory::PlayerLookupRow {
        account_id,
        steam_id64: steam_id64.to_owned(),
        display_name,
        country,
        avatar_url,
    };
    let (overall, first_record_at, last_record_at) =
        overall_stats(state, account_id, None, None, None, false).await?;
    let modes = mode_stats_dtos(state, account_id).await?;
    Ok(ComparePlayerSide {
        profile: profile_dto(&player, first_record_at, last_record_at),
        overall,
        modes,
    })
}

pub async fn compare(state: &AppState, query: &CompareQuery) -> Result<CompareResponse, AppError> {
    let left_param = query.left.trim();
    let right_param = query.right.trim();
    if left_param.is_empty() || right_param.is_empty() {
        return Err(AppError::validation("`left` and `right` are required"));
    }
    let left = directory::find_player(state.db(), left_param)
        .await?
        .ok_or_else(|| AppError::not_found("left player"))?;
    let right = directory::find_player(state.db(), right_param)
        .await?
        .ok_or_else(|| AppError::not_found("right player"))?;
    if left.account_id == right.account_id {
        return Err(AppError::validation("players must be different"));
    }

    let mode = mode_or_default(state, query.mode.as_deref()).await?;
    let style = style_or_default(state, query.style.as_deref()).await?;
    let time_type = parse_time_type_filter(query.time_type.as_deref())?;
    let filter = validate_sort(query.filter.as_deref(), COMPARE_FILTERS, "all")?;
    let page = query.page_params()?;

    let key = format!(
        "compare:{}:{}:{}:{}:{}:{}:{}:{}",
        left.steam_id64,
        right.steam_id64,
        mode.code,
        style.code,
        time_type_label(time_type),
        filter,
        page.page,
        page.page_size
    );

    cache::get_or_load(state, &key, cache::TTL_PLAYER_DETAIL, || async {
        let left_side = compare_side(
            state,
            left.account_id,
            &left.steam_id64,
            left.display_name.clone(),
            left.country.clone(),
            left.avatar_url.clone(),
        )
        .await?;
        let right_side = compare_side(
            state,
            right.account_id,
            &right.steam_id64,
            right.display_name.clone(),
            right.country.clone(),
            right.avatar_url.clone(),
        )
        .await?;

        let left_tiers = player_repo::tier_progress(
            state.db(),
            left.account_id,
            mode.id,
            time_type,
            Some(style.id),
        )
        .await?;
        let right_tiers = player_repo::tier_progress(
            state.db(),
            right.account_id,
            mode.id,
            time_type,
            Some(style.id),
        )
        .await?;
        let mut tiers: BTreeMap<Option<i16>, CompareTier> = BTreeMap::new();
        for row in &left_tiers {
            tiers.insert(
                row.tier,
                CompareTier {
                    tier: row.tier,
                    total_maps: row.map_count,
                    left_completed: row.completed_count,
                    right_completed: 0,
                    left_points: None,
                    right_points: None,
                },
            );
        }
        for row in &right_tiers {
            tiers
                .entry(row.tier)
                .and_modify(|tier| tier.right_completed = row.completed_count)
                .or_insert(CompareTier {
                    tier: row.tier,
                    total_maps: row.map_count,
                    left_completed: 0,
                    right_completed: row.completed_count,
                    left_points: None,
                    right_points: None,
                });
        }

        let rows = player_repo::compare_maps(
            state.db(),
            left.account_id,
            right.account_id,
            mode.id,
            style.id,
            time_type,
            filter,
            page.limit,
            page.offset,
        )
        .await?;
        let total = player_repo::compare_maps_total(
            state.db(),
            left.account_id,
            right.account_id,
            mode.id,
            style.id,
            time_type,
            filter,
        )
        .await?;

        let maps = rows
            .into_iter()
            .map(|row| {
                let left_best = CompareBest {
                    completed: row.left_time_ns.is_some(),
                    time_ns: row.left_time_ns,
                    time_seconds: row.left_time_ns.map(seconds_string_3),
                    rank: row.left_rank,
                    points: None,
                    last_record_at: row.left_record_at,
                };
                let right_best = CompareBest {
                    completed: row.right_time_ns.is_some(),
                    time_ns: row.right_time_ns,
                    time_seconds: row.right_time_ns.map(seconds_string_3),
                    rank: row.right_rank,
                    points: None,
                    last_record_at: row.right_record_at,
                };
                let status = match (left_best.completed, right_best.completed) {
                    (true, true) => "both",
                    (true, false) => "left_only",
                    (false, true) => "right_only",
                    (false, false) => "neither",
                };
                let time_diff_ns = match (row.left_time_ns, row.right_time_ns) {
                    (Some(left), Some(right)) => Some(left - right),
                    _ => None,
                };
                CompareMapRow {
                    map_id: row.map_id,
                    slug: map_slug(&row.name),
                    name: row.name,
                    tier: row.tier,
                    left: left_best,
                    right: right_best,
                    status: status.to_owned(),
                    time_diff_ns,
                    left_points_diff: None,
                }
            })
            .collect();

        Ok(CompareResponse {
            left: left_side,
            right: right_side,
            mode: mode.to_ref(),
            time_type: time_type_label(time_type).to_owned(),
            tiers: {
                let mut tiers: Vec<CompareTier> = tiers.into_values().collect();
                tiers.sort_by_key(|tier| (tier.tier.is_none(), tier.tier));
                tiers
            },
            filter: filter.to_owned(),
            maps,
            meta: PageMeta::from_total(total, page),
            ranking_metric: RANKING_METRIC.to_owned(),
            points_available: false,
        })
    })
    .await
}

pub async fn records(
    state: &AppState,
    query: &RecordQuery,
) -> Result<RecordListResponse, AppError> {
    let page = query.page_params()?;
    let mode = match validate_code(query.mode.as_deref())? {
        Some(code) => Some(mode_or_default(state, Some(&code)).await?),
        None => None,
    };
    let style = match validate_code(query.style.as_deref())? {
        Some(code) => Some(style_or_default(state, Some(&code)).await?),
        None => None,
    };
    let time_type = parse_time_type_filter(query.time_type.as_deref())?;
    let sort = validate_sort(query.sort.as_deref(), RECORD_SORTS, "uploaded")?;
    let player = validate_search(query.player.as_deref())?;
    let map = validate_search(query.map.as_deref())?;
    let from = parse_date(query.from.as_deref(), "from")?;
    let to = parse_date(query.to.as_deref(), "to")?;
    let tier = match query.tier {
        Some(value) if !(0..=10).contains(&value) => {
            return Err(AppError::validation("`tier` must be between 0 and 10"));
        }
        value => value,
    };

    let filters = RecordFilters {
        player_query: player.clone(),
        map_query: map.clone(),
        mode_id: mode.as_ref().map(|mode| mode.id),
        style_id: style.as_ref().map(|style| style.id),
        time_type,
        tier,
        world_record: query.world_record,
        from,
        to,
        ..Default::default()
    };

    let key = format!(
        "records:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        player.as_deref().unwrap_or(""),
        map.as_deref().unwrap_or(""),
        mode.as_ref()
            .map(|mode| mode.code.as_str())
            .unwrap_or("all"),
        style
            .as_ref()
            .map(|style| style.code.as_str())
            .unwrap_or("all"),
        time_type_label(time_type),
        tier.map(|value| value.to_string())
            .unwrap_or_else(|| "all".to_owned()),
        query
            .world_record
            .map(|value| value.to_string())
            .unwrap_or_else(|| "all".to_owned()),
        query.from.as_deref().unwrap_or(""),
        query.to.as_deref().unwrap_or(""),
        sort,
        page.page,
        page.page_size
    );

    cache::get_or_load(state, &key, cache::TTL_RECORD_LIST, || async {
        let rows =
            record_repo::list_records(state.db(), &filters, sort, page.limit, page.offset).await?;
        let total = record_repo::count_records(state.db(), &filters).await?;
        Ok(RecordListResponse {
            items: rows.into_iter().map(record_summary_from_row).collect(),
            meta: PageMeta::from_total(total, page),
        })
    })
    .await
}

pub async fn recent_records(
    state: &AppState,
    query: &RecentRecordsQuery,
) -> Result<RecentRecordsResponse, AppError> {
    let limit = query.resolved_limit()?;
    let mode = match validate_code(query.mode.as_deref())? {
        Some(code) => Some(mode_or_default(state, Some(&code)).await?),
        None => None,
    };
    let key = format!(
        "recent-records:{}:{}",
        limit,
        mode.as_ref()
            .map(|mode| mode.code.as_str())
            .unwrap_or("all")
    );

    cache::get_or_load(state, &key, cache::TTL_RECENT_RECORDS, || async {
        let filters = RecordFilters {
            mode_id: mode.as_ref().map(|mode| mode.id),
            ..Default::default()
        };
        let rows = record_repo::recent_records(state.db(), &filters, i64::from(limit)).await?;
        let items: Vec<RecordSummary> = rows.into_iter().map(record_summary_from_row).collect();
        Ok(RecentRecordsResponse {
            count: items.len(),
            items,
            limit,
        })
    })
    .await
}

pub async fn recent_world_records(
    state: &AppState,
    query: &RecentWorldRecordsQuery,
) -> Result<Vec<WorldRecordEntry>, AppError> {
    let limit = query.resolved_limit()?;
    let mode = match validate_code(query.mode.as_deref())? {
        Some(code) => Some(mode_or_default(state, Some(&code)).await?),
        None => None,
    };
    let time_type = parse_time_type_filter(query.time_type.as_deref())?;
    let key = format!(
        "world-records:{}:{}:{}",
        limit,
        mode.as_ref()
            .map(|mode| mode.code.as_str())
            .unwrap_or("all"),
        time_type_label(time_type)
    );

    cache::get_or_load(state, &key, cache::TTL_WORLD_RECORDS, || async {
        let rows = record_repo::recent_world_records(
            state.db(),
            mode.as_ref().map(|mode| mode.id),
            time_type,
            i64::from(limit),
        )
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| {
                let achieved_at = row.achieved_at.unwrap_or(row.created_at);
                WorldRecordEntry {
                    record: record_summary_from_row(row),
                    achieved_at,
                    previous_time_ns: None,
                    improvement_ns: None,
                    improvement_seconds: None,
                }
            })
            .collect())
    })
    .await
}

pub async fn active_players(
    state: &AppState,
    query: &ActivePlayersQuery,
) -> Result<ActivePlayersResponse, AppError> {
    let (days, limit) = query.resolve()?;
    cache::get_or_load(
        state,
        &format!("active-players:{days}:{limit}"),
        cache::TTL_ACTIVE_PLAYERS,
        || async {
            let rows = record_repo::active_players(state.db(), days, i64::from(limit)).await?;
            Ok(ActivePlayersResponse {
                items: rows
                    .into_iter()
                    .map(|row| ActivePlayer {
                        player: player_ref(
                            row.account_id,
                            row.steam_id64,
                            row.display_name,
                            row.avatar_url,
                        ),
                        records: row.records,
                        last_record_at: row.last_record_at,
                    })
                    .collect(),
                days,
            })
        },
    )
    .await
}

pub async fn record_detail(
    state: &AppState,
    record_id: i64,
) -> Result<RecordDetailResponse, AppError> {
    let row = record_repo::find_record(state.db(), record_id)
        .await?
        .ok_or_else(|| AppError::not_found("record"))?;
    let replay_id = row.replay_id;
    let server = row
        .server_id
        .zip(row.server_name.clone())
        .map(|(id, name)| ServerRef { id, name });
    Ok(RecordDetailResponse {
        record: record_summary_from_row(row),
        server,
        replay_id,
    })
}

pub async fn maps(state: &AppState, query: &MapListQuery) -> Result<MapListResponse, AppError> {
    let mode = mode_or_default(state, query.mode.as_deref()).await?;
    let time_type = parse_time_type_filter(query.time_type.as_deref())?;
    let search = validate_search(query.search.as_deref())?;
    let sort = validate_sort(query.sort.as_deref(), MAP_SORTS, "completions")?;
    let page = query.page_params()?;
    let tier = match query.tier {
        Some(value) if !(0..=10).contains(&value) => {
            return Err(AppError::validation("`tier` must be between 0 and 10"));
        }
        value => value,
    };

    let key = format!(
        "maps:{}:{}:{}:{}:{}:{}:{}",
        mode.code,
        time_type_label(time_type),
        search.as_deref().unwrap_or(""),
        tier.map(|value| value.to_string())
            .unwrap_or_else(|| "all".to_owned()),
        sort,
        page.page,
        page.page_size
    );

    cache::get_or_load(state, &key, cache::TTL_MAP_LIST, || async {
        let rows = map_repo::list_maps(
            state.db(),
            mode.id,
            time_type,
            search.as_deref(),
            tier,
            sort,
            page.limit,
            page.offset,
        )
        .await?;
        let total =
            map_repo::count_maps(state.db(), mode.id, time_type, search.as_deref(), tier).await?;

        Ok(MapListResponse {
            mode: mode.to_ref(),
            time_type: time_type_label(time_type).to_owned(),
            items: rows.into_iter().map(map_summary).collect(),
            meta: PageMeta::from_total(total, page),
        })
    })
    .await
}

fn map_summary(row: map_repo::MapRow) -> MapSummary {
    MapSummary {
        id: row.id,
        slug: map_slug(&row.name),
        name: row.name,
        workshop_id: row.workshop_id,
        description: row.description,
        tier: row.tier,
        courses: row.courses,
        completed_players: row.completed_players,
        records_count: row.records_count,
        wr_time_ns: row.wr_time_ns,
        wr_time_seconds: row.wr_time_ns.map(seconds_string_3),
        wr_player_name: row.wr_player_name,
        created_at: row.created_at,
    }
}

pub async fn map_detail(
    state: &AppState,
    slug: &str,
    query: &MapDetailQuery,
) -> Result<MapDetailResponse, AppError> {
    let map = directory::find_map_by_slug(state.db(), slug)
        .await?
        .ok_or_else(|| AppError::not_found("map"))?;
    let mode = mode_or_default(state, query.mode.as_deref()).await?;
    let time_type = parse_time_type_filter(query.time_type.as_deref())?;

    let key = format!(
        "map:{}:{}:{}",
        map.id,
        mode.code,
        time_type_label(time_type)
    );

    cache::get_or_load(state, &key, cache::TTL_MAP_DETAIL, || async {
        let row = map_repo::find_map_row(state.db(), map.id, mode.id, time_type)
            .await?
            .unwrap_or(map_repo::MapRow {
                id: map.id,
                name: map.name.clone(),
                workshop_id: map.workshop_id,
                description: map.description.clone(),
                tier: None,
                courses: 0,
                completed_players: 0,
                records_count: 0,
                wr_time_ns: None,
                wr_player_name: None,
                created_at: map.created_at,
                approved_at: map.approved_at,
            });

        let supported = map_repo::supported_modes(state.db(), map.id).await?;
        let courses = map_repo::map_courses(state.db(), map.id, mode.id, None).await?;
        let (stats, modes, time_types) =
            map_repo::map_statistics(state.db(), map.id, mode.id).await?;
        let completion_rate = if stats.total_players > 0 {
            stats.completed_players as f64 / stats.total_players as f64
        } else {
            0.0
        };

        Ok(MapDetailResponse {
            map: map_summary(row),
            supported_modes: supported.into_iter().map(|mode| mode.to_ref()).collect(),
            courses: courses
                .into_iter()
                .map(|course| MapCourseDto {
                    id: course.id,
                    stage: course.stage,
                    name: course.name,
                    nub_tier: course.nub_tier,
                    pro_tier: course.pro_tier,
                    completed_players: course.completed_players,
                    records_count: course.records_count,
                })
                .collect(),
            statistics: MapStatistics {
                total_players: stats.total_players,
                completed_players: stats.completed_players,
                pro_completed_players: stats.pro_completed_players,
                nub_completed_players: stats.nub_completed_players,
                completion_rate,
                record_count: stats.record_count,
                records_by_mode: modes
                    .into_iter()
                    .map(|row| ModeCount {
                        mode: ModeRow {
                            id: row.mode_id,
                            code: row.code,
                            display_name: row.display_name,
                            description: None,
                        }
                        .to_ref(),
                        records: row.records,
                    })
                    .collect(),
                records_by_time_type: time_types
                    .into_iter()
                    .map(|row| TimeTypeCount {
                        time_type: row.time_type,
                        records: row.records,
                    })
                    .collect(),
            },
            authors: None,
        })
    })
    .await
}

pub async fn map_leaderboard(
    state: &AppState,
    slug: &str,
    query: &MapLeaderboardQuery,
) -> Result<MapLeaderboardResponse, AppError> {
    let map = directory::find_map_by_slug(state.db(), slug)
        .await?
        .ok_or_else(|| AppError::not_found("map"))?;
    let mode = mode_or_default(state, query.mode.as_deref()).await?;
    let style = style_or_default(state, query.style.as_deref()).await?;
    let time_type = parse_time_type_filter(query.time_type.as_deref())?;
    let stage = query.stage.unwrap_or(0);
    if !(0..100).contains(&stage) {
        return Err(AppError::validation("`stage` must be between 0 and 99"));
    }
    let course = directory::find_course(state.db(), map.id, stage)
        .await?
        .ok_or_else(|| AppError::not_found("course"))?;
    let page = query.page_params()?;

    let key = format!(
        "map-leaderboard:{}:{}:{}:{}:{}:{}:{}",
        map.id,
        stage,
        mode.code,
        style.code,
        time_type_label(time_type),
        page.page,
        page.page_size
    );

    cache::get_or_load(state, &key, cache::TTL_LEADERBOARD, || async {
        let filters = RecordFilters {
            map_id: Some(map.id),
            course_stage: Some(stage),
            mode_id: Some(mode.id),
            style_id: Some(style.id),
            time_type,
            ..Default::default()
        };
        let rows = record_repo::list_records(state.db(), &filters, "time", page.limit, page.offset)
            .await?;
        let total = record_repo::count_records(state.db(), &filters).await?;
        Ok(MapLeaderboardResponse {
            map: map.to_ref(),
            course: crate::dto::public::common::CourseRef {
                id: course.id,
                stage: course.stage,
                name: course.name,
            },
            mode: mode.to_ref(),
            style: style.to_ref(),
            time_type: time_type_label(time_type).to_owned(),
            items: rows.into_iter().map(record_summary_from_row).collect(),
            meta: PageMeta::from_total(total, page),
        })
    })
    .await
}

pub async fn servers(
    state: &AppState,
    query: &ServerQuery,
) -> Result<ServerListResponse, AppError> {
    let page = query.page_params()?;
    let search = validate_search(query.search.as_deref())?;
    let max_age = state.config().source_fallback_max_age.as_secs() as i64;
    if source_repo::external_source_fresh(state.db(), max_age).await? {
        let external = source_repo::list_external_servers(state.db(), max_age).await?;
        if !external.is_empty() {
            let filtered: Vec<_> = external
                .into_iter()
                .filter(|server| {
                    query.online != Some(false)
                        && search.as_deref().is_none_or(|needle| {
                            server.name.to_lowercase().contains(&needle.to_lowercase())
                                || server.host.contains(needle)
                        })
                })
                .collect();
            let total = filtered.len() as i64;
            let start = page.offset as usize;
            let items = filtered
                .into_iter()
                .skip(start)
                .take(page.limit as usize)
                .map(|server| ServerSummary {
                    id: server.id,
                    name: server.name,
                    community: server.label,
                    ip: server.host,
                    port: server.port,
                    online: server.is_online,
                    status: if server.is_online {
                        "online"
                    } else {
                        "offline"
                    }
                    .to_owned(),
                    heartbeat_age_seconds: Some(
                        (Utc::now() - server.last_seen_at).num_seconds().max(0),
                    ),
                    current_map_id: None,
                    current_map_slug: server.map_name.as_deref().map(map_slug),
                    current_map_name: server.map_name,
                    plugin_version: None,
                    last_heartbeat_at: Some(server.last_seen_at),
                    players: Some(i64::from(server.online_players)),
                    max_players: Some(i64::from(server.max_players)),
                    mode: None,
                    region: None,
                    server_type: "external".to_owned(),
                    game: "csgo".to_owned(),
                })
                .collect();
            return Ok(ServerListResponse {
                items,
                meta: PageMeta::from_total(total, page),
            });
        }
    }
    let key = format!(
        "servers:{}:{}:{}:{}",
        search.as_deref().unwrap_or(""),
        query
            .online
            .map(|value| value.to_string())
            .unwrap_or_else(|| "all".to_owned()),
        page.page,
        page.page_size
    );

    cache::get_or_load(state, &key, cache::TTL_SERVER_LIST, || async {
        let rows = server_repo::list_servers(
            state.db(),
            search.as_deref(),
            query.online,
            page.limit,
            page.offset,
        )
        .await?;
        let total = server_repo::count_servers(state.db(), search.as_deref(), query.online).await?;
        Ok(ServerListResponse {
            items: rows.into_iter().map(server_summary).collect(),
            meta: PageMeta::from_total(total, page),
        })
    })
    .await
}

fn server_summary(row: server_repo::ServerRow) -> ServerSummary {
    let current_map_slug = row.map_slug();
    let heartbeat_age_seconds = row
        .last_heartbeat_at
        .map(|heartbeat| (Utc::now() - heartbeat).num_seconds().max(0));
    let status = match (row.online, row.last_heartbeat_at, heartbeat_age_seconds) {
        (true, Some(_), Some(age)) if age < 120 => "online",
        (false, Some(_), Some(age)) if age >= 120 => "expired",
        (false, Some(_), _) => "offline",
        (_, None, _) => "unknown",
        _ => "offline",
    };

    ServerSummary {
        id: row.id,
        name: row.name,
        community: row.community,
        ip: row.ip,
        port: row.port,
        online: row.online,
        status: status.to_owned(),
        heartbeat_age_seconds,
        current_map_id: row.current_map_id,
        current_map_slug,
        current_map_name: row.current_map_name,
        plugin_version: row.plugin_version,
        last_heartbeat_at: row.last_heartbeat_at,
        players: Some(i64::from(row.online_players)),
        max_players: Some(i64::from(row.max_players)),
        mode: row.current_mode,
        region: row.region,
        server_type: row.server_type,
        game: row.game,
    }
}

pub async fn online_players(
    state: &AppState,
    query: &OnlinePlayersQuery,
) -> Result<OnlinePlayersResponse, AppError> {
    let limit = query.limit.unwrap_or(100);
    if !(1..=200).contains(&limit) {
        return Err(AppError::validation("`limit` must be between 1 and 200"));
    }
    let rows =
        game_server_repo::public_online_players(state.db(), query.server_id, i64::from(limit))
            .await?;
    Ok(OnlinePlayersResponse {
        count: rows.len(),
        items: rows
            .into_iter()
            .map(|row| OnlinePlayerSummary {
                player: player_ref(row.account_id, row.steam_id64, Some(row.display_name), None),
                server_id: row.server_id,
                server_name: row.server_name,
                current_map_name: row.map_name,
                mode: row.mode_code,
                status: row.status,
                connected_at: row.connected_at,
                online_seconds: (Utc::now() - row.connected_at).num_seconds().max(0),
            })
            .collect(),
    })
}

pub async fn server_detail(state: &AppState, server_id: i64) -> Result<ServerSummary, AppError> {
    let row = server_repo::find_server(state.db(), server_id)
        .await?
        .ok_or_else(|| AppError::not_found("server"))?;
    Ok(server_summary(row))
}

pub async fn search(state: &AppState, query: &SearchQuery) -> Result<SearchResponse, AppError> {
    let q = query.q.trim();
    if q.chars().count() < 2 || q.chars().count() > 64 {
        return Err(AppError::validation(
            "`q` must be between 2 and 64 characters",
        ));
    }
    let key = format!("search:{}", q.to_lowercase());
    cache::get_or_load(state, &key, cache::TTL_SEARCH, || async {
        let players = directory::search_players(state.db(), q, 5).await?;
        let maps = directory::search_maps(state.db(), q, 5).await?;
        Ok(SearchResponse {
            query: q.to_owned(),
            players: players.into_iter().map(|row| row.to_ref()).collect(),
            maps: maps.into_iter().map(|row| row.to_ref()).collect(),
        })
    })
    .await
}

/// Exposed for phase 5 write paths and for diagnostics.
/// Best-effort cache invalidation. `false` means Redis was unavailable; the
/// caller must still report the data change as applied (PostgreSQL is truth).
pub async fn invalidate_cache(state: &AppState) -> bool {
    cache::invalidate_all(state).await
}

/// Re-populates the most frequently requested cache entry after invalidation.
pub async fn warm_statistics(state: &AppState) -> Result<(), AppError> {
    cache::warm(state, "statistics", cache::TTL_STATISTICS, || async {
        let row = stats_repo::statistics(state.db()).await?;
        Ok(PublicStatistics {
            players: row.players,
            maps: row.maps,
            courses: row.courses,
            records: row.records,
            world_records: row.world_records,
            servers_online: row.servers_online,
            generated_at: Utc::now(),
        })
    })
    .await
}

pub async fn cache_version_value(state: &AppState) -> String {
    cache::cache_version_value(state).await
}
