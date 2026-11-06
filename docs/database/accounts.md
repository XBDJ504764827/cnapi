# Accounts 模块

> 表：`accounts`、`player_profiles`、`player_mode_stats`、`player_identities`
> 迁移：`20261103000003_accounts.sql`

## 1. 研究依据

| 来源 | 字段/规则 |
| --- | --- |
| GOKZ `Players`（gokz-localdb/db/sql.sp） | `SteamID32 INTEGER UNSIGNED`、`Alias`、`Country`、`IP`、`Cheater`、`LastPlayed`、`Created` |
| GlobalAPI v2 `APIPlayer` | `steamid64`、`steam_id`（Steam2）、`is_banned`、`total_records`、`name` |
| GlobalAPI v2 `APIPlayerRank` | `points`、`points_overall`、`average`、`rating`、`finishes` |
| cs2kz `Players` | `id`（SteamID64, INT8 UNSIGNED）、`name`、`ip_address`、`preferences`、`vnl_rating`、`ckz_rating`、`first_joined_at`、`last_joined_at`、`prime_verified` |

**关键结论**：上游对 Steam 身份的表示不一致（GOKZ 用 32 位账号 ID，GlobalAPI 同时返回 Steam2 与 SteamID64，cs2kz 只用 SteamID64）。CNGOKZAPI 以 **SteamID64 为唯一真值**，Steam2 仅作为旧数据导入的辅助字段。

## 2. 表结构

### 2.1 `accounts`

| 列 | 类型 | 约束 | 说明 |
| --- | --- | --- | --- |
| `id` | `bigint identity` | PK | 内部主键 |
| `steam_id64` | `numeric(20,0)` | NOT NULL, CHECK 0..18446744073709551615 | 唯一真值 |
| `steam_id2` | `text` | CHECK `^STEAM_[0-5]:[01]:[0-9]{1,10}$` | 旧格式，可空 |
| `created_at` | `timestamptz` | NOT NULL DEFAULT now() | |
| `updated_at` | `timestamptz` | NOT NULL DEFAULT now() | 触发器维护 |
| `deleted_at` | `timestamptz` | | 软删除 |

索引：

- `uq_accounts_steam_id64_active`：`(steam_id64) WHERE deleted_at IS NULL` — 活跃账号唯一。
- `uq_accounts_steam_id2_active`：`(steam_id2) WHERE deleted_at IS NULL AND steam_id2 IS NOT NULL`。
- `trg_accounts_updated_at`：`set_updated_at()`。

**为什么不是 `bigint`**：SteamID64 是 64 位无符号整数，范围上限 `18446744073709551615`，超过 `bigint`（有符号 64 位）上限 `9223372036854775807`。`NUMERIC(20,0)` 可精确表示全部取值，且可进行数值比较与索引；`CHECK` 防止写入非法值。Rust 侧使用 `BigDecimal`（`sqlx::types::BigDecimal`），对外 JSON 使用**十进制字符串**（与 GlobalAPI 一致）。

### 2.2 `player_profiles`

| 列 | 类型 | 说明 |
| --- | --- | --- |
| `account_id` | `bigint` PK → `accounts(id)` ON DELETE CASCADE | 1:1 |
| `display_name` | `text` NOT NULL，1–64 字符 | 最新昵称 |
| `country` | `text` | 原样保存（GOKZ 存自由文本国家，未做 ISO 归一化） |
| `last_ip` | `inet` | 最近 IP（当前上游均为 IPv4） |
| `preferences` | `jsonb` NOT NULL DEFAULT `{}` | 插件偏好（cs2kz `preferences`） |
| `prime_verified` | `boolean` NOT NULL DEFAULT false | cs2kz `prime_verified` |
| `first_joined_at` / `last_joined_at` | `timestamptz` | 首次/最近进入服务器 |
| `created_at` / `updated_at` | `timestamptz` | |

索引：`idx_player_profiles_display_name` on `lower(display_name)`（前缀/等值查询；模糊搜索的 `pg_trgm` 索引留待需要时再加）。

### 2.3 `player_mode_stats`

按（账号, 移动模式）缓存的派生统计，字段对齐 GlobalAPI `APIPlayerRank`：

| 列 | 类型 | 说明 |
| --- | --- | --- |
| `account_id` + `mode_id` | PK | |
| `points` / `points_overall` | `integer` | GlobalAPI `points` / `points_overall` |
| `rating` / `average` | `numeric(10,4)` | `rating`、`average` |
| `finishes` / `total_records` | `integer` | |
| `updated_at` | `timestamptz` | 触发器维护 |

索引：`idx_player_mode_stats_points (mode_id, points DESC)`。

> 这些数值**不是真值**，必须可由 `records` 重算。第三阶段实现计算任务与刷新策略前，表保持为空。

### 2.4 `player_identities`

外部身份关联（Discord、论坛、QQ 等），Steam 身份仍以 `accounts` 为准。

| 列 | 类型 | 说明 |
| --- | --- | --- |
| `id` | `bigint` PK | |
| `account_id` | `bigint` → `accounts(id)` ON DELETE CASCADE | |
| `provider` | `text`，`^[a-z][a-z0-9_-]*$` | 例如 `discord`、`forum` |
| `identifier` | `text` | 外部 ID |
| `verified_at` | `timestamptz` | 验证时间，可空 |
| `created_at` / `last_seen_at` | `timestamptz` | |

唯一索引：`uq_player_identities_provider_identifier (provider, identifier)`；普通索引：`idx_player_identities_account`。

## 3. 生命周期

```
插件首次见到玩家
   ├─ 按 steam_id64 查 accounts（含软删除过滤）
   ├─ 不存在 → INSERT accounts + INSERT player_profiles（昵称/IP）
   └─ 存在   → UPDATE player_profiles（昵称、last_ip、last_joined_at）
```

- 重命名保留最新昵称，不保存历史（上游也只存当前昵称；昵称历史属于第三阶段可选功能）。
- `is_banned` / `Cheater` **不落库到账号表**：封禁状态由 `bans` 推导，避免双写不一致。
- 账号软删除后，`records`、`bans` 仍然引用该行，历史保留；`player_profiles` 随账号级联删除（隐私删除时可只保留 `accounts.id` 与成绩）。

## 4. 典型查询

```sql
-- SteamID64 登录/查询（参数为十进制字符串，转 numeric）
SELECT id, steam_id64 FROM accounts
WHERE steam_id64 = $1::numeric AND deleted_at IS NULL;

-- 带统计的玩家概览
SELECT a.id, a.steam_id64, p.display_name, p.country,
       s.mode_id, s.points, s.rating, s.finishes
FROM accounts a
LEFT JOIN player_profiles p ON p.account_id = a.id
LEFT JOIN player_mode_stats s ON s.account_id = a.id
WHERE a.steam_id64 = $1::numeric AND a.deleted_at IS NULL;

-- 幂等注册（并发安全）
INSERT INTO accounts (steam_id64) VALUES ($1::numeric)
ON CONFLICT (steam_id64) WHERE deleted_at IS NULL DO NOTHING;
```

> 注意：`ON CONFLICT` 推断部分唯一索引需要写 `WHERE deleted_at IS NULL` 与索引谓词一致。

## 5. 待确认事项

1. `country` 是否需要归一化为 ISO-3166-1 alpha-2（上游 GOKZ 存自由文本，无法直接转换）。
2. 昵称历史是否需要独立表（当前只保留最新值）。
3. 账号合并/多 Steam 身份（同一人多个账号）是否需要显式模型。
4. `player_mode_stats` 的积分公式、刷新频率、是否保留历史快照。
