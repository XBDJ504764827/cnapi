# Maps 模块

> 表：`maps`、`map_versions`、`map_courses`、`map_modes`、`course_tiers`
> 迁移：`20261103000004_maps.sql`

## 1. 研究依据

| 来源 | 字段/规则 |
| --- | --- |
| GOKZ `Maps` / `MapCourses` | `MapID` 自增、`Name` 唯一、`LastPlayed`、`Created`；关卡表 `UNIQUE(MapID, Course)`；`GOKZ_MAX_COURSES = 100`，`Course 0` 为主线，`1..` 为 bonus |
| GlobalAPI v2 `APIMap` | `id`、`name`、`filesize`、`validated`、`difficulty`（单值 tier）、`created_on`、`updated_on`、`approved_by_steamid64` |
| cs2kz `Maps` | `workshop_id`、`name`、`description`、`state`（-1 invalid / 0 in-testing / 1 approved）、`vpk_checksum BINARY(16)`、`approved_at` |
| cs2kz `Courses` / `CourseFilters` | 关卡独立成表；filter = mode + `nub_tier`/`pro_tier`（1–10）+ `state`（unranked/pending/ranked） |

## 2. 表结构

### 2.1 `maps`

| 列 | 类型 | 约束/说明 |
| --- | --- | --- |
| `id` | `bigint identity` PK | |
| `name` | `text` NOT NULL，1–128 | 不含 `.bsp`；大小写不敏感唯一 |
| `workshop_id` | `bigint` | CS2 Workshop 文件 ID；CS:GO 地图为空 |
| `description` | `text` | |
| `status` | `text` NOT NULL DEFAULT `in_testing` | `in_testing` / `approved` / `disabled`（合并 cs2kz MapState 与 GlobalAPI validated） |
| `approved_at` | `timestamptz` | 审核通过时间 |
| `approved_by_account_id` | `bigint` → `accounts(id)` ON DELETE SET NULL | 审核人 |
| `created_at` / `updated_at` | `timestamptz` | |
| `deleted_at` | `timestamptz` | 软删除 |

索引：

- `uq_maps_name_active`：`(lower(name)) WHERE deleted_at IS NULL`
- `uq_maps_workshop_id`：`(workshop_id) WHERE workshop_id IS NOT NULL AND deleted_at IS NULL`
- `idx_maps_status`：`(status, updated_at DESC) WHERE deleted_at IS NULL`

### 2.2 `map_versions`

保存地图内容版本，用于文件校验与变更追踪。

| 列 | 类型 | 说明 |
| --- | --- | --- |
| `id` | `bigint` PK | |
| `map_id` | `bigint` → `maps(id)` ON DELETE CASCADE | |
| `version_label` | `text` | 可选的发布标记（如 workshop 更新时间） |
| `file_size_bytes` | `bigint` | GlobalAPI `filesize` / GOKZ 本地文件大小校验 |
| `checksum` | `bytea` | cs2kz `vpk_checksum`（16 字节）；算法未在上游文档化 |
| `checksum_algorithm` | `text` | 例如 `md5`、`sha256`；与 `checksum` 同时为空或同时有值 |
| `is_current` | `boolean` | 当前版本标记 |
| `created_by_account_id` | `bigint` → `accounts(id)` ON DELETE SET NULL | |
| `created_at` | `timestamptz` | |

索引：

- `uq_map_versions_current`：`(map_id) WHERE is_current`（每个地图最多一个当前版本）
- `uq_map_versions_checksum`：`(map_id, checksum_algorithm, checksum) WHERE checksum IS NOT NULL`
- `idx_map_versions_map`：`(map_id, created_at DESC)`

> 为什么不把 `workshop_id` 放在版本表：两个上游来源都把 workshop ID 视为地图身份的一部分（cs2kz 每张地图一个 workshop_id）。版本表只描述内容变化。

### 2.3 `map_courses`

| 列 | 类型 | 约束/说明 |
| --- | --- | --- |
| `id` | `bigint` PK | |
| `map_id` | `bigint` → `maps(id)` ON DELETE CASCADE | |
| `stage` | `smallint` NOT NULL | `0` = 主线，`1..99` = bonus（GOKZ 编号） |
| `name` | `text` NOT NULL | 展示名；cs2kz 使用自由文本关卡名 |
| `description` | `text` | |
| `created_at` / `updated_at` | `timestamptz` | |

