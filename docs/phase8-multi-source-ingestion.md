# 第八阶段：GOKZ-Global / kzgo.eu 多数据源接入与自有积分

> 本文件区分三类事实：**真实接通并验证**、**代码已实现但环境未验证**、**受上游接口限制无法完成**。

## 1. 数据接口调查结果

| 项目 | 调查结果 |
|---|---|
| GOKZ-Global 是否有公开 API | 有。官方 `GlobalAPI.inc` 指向 `https://kztimerglobal.com/api/v2.0`；实际是本项目所说的 GOKZ-Global / KZTimerGlobal API |
| 是否支持成绩增量查询 | 有 `records/top/recent`，支持 `created_since`、`offset`、`limit`（官方 native 与 kzgo.eu 前端均使用） |
| 是否支持历史成绩获取 | 支持按 map/stage/mode/tickrate/teleports 查询 `records/top`，可用 `limit` 拉取单 Filter 排行榜；没有发现“任意全量 dump”公开接口 |
| 是否可以获取完整原始成绩 | 不能确认存在一次性全量导出；需要分 Filter/玩家或 recent 增量拉取。完整历史回填受限于请求量与 rate limit |
| 是否提供稳定记录 ID | 有，record 响应包含 `id`；KZGO WR 接口不返回该 ID |
| 是否支持分页 | 支持 `offset` / `limit` |
| 是否存在请求频率限制 | 未找到公开固定数值；kzgo.eu 前端明确处理 429/过多请求，生产必须限速与退避 |
| 是否允许第三方数据使用 | 公开 GET 接口无认证即可访问，但未找到明确书面授权条款；生产前应确认使用许可与限流要求 |
| 是否有其他可靠接入方式 | 官方 GlobalAPI.inc native、kzgo.eu 自有 API；没有发现官方 Webhook |
| 本环境是否真实连通 GOKZ-Global | **否**：`kztimerglobal.com` 在本执行环境连接超时。代码适配器已实现，但只能标记为“未在线验证” |
| 本环境是否真实连通 kzgo.eu | **是**：真实拉取并写入 990 张地图、4892 条 WR、2 台在线服务器 |

### kzgo.eu 接口

| 接口 | 用途 | 数据完整度 |
|---|---|---|
| `GET /api/maps` | 地图、id、tier、Workshop、bonus 数、sp/vp、mapper | 实时验证 |
| `GET /api/wrs/kz_timer` | KZT 当前 WR（PRO/NUB） | 实时验证；只有当前 WR，不是全部成绩 |
| `GET /api/wrs/kz_simple` | SKZ 当前 WR | 实时验证 |
| `GET /api/wrs/kz_vanilla` | VNL 当前 WR | 实时验证 |
| `GET /api/servers` | 服务器在线状态、地图、人数 | 实时验证 |
| `GET /api/completions/{mode}` | 完成统计 | 已调查，未作为成绩来源 |
| `GET /api/steam/{steamid64}` | 头像/昵称/国家 | 未批量同步 |

**关键限制**：kzgo.eu 自有 API 不提供完整 records 历史。其前端通过 `kztimerglobal.com/api/v2.0/records/top*` 获取成绩。因此不能把 kzgo.eu 当作完整原始成绩来源。

---

## 2. 数据接入架构

```text
GOKZ-Global Adapter ─┐
                     │
kzgo.eu Adapter ─────┼──> Unified Record Ingestion
                     │          │
CNGOKZ Plugin ───────┘          ├─ raw_records（原始 payload）
                                ├─ record_source_links（来源溯源）
                                ├─ records（canonical 成绩）
                                └─ DB trigger → Filter recalc
                                        │
                                        ▼
                          CNGOKZ phase-6 scoring formula
                                        │
                                        ▼
                           player_filter_points / mode points
```

外部 `points` / `rank` 即使出现在原始 payload 中，也不会写入 CNGOKZ 积分字段，不参与公开积分。

---

## 3. 数据库模型

新增迁移：`apps/api/migrations/20261105000002_phase8_sources_and_scoring_audit.sql`

