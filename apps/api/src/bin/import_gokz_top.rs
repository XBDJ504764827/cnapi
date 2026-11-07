//! 一次性导入 gokz.top 的数据（地图信息 / 地图等级 / 全部完成记录）。
//!
//! 只用于补齐历史数据；导入后站点仍以 kzgo.eu 为主数据源，本程序不需要接进
//! 常驻 worker，也不应该被反复运行（重复运行是幂等的，靠 external_ref 去重）。
//!
//! 用法：
//!   DATABASE_URL=postgres://... cargo run --release --bin import-gokz-top -- \
//!       [--only=maps|records|all] [--dry-run] [--max-maps=N] [--api=URL]
//!
//! 说明：
//! * gokz.top 的跳数（/v1/jumpstats）没有地图字段，而 `jumpstats.map_id` 是
//!   NOT NULL，因此跳数**不导入**，避免编造关联。
//! * 等级只填空白：已有等级（来自 kzgo.eu）不会被覆盖。
//! * 记录用 `external_ref = gokz.top 的 record uuid` 去重，可安全重跑。

use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::Deserialize;
use sqlx::{PgPool, postgres::PgPoolOptions};

const DEFAULT_API: &str = "https://api.gokz.top/v1";
const DEFAULT_PAGE: usize = 100;
const SOURCE: &str = "gokz_top_import";
/// 累积到这么多行再提交一次 INSERT。
const FLUSH_ROWS: usize = 1000;

#[derive(Debug)]
struct Args {
    api: String,
    only: String,
    dry_run: bool,
    max_maps: Option<usize>,
    page: usize,
    /// 同时抓取的地图数。单页上限 100 条、单页约 2 秒，串行要十几小时。
    concurrency: usize,
    /// 跳过已经导入过的地图，便于中断后继续。
    resume: bool,
    /// 重建世界纪录指针时无条件覆盖（用于清掉历史错误数据后重算）。
    rebuild_wrs: bool,
    /// Per-map cap; the leaderboard is ordered by time, so this is "top N".
    max_per_map: Option<usize>,
    /// Rebuild the world-record pointers only, without fetching anything.
    wrs_only: bool,
}

impl Args {
    fn parse() -> Self {
        let mut args = Self {
            api: DEFAULT_API.to_owned(),
            only: "all".to_owned(),
            dry_run: false,
            max_maps: None,
            page: DEFAULT_PAGE,
            concurrency: 24,
            resume: true,
            rebuild_wrs: false,
            max_per_map: None,
            wrs_only: false,
        };
        for raw in std::env::args().skip(1) {
            if let Some(value) = raw.strip_prefix("--api=") {
                args.api = value.trim_end_matches('/').to_owned();
            } else if let Some(value) = raw.strip_prefix("--only=") {
                args.only = value.to_owned();
            } else if let Some(value) = raw.strip_prefix("--max-maps=") {
                args.max_maps = value.parse().ok();
            } else if let Some(value) = raw.strip_prefix("--page=") {
                args.page = value.parse().unwrap_or(DEFAULT_PAGE).clamp(10, 500);
            } else if let Some(value) = raw.strip_prefix("--concurrency=") {
                args.concurrency = value.parse().unwrap_or(24).clamp(1, 32);
            } else if let Some(value) = raw.strip_prefix("--max-per-map=") {
                args.max_per_map = value.parse().ok();
            } else if raw == "--wrs-only" {
                args.wrs_only = true;
            } else if raw == "--rebuild-wrs" {
                args.rebuild_wrs = true;
            } else if raw == "--no-resume" {
                args.resume = false;
            } else if raw == "--dry-run" {
                args.dry_run = true;
            }
        }
        args
    }
}

#[derive(Deserialize)]
struct GokzMap {
    id: i64,
    name: String,
    workshop_id: Option<i64>,
    bonus_count: Option<i32>,
    tiers: Option<HashMap<String, i16>>,
}

#[derive(Deserialize)]
struct LeaderboardPage {
    count: i64,
    data: Vec<LeaderboardItem>,
}

