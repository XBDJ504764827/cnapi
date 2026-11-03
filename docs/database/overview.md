# 数据库设计总览

第二阶段目标：在**不实现业务接口**的前提下，为玩家、地图、成绩、排行榜、服务器、Jumpstats、Replay 和封禁建立可长期演进的 PostgreSQL 数据模型。

- 数据库：PostgreSQL 18
- 迁移：SQLx Migrate（`apps/api/migrations/`）
- Rust 模型：`apps/api/src/models/`；未来 API 契约：`apps/api/src/dto/`
- 阶段 1 的 `system_events` / `app_settings` 保持不变

## 1. 研究来源

设计以真实源码与迁移为依据。三个上游仓库均在第二阶段开始时克隆到本地并阅读（未修改上游代码）：

| 项目 | 版本/提交 | 研究内容 |
| --- | --- | --- |
| [KZGlobalTeam/gokz](https://github.com/KZGlobalTeam/gokz) | tag `3.8.0`，commit `3a601e1`（2026-10-15） | `gokz-localdb/db/sql.sp` 本地表结构；`include/gokz/{core,jumpstats,global}.inc` 的 Mode/Style/TimeType/JumpType 枚举与 GlobalAPI 模式名；`gokz-replays` 回放格式版本；`include/GlobalAPI.inc`、`include/GlobalAPI/responses.inc` 的 GlobalAPI v2 接口与响应字段 |
| [KZGlobalTeam/cs2kz-api](https://github.com/KZGlobalTeam/cs2kz-api) | commit `c938925`（2026-10-04） | `crates/cs2kz/migrations/0001_initial.up.sql` 及 0002–0006；`src/mode.rs`、`styles.rs`、`jumpstats/jump_type.rs`、`maps/state.rs`、`maps/courses/filters/mod.rs`、`records.rs`、`players/mod.rs`、`servers/mod.rs`、`replays*` |
| [KZGlobalTeam/csgo-legacy-globalapi-plugin](https://github.com/KZGlobalTeam/csgo-legacy-globalapi-plugin) | commit `bec212c`（2018-03-20，GlobalAPI v1 时代） | `GlobalAPI-Core.sp` 端点常量与模式名；`records.sp`/`bans.sp`/`server.sp` 的请求体与响应字段；`GlobalAPI-Jumpstats/apicalls/postjumpstat.sp` |

局限与未获取的资料（详见 [globalapi-compatibility.md](../globalapi-compatibility.md#9-研究局限)）：

- GlobalAPI **v2 服务端实现**不在上述仓库中；POST 请求体字段只能从插件 native 参数与响应字段反推，未与后端源码核对。
- KZTimer（CS:GO 时代的另一套计时器/网站）源码与数据库不在研究范围内；`kztimer` 模式仅作为 GlobalAPI 模式字符串出现。
- `style` 在 GOKZ 中只有 `Style_Normal`；cs2kz 的 `styles` 是服务器设置位掩码（auto-bhop/legacy-jump/auto-unduck），不是移动风格。侧跳/半侧跳等风格未在任何被研究源码中确认。

## 2. 设计原则

1. **主键统一**：业务表一律 `BIGINT GENERATED ALWAYS AS IDENTITY`；引用表用 `SMALLINT`。阶段 1 的系统表保留 `UUID`（`system_events`），这是有意的例外，不参与业务外键。
2. **SteamID64 无溢出**：`accounts.steam_id64 NUMERIC(20,0)` + `CHECK (0 <= steam_id64 <= 18446744073709551615)`。**不使用** `BIGINT`/`BIGSERIAL`，因为它们是有符号 64 位。
3. **时间统一 UTC**：所有时间列 `TIMESTAMPTZ`；默认 `now()`；容器与数据库时区设置为 UTC。
4. **成绩时间无浮点误差**：`records.time_ns BIGINT`（纳秒）。源数据是 float 秒（GlobalAPI `time`、cs2kz `Seconds`、GOKZ `float time`），入库前转换为整数纳秒；128 tick 的 7.8125 ms 可在纳秒下精确表示。API 层同时提供纳秒整数与十进制秒字符串，不返回 JSON 浮点。
5. **TP/PRO 规则显式化**：`time_type` 与 `teleports` 由 CHECK 约束互相校验（PRO ⇔ 0 传送）。
6. **软删除策略**：
   - `accounts`、`maps`、`servers`、`records`、`jumpstats`、`replays` 使用 `deleted_at`；唯一索引均为 `WHERE deleted_at IS NULL` 的部分索引，允许软删除后重新注册。
   - `bans` 永不软删除，只能通过 `ban_revocations` 撤销，保留审计事实。
   - 硬删除仅用于数据保留策略（例如 Replay 文件过期后删除对象并移除元数据），由第三阶段的清理任务执行。
7. **引用数据表**：`movement_modes`、`movement_styles`、`jump_types` 存代码值、展示名与上游枚举映射，避免在代码与 SQL 里各自硬编码字符串。
8. **幂等与并发**：
   - 插件提交使用 `(server_id, submission_id)` 唯一索引；导入使用 `(source, external_ref)` 唯一索引。
   - 世界纪录指针 `course_world_records` 采用条件 upsert（`WHERE EXCLUDED.time_ns < current.time_ns`），慢成绩永远不会覆盖快成绩；测试 `world_record_upsert_cannot_be_overwritten_by_slower_runs` 覆盖该行为。
9. **不做过早优化**：不引入分区、物化视图、微服务或跨库同步。大表（records/jumpstats）预留了按自然键读取的索引与时间列，后续需要时再分区。
10. **避免冗余但保留高频查询路径**：`records.map_id`、`replays.account_id/map_id` 是受控冗余，分别用复合外键/上传时快照保证不漂移。

## 3. 表清单

| 模块 | 表 | 说明 |
| --- | --- | --- |
| 引用数据 | `movement_modes` | 移动模式（vanilla/simple/kztimer/classic） |
| 引用数据 | `movement_styles` | 移动风格（当前仅 `normal`） |
| 引用数据 | `jump_types` | Jumpstat 类型 + GOKZ/cs2kz 枚举映射 |
| 账户 | `accounts` | SteamID64 身份 |
| 账户 | `player_profiles` | 昵称、国家、最近 IP、偏好、Prime |
| 账户 | `player_mode_stats` | 按模式缓存的积分/rating 等派生统计 |
| 账户 | `player_identities` | 外部身份关联（discord/forum…） |
| 地图 | `maps` | 地图基础信息与审核状态 |
| 地图 | `map_versions` | 文件大小/校验和等内容版本 |
| 地图 | `map_courses` | 关卡（stage 0 = 主线，1.. = bonus） |
| 地图 | `map_modes` | 地图支持的移动模式 |
| 地图 | `course_tiers` | 按关卡+模式的 NUB/PRO 难度与排名状态 |
| 服务器 | `servers` | 服务器注册、API Key 哈希、心跳、当前地图 |
| 成绩 | `records` | 单次跑图成绩（含 TP/PRO、tickrate、来源服务器、审核） |
| 成绩 | `record_history` | 成绩变更审计（审核、修正、软删除） |
| 成绩 | `course_world_records` | 每个（关卡, 模式, 风格, TP/PRO）当前 WR 指针 |
| Jumpstats | `jumpstats` | 跳跃测量数据 |
| Replay | `replays` | Replay 对象元数据（R2 object key、大小、校验值、保留期） |
| 管理 | `bans` | 封禁记录（追加式，过期由时间推导） |
| 管理 | `ban_revocations` | 解封记录 |

## 4. 迁移清单

| 版本 | 文件 | 内容 |
| --- | --- | --- |
| 20261103000001 | `init_system_tables.sql` | 阶段 1：`system_events`、`app_settings` |
| 20261103000002 | `kz_reference_data.sql` | `set_updated_at()` + `movement_modes`/`movement_styles`/`jump_types` 及种子数据 |
| 20261103000003 | `accounts.sql` | `accounts`、`player_profiles`、`player_mode_stats`、`player_identities` |
| 20261103000004 | `maps.sql` | `maps`、`map_versions`、`map_courses`、`map_modes`、`course_tiers` |
| 20261103000005 | `servers.sql` | `servers` |
| 20261103000006 | `records.sql` | `records`、`record_history`、`course_world_records` |
| 20261103000007 | `jumpstats.sql` | `jumpstats` |
| 20261103000008 | `replays.sql` | `replays` |
| 20261103000009 | `moderation.sql` | `bans`、`ban_revocations` |

迁移在 API 启动时自动执行（`sqlx::migrate!`）。`apps/api/build.rs` 让 Cargo 监听 `migrations/` 目录，新增迁移会触发重新编译（否则宏内嵌的迁移列表不会更新）。

## 5. 命名与类型约定

- 表名：小写复数；关联表用 `<a>_<b>`（如 `map_modes`）。
- 主键：`id`；外键：`<entity>_id`。
- 状态列：`TEXT` + 命名 CHECK（`ck_<table>_<field>`），值使用 snake_case，并在 Rust 中用 `sqlx::Type` 映射。
- 唯一索引：`uq_<table>_<purpose>`；普通索引：`idx_<table>_<purpose>`；外键：`fk_<table>_<reference>`；检查：`ck_<table>_<rule>`。
- 金额/评分/距离等精确值：`NUMERIC`，禁止 `FLOAT4/FLOAT8`（上游使用 float 的字段在入库前转换为 `NUMERIC` 或整数）。
- JSON 仅用于来源不确定或高度可变的载荷（`player_profiles.preferences`、`jumpstats.raw_metrics`、`system_events.metadata`）。

## 6. 与第一阶段的关系

- 现有接口（`/health*`、`/api/v1/system/*`）不受影响；新增迁移只创建表，不修改 `system_events`/`app_settings`。
- 第二阶段不注册任何新 HTTP 路由，OpenAPI 文档保持不变（`docs/openapi.json` 未变）。
- 现有测试全部保留并通过；新增 `tests/schema_integration.rs` 验证 schema。

## 7. 待确认的业务规则

| # | 问题 | 当前处理 | 影响 |
| --- | --- | --- | --- |
| 1 | GlobalAPI 的 leaderboard 是否按 tickrate 区分 | `records.tickrate` 仅存储，未进入 WR/榜单唯一键 | 若需区分，需调整 `course_world_records` 主键并新增索引 |
| 2 | 侧跳/半侧跳/space 等 style 是否纳入 | `movement_styles` 只 seed `normal` | 新增 style 只需插入引用行，不改表结构 |
| 3 | NUB/PRO 之外的排行维度（例如 tickrate 分组、分段榜单） | 未建模 | 可通过新增 `course_world_records` 维度列或独立榜单表扩展 |
| 4 | `player_mode_stats.points/rating` 的计算公式与刷新频率 | 仅建表，由第三阶段计算任务写入 | 不影响现有数据完整性 |
| 5 | GlobalAPI `difficulty` 与 cs2kz NUB/PRO tier 的对应关系 | 统一存 `course_tiers.nub_tier/pro_tier`（1–10） | 导入时若只有单值难度，写入 NUB/PRO 两列需确认规则 |
| 6 | 地图作者/mapper 数据 | 未建模（上游有 Mappers/CourseMappers，但需求未要求） | 后续可新增 `map_authors`/`course_authors` |
| 7 | 服务器在线判定的心跳阈值 | 未固定；文档建议 2× 心跳间隔 | 读取端需要统一常量 |
| 8 | 封禁的 ban_type 取值 | 自由文本（GlobalAPI 未定义枚举） | 若需过滤，需要约定规范值 |

## 8. 数据保留策略（草案）

| 数据 | 保留策略 |
| --- | --- |
| `records` / `record_history` | 永久保留；软删除仅隐藏，审计行不删除 |
| `accounts` / `player_profiles` | 永久保留；账号删除请求走软删除 + 匿名化流程（第三阶段定义） |
| `jumpstats` | 默认永久；可按保留期批量归档（未来任务） |
| `replays` | 按 `expires_at` 清理对象存储；元数据在对象删除后保留 `status='deleted'` 一段时间用于审计 |
| `servers` | 软删除；API Key 立即吊销（`api_key_revoked_at`） |
| `bans` / `ban_revocations` | 审计数据，永久保留 |
| `system_events` | 建议 90–180 天，具体由运维任务决定（阶段 1 未实现） |

## 9. 验证情况

- 9 个迁移在 PostgreSQL 18.6 上从零执行成功（本地实例验证）。
- 新增 11 个 schema 集成测试（表/索引/约束/枚举/触发器/WR 并发/软删除唯一性），全部通过。
- 第一阶段 34 个测试保持通过，总计 **53 个测试通过**（29 单元 + 10 HTTP + 11 schema + 3 真实依赖集成）。
- 详细列定义见各模块文档，表关系见 [er-diagram.md](er-diagram.md)。
