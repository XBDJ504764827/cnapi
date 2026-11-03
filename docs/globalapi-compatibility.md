# GlobalAPI 兼容性分析

本文档记录 GlobalAPI（KZTimer 全球记录系统）的数据格式，以及它与 CNGOKZAPI 自有数据模型之间的映射关系、差异和兼容建议。**CNGOKZAPI 不复制 GlobalAPI 的数据库结构**；所有映射都在接入层完成。

## 1. 研究来源与版本

| 来源 | 版本/提交 | 得到的信息 |
| --- | --- | --- |
| `KZGlobalTeam/csgo-legacy-globalapi-plugin` | commit `bec212c`（2018-03-20） | GlobalAPI **v1**：使用 `https://kztimerglobal.com/api/v1/...`；`X-ApiKey` 认证；`POST /records`、`POST /bans`、`GET /records/top`、`GET /players/steamid/{steamid}`、`GET /auth/status`、`GET /modes` 的真实 JSON |
| `KZGlobalTeam/gokz` | tag `3.8.0`，commit `3a601e1` | GlobalAPI **v2.0**（`GlobalAPI_Backend_Version "v2.0"`，`https://kztimerglobal.com/api/v2.0`）；`include/GlobalAPI.inc` 的 native 参数、`include/GlobalAPI/responses.inc` 的响应字段；`gokz-global/send_run.sp` 对 `record_id`/`top_100`/`top_100_overall` 的使用 |
| `KZGlobalTeam/cs2kz-api` | commit `c938925` | cs2kz 自有 API/MySQL schema，仅用于对照（**不是** GlobalAPI） |

> GlobalAPI 的**服务端实现**不在以上仓库中。v2 的 POST 请求体字段没有官方 schema，本文档中涉及 v2 请求体的结论均标注为“由 native 参数反推”。

## 2. GlobalAPI 协议概览

### 2.1 认证

- 请求头：`X-ApiKey: <server key>`（v1 插件与 v2 插件一致）。
- `GET /api/{version}/auth/status`：
  - v1 响应被解析为 `{ "isValid": bool }`（`GlobalAPI-Core/server.sp`）。
  - v2 响应通过 `APIAuth` 读取 `Type`、`IsValid`、`Identity`（`responses.inc`）。`Identity` 的含义未在源码中说明。
- 没有服务器注册接口：Key 由管理员线下发放（v1 插件从 `cfg/sourcemod/globalrecords.cfg` 读取）。

### 2.2 端点