#[derive(Deserialize)]
struct LeaderboardItem {
    uuid: String,
    player: GokzPlayer,
    map_name: String,
    stage: i32,
    mode: String,
    tickrate: Option<i16>,
    time: f64,
    teleports: i32,
    created_on: String,
    #[serde(default)]
    is_valid: bool,
}

#[derive(Deserialize)]
struct GokzPlayer {
    steamid64: String,
}

/// 一行待插入的记录（已完成本地 id 解析）。
struct PendingRecord {
    account_id: i64,
    map_id: i64,
    course_id: i64,
    mode_id: i16,
    style_id: i16,
    tickrate: i16,
    teleports: i32,
    time_type: &'static str,
    time_ns: i64,
    external_ref: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL is required")?;

    println!(
        "api={} only={} dry_run={}",
        args.api, args.only, args.dry_run
    );

    let pool = PgPoolOptions::new()
        .max_connections(32)
        .connect(&database_url)
        .await
        .context("connect database")?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .user_agent(concat!("CNGOKZAPI-import/", env!("CARGO_PKG_VERSION")))
        .build()?;

    let mut maps: Vec<GokzMap> = fetch_all_maps(&client, &args).await?;
    if let Some(limit) = args.max_maps {
        maps.truncate(limit);
    }
    println!("fetched {} maps from gokz.top", maps.len());

    if args.only == "maps" || args.only == "all" {
        sync_map_metadata(&pool, &maps, args.dry_run).await?;
    }
    if args.wrs_only {
        refresh_world_records(&pool, true).await?;
    } else if args.only == "records" || args.only == "all" {
        sync_records(&pool, &client, &maps, &args).await?;
        if !args.dry_run {
            refresh_world_records(&pool, args.rebuild_wrs).await?;
        }
    }

    println!("done");
    Ok(())
}

async fn fetch_all_maps(client: &reqwest::Client, args: &Args) -> Result<Vec<GokzMap>> {
    let mut all = Vec::new();
    let mut offset = 0usize;
    loop {
        let url = format!("{}/maps?limit={}&offset={}", args.api, args.page, offset);
        let batch: Vec<GokzMap> = client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("GET {url}"))?
            .error_for_status()?
            .json()
            .await
            .context("decode maps page")?;
        let received = batch.len();
        all.extend(batch);
        if received < args.page {
            break;
        }
        offset += received;
        if offset.is_multiple_of(1000) {
            println!("  … {offset} maps");
        }
    }
    Ok(all)
}

