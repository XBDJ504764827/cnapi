# Records 模块

> 表：`records`、`record_history`、`course_world_records`
> 迁移：`20261103000006_records.sql`

## 1. 研究依据

| 来源 | 字段/规则 |
| --- | --- |
| GOKZ `Times`（gokz-localdb） | `SteamID32`、`MapCourseID`、`Mode`、`Style`、`RunTime`（整数，本地库为有损存储）、`Teleports`、`Created` |
| GOKZ `GOKZ_GetTimeTypeEx` | `teleports == 0` → `TimeType_Pro`，否则 `TimeType_Nub` |
| GlobalAPI v1 `POST /records` | `{steam_id (Steam2), map_id, mode (字符串), stage, tickrate, teleports, time (float 秒)}`；响应 `{place, top_100, top_100_overall, record_id}` |
| GlobalAPI v2 `APIRecord` | `id`、`steamid64`、`steam_id`、`player_name`、`server_id`、`map_id`、`stage`、`mode`、`tickrate`、`time`（float）、`teleports`、`created_on`、`updated_on`、`updated_by`、`server_name`、`map_name` |
| cs2kz `Records` | `id UUID`、`player_id`、`server_id`、`filter_id`、`styles`（位掩码）、`teleports`、`time FLOAT8`、`plugin_version_id`；榜单用 `BestNubRecords`/`BestProRecords`，filter 编码 mode+tickrate+has_teleports |

## 2. 表结构

### 2.1 `records`

| 列 | 类型 | 约束/说明 |
| --- | --- | --- |
| `id` | `bigint identity` PK | |
| `account_id` | `bigint` → `accounts(id)` | 玩家 |
| `map_id` | `bigint` → `maps(id)` | 冗余列（见下方复合外键） |
| `course_id` | `bigint` | 关卡 |
| `mode_id` | `smallint` → `movement_modes(id)` | 移动模式 |
| `style_id` | `smallint` → `movement_styles(id)` | 移动风格 |
| `server_id` | `bigint` → `servers(id)` ON DELETE SET NULL | 来源服务器；导入数据可为空 |
| `server_plugin_version` | `text` | 提交时插件版本（cs2kz 存 `plugin_version_id`，这里保留原始版本串） |
| `tickrate` | `smallint` NOT NULL CHECK 1–1000 | GlobalAPI 每条成绩都带 tickrate |
| `teleports` | `integer` NOT NULL CHECK >= 0 | TP 次数 |
| `time_type` | `text` NOT NULL | `nub` / `pro` |
| `time_ns` | `bigint` NOT NULL CHECK > 0 | 成绩时长（纳秒） |
| `status` | `text` NOT NULL DEFAULT `pending` | `pending` / `approved` / `rejected` |
| `source` | `text` NOT NULL DEFAULT `plugin` | `plugin` / `admin` / `globalapi_import` / `cs2kz_import` / `legacy_import` |
| `submission_id` | `uuid` | 插件幂等键 |
| `external_ref` | `text` | 上游成绩 ID（GlobalAPI id / cs2kz UUID） |
| `reviewed_by_account_id` | `bigint` → `accounts(id)` ON DELETE SET NULL | 人工审核人 |
| `reviewed_at` / `review_note` | `timestamptz` / `text` | |
| `created_at` | `timestamptz` DEFAULT now() | 入库时间（导入时可显式写入上游 `created_on`） |
| `updated_at` | `timestamptz` | |
| `deleted_at` | `timestamptz` | 软删除 |

约束：

- `fk_records_course_map FOREIGN KEY (course_id, map_id) REFERENCES map_courses (id, map_id)`：保证 `map_id` 与 `course_id` 一致，堵住冗余列漂移的唯一入口。
- `ck_records_tickrate`、`ck_records_teleports`、`ck_records_time_ns`。
- `ck_records_time_type`：`nub`/`pro`。
- `ck_records_time_type_matches_teleports`：`pro ⇔ teleports = 0`（GOKZ 规则）。
- `ck_records_status`、`ck_records_source`：与 Rust 枚举一一对应。

索引：

