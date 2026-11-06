# 第七阶段：CS:GO KZ 游戏服务器接入与数据同步

> 技术栈：**CS:GO (Source 1)** + LinuxGSM + MetaMod:Source + SourceMod + GOKZ。
> 本阶段不使用 SwiftlyS2、CS2KZ 或任何 CS2 专用接口。

## 1. 现有项目审查结论

| 项目 | 现状 | 第七阶段处理 |
| --- | --- | --- |
| API | Rust 1.97 + Axum 0.8 + SQLx 0.9，`/api/v1` 统一响应 | 复用现有路由与错误结构，新增 `/api/v1/game-servers/*`、`/api/v1/admin/game-servers/*` |
| 服务器表 | `servers` 已有 Token 哈希、前缀、撤销时间、地图、心跳 | 增量扩展 CS:GO 元数据、启用开关、在线人数，不新建平行表 |
| 成绩 | `records` 已有 `source=plugin`、`submission_id`、WR 指针、审核 | 新增 `submission_key` 幂等列；自动提交写 `source=plugin` |
| PB/WR/积分 | 第六阶段 Filter + `filter_recalc_tasks` + worker 已实现 | 成绩插入触发对应 Filter 重算；本阶段不修改积分公式 |
| Replay | `replays` 已有 R2/S3 元数据模型，没有二进制上传客户端 | 新增元数据注册/幂等/补偿任务；二进制由独立 uploader 处理 |
| 公开门户 | 已有服务器、成绩、玩家、地图接口 | 服务器人数/模式/在线玩家改为真实心跳数据 |
| 管理后台 | 服务器页是“接口未接入”空壳 | 列表、创建、详情、启停、Token 轮换/撤销、同步日志/任务 |

## 2. 数据链路

```text
CS:GO Dedicated Server (LinuxGSM)
  MetaMod:Source + SourceMod + GOKZ + SteamWorks + SMJansson
        │  cngokz_sync.smx
        ├─ heartbeat (25s)
        ├─ players/sync (30s + 加入/离开事件)
        ├─ GOKZ_OnTimerEnd_Post ──► local durable record queue
        ├─ GOKZ_RP_OnTimerEnd_Post ──► replay manifest (file path only)
        └─ config/sync ──► data/cngokz_config.json
        │ HTTPS + X-Server-Token
        ▼
CNGOKZAPI
  token auth → validation → PostgreSQL transaction
        ├─ records (approved, source=plugin)
        ├─ course_world_records (conditional faster-only upsert)
        ├─ record_history
        ├─ server_sync_tasks (replay compensation)
        └─ DB trigger: scoring_filters dirty + filter_recalc_tasks pending
        ▼
phase-6 recalc worker → player_filter_points → player_mode_points
        ▼
Redis public cache invalidation + public portal
```

## 3. 服务器身份认证

- 每台服务器一个独立 Token：`cngokz_srv_<64 hex>`。
- 数据库只保存 `sha256(token)` 与前 12 位前缀；明文只在创建/轮换响应中出现一次。
- 请求头：`X-Server-Token: cngokz_srv_...`（也兼容 `Authorization: Bearer ...`）。
- 错误规则：
  - 缺少/错误 Token → `401 unauthorized`；
  - Token 已撤销 → `401`；
  - 服务器被管理员禁用 → `403 forbidden`；
  - 心跳可以更新状态，但禁用服务器不能提交心跳、玩家或成绩。
- Token 轮换后旧 Token 立即失效；撤销后只能由管理员重新轮换。

## 4. 数据库迁移

新增：`apps/api/migrations/20261105000001_phase7_game_server_sync.sql`

扩展 `servers`：

| 列 | 说明 |
| --- | --- |
| `game` | 固定 `csgo` |
| `region` / `server_type` | 区域、community/official/event/testing |
| `query_port` | 可选 A2S 查询端口 |
| `max_players` / `online_players` | 心跳人数 |
| `current_mode_id` | 当前移动模式 |
| `game_version` | CS:GO Dedicated Server 版本 |
| `is_enabled` | 管理员启用开关 |

新增表：

| 表 | 用途 |
| --- | --- |
| `server_heartbeats` | 心跳历史（服务器详情/诊断） |
| `server_player_sessions` | 真实在线会话；`(server_id, account_id) WHERE disconnected_at IS NULL` 唯一 |
| `server_sync_logs` | 成绩/Replay/拒绝/认证等审计日志（不存 Token） |
| `server_sync_tasks` | Replay 上传补偿、失败重试任务 |

成绩幂等：