| 表/列 | 说明 |
|---|---|
| `record_sources` | source code/type、启用、优先级、健康状态、连续失败、cursor、config |
| `source_sync_runs` | 每次同步的类型、状态、统计、错误 |
| `raw_records` | 来源原始记录 ID、payload hash、原始 JSON、处理状态、canonical id |
| `record_source_links` | canonical 成绩 ↔ 来源，匹配方式、置信度、字段冲突 |
| `map_source_links` | 外部 map id ↔ 本地 canonical map |
| `external_servers` | kzgo.eu 服务器快照，公开服务器列表的优先展示源 |
| `records.source_server_name` | 外部成绩的服务器名称快照 |
| `player_filter_points.scoring_version/base_points/rank_factor/dist_points` | 自有积分计算审计 |

`records` 仍是唯一 canonical 成绩表，没有创建第二套计分成绩表。

---

## 4. 去重与冲突处理

1. **来源内去重**：`raw_records (source_id, external_record_id)` 唯一。
2. **跨来源强匹配**：来源稳定 record id 先关联；同一 canonical 记录只计分一次。
3. **高置信度匹配**：SteamID64、map、course、mode、style、time_type、teleports、time_ns 完全相同，且 `created_at` 相差 ≤ 600 秒。
4. **不匹配**：相同成绩但时间差超过窗口，保留为独立成绩，避免把玩家重复完成的相同时间静默合并。
5. **原始数据不覆盖**：`raw_records` 只追加；链接保存匹配方式。
6. **字段冲突**：schema 已提供 `field_conflicts JSONB`；当前适配器对完全相同字段的跨源匹配写空冲突，差异字段不会被静默覆盖。

已测试：KZGO 与 GOKZ-Global 同一条成绩 → 1 条 canonical、2 条 raw、2 条来源链接。

---

## 5. 自有积分引擎

- 复用第六阶段公式，未修改公式。
- Filter = map + course + mode + record_type。
- KZT/SKZ/VNL、NUB/PRO 独立。
- WR = 1000；PB 才计分；慢成绩不覆盖 WR。
- `player_filter_points` 新增审计列：`scoring_version`、`base_points`、`rank_factor`、`dist_points`。
- `scoring_version` 当前为 `phase6-v1`。
- 外部平台积分不参与计算。

---

## 6. 同步实现情况

### 已真实接通并执行

`KZGO Adapter`：

- 地图/tier/Workshop/bonus/course/mode 同步。
- 三模式当前 WR 同步。
- 服务器快照同步。
- 真实执行结果：990 maps、4892 WRs seen、4884 canonical records created、2 servers。
- 8 条 WR 因地图/Course 不匹配被跳过并计入 stats。

### 代码已实现但未在线验证

`GOKZ-Global Adapter`：

- 使用 `GOKZ_GLOBAL_BASE_URL`（默认 `https://kztimerglobal.com/api/v2.0`）。
- `records/top/recent?created_since=...&modes_list_string=...` 增量拉取。
- 支持 Steam2 → SteamID64、time_ns、teleports、created_on、server_name。
- 因当前环境到 `kztimerglobal.com` 连接超时，**未取得真实响应样本，未声明在线验收通过**。
- 默认 `GOKZ_GLOBAL_SYNC_ENABLED=false`，生产需确认网络与使用许可后启用。

### 仍然受限

- 没有发现 GlobalAPI 官方 Webhook。
- 没有发现一次性全量历史 dump。
- 完整历史回填需要按 Filter/玩家分页拉取，尚未实现独立 backfill worker。
- kzgo.eu 只能提供当前 WR，不能替代完整成绩历史。

---

## 7. 数据源健康与容灾

状态：

- `healthy`：最近同步成功。
- `degraded`：失败但未达到离线阈值。
- `offline`：连续失败 ≥ 3 次。
- `disabled`：管理员禁用。
- `unknown`：尚未同步。

公开服务器读取：