| 索引 | 用途 |
| --- | --- |
| `uq_records_submission (server_id, submission_id) WHERE ...` | 插件重试幂等 |
| `uq_records_external_ref (source, external_ref) WHERE ...` | 导入幂等 |
| `idx_records_leaderboard (course_id, mode_id, style_id, time_type, time_ns) WHERE status='approved' AND deleted_at IS NULL` | 榜单 / 世界纪录查询 |
| `idx_records_account (account_id, created_at DESC)` | 玩家成绩列表 |
| `idx_records_map (map_id, created_at DESC)` | 地图最近成绩 |
| `idx_records_server (server_id, created_at DESC)` | 服务器提交审计 |
| `idx_records_review_queue (created_at) WHERE status='pending' AND deleted_at IS NULL` | 审核队列 |

### 2.2 `record_history`

追加式审计，不提供 UPDATE/DELETE 语义（除级联删除成绩时随之删除）。

| 列 | 类型 | 说明 |
| --- | --- | --- |
| `id` | `bigint` PK | |
| `record_id` | `bigint` → `records(id)` ON DELETE CASCADE | |
| `action` | `text` | `created` / `status_changed` / `time_corrected` / `reviewed` / `soft_deleted` / `restored` / `other` |
| `old_status` / `new_status` | `text` | 审核状态变化 |
| `old_time_ns` / `new_time_ns` | `bigint` | 成绩修正前后 |
| `changed_by_account_id` | `bigint` → `accounts(id)` ON DELETE SET NULL | 操作者 |
| `reason` | `text` | |
| `metadata` | `jsonb` DEFAULT `{}` | 结构化补充（例如审核规则命中） |
| `created_at` | `timestamptz` | |

索引：`(record_id, created_at DESC)`、`(changed_by_account_id, created_at DESC)`。

### 2.3 `course_world_records`

每个（关卡, 模式, 风格, TP/PRO）维度的当前 WR 指针。

| 列 | 类型 | 说明 |
| --- | --- | --- |
| `course_id` + `mode_id` + `style_id` + `time_type` | PK | 榜单维度 |
| `record_id` | `bigint` → `records(id)` ON DELETE CASCADE | |
| `time_ns` | `bigint` | 缓存的获胜时间（应用层保证与 record 一致） |
| `achieved_at` | `timestamptz` | |
| `updated_at` | `timestamptz` | 触发器维护 |

索引：`idx_course_world_records_record (record_id)`。

## 3. 成绩时间精度

| 环节 | 表示 |
| --- | --- |
| 插件 / 上游 API | `float` 秒（GOKZ `float time`、GlobalAPI `time`、cs2kz `Seconds`） |
| CNGOKZAPI 数据库 | `time_ns BIGINT`（整数纳秒） |
| CNGOKZAPI API（第三阶段） | `time_ns`（精确排序）+ `time_seconds`（十进制字符串，9 位小数） |
| 展示 | 由前端格式化，不参与比较 |

转换规则：`time_ns = round(seconds × 1_000_000_000)`。128 tick 的一帧 = 7.8125 ms = 7 812 500 ns，可被精确表示；64 tick = 15 625 000 ns 同样精确。禁止把 `time_ns` 再转成 `f64` 参与排序。

> 参考：cs2kz 的 `Seconds` 在数据库中是 `FLOAT8`，GlobalAPI 也是浮点秒。CNGOKZAPI 采用整数纳秒是为了避免“相同成绩因浮点舍入出现不同排序”以及 `time = 9.999999999` 之类的边界问题。

## 4. TP / PRO 规则

依据 GOKZ `GOKZ_GetTimeTypeEx`：

```text
teleports == 0  → PRO
teleports  > 0  → NUB
```

数据库中由 CHECK 约束强制：

```sql
CHECK ((time_type = 'pro' AND teleports = 0) OR (time_type = 'nub' AND teleports > 0))
```

因此不存在“标为 PRO 但有传送”或“标为 NUB 但零传送”的脏数据；写入端也无需信任客户端传入的 time_type（服务端可强制计算）。

## 5. 幂等、重复提交与并发 WR