```sql
ALTER TABLE records ADD COLUMN submission_key TEXT;
CREATE UNIQUE INDEX uq_records_submission_key
  ON records (server_id, submission_key)
  WHERE submission_key IS NOT NULL AND server_id IS NOT NULL;
```

迁移只做增量变更，不删除已有成绩、服务器或配置。API 启动时由 `sqlx::migrate!` 自动执行；也可用 `sqlx migrate run`。

## 5. API 清单

### 5.1 服务器侧（`X-Server-Token`）

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| POST | `/api/v1/game-servers/heartbeat` | 心跳、地图、模式、人数、版本 |
| POST | `/api/v1/game-servers/players/connect` | 玩家连接/认证成功 |
| POST | `/api/v1/game-servers/players/disconnect` | 玩家断开 |
| POST | `/api/v1/game-servers/players/sync` | 全量校准（只影响本服务器） |
| POST | `/api/v1/game-servers/records` | 提交 GOKZ 成绩，幂等 |
| GET | `/api/v1/game-servers/records/status?record_identifier=` | 成绩/Replay/积分状态 |
| POST | `/api/v1/game-servers/replays` | Replay 元数据（R2/S3 key、大小、SHA-256） |
| GET | `/api/v1/game-servers/config/sync?version=` | 地图/Course/Mode/Style/Tier 快照 |
| GET | `/api/v1/game-servers/maps/sync?version=` | 同上，兼容别名 |

### 5.2 管理侧（`X-Maintenance-Token`）

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/v1/admin/game-servers` | 列表、搜索、启用筛选、分页 |
| POST | `/api/v1/admin/game-servers` | 创建服务器并一次性返回 Token |
| GET | `/api/v1/admin/game-servers/{id}` | 详情：心跳、玩家、成绩、Replay、日志、任务 |
| PATCH | `/api/v1/admin/game-servers/{id}` | 编辑名称/区域/类型/人数/查询端口 |
| POST | `/api/v1/admin/game-servers/{id}/enable` | 启用 |
| POST | `/api/v1/admin/game-servers/{id}/disable` | 禁用并断开在线会话 |
| POST | `/api/v1/admin/game-servers/{id}/token/rotate` | 轮换 Token（新 Token 只返回一次） |
| POST | `/api/v1/admin/game-servers/{id}/token/revoke` | 撤销 Token |
| POST | `/api/v1/admin/game-servers/tasks/{task_id}/retry` | 重试失败的 Replay 补偿任务 |

### 5.3 公开门户

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/v1/public/servers` | 真实人数、最大人数、模式、状态 |
| GET | `/api/v1/public/servers/{id}` | 单台服务器 |
| GET | `/api/v1/public/players/online` | 心跳正常服务器的在线玩家 |

## 5.4 网站数据源调整：kzgo.eu 优先，插件兜底

当前网站数据源方向调整为：

```text
优先：kzgo.eu + KZTimerGlobal GlobalAPI 同步到本地数据库
兜底：CNGOKZ 自有 CS:GO 服务器插件上报
```

审查 kzgo.eu 前端后确认，它不是一个单一数据源：

| KZGO 页面数据 | 实际来源 | 可用接口 |
| --- | --- | --- |
| 地图、难度 tier、Workshop、mapper | kzgo.eu 自有后端 | `GET https://kzgo.eu/api/maps` |
| 服务器在线状态/地图/人数 | kzgo.eu 自有后端 | `GET https://kzgo.eu/api/servers` |
| 地图 WR（PRO/NUB、KZT/SKZ/VNL） | kzgo.eu 自有后端 | `GET https://kzgo.eu/api/wrs/kz_timer`、`kz_simple`、`kz_vanilla` |
| 各模式完成统计 | kzgo.eu 自有后端 | `GET https://kzgo.eu/api/completions/{kz_mode}` |
| Steam 头像/昵称 | kzgo.eu 自有后端 | `GET https://kzgo.eu/api/steam/{steamid64}` |
| 成绩历史、玩家排行、Jumpstats | KZTimerGlobal GlobalAPI | `https://kztimerglobal.com/api/v2.0/records/top/...`、`player_ranks`、`jumpstats` |

因此“同步 kzgo.eu 数据”需要分成两层：

1. **KZGO 自有 API**：同步地图、tier、当前服务器、当前 WR、完成统计。
2. **GlobalAPI**：同步完整成绩、玩家排行、Jumpstats；这是 kzgo.eu 页面实际展示成绩历史的接口。

