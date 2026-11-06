//! Shared public API contracts: pagination, refs and validation helpers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::errors::AppError;
use crate::models::TimeType;

pub const MAX_PAGE_SIZE: u32 = 100;
pub const DEFAULT_PAGE_SIZE: u32 = 20;

#[derive(Debug, Clone, Copy, Default, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PageQuery {
    /// 1-based page number.
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    /// Page size (1-100).
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct PageParams {
    pub page: u32,
    pub page_size: u32,
    pub limit: i64,
    pub offset: i64,
}

impl PageQuery {
    pub fn resolve(&self) -> Result<PageParams, AppError> {
        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(DEFAULT_PAGE_SIZE);

        if page == 0 {
            return Err(AppError::validation("`page` must be at least 1"));
        }
        if page_size == 0 || page_size > MAX_PAGE_SIZE {
            return Err(AppError::validation(format!(
                "`page_size` must be between 1 and {MAX_PAGE_SIZE}"
            )));
        }

        Ok(PageParams {
            page,
            page_size,
            limit: i64::from(page_size),
            offset: i64::from((page - 1) * page_size),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PageMeta {
    pub page: u32,
    pub page_size: u32,
    pub total: i64,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub meta: PageMeta,
}

impl PageMeta {
    pub fn from_total(total: i64, page: PageParams) -> Self {
        let total_pages = if total <= 0 {
            0
        } else {
            (total as u64).div_ceil(u64::from(page.page_size)) as u32
        };
        Self {
            page: page.page,
            page_size: page.page_size,
            total,
            total_pages,
        }
    }
}

impl<T> Paginated<T> {
    pub fn new(items: Vec<T>, total: i64, page: PageParams) -> Self {
        Self {
            items,
            meta: PageMeta::from_total(total, page),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlayerRef {
    pub account_id: i64,
    /// Decimal string; SteamID64 exceeds JSON safe integer range.
    pub steam_id64: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MapRef {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CourseRef {
    pub id: i64,
    /// 0 = main course, >0 = bonus stage.
    pub stage: i16,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModeRef {
    pub id: i16,
    pub code: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StyleRef {
    pub id: i16,
    pub code: String,
    pub display_name: String,
}

/// Stable, URL-safe map slug. Map names are already lowercase and URL-safe in
/// the database; spaces and unsafe characters are replaced for safety.
pub fn map_slug(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    for character in name.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_matches('-').to_owned()
}

/// Nanoseconds -> decimal seconds string with 9 fraction digits.
pub fn seconds_string(time_ns: i64) -> String {
    let seconds = time_ns / 1_000_000_000;
    let nanos = time_ns.unsigned_abs() % 1_000_000_000;
    format!("{seconds}.{nanos:09}")
}

/// Nanoseconds -> decimal seconds string with 3 fraction digits (community style).
pub fn seconds_string_3(time_ns: i64) -> String {
    let rounded = (time_ns + 500_000) / 1_000_000;
    let seconds = rounded / 1_000;
    let millis = rounded.unsigned_abs() % 1_000;
    format!("{seconds}.{millis:03}")
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Timestamped {
    pub created_at: DateTime<Utc>,
}

/// Parses `time_type` query values (`nub`, `pro`, `all`).
pub fn parse_time_type_filter(value: Option<&str>) -> Result<Option<TimeType>, AppError> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        None | Some("all") => Ok(None),
        Some("nub") => Ok(Some(TimeType::Nub)),
        Some("pro") => Ok(Some(TimeType::Pro)),
        Some(other) => Err(AppError::validation(format!(
            "`time_type` must be one of `all`, `nub`, `pro`, got `{other}`"
        ))),
    }
}

pub fn validate_search(value: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if value.chars().count() > 64 {
        return Err(AppError::validation(
            "`search` must be at most 64 characters",
        ));
    }
    Ok(Some(value.to_owned()))
}

pub fn validate_code(value: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if value.len() > 32
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        })
    {
        return Err(AppError::validation("invalid code parameter"));
    }
    Ok(Some(value.to_owned()))
}

pub fn validate_sort<'a>(
    value: Option<&'a str>,
    allowed: &[&'a str],
    default: &'static str,
) -> Result<&'a str, AppError> {
    let value = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(default);
    if allowed.contains(&value) {
        Ok(value)
    } else {
        Err(AppError::validation(format!(
            "`sort` must be one of: {}",
            allowed.join(", ")
        )))
    }
}