### 5.1 重复提交

1. 原生插件：每条成绩带 `submission_id`（UUID，插件生成）。重试不会产生第二行：

```sql
INSERT INTO records (..., submission_id) VALUES (...)
ON CONFLICT (server_id, submission_id) WHERE submission_id IS NOT NULL AND server_id IS NOT NULL
DO NOTHING
RETURNING id;
```

2. 导入：使用 `(source, external_ref)` 唯一索引，重复导入同理。

3. GlobalAPI v1/v2 原始协议**没有**提交 ID。其兼容层在导入时必须依赖 `external_ref`（GlobalAPI record id），或者在没有 id 的情况下按内容去重；仅靠内容去重可能误杀合法的相同成绩，因此该策略作为待确认项（见 [globalapi-compatibility.md](../globalapi-compatibility.md)）。

### 5.2 并发世界纪录更新

`course_world_records` 只能通过条件 upsert 更新：

```sql
INSERT INTO course_world_records
    (course_id, mode_id, style_id, time_type, record_id, time_ns, achieved_at)
VALUES ($1, $2, $3, $4, $5, $6, now())
ON CONFLICT (course_id, mode_id, style_id, time_type) DO UPDATE
SET record_id   = EXCLUDED.record_id,
    time_ns     = EXCLUDED.time_ns,
    achieved_at = EXCLUDED.achieved_at
WHERE EXCLUDED.time_ns < course_world_records.time_ns;
```

- 并发的两个快慢成绩不会互相覆盖：`WHERE` 子句在行锁下重新求值。
- 相同时间（`time_ns` 相等）不更新，先到者保持 WR；如需“同分取更早/更晚”，在此处补充规则（待确认）。
- 集成测试 `world_record_upsert_cannot_be_overwritten_by_slower_runs` 覆盖了该行为。

### 5.3 成绩审核与 WR 的关系

- 只有 `status = 'approved'` 且未软删除的成绩进入榜单索引。
- 审核把 `pending → approved` 时，应在**同一事务**内重算对应维度的 WR（条件 upsert），确保指针与成绩状态一致。
- `rejected` / `deleted` 的成绩如果曾是 WR，需要回退到次优成绩：由第三阶段的重算任务处理（当前 schema 未保存“次优”缓存，直接查 `records` 即可）。

## 6. 典型查询

```sql
-- 某关卡某模式的 PRO 榜单 Top 50（精确整数排序）
SELECT r.id, p.display_name, r.time_ns, r.created_at
FROM records r
JOIN player_profiles p ON p.account_id = r.account_id
WHERE r.course_id = $1 AND r.mode_id = $2 AND r.style_id = $3
  AND r.time_type = 'pro' AND r.status = 'approved' AND r.deleted_at IS NULL
ORDER BY r.time_ns
LIMIT 50;

-- 玩家最佳 NUB 成绩（每关卡）
SELECT DISTINCT ON (course_id) course_id, time_ns
FROM records
WHERE account_id = $1 AND time_type = 'nub'
  AND status = 'approved' AND deleted_at IS NULL
ORDER BY course_id, time_ns;

-- 审核队列
SELECT id, account_id, course_id, time_ns, created_at
FROM records
WHERE status = 'pending' AND deleted_at IS NULL
ORDER BY created_at
LIMIT 100;
```

## 7. 待确认事项

1. 榜单是否按 `tickrate` 拆分（GlobalAPI 的 `records/top` 支持 tickrate 参数；当前 `course_world_records` 主键未包含 tickrate）。
2. 是否需要按 `style` 拆分的正式榜单（当前 style 维度已在主键中，但只有 `normal` 有数据）。
3. 相同 `time_ns` 的排名规则（并列 / 先到者优先）。
4. 成绩审核策略：自动通过（插件校验）与人工抽检的比例、`pending` 是否只用于可疑成绩。
5. GlobalAPI `place` / `top_100` / `top_100_overall` 是响应时计算值，不落库；第三阶段兼容层按需实时计算。
6. 是否需要保存每 tick 的检查点/分段数据（当前只有总时间，属于 Replay/未来功能）。