/// 写入地图基础信息：workshop_id、主线课程、bonus 课程、四套等级（只补空白）。
async fn sync_map_metadata(pool: &PgPool, maps: &[GokzMap], dry_run: bool) -> Result<()> {
    let modes: HashMap<String, i16> =
        sqlx::query_as::<_, (String, i16)>("SELECT code, id FROM movement_modes")
            .fetch_all(pool)
            .await?
            .into_iter()
            .collect();

    let mut created_maps = 0usize;
    let mut updated_workshop = 0usize;
    let mut created_courses = 0usize;
    let mut filled_tiers = 0usize;

    for map in maps {
        if dry_run {
            continue;
        }

        // 名称唯一（lower(name)），存在则不动
        sqlx::query(
            r#"
            INSERT INTO maps (name, workshop_id, status, approved_at)
            VALUES ($1, $2, 'approved', now())
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(&map.name)
        .bind(map.workshop_id)
        .execute(pool)
        .await
        .with_context(|| format!("insert map {}", map.name))?;

        let map_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM maps WHERE lower(name) = lower($1) AND deleted_at IS NULL",
        )
        .bind(&map.name)
        .fetch_optional(pool)
        .await?;
        let Some(map_id) = map_id else {
            // workshop_id 冲突等极端情况下没插进去，跳过这张图
            eprintln!("  ! skip map {} (insert conflict)", map.name);
            continue;
        };
        created_maps += 1;

        // workshop_id 只补空值，且避免撞唯一索引
        if let Some(workshop_id) = map.workshop_id {
            let updated = sqlx::query(
                r#"
                UPDATE maps SET workshop_id = $2
                 WHERE id = $1 AND workshop_id IS NULL
                   AND NOT EXISTS (
                       SELECT 1 FROM maps WHERE workshop_id = $2 AND id <> $1 AND deleted_at IS NULL
                   )
                "#,
            )
            .bind(map_id)
            .bind(workshop_id)
            .execute(pool)
            .await?;
            updated_workshop += updated.rows_affected() as usize;
        }

        // 主线课程
        let main_course: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO map_courses (map_id, stage, name) VALUES ($1, 0, 'Main')
            ON CONFLICT (map_id, stage) DO UPDATE SET name = map_courses.name
            RETURNING id
            "#,
        )
        .bind(map_id)
        .fetch_one(pool)
        .await?;

        // bonus 课程
        let bonus = map.bonus_count.unwrap_or(0).clamp(0, 32);
        for stage in 1..=bonus {
            let inserted = sqlx::query(
                r#"
                INSERT INTO map_courses (map_id, stage, name)
                VALUES ($1, $2, 'Bonus ' || $2)
                ON CONFLICT (map_id, stage) DO NOTHING
                "#,
            )
            .bind(map_id)
            .bind(stage as i16)
            .execute(pool)
            .await?;
            created_courses += inserted.rows_affected() as usize;
        }

        // 等级：OVR 不是一种移动模式，跳过；其余三种映射到我们的 mode code
        if let Some(tiers) = map.tiers.as_ref() {
            for (gokz_mode, code) in [("KZT", "kztimer"), ("SKZ", "simple"), ("VNL", "vanilla")] {
                let Some(tier) = tiers.get(gokz_mode).copied() else {
                    continue;
                };
                if !(1..=10).contains(&tier) {
                    continue;
                }
                let Some(mode_id) = modes.get(code).copied() else {
                    continue;
                };
                let result = sqlx::query(
                    r#"
                    INSERT INTO course_tiers (course_id, mode_id, nub_tier, pro_tier, status)
                    VALUES ($1, $2, $3, $3, 'ranked')
                    ON CONFLICT (course_id, mode_id) DO UPDATE
                       SET nub_tier = COALESCE(course_tiers.nub_tier, EXCLUDED.nub_tier),
                           pro_tier = COALESCE(course_tiers.pro_tier, EXCLUDED.pro_tier),
                           status = CASE WHEN course_tiers.status = 'unranked'
                                         THEN 'ranked' ELSE course_tiers.status END,
                           updated_at = now()
                    "#,
                )
                .bind(main_course)
                .bind(mode_id)
                .bind(tier)
                .execute(pool)
                .await?;
                filled_tiers += result.rows_affected() as usize;
            }
        }
    }

    println!(
        "maps: processed={created_maps} workshop_updated={updated_workshop} bonus_courses={created_courses} tiers_unchanged_or_filled={filled_tiers}"
    );
    Ok(())
}

/// 共享缓存：账号、课程、地图名 → id。并发任务通过 Mutex 复用。
#[derive(Default)]
struct Caches {
    accounts: HashMap<String, i64>,
    courses: HashMap<(i64, i16), i64>,
    map_ids: HashMap<String, i64>,
    modes: HashMap<String, i16>,
    style_id: i16,
}