如果只同步 `kzgo.eu/api`，无法得到完整成绩历史，因为该 API 不暴露完整 records 列表。

推荐落地顺序：

1. 新增外部同步状态表与同步 worker（地图/tier/server/WR）。
2. 增加 GlobalAPI 导入适配器（记录当前 WR 之外的完整 records 时使用）。
3. 公开 API 读取优先级：外部源新鲜 → 外部数据；外部源过期/失败 → CNGOKZ 插件数据。
4. 插件恢复开发时，按 `kzcharm/gokz-top-plugins` 目录结构重建。

本文件中的服务器接入 API 仍然保留，作为第二步兜底链路，不删除。

## 6. SourceMod 插件（延期）

> 当前方向调整：**网站功能优先，插件暂停开发**。原单文件插件原型已移除，不作为交付物。
>
> 后续插件必须按 [kzcharm/gokz-top-plugins](https://github.com/kzcharm/gokz-top-plugins) 的结构组织：
>
> ```text
> addons/sourcemod/
> ├── scripting/
> │   ├── cngokz-sync.sp
> │   └── cngokz-sync/
> │       ├── heartbeat.sp
> │       ├── players.sp
> │       ├── records.sp
> │       ├── records/queue.sp
> │       ├── records/http.sp
> │       ├── replays.sp
> │       └── config.sp
> ├── plugins/.gitkeep
> └── translations/
> cfg/sourcemod/gokz/
> ```
>
> 下面保留的是接入设计，不代表当前已交付插件文件。

文件（后续）：

```text
addons/sourcemod/scripting/cngokz-sync.sp
addons/sourcemod/scripting/cngokz-sync/*.sp
cfg/sourcemod/gokz/cngokz-sync.cfg
```

依赖：

- SourceMod 1.11+
- MetaMod:Source
- GOKZ 3.8+
- SteamWorks 扩展
- SMJansson（`smjansson` / `json` 扩展）

使用的 GOKZ 真实接口：

| 接口 | 用途 |
| --- | --- |
| `GOKZ_OnTimerEnd_Post(client, course, time, teleportsUsed)` | 有效成绩事件 |
| `GOKZ_RP_OnTimerEnd_Post(client, filePath, course, time, teleportsUsed)` | Replay 文件路径 |
| `GOKZ_GetCoreOption(client, Option_Mode)` | KZT / SKZ / VNL |
| `GOKZ_GetTimeType` | 由 teleports 推导 NUB / PRO（API 同样校验） |

不解析聊天消息，不修改 GOKZ 核心代码。

插件可靠性：

- 成绩先写入 `data/cngokz_sync_queue.txt`，最多 256 条；HTTP 成功后删除，插件重载/服务器重启后恢复。
- 401/403 停止 60 秒并记录日志；其他失败指数退避（5s → 300s）。
- 所有 HTTP 都是 SteamWorks 异步请求，不在游戏帧/回调中阻塞。
- 心跳与玩家全量校准独立定时器；全量校准只清理本服务器的会话。
- Replay 只记录 manifest，不在 SourceMod 主线程读取/上传大文件。

## 7. LinuxGSM 部署

```bash
# 以 CS:GO 服务器用户执行
cd ~/csgo/serverfiles/csgo/addons/sourcemod

# 1. 确认 MetaMod / SourceMod / GOKZ / SteamWorks / SMJansson 已安装
ls addons/metamod.vdf
ls addons/sourcemod/extensions/steamworks.ext.so
ls addons/sourcemod/extensions/smjansson.ext.so
ls plugins/gokz-core.smx

# 2. 复制插件源码并编译（或在构建机编译后上传 .smx）
#    spcomp -i addons/sourcemod/scripting/include \
#      -o addons/sourcemod/plugins/cngokz_sync.smx \
#      addons/sourcemod/scripting/cngokz_sync.sp

# 3. 配置 Token（不要提交到 Git）
cp addons/sourcemod/configs/cngokz_sync.cfg cfg/sourcemod/cngokz_sync.cfg
chmod 600 cfg/sourcemod/cngokz_sync.cfg
$EDITOR cfg/sourcemod/cngokz_sync.cfg

# 4. 重启服务器或重载插件
./lgsm restart
# 或在服务器控制台：
# sm plugins reload cngokz_sync
```

`cngokz_sync.cfg`：

```cfg
sm_cngokz_enabled "1"
sm_cngokz_api_base "https://api.example.com"
sm_cngokz_server_token "cngokz_srv_REPLACE_ME"
sm_cngokz_heartbeat_seconds "25"
sm_cngokz_player_sync_seconds "30"
```

检查：

```text
sm_cngokz_sync_status
sm_cngokz_sync_config
```

`sm_cngokz_sync_status` 不打印 Token 明文。

## 8. Replay 上传

SourceMod 只写入：

```text
addons/sourcemod/data/cngokz_replays.log
```

每行包含玩家、地图、Course、模式、时间、`record_identifier` 和本地 Replay 路径。
独立上传器 `tools/cngokz-replay-uploader.py` 负责：

1. `rclone copyto` 上传到 R2/S3；
2. 计算文件大小和 SHA-256；
3. POST `/api/v1/game-servers/replays`；
4. 成功后从 manifest 删除；失败保留等待下次重试。

Cron 示例：

```cron
*/2 * * * * /usr/local/bin/cngokz-replay-uploader \
  --api-base https://api.example.com \
  --token-file /etc/cngokz/server-token \
  --bucket cngokz-replays \
  --rclone-remote r2 \
  --manifest /home/csgo/serverfiles/csgo/addons/sourcemod/data/cngokz_replays.log
```

成绩接收不等 Replay：Replay 失败不会删除已接收成绩，`server_sync_tasks` 保留补偿任务。

## 9. API 环境变量

本阶段没有新增必需环境变量。沿用：

```env
DATABASE_URL=postgres://...
REDIS_URL=redis://...
MAINTENANCE_TOKEN=...          # 管理侧接口
PUBLIC_RATE_LIMIT_PER_MINUTE=120
```

管理后台前端：

```env
VITE_MAINTENANCE_TOKEN=...     # 仅管理后台构建/运行环境使用
```

不要把服务器 Token 或维护 Token 写入仓库。

## 10. 故障排查

| 现象 | 检查 |
| --- | --- |
| 后台显示 `unknown` | 插件是否加载、`sm_cngokz_api_base` 是否可达、Token 是否设置 |
| 心跳 `401` | Token 是否被轮换/撤销；`sm_cngokz_sync_status` |
| 心跳 `403` | 管理员是否禁用了服务器 |
| 在线玩家为空 | 心跳是否过期（>2 分钟）；`server_player_sessions` 是否被本服务器全量校准清理 |
| 成绩未提交 | `sm_cngokz_sync_status` 的 queue 长度；SourceMod 日志中的 HTTP 错误 |
| 成绩重复 | API 以 `(server_id, record_identifier)` 幂等；返回 `duplicate=true` |
| 没有积分 | 该 `(map, course, mode)` 是否配置了 `course_mode_tiers`；查看 `filter_recalc_tasks` |
| Replay 未显示 | manifest/uploader 是否运行；`server_sync_tasks` 是否有失败任务 |
| 迁移失败 | `sqlx migrate run` 的报错；不要手工删表或重置生产库 |

## 11. 测试与验收状态

本次保留的后端接入 API 仍已自动验证（临时数据库 + 真实 Axum 路由 + 真实 PostgreSQL trigger）：

- 服务器认证：正确/错误/撤销 Token、禁用服务器；
- 心跳与 Token 轮换；
- 在线玩家：同玩家多服务器隔离、全量校准、公开在线玩家；
- 成绩：幂等、PB、WR、慢成绩不覆盖 WR、Filter 重算入队；
- Replay：注册、重复注册、状态查询；
- 配置版本：变更/未变更。

命令：

```bash
DATABASE_URL=postgres://... cargo test --all-targets
```

原插件原型曾使用 SourceMod 1.11 `spcomp` 编译通过，但**已因目录规范不符合要求而暂停并移除**；后续不再按当前结构继续开发。

**未执行 / 未验证**：

- 没有真实 CS:GO Dedicated Server + LinuxGSM + GOKZ 的在线联调环境；因此“真实服务器完成一次成绩并自动入库”的最终验收仍需在目标服务器执行。
- Replay 二进制上传仅提供独立 uploader，未在真实 R2/S3 bucket 上执行。
- 未在 GitHub Actions 的真实 runner 上复跑本阶段全量检查。

## 12. 生产注意事项

- `server_heartbeats` 与 `server_sync_logs` 需要定期保留策略（例如 30/90 天）；当前迁移未自动删除历史。
- 禁用服务器会立即断开其在线会话并拒绝后续心跳/成绩。
- 不要修改已合并迁移；后续字段调整新增迁移。
- 不要把 Token 写入普通日志、Git、截图或前端公共 bundle。
- 积分重算仍由第六阶段 worker 异步执行，成绩接口不会同步等待重算完成。