v2（`GlobalAPI.inc` 中的 native 注释）：

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/{v}/auth/status` | Key 校验 |
| GET | `/api/{v}/bans` | 封禁列表（可按类型/SteamID 等过滤） |
| POST | `/api/{v}/bans` | 创建封禁 |
| GET | `/api/{v}/jumpstats` | Jumpstats 列表（过滤参数见 native） |
| POST | `/api/{v}/jumpstats` | 创建 Jumpstat |
| GET | `/api/{v}/jumpstats/{jump_type}/top` | 类型排行榜 |
| GET | `/api/{v}/jumpstats/{jump_type}/top30` | 类型 Top30 |
| GET | `/api/{v}/maps`、`/maps/{id}`、`/maps/name/{name}` | 地图 |
| GET | `/api/{v}/modes`、`/modes/id/{id}`、`/modes/name/{name}` | 模式 |
| GET | `/api/{v}/players`、`/players/steamid/{steamid}`、`/players/steamid/{steamid}/ip/{ip}` | 玩家 |
| GET | `/api/{v}/player_ranks` | 玩家积分/排名 |
| GET | `/api/{v}/record_filters`、`/record_filters/distributions` | 成绩过滤维度与积分分布 |
| POST | `/api/{v}/records` | 提交成绩 |
| GET | `/api/{v}/records/place/{id}` | 名次（已废弃） |
| GET | `/api/{v}/records/top`、`/records/top/recent`、`/records/top/world_records` | 榜单 |
| GET | `/api/{v}/records/replay/list`、`/records/{recordId}/replay`、`/records/replay/{replayId}` | Replay 列表/下载 |
| POST | `/api/{v}/records/{recordId}/replay` | 上传 Replay（二进制） |
| GET | `/api/{v}/servers`、`/servers/{id}`、`/servers/name/{name}` | 服务器 |

v1（legacy 插件实际调用）：

```text
GET  /api/v1/auth/status
GET  /api/v1/players/steamid/{steamid}
POST /api/v1/records
GET  /api/v1/records/top?map_name=&tickrate=&limit=&modes_list=&stage=&has_teleports=
POST /api/v1/bans/
GET  /api/v1/maps
GET  /api/v1/modes/name/{mode}
```

### 2.3 真实请求/响应片段（v1 源码原文）

`POST /records`（`GlobalAPI-Core/records.sp`），`time` 为 float 秒：

```json
{ "steam_id": "STEAM_1:0:12345678", "map_id": 1234, "mode": "kz_timer", "stage": 0, "tickrate": 128, "teleports": 0, "time": 12.345 }
```

响应解析（`OnPostRecordInfo_Data`）：

```json
{ "place": 1, "top_100": 1, "top_100_overall": 1 }
```

`POST /bans/`（`GlobalAPI-Core/bans.sp`）：

```json
{ "steam_id": "STEAM_1:0:12345678", "ban_type": "global", "notes": "...", "stats": "...", "ip": "1.2.3.4" }
```

## 3. 字段映射

### 3.1 玩家 / 账户

| GlobalAPI 字段 | 类型 | CNGOKZAPI 字段 | 类型 | 转换 |
| --- | --- | --- | --- | --- |
| `steamid64` | string | `accounts.steam_id64` | `numeric(20,0)` | 十进制字符串 → NUMERIC；范围校验 |
| `steam_id` | string | `accounts.steam_id2` | `text` | 校验 `^STEAM_[0-5]:[01]:[0-9]+$`；与 SteamID64 必须一致（可互相推导） |
| `name` / `player_name` | string | `player_profiles.display_name` | `text` | 只保留最新值；不写入成绩行 |
| `is_banned` | bool | 由 `bans` 推导 | — | `EXISTS (active ban)`，不落库 |
| `total_records` | int | `player_mode_stats.total_records` | int | 派生统计，可重算 |
| `finishes` | int | `player_mode_stats.finishes` | int | 派生 |
| `points` / `points_overall` | int | `player_mode_stats.points` / `points_overall` | int | 派生（计算规则未公开） |
| `average` / `rating` | float | `player_mode_stats.average` / `rating` | `numeric(10,4)` | 浮点 → NUMERIC(10,4)（精度取舍需确认） |
| `ip`（关联查询） | string | `player_profiles.last_ip` | `inet` | 字符串 → INET，校验 IPv4/IPv6 |

### 3.2 地图 / 关卡

| GlobalAPI 字段 | 类型 | CNGOKZAPI 字段 | 类型 | 转换 |
| --- | --- | --- | --- | --- |
| `id` | int | 不直接使用；可写入外部映射 | — | 需要 `external_ref`/映射表（见第 6 节） |
| `name` | string | `maps.name` | `text` | 大小写不敏感唯一；去掉 `.bsp` |
| `filesize` | int | `map_versions.file_size_bytes` | `bigint` | 每次内容变更新增版本行 |
| `validated` | bool | `maps.status` | `text` | `true → approved`，`false → in_testing`（无法区分 disabled） |
| `difficulty` | int (单值) | `course_tiers.nub_tier` + `pro_tier` | smallint 1–10 | **不兼容**：单值无法直接拆成 NUB/PRO，需业务规则（待确认） |
| `created_on` / `updated_on` | timestamp | `maps.created_at` / `updated_at` | timestamptz | 导入时可覆盖 `created_at` |
| `approved_by_steamid64` | string | `maps.approved_by_account_id` | bigint FK | SteamID64 → accounts 查找 |

GlobalAPI 的 `stage`（0 = 主线，1.. = bonus）对应 `map_courses.stage`，语义一致。

### 3.3 成绩

| GlobalAPI 字段 | 类型 | CNGOKZAPI 字段 | 类型 | 转换 |
| --- | --- | --- | --- | --- |
| `id` / `record_id` | int | `records.external_ref` | text | 保留原始 id 用于幂等与回查 |
| `steam_id` | string | `records.account_id` | bigint FK | Steam2 → SteamID64 → accounts |
| `steamid64` | string | 同上 | | 优先使用 |
| `player_name` | string | 不存储 | — | 响应时从 `player_profiles` join |
| `map_id` | int | `records.map_id` / `course_id` | bigint FK | 通过外部映射找到 CNGOKZ 地图；`stage` → `map_courses.stage` |
| `map_name` / `server_name` | string | 不存储 | — | 派生 |
| `server_id` | int | `records.server_id` | bigint FK | 外部服务器 id → `servers.id` 映射 |
| `mode` | string（`kz_vanilla`/`kz_simple`/`kz_timer`） | `records.mode_id` | smallint FK | 字符串 → `movement_modes.code`（`vanilla`/`simple`/`kztimer`） |
| `stage` | int | `map_courses.stage` | smallint | 直接对应（0 = 主线） |
| `tickrate` | int | `records.tickrate` | smallint | 1–1000 校验 |
| `teleports` | int | `records.teleports` | int | 直接对应 |
| `has_teleports`（榜单过滤） | bool | 由 `teleports` 推导 | — | `teleports = 0` ⇔ PRO / 无传送 |
| `time` | float 秒 | `records.time_ns` | bigint 纳秒 | `round(seconds * 1e9)`；API 输出十进制秒字符串 |
| `created_on` / `updated_on` | timestamp | `records.created_at` / `updated_at` | timestamptz | |
| `updated_by` / `updated_by_id` | int | `records.reviewed_by_account_id` | bigint FK | 需要外部管理员 id → accounts 映射 |
| `place` / `top_100` / `top_100_overall` | int | 不存储 | — | 响应时实时计算 |
| `style` | **不存在** | `records.style_id` | smallint FK | GlobalAPI 无 style 字段；默认 `normal` |

### 3.4 Jumpstats

| GlobalAPI 字段 | 类型 | CNGOKZAPI 字段 | 类型 | 转换 |
| --- | --- | --- | --- | --- |
| `id` | int | `jumpstats.external_ref` | text | 幂等/回查 |
| `steam_id` / `steamid64` | string | `jumpstats.account_id` | bigint FK | 同成绩 |
| `server_id` | int | `jumpstats.server_id` | bigint FK | 外部 id → `servers.id` |
| `jump_type` | int | `jumpstats.jump_type_id` | smallint FK | **int → code 的映射未在源码中定义**；`jump_types.gokz_code` 记录了 GOKZ 枚举值，导入前必须确认 GlobalAPI 的编号（待确认） |
| `distance` | float | `jumpstats.distance_units` | `numeric(12,4)` | 四舍五入到 4 位小数 |
| `tickrate` | int | `jumpstats.tickrate` | smallint | |
| `strafe_count` | int | `jumpstats.strafe_count` | smallint | |
| `msl_count` | int | `jumpstats.msl_count` | smallint | |
| `is_crouch_bind` / `is_forward_bind` / `is_crouch_boost` | bool | 同名列 | boolean | |
| `jump_json_info` | string(JSON) | `jumpstats.raw_metrics` | jsonb | JSON 字符串解析；字段含义未文档化 |
| `updated_by_id` | int | 不存储 | — | 只在审核场景使用（当前无审核字段） |
| `created_on` / `updated_on` | timestamp | `created_at` / `updated_at` | timestamptz | |

### 3.5 服务器

| GlobalAPI 字段 | 类型 | CNGOKZAPI 字段 | 类型 | 转换 |
| --- | --- | --- | --- | --- |
| `id` | int | 外部映射（未建表） | — | 成绩/封禁导入需要 |
| `name` | string | `servers.name` | text | 大小写不敏感唯一 |
| `ip` / `port` | string / int | `servers.ip` / `port` | inet / int | |
| `owner_steamid64` | string | `servers.owner_account_id` | bigint FK | SteamID64 → accounts |
| `X-ApiKey` | header | `servers.api_key_hash` | bytea | 只存哈希；插件持有明文 |

### 3.6 封禁

| GlobalAPI 字段 | 类型 | CNGOKZAPI 字段 | 类型 | 转换 |
| --- | --- | --- | --- | --- |
| `id` | int | `bans.external_ref` | text | |
| `steamid64` / `steam_id` | string | `bans.account_id` | bigint FK | 优先 SteamID64 |
| `ip` | string | `bans.ip` | inet | |
| `ban_type` | string | `bans.ban_type` | text | 原样保存（无枚举） |
| `notes` | string | `bans.reason` | text | 直接对应 |
| `stats` | string | `bans.stats` | text | 原样保存 |
| `expires_on` | timestamp | `bans.expires_at` | timestamptz | 空值/0 表示永久（v1 插件不发送该字段；v2 响应有） |
| `server_id` | int | `bans.issued_by_server_id` | bigint FK | 服务器自动封禁 |
| `updated_by_id` | int | `bans.issued_by_account_id` | bigint FK | 管理员封禁 |
| `created_on` | timestamp | `bans.issued_at` | timestamptz | |
| `updated_on` | timestamp | `bans.updated_at` | timestamptz | |
| 解封 | **无对应端点** | `ban_revocations` | 表 | GlobalAPI 未暴露解封 API |

### 3.7 Replay

| GlobalAPI | CNGOKZAPI | 说明 |
| --- | --- | --- |
| `POST /records/{recordId}/replay` 二进制上传 | `replays` 元数据 + R2 对象 | GlobalAPI 不返回元数据；CNGOKZAPI 额外记录 bucket/object_key/size/sha256 |
| `GET /records/{recordId}/replay` | 查询 `replays` 后返回对象 | 需要鉴权与保留期策略 |
| `replayId` | `replays.id` | 无上游映射字段；如需回查 GlobalAPI，加 `external_ref` 列（当前未加） |

## 4. 枚举值差异

| 维度 | GlobalAPI | GOKZ（本地） | cs2kz | CNGOKZAPI |
| --- | --- | --- | --- | --- |
| 移动模式 | `kz_vanilla`、`kz_simple`、`kz_timer` | `Mode_Vanilla=0`、`Mode_SimpleKZ=1`、`Mode_KZTimer=2` | `Vanilla=1`、`Classic=2` | `movement_modes.code`: `vanilla`、`simple`、`kztimer`、`classic` |
| TP/PRO | `teleports` 计数 + `has_teleports` 过滤 | `TimeType_Nub=0`、`TimeType_Pro=1` | 由 filter（`has_teleports`）区分 | `time_type`: `nub`/`pro`，由 `teleports` 强制 |
| 风格 | **无字段** | `Style_Normal=0`，`STYLE_COUNT=1` | `Styles` 位掩码 = 服务器设置（auto-bhop/legacy-jump/auto-unduck），不是移动风格 | `movement_styles.code`，当前仅 `normal` |
| 成绩时间 | float 秒 | `float time`；本地 `RunTime` 整数（有损） | `FLOAT8` 秒 | `time_ns` 整数纳秒 |
| Jump 类型 | int（编号含义未文档化） | `LongJump=0` … `Invalid=11`（含 LowpreBhop/LowpreWeirdJump/Other） | `LongJump=0` … `Fall=7` | `jump_types.code` + `gokz_code`/`cs2kz_code` 映射列 |
| 地图状态 | `validated` bool | 无（文件大小校验） | `MapState`: Invalid=-1 / InTesting=0 / Approved=1 | `maps.status`: `in_testing`/`approved`/`disabled` |
| 地图难度 | `difficulty` 单值 | 无 | `nub_tier`/`pro_tier` 1–10 + filter state | `course_tiers` per mode |
| 封禁类型 | 自由文本 | 无 | 无枚举（`reason` + `BannedBy`） | 自由文本 + `source` 枚举 |
| 触发来源 | 无 | 无 | 服务器/管理员（`BannedBy`） | `source` + `issued_by_*` |

## 5. 数据转换规则

### 5.1 Steam2 → SteamID64

GlobalAPI v1 只发送 Steam2（`STEAM_X:Y:Z`）。转换公式（公有账号，universe 1）：

```text
account_id = Z * 2 + Y
steam_id64 = 76561197960265728 + account_id
```

规则：

- 必须校验 `X`（universe）与 `Y`（0/1）；`Z` 为无符号 32 位。
- 若 SteamID64 与 Steam2 同时存在，必须以 SteamID64 为准，并校验两者一致，否则拒绝该条数据并记录 `system_events`。
- 转换在接入层完成，数据库只存 SteamID64；`steam_id2` 仅用于核对/导入溯源。

### 5.2 时间

- 输入：float 秒（GlobalAPI `time`）。
- 存储：`time_ns = round(seconds × 1_000_000_000)`，必须为正数。
- 输出（CNGOKZAPI）：`time_ns` + `time_seconds`（9 位小数十进制字符串）。
- GlobalAPI 兼容响应：输出 float 秒（`time_ns / 1e9`），这是唯一允许出现浮点的位置，并接受精度损失。

### 5.3 地图 / 关卡 / 服务器身份

GlobalAPI 使用自增整数 id，CNGOKZAPI 使用自有 BIGINT。两者之间**必须有显式映射**，不能假设 id 相同：

- 地图：优先按 `name` 匹配；`stage` 直接映射 `map_courses.stage`。
- 服务器：按 `ip + port` 或 `name` 匹配；无法匹配时拒绝提交或创建待审核记录（待确认）。
- 导入的成绩/封禁通过 `(source, external_ref)` 保持幂等；地图/服务器的外部 id 映射表尚未建立（见第 6 节建议）。

### 5.4 审核字段

GlobalAPI `updated_by_id` 是上游管理员 id，需要映射到 `accounts.id`。若无法映射，`reviewed_by_account_id` 置空，并在 `record_history.metadata` 中保留原始 id，避免丢失溯源信息。

## 6. 不兼容字段与模型差异

| 项 | 差异 | 处理 |
| --- | --- | --- |
| `place`、`top_100`、`top_100_overall` | GlobalAPI 响应中的实时名次 | 不落库；兼容响应层实时计算 |
| `player_name`、`steam_id`、`map_name`、`server_name` | GlobalAPI 响应内联字符串 | 不冗余存储；join 生成 |
| `difficulty`（单值） | 无法表达 NUB/PRO 难度差异 | 导入时需业务规则；否则只写 `nub_tier` 或保持空 |
| `style` | GlobalAPI 无此维度 | 统一写 `normal`；无法从 GlobalAPI 导入侧跳数据 |
| `tickrate` 分组 | GlobalAPI 榜单按 tickrate 查询；CNGOKZAPI 当前 WR 主键不含 tickrate | 待确认；见 `records.md` |
| `record_filters` 表 | GlobalAPI 将 (map, stage, mode, tickrate, has_teleports) 作为一等实体；CNGOKZAPI 用（course, mode, style, time_type）组合并保留 tickrate 列 | 兼容查询层可按需组合 |
| `stats`（封禁） | GlobalAPI 自由文本 | 原样保存 |
| 提交幂等键 | GlobalAPI 协议无提交 ID | 见 5.3 / 第 7 节；导入依赖 `external_ref` |
| 解封 | GlobalAPI 无解封端点 | CNGOKZAPI 提供内部解封模型，不对外兼容 |
| Replay 元数据 | GlobalAPI 不返回 | CNGOKZAPI 自建元数据 |

## 7. 兼容接口设计建议（第三阶段）

1. **接入层与核心模型分离**
   - 新增 `POST /api/v1/ingest/globalapi/records`、`/jumpstats`、`/bans` 等入口，由独立的 adapter 模块把 GlobalAPI 字段转换成内部 `records`/`jumpstats`/`bans` 写入。
   - 核心 service/repository 不感知 GlobalAPI 字段名。

2. **认证兼容**
   - 接受 `X-ApiKey` 头：SHA-256 → `servers.api_key_hash` 查找；轮换/吊销逻辑与原生协议一致。
   - `GET /api/v1/ingest/globalapi/auth/status` 返回 `{ "isValid": true }`（v1）或 v2 的 `Type`/`IsValid`/`Identity`（`Identity` 建议返回 `servers.id`，但需确认上游语义）。

3. **响应兼容**
   - 成绩提交成功后返回 `record_id`、`place`、`top_100`、`top_100_overall`：这些值由查询实时计算，不落库。
   - `place` 基于（course, mode, style, time_type, 时间）实时排序；具体口径（是否包含 tickrate 分组）待确认。

4. **外部 id 映射**
   - 建议新增 `external_entity_ids(source, entity_type, entity_id, external_id)` 通用映射表（唯一约束 `(source, entity_type, external_id)`），用于地图、服务器、管理员账号。
   - 当前 schema 只在 `records`/`jumpstats`/`bans` 上保留了 `external_ref`，地图/服务器映射缺失是已知限制。

5. **幂等**
   - GlobalAPI 没有提交 ID：如果上游无法提供，建议在接入层对 `(server_id, steam_id, map_id, stage, mode, tickrate, teleports, time_ns)` 做短期去重窗口（例如 10 秒），并记录 `source='globalapi_import'` 的 `external_ref`（若响应中拿到 `record_id` 则回填）。
   - 不建议把内容哈希作为永久唯一键：合法的重复成绩在理论上存在。

6. **版本策略**
   - CNGOKZAPI 原生 API 使用 `/api/v1`；GlobalAPI 兼容入口建议放在 `/api/v1/ingest/globalapi/*`，避免污染原生契约。
   - 兼容层只承诺字段级别兼容，不承诺与 GlobalAPI 内部数据库结构一致。

## 8. 待确认事项

1. GlobalAPI v2 各 POST 端点的**完整请求体**（当前由 native 参数反推）。
2. GlobalAPI `jump_type` 整数与 GOKZ/cs2kz 枚举的对应关系。
3. `APIAuth.Identity` 的语义（服务器 id？账号 id？）。
4. GlobalAPI 榜单是否按 tickrate 拆分，`has_teleports` 与 NUB/PRO 的精确关系。
5. `difficulty` 单值如何映射到 NUB/PRO tier。
6. `expires_on` 的永久封禁表示（空值 / `0000-00-00` / 特定日期）。
7. `updated_by_id` 引用的管理员/服务器实体如何映射到 CNGOKZAPI 账号。
8. 上游 `place`/`top_100` 的计算口径（是否排除封禁玩家、是否按模式/tickrate）。

## 9. 研究局限

- GlobalAPI v2 服务端源码未公开在所列仓库中；请求字段与错误响应格式无法完全核实。
- Legacy 插件最后提交为 2018 年，GlobalAPI v1 可能已经下线或行为变更。
- KZTimer（另一套计时器与网站）源码未纳入研究，`kz_timer` 模式的完整规则（计时精度、传送规则、风格）未知。
- 本阶段未抓取线上 `kztimerglobal.com` 的实际响应进行比对（避免依赖外部服务的实时行为）；所有结论均来自源码与迁移文件。