async fn sync_records(
    pool: &PgPool,
    client: &reqwest::Client,
    maps: &[GokzMap],
    args: &Args,
) -> Result<()> {
    let caches = Caches {
        modes: sqlx::query_as::<_, (String, i16)>("SELECT code, id FROM movement_modes")
            .fetch_all(pool)
            .await?
            .into_iter()
            .collect(),
        style_id: sqlx::query_scalar("SELECT id FROM movement_styles WHERE code = 'normal'")
            .fetch_one(pool)
            .await?,
        ..Default::default()
    };
    let mut caches = caches;
    for (steam_id, id) in sqlx::query_as::<_, (String, i64)>(
        "SELECT steam_id64::text, id FROM accounts WHERE deleted_at IS NULL",
    )
    .fetch_all(pool)
    .await?
    {
        caches.accounts.insert(steam_id, id);
    }
    for (map_name, map_id) in
        sqlx::query_as::<_, (String, i64)>("SELECT name, id FROM maps WHERE deleted_at IS NULL")
            .fetch_all(pool)
            .await?
    {
        caches.map_ids.insert(map_name.to_lowercase(), map_id);
    }
    for (map_id, stage, course_id) in
        sqlx::query_as::<_, (i64, i16, i64)>("SELECT map_id, stage, id FROM map_courses")
            .fetch_all(pool)
            .await?
    {
        caches.courses.insert((map_id, stage), course_id);
    }

    let caches = std::sync::Arc::new(tokio::sync::Mutex::new(caches));
    let client = client.clone();
    let pool = pool.clone();
    let api = args.api.clone();
    let page_size = args.page;
    let dry_run = args.dry_run;
    let resume = args.resume;
    let max_per_map = args.max_per_map;

    let total_maps = maps.len();
    let mut processed = 0usize;
    let mut total_rows = 0usize;
    let mut results = tokio::task::JoinSet::new();
    let mut index = 0usize;
    let mut inflight = 0usize;

    while index < maps.len() || inflight > 0 {
        while inflight < args.concurrency && index < maps.len() {
            let map = GokzMap {
                id: maps[index].id,
                name: maps[index].name.clone(),
                workshop_id: maps[index].workshop_id,
                bonus_count: maps[index].bonus_count,
                tiers: None,
            };
            index += 1;
            inflight += 1;
            let pool = pool.clone();
            let client = client.clone();
            let caches = caches.clone();
            let api = api.clone();
            results.spawn(async move {
                let rows = import_one_map(
                    &pool,
                    &client,
                    &caches,
                    &map,
                    &api,
                    page_size,
                    dry_run,
                    resume,
                    max_per_map,
                )
                .await
                .unwrap_or_else(|error| {
                    eprintln!("  ! {} failed: {error}", map.name);
                    0
                });
                (map.name, rows)
            });
        }

        if let Some(joined) = results.join_next().await {
            inflight -= 1;
            let (name, rows) = joined.unwrap_or_else(|error| {
                eprintln!("  ! task panicked: {error}");
                (String::from("?"), 0)
            });
            processed += 1;
            total_rows += rows;
            if rows > 0 || processed.is_multiple_of(50) {
                println!("  [{processed}/{total_maps}] {name} -> {rows} rows (total {total_rows})");
            }
        }
    }

    println!("records: rows={total_rows}");
    Ok(())
}

/// 拉一页排行榜，失败重试 3 次（gokz.top 偶发超时与截断）。
async fn fetch_page(
    client: &reqwest::Client,
    api: &str,
    map_id: i64,
    page_size: usize,
    offset: usize,
) -> Result<LeaderboardPage> {
    let url = format!("{api}/maps/{map_id}/leaderboard?limit={page_size}&offset={offset}");
    let mut last_error = None;
    for attempt in 1..=3 {
        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => match response.json().await {
                Ok(page) => return Ok(page),
                Err(error) => last_error = Some(anyhow::anyhow!("decode {url}: {error}")),
            },
            Ok(response) => {
                last_error = Some(anyhow::anyhow!("{url} -> {}", response.status()));
            }
            Err(error) => last_error = Some(anyhow::anyhow!("GET {url}: {error}")),
        }
        if attempt < 3 {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("unknown error")))
}