约束与索引：

- `UNIQUE (map_id, stage)`（`uq_map_courses_map_stage`）
- `UNIQUE (id, map_id)`（`uq_map_courses_id_map`）— 供 `records` 的复合外键使用，保证成绩的 `map_id` 与 `course_id` 属于同一张地图
- `idx_map_courses_map (map_id, stage)`

### 2.4 `map_modes`

显式记录地图支持哪些移动模式（存在即支持）。没有该行表示“未确认/不支持”。

| 列 | 类型 |
| --- | --- |
| `map_id` | `bigint` → `maps(id)` ON DELETE CASCADE |
| `mode_id` | `smallint` → `movement_modes(id)` ON DELETE CASCADE |
| `created_at` | `timestamptz` |
| PK | `(map_id, mode_id)` |

索引：`idx_map_modes_mode (mode_id, map_id)`。

### 2.5 `course_tiers`

| 列 | 类型 | 说明 |
| --- | --- | --- |
| `course_id` + `mode_id` | PK | |
| `nub_tier` | `smallint` 1–10，可空 | NUB 榜单难度 |
| `pro_tier` | `smallint` 1–10，可空 | PRO 榜单难度 |
| `status` | `text` DEFAULT `unranked` | `unranked` / `pending` / `ranked` |
| `notes` | `text` | 审核备注 |
| `reviewed_by_account_id` | `bigint` → `accounts(id)` ON DELETE SET NULL | |
| `reviewed_at` | `timestamptz` | |
| `created_at` / `updated_at` | `timestamptz` | |

索引：`idx_course_tiers_mode_tier (mode_id, nub_tier, pro_tier)`。

**设计说明**：GlobalAPI 只暴露一个 `difficulty`；cs2kz 每个 filter 同时有 NUB/PRO tier。CNGOKZAPI 采用后者，因为它能表达“同一关卡 TP 与 PRO 难度不同”的真实情况。导入 GlobalAPI 单值难度时如何填充两列是待确认项（见总览）。

## 3. 生命周期

```
管理员创建地图 (status=in_testing)
  ├─ 可选：写入 map_versions（文件大小/校验和）
  ├─ 创建 map_courses（stage 0..n）
  ├─ 建立 map_modes（支持的移动模式）
  └─ 审核通过 → status=approved, approved_at, approved_by_account_id
审核不通过/下架 → status=disabled（保留成绩）
删除 → deleted_at（软删除，成绩仍引用）
```

- `course_tiers` 的 `status` 与 `course_world_records` 相互独立：未 ranked 的关卡仍然可以记录成绩，但不会出现在正式排行榜。
- 地图改名会破坏历史查询的稳定展示；建议将 `name` 视为不可变，变更需新增迁移/管理流程（待确认）。

## 4. 典型查询

```sql
-- 地图列表（含主线关卡数）
SELECT m.id, m.name, m.status, m.workshop_id,
       count(c.id) FILTER (WHERE c.stage = 0) AS main_courses
FROM maps m
LEFT JOIN map_courses c ON c.map_id = m.id
WHERE m.deleted_at IS NULL AND m.status = 'approved'
GROUP BY m.id
ORDER BY m.name;

-- 关卡 + 每模式难度
SELECT c.id, c.stage, c.name, mm.code AS mode,
       t.nub_tier, t.pro_tier, t.status
FROM map_courses c
JOIN map_modes mm ON mm.map_id = c.map_id
LEFT JOIN course_tiers t ON t.course_id = c.id AND t.mode_id = mm.mode_id
WHERE c.map_id = $1
ORDER BY c.stage, mm.code;

-- 当前内容版本校验
SELECT v.* FROM map_versions v
WHERE v.map_id = $1 AND v.is_current;
```

## 5. 待确认事项

1. 地图改名/作者变更的管理流程。
2. 是否需要 mapper（作者）关联表（cs2kz 有 `Mappers`/`CourseMappers`，本阶段未建模）。
3. GlobalAPI 单值 `difficulty` → `nub_tier`/`pro_tier` 的转换规则。
4. 地图下架（disabled）后成绩是否仍计入玩家积分。
5. Workshop 地图更新后旧版本成绩的处理策略（标记版本、还是按当前版本计入）。
