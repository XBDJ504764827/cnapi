# Moderation 模块

> 表：`bans`、`ban_revocations`
> 迁移：`20261103000009_moderation.sql`

## 1. 研究依据

| 来源 | 字段/规则 |
| --- | --- |
| cs2kz `Bans` | `player_id`、`player_ip`、`banned_by`（`Server(server_id)` 或 `Admin(user_id)` 单列编码）、`reason`、`plugin_version_id`、`created_at`、`expires_at`（CHECK >= created_at） |
| cs2kz `Unbans` | `ban_id` PK、`admin_id`、`reason`、`created_at` |
| GlobalAPI v1 `POST /bans` | `{steam_id, ban_type, notes, stats, ip}` |
| GlobalAPI v2 `APIBan` | `id`、`steamid64`、`steam_id`、`player_name`、`notes`、`stats`、`ban_type`、`expires_on`、`server_id`、`created_on`、`updated_on`、`updated_by_id` |
| GOKZ `Players.Cheater` | 布尔标记，不是封禁记录 |

## 2. 表结构

### 2.1 `bans`

| 列 | 类型 | 约束/说明 |
| --- | --- | --- |
| `id` | `bigint identity` PK | |
| `account_id` | `bigint` NOT NULL → `accounts(id)` | 被处罚玩家 |
| `ip` | `inet` | 封禁时的 IP（GlobalAPI 携带） |
| `ban_type` | `text` DEFAULT `global`，1–64 | 自由文本（GlobalAPI 未定义枚举） |
| `reason` | `text` NOT NULL，1–1024 | |
| `stats` | `text` | GlobalAPI `stats` 原文 |
| `issued_by_account_id` | `bigint` → `accounts(id)` ON DELETE SET NULL | 管理员 |
| `issued_by_server_id` | `bigint` → `servers(id)` ON DELETE SET NULL | 服务器自动封禁 |
| `issued_at` | `timestamptz` DEFAULT now() | 开始时间 |
| `expires_at` | `timestamptz` | 结束时间；NULL = 永久 |
| `status` | `text` DEFAULT `active` | `active` / `revoked`（**不存 expired**，由时间推导） |
| `source` | `text` DEFAULT `admin` | `admin` / `server` / `globalapi_import` |
| `external_ref` | `text` | 上游 ID |
| `created_at` / `updated_at` | `timestamptz` | |

约束：

- `ck_bans_period`：`expires_at > issued_at`。
- `ck_bans_issuer`：`issued_by_account_id` 或 `issued_by_server_id` 至少一个非空（记录责任方）。
- `ck_bans_status` / `ck_bans_source`。

索引：

| 索引 | 用途 |
| --- | --- |
| `uq_bans_external_ref (external_ref) WHERE NOT NULL` | 导入幂等 |
| `idx_bans_account_active (account_id, issued_at DESC) WHERE status='active'` | 玩家当前封禁 |
| `idx_bans_expiry (expires_at) WHERE status='active' AND expires_at IS NOT NULL` | 到期扫描 |
| `idx_bans_issued_by_admin` / `idx_bans_issued_by_server` | 审计 |

### 2.2 `ban_revocations`

| 列 | 类型 | 说明 |
| --- | --- | --- |
| `id` | `bigint identity` PK | |
| `ban_id` | `bigint` NOT NULL UNIQUE → `bans(id)` ON DELETE CASCADE | 一个封禁最多一次解封 |
| `revoked_by_account_id` | `bigint` → `accounts(id)` ON DELETE SET NULL | 管理员 |
| `revoked_by_server_id` | `bigint` → `servers(id)` ON DELETE SET NULL | 服务器撤销 |
| `reason` | `text` NOT NULL，1–1024 | |
| `revoked_at` | `timestamptz` DEFAULT now() | |

约束：`ck_ban_revocations_actor`（至少一个操作者）。

## 3. 状态语义

```text
                    ┌────────────── expires_at 到期（时间推导）──────────────▶ 失效
active ─────────────┤
                    └────────────── ban_revocations 存在 ──▶ revoked
```

- **不存储 `expired`**：避免后台任务与时钟不一致；查询时统一使用：

```sql
WHERE b.status = 'active'
  AND (b.expires_at IS NULL OR b.expires_at > now())
```

- 解封是**追加记录**：插入 `ban_revocations` 的同时把 `bans.status` 置为 `revoked`（同一事务）。原始封禁原因、时间、操作者不变。
- 封禁记录不可删除（审计要求）；如需“删除错误封禁”，使用解封并附原因。
- IP 封禁：`bans.ip` 与 `account_id` 同时存在；查询当前是否被封禁需同时匹配账号与 IP 规则（第三阶段定义 IP 匹配范围，例如精确 IP 还是网段）。

## 4. 典型查询

```sql
-- 玩家当前有效封禁
SELECT b.id, b.ban_type, b.reason, b.issued_at, b.expires_at
FROM bans b
WHERE b.account_id = $1
  AND b.status = 'active'
  AND (b.expires_at IS NULL OR b.expires_at > now())
ORDER BY b.issued_at DESC;

-- 解封（同一事务）
INSERT INTO ban_revocations (ban_id, revoked_by_account_id, reason)
VALUES ($1, $2, $3);
UPDATE bans SET status = 'revoked' WHERE id = $1;

-- 即将到期的封禁（监控/提醒）
SELECT id, account_id, expires_at
FROM bans
WHERE status = 'active' AND expires_at BETWEEN now() AND now() + interval '1 day';
```

## 5. 与 GOKZ `Cheater` 标记的关系

GOKZ 的本地 `Cheater` 布尔值并非封禁记录，也不能区分来源/时长。CNGOKZAPI 不保存该字段：

- “是否被封禁”由 `bans` 推导（类似 cs2kz 的 `is_banned` 计算字段）。
- 若未来需要“标记为作弊但不封禁”，应新增独立标记表并记录操作者，而不是复用 `bans`。

## 6. 待确认事项

1. `ban_type` 的规范取值（GlobalAPI 未定义；当前自由文本）。
2. IP 封禁匹配规则（精确 IP / 网段 / 是否只作为辅助证据）。
3. 封禁是否影响已有成绩（删除/隐藏/保留但标记）。
4. 解封是否需要审批流程/双人复核。
5. 是否需要封禁申诉与工单模型（当前未建）。