/// 导入单张地图的全部完成记录。返回写入行数。
///
/// 先取第一页拿到 `count`，再决定是否跳过：只有「已导入行数 ≥ 上游总数」才算做完，
/// 这样上次中断在半路的地图会被重新拉一遍（`external_ref` 保证不会重复写入）。
#[allow(clippy::too_many_arguments)]
async fn import_one_map(
    pool: &PgPool,
    client: &reqwest::Client,
    caches: &tokio::sync::Mutex<Caches>,
    map: &GokzMap,
    api: &str,
    page_size: usize,
    dry_run: bool,
    resume: bool,
    max_per_map: Option<usize>,
) -> Result<usize> {
    let our_map_id = {
        let guard = caches.lock().await;
        guard.map_ids.get(&map.name.to_lowercase()).copied()
    };
    let Some(our_map_id) = our_map_id else {
        return Ok(0);
    };

    let mut offset = 0usize;
    let mut written = 0usize;
    let mut checked_resume = false;
    let mut buffer: Vec<PendingRecord> = Vec::with_capacity(FLUSH_ROWS * 2);

    'pages: loop {
        let page = fetch_page(client, api, map.id, page_size, offset).await?;
        if page.data.is_empty() {
            break;
        }

        if !checked_resume {
            checked_resume = true;
            if resume && !dry_run {
                let existing: i64 = sqlx::query_scalar(
                    "SELECT count(*) FROM records WHERE map_id = $1 AND source = $2",
                )
                .bind(our_map_id)
                .bind(SOURCE)
                .fetch_one(pool)
                .await?;
                let target = match max_per_map {
                    Some(max) => page.count.min(max as i64),
                    None => page.count,
                };
                if existing >= target {
                    return Ok(0);
                }
            }
        }

        for item in &page.data {
            if let Some(max) = max_per_map
                && written + buffer.len() >= max
            {
                break 'pages;
            }
            with_record(&mut buffer, item, our_map_id, caches, pool, dry_run).await?;
        }
        if buffer.len() >= FLUSH_ROWS {
            if !dry_run {
                insert_records(pool, &buffer).await?;
            }
            written += buffer.len();
            buffer.clear();
        }

        offset += page.data.len();
        if offset as i64 >= page.count {
            break;
        }
    }

    if !buffer.is_empty() {
        if !dry_run {
            insert_records(pool, &buffer).await?;
        }
        written += buffer.len();
    }

    Ok(written)
}

#[allow(clippy::too_many_arguments)]
async fn with_record(
    pending: &mut Vec<PendingRecord>,
    item: &LeaderboardItem,
    our_map_id: i64,
    caches: &tokio::sync::Mutex<Caches>,
    pool: &PgPool,
    dry_run: bool,
) -> Result<()> {
    if !item.is_valid {
        return Ok(());
    }

    let (mode_id, style_id, map_id, known_course) = {
        let guard = caches.lock().await;
        let Some(&mode_id) = guard.modes.get(match item.mode.as_str() {
            "KZT" => "kztimer",
            "SKZ" => "simple",
            "VNL" => "vanilla",
            _ => return Ok(()),
        }) else {
            return Ok(());
        };
        let Some(&map_id) = guard.map_ids.get(&item.map_name.to_lowercase()) else {
            return Ok(());
        };
        let known = guard.courses.get(&(map_id, item.stage as i16)).copied();
        (mode_id, guard.style_id, map_id, known)
    };
    let _ = our_map_id;

    let stage = item.stage as i16;
    let course_id = match known_course {
        Some(id) => id,
        None if dry_run => 0,
        None => {
            let id: i64 = sqlx::query_scalar(
                r#"
                INSERT INTO map_courses (map_id, stage, name)
                VALUES ($1, $2, CASE WHEN $2 = 0 THEN 'Main' ELSE 'Bonus ' || $2 END)
                ON CONFLICT (map_id, stage) DO UPDATE SET name = map_courses.name
                RETURNING id
                "#,
            )
            .bind(map_id)
            .bind(stage)
            .fetch_one(pool)
            .await?;
            caches.lock().await.courses.insert((map_id, stage), id);
            id
        }
    };

    let known_account = caches
        .lock()
        .await
        .accounts
        .get(&item.player.steamid64)
        .copied();
    let account_id = match known_account {
        Some(id) => id,
        None if dry_run => 0,
        None => {
            let id: i64 = sqlx::query_scalar(
                r#"
                INSERT INTO accounts (steam_id64) VALUES ($1::numeric)
                ON CONFLICT (steam_id64) WHERE deleted_at IS NULL
                DO UPDATE SET updated_at = now()
                RETURNING id
                "#,
            )
            .bind(&item.player.steamid64)
            .fetch_one(pool)
            .await?;
            caches
                .lock()
                .await
                .accounts
                .insert(item.player.steamid64.clone(), id);
            id
        }
    };

    let time_type = if item.teleports == 0 { "pro" } else { "nub" };
    let time_ns = (item.time * 1e9).round().max(1.0) as i64;
    let created_at = chrono::DateTime::parse_from_rfc3339(&item.created_on)
        .map(|value| value.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now());

    pending.push(PendingRecord {
        account_id,
        map_id,
        course_id,
        mode_id,
        style_id,
        tickrate: item.tickrate.unwrap_or(128).clamp(1, 1000),
        teleports: item.teleports,
        time_type,
        time_ns,
        external_ref: item.uuid.clone(),
        created_at,
    });
    Ok(())
}