1. kzgo 数据源 `last_success_at` 在 `SOURCE_FALLBACK_MAX_AGE_SECS` 内且存在 `external_servers` 快照 → 使用 kzgo 服务器列表。
2. 外部源过期、失败或为空 → 回退到 CNGOKZ 自有 `servers` 心跳。
3. 两种来源都不可用 → 返回已有数据库数据，不清空。

成绩数据已写入 canonical 表；外部源故障不会删除已有成绩或积分。

---

## 8. 管理后台

新增 `/admin/sources`：

- 三个来源：`gokz_global`、`kzgo`、`cngokz_plugin`。
- 状态、最近成功/失败、连续失败、原始记录数、canonical 链接数、cursor。
- 启用/禁用。
- 手动触发同步（后台任务）。
- 同步运行记录与错误。
- 每条 canonical 成绩可查询来源审计：`GET /api/v1/admin/records/{id}/sources`。

---

## 9. API

```text
GET   /api/v1/admin/sources
PATCH /api/v1/admin/sources/{code}
GET   /api/v1/admin/sources/{code}/runs
POST  /api/v1/admin/sources/{code}/sync
GET   /api/v1/admin/records/{record_id}/sources
GET   /api/v1/admin/external-servers
```

管理接口沿用 `X-Maintenance-Token`。

---

## 10. 环境变量

```env
KZGO_BASE_URL=https://kzgo.eu
KZGO_SYNC_ENABLED=true
GOKZ_GLOBAL_BASE_URL=https://kztimerglobal.com/api/v2.0
GOKZ_GLOBAL_SYNC_ENABLED=false
SOURCE_SYNC_INTERVAL_SECS=300
SOURCE_HTTP_TIMEOUT_MS=15000
SOURCE_FALLBACK_MAX_AGE_SECS=900
```

---

## 11. 测试结果

自动测试（临时数据库 + 真实 PostgreSQL）：

- 共 79 个测试通过（含新增 source ingestion 2 个、unit 3 个）。
- KZGO/GOKZ 同成绩去重 + 来源审计。
- 相同时间但超出去重窗口 → 保留独立成绩。
- 数据源健康状态、同步运行、外部服务器快照。
- 真实 KZGO 时间戳与 Steam2 → SteamID64。
- 既有第六阶段积分、公开门户、服务器接入测试全部通过。

真实网络验证（`source_live`，`#[ignore]`）：

```bash
DATABASE_URL=... cargo test -p cngokz-api --test source_live -- --ignored --nocapture
```

结果：`success`；990 maps、4892 WR raw、4884 canonical、2 servers。

未验证：

- GOKZ-Global 真实响应（本环境连接超时）。
- 完整历史成绩回填。
- GitHub Actions 真实 runner 网络环境。
- 生产 R2/S3 Replay。

---

## 12. 生产部署步骤

1. 备份数据库。
2. 执行迁移：

```bash
cd apps/api
sqlx migrate run
```

3. 配置数据源环境变量。
4. 启动 API；source worker 按 interval 同步。
5. 打开 `/admin/sources` 检查 `kzgo` 状态。
6. 手动触发一次同步，确认：
   - `maps_seen > 500`
   - `wrs_created > 0`
   - `servers_seen > 0`
7. 检查 `/api/v1/public/servers` 是否使用外部快照。
8. 在确认 GlobalAPI 网络与授权后启用 `GOKZ_GLOBAL_SYNC_ENABLED=true`。

回滚：

- 代码回滚优先；迁移为增量，不建议删除新表。
- 外部源故障时禁用对应 source，公开服务器自动回退插件心跳。

---

## 13. 当前限制与后续

1. GOKZ-Global 直连未在本环境验证，必须先在目标生产网络测试。
2. 完整历史记录不是 kzgo.eu 能提供的；需要实现 GlobalAPI 分 Filter backfill worker。
3. 外部 Mapper 名称尚未写入独立表。
4. kzgo 在线玩家只提供名称，没有 SteamID，暂不作为公开玩家身份来源。
5. 需要确认 GlobalAPI 第三方使用许可、限流与 User-Agent 规范。
6. 数据源日志/raw payload 需要保留期与容量策略。
