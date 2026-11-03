# Jumpstats 模块

> 表：`jumpstats`（引用 `jump_types`）
> 迁移：`20261103000007_jumpstats.sql`

## 1. 研究依据

| 来源 | 字段/语义 |
| --- | --- |
| GOKZ `Jump` struct（jumpstats.inc） | `type`、`distance`（float 游戏单位）、`strafes`、`sync`（%）、`preSpeed`/`maxSpeed`、`block`、`duration`（tick）、`deadair`、`overlap`、`deviation`、`height`、`width` 等 |
| GOKZ 本地 `Jumpstats` 表 | `JumpType`、`Mode`、`Distance`（整数）、`IsBlockJump`、`Block`、`Strafes`、`Sync`、`Pre`、`Max`、`Airtime`（整数） |
| GlobalAPI v2 `POST /jumpstats` | `steamId`、`jump_type`（int）、`distance`（float）、`jump_json_info`（JSON 字符串）、`tickrate`、`msl_count`、`is_crouch_bind`、`is_forward_bind`、`is_crouch_boost`、`strafe_count` |
| GlobalAPI v2 `APIJumpstat` | `id`、`server_id`、`steamid64`、`steam_id`、`player_name`、`jump_type`、`distance`、`tickrate`、`msl_count`、`strafe_count`、`is_crouch_bind`、`is_forward_bind`、`is_crouch_boost`、`updated_by_id`、`created_on`、`updated_on` |
| cs2kz `Jumps` | `type`、`mode`、`styles`（位掩码）、`time`（秒）、`strafes`、`distance`、`sync`、`pre`、`max`、`overlap`、`bad_angles`、`dead_air`、`height`、`airpath`、`deviation`、`average_width`（均为 float） |

## 2. 表结构

| 列 | 类型 | 约束/说明 |
| --- | --- | --- |
| `id` | `bigint identity` PK | |
| `account_id` | `bigint` → `accounts(id)` | 玩家 |
| `map_id` | `bigint` → `maps(id)` | 地图 |
| `server_id` | `bigint` → `servers(id)` ON DELETE SET NULL | 来源服务器 |
| `mode_id` | `smallint` → `movement_modes(id)` | 移动模式 |
| `style_id` | `smallint` → `movement_styles(id)` | 移动风格 |
| `jump_type_id` | `smallint` → `jump_types(id)` | 跳跃类型 |
| `distance_units` | `numeric(12,4)` NOT NULL CHECK >= 0 | 距离（游戏单位，精确小数） |
| `strafe_count` | `smallint` CHECK >= 0 | 转身次数 |
| `sync_percent` | `numeric(6,3)` CHECK 0–100 | 同步率 |
| `airtime_ns` | `bigint` CHECK > 0 | 滞空时间（纳秒，可空） |
| `block` | `integer` CHECK >= 0 | Block 编号（GOKZ `Jump.block`） |
| `is_block_jump` | `boolean` NOT NULL DEFAULT false | 是否 Block Jump |
| `msl_count` | `smallint` CHECK >= 0 | GlobalAPI `msl_count` |
| `is_crouch_bind` / `is_forward_bind` / `is_crouch_boost` | `boolean` NOT NULL DEFAULT false | GlobalAPI 反作弊标记 |
| `tickrate` | `smallint` NOT NULL CHECK 1–1000 | |
| `source` | `text` NOT NULL DEFAULT `plugin` | 与 records 相同的来源枚举 |
| `submission_id` | `uuid` | 幂等键（`server_id` + `submission_id` 唯一） |
| `external_ref` | `text` | 导入时的上游 ID（`source` + `external_ref` 唯一） |
| `raw_metrics` | `jsonb` NOT NULL DEFAULT `{}` | 来源原始指标（见下文） |
| `created_at` / `updated_at` | `timestamptz` | |
| `deleted_at` | `timestamptz` | 软删除 |

索引：

| 索引 | 用途 |
| --- | --- |
| `idx_jumpstats_top (jump_type_id, distance_units DESC) WHERE deleted_at IS NULL` | 全服 Jumpstats 榜 |
| `idx_jumpstats_map_top (map_id, jump_type_id, distance_units DESC) WHERE deleted_at IS NULL` | 地图 Jumpstats 榜 |
| `idx_jumpstats_account (account_id, created_at DESC)` | 玩家记录 |
| `idx_jumpstats_server (server_id, created_at DESC)` | 服务器审计 |
| `uq_jumpstats_submission` / `uq_jumpstats_external_ref` | 幂等 |

## 3. 指标语义与 `raw_metrics`

上游对同名指标的语义并不一致：

- GOKZ `Jump.preSpeed` / `maxSpeed` 是**速度**（units/s），`sync` 是**百分比**。
- cs2kz `pre` / `max` 在 API 中以 float 暴露，命名简短，未在源码注释中说明是速度还是百分比。
- GlobalAPI 的 `jump_json_info` 是一个由插件构造的 JSON 字符串，字段未在上游仓库中定义。

因此本阶段**只提升语义明确的指标**为列：`distance_units`、`strafe_count`、`sync_percent`、`airtime_ns`、`block`、`is_block_jump`、`msl_count`、绑定标记、`tickrate`。其余指标原样保存在 `raw_metrics`：

```json
{
  "pre": 285.31,
  "max": 312.4,
  "overlap": 0.0,
  "bad_angles": 12.5,
  "dead_air": 0.0,
  "height": 64.0,
  "airpath": 288.7,
  "deviation": 3.2,
  "average_width": 42.1
}
```

后续确认单位后，可用新迁移把它们提升为独立列（`ALTER TABLE ... ADD COLUMN` + 回填），不会破坏现有数据。

## 4. 幂等与重复提交

- 与成绩相同的策略：`(server_id, submission_id)` 唯一、`(source, external_ref)` 唯一。
- GlobalAPI 没有提交 ID，导入时优先使用上游 `id` 作为 `external_ref`。
- Jumpstats 数据量预计远大于成绩（每玩家每次跳跃一条），索引已按“榜单（类型+距离）”“玩家”“地图”“服务器”设计；暂不分区。

## 5. 典型查询

```sql
-- 某跳跃类型全服 Top 20
SELECT j.id, p.display_name, j.distance_units, j.created_at
FROM jumpstats j
JOIN player_profiles p ON p.account_id = j.account_id
JOIN jump_types t ON t.id = j.jump_type_id
WHERE t.code = 'long-jump' AND j.deleted_at IS NULL
ORDER BY j.distance_units DESC
LIMIT 20;

-- 玩家在某地图的 Jumpstats
SELECT t.code, j.distance_units, j.created_at
FROM jumpstats j
JOIN jump_types t ON t.id = j.jump_type_id
WHERE j.account_id = $1 AND j.map_id = $2 AND j.deleted_at IS NULL
ORDER BY j.created_at DESC
LIMIT 100;
```

## 6. 待确认事项

1. `pre`/`max`/`overlap` 等指标的单位与统计方式（速度 vs 百分比）。
2. GOKZ `duration`（tick）→ `airtime_ns` 的换算是否需要按 tickrate 进行（预计是 `duration * tick_interval`）。
3. Jumpstats 是否需要审核状态（当前只有软删除；GlobalAPI 有 `updated_by_id` 字段）。
4. 是否需要按 `msl_count` / crouch bind 等反作弊标记过滤榜单。
5. 数据保留周期（Jumpstats 增长最快）。