async fn insert_records(pool: &PgPool, rows: &[PendingRecord]) -> Result<()> {
    let account_ids: Vec<i64> = rows.iter().map(|row| row.account_id).collect();
    let map_ids: Vec<i64> = rows.iter().map(|row| row.map_id).collect();
    let course_ids: Vec<i64> = rows.iter().map(|row| row.course_id).collect();
    let mode_ids: Vec<i16> = rows.iter().map(|row| row.mode_id).collect();
    let style_ids: Vec<i16> = rows.iter().map(|row| row.style_id).collect();
    let tickrates: Vec<i16> = rows.iter().map(|row| row.tickrate).collect();
    let teleports: Vec<i32> = rows.iter().map(|row| row.teleports).collect();
    let time_types: Vec<&str> = rows.iter().map(|row| row.time_type).collect();
    let time_ns: Vec<i64> = rows.iter().map(|row| row.time_ns).collect();
    let external_refs: Vec<&str> = rows.iter().map(|row| row.external_ref.as_str()).collect();
    let created_at: Vec<chrono::DateTime<chrono::Utc>> =
        rows.iter().map(|row| row.created_at).collect();

    sqlx::query(
        r#"
        INSERT INTO records (
            account_id, map_id, course_id, mode_id, style_id,
            tickrate, teleports, time_type, time_ns,
            status, source, external_ref, created_at
        )
        SELECT account_id, map_id, course_id, mode_id, style_id,
               tickrate, teleports, time_type, time_ns,
               'approved', $12, external_ref, created_at
          FROM unnest(
                 $1::bigint[], $2::bigint[], $3::bigint[], $4::smallint[], $5::smallint[],
                 $6::smallint[], $7::int[], $8::text[], $9::bigint[], $10::text[],
                 $11::timestamptz[]
               ) AS t(account_id, map_id, course_id, mode_id, style_id,
                      tickrate, teleports, time_type, time_ns, external_ref, created_at)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(&account_ids)
    .bind(&map_ids)
    .bind(&course_ids)
    .bind(&mode_ids)
    .bind(&style_ids)
    .bind(&tickrates)
    .bind(&teleports)
    .bind(&time_types)
    .bind(&time_ns)
    .bind(&external_refs)
    .bind(&created_at)
    .bind(SOURCE)
    .execute(pool)
    .await
    .context("batch insert records")?;
    Ok(())
}

/// 用导入后的数据重算世界纪录指针（只会变快，不会变慢）。
async fn refresh_world_records(pool: &PgPool, rebuild: bool) -> Result<()> {
    let result = sqlx::query(
        r#"
        INSERT INTO course_world_records
            (course_id, mode_id, style_id, time_type, record_id, time_ns, achieved_at)
        SELECT DISTINCT ON (course_id, mode_id, style_id, time_type)
               course_id, mode_id, style_id, time_type, id, time_ns, created_at
          FROM records
         WHERE status = 'approved' AND deleted_at IS NULL
         ORDER BY course_id, mode_id, style_id, time_type, time_ns ASC, id ASC
        ON CONFLICT (course_id, mode_id, style_id, time_type) DO UPDATE
           SET record_id = EXCLUDED.record_id,
               time_ns = EXCLUDED.time_ns,
               achieved_at = EXCLUDED.achieved_at
         WHERE $1 OR EXCLUDED.time_ns < course_world_records.time_ns
        "#,
    )
    .bind(rebuild)
    .execute(pool)
    .await
    .context("refresh world records")?;
    println!(
        "world records refreshed: {} rows touched",
        result.rows_affected()
    );
    Ok(())
}
