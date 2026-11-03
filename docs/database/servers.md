# Servers 模块

> 表：`servers`
> 迁移：`20261103000005_servers.sql`

## 1. 研究依据

| 来源 | 字段/规则 |
| --- | --- |
| GlobalAPI v1 插件 | 通过 `X-ApiKey` 请求头认证；Key 由管理员线下配置（`cfg/sourcemod/globalrecords.cfg`），无注册接口 |
| GlobalAPI v2 `APIServer` | `name`、`ip`、`port`、`owner_steamid64` |
| cs2kz `Servers` | `name UNIQUE`、`host`、`port`、`owner_id`、`access_key BINARY(16) UNIQUE`、`approved_at`、`last_connected_at`、`UNIQUE(host, port)` |

**结论**：上游都没有把“API Key 明文”存库（GlobalAPI 由插件持有明文，cs2kz 存 16 字节随机 access key）。CNGOKZAPI 只存 Key 哈希。

## 2. 表结构

| 列 | 类型 | 约束/说明 |
| --- | --- | --- |
| `id` | `bigint identity` PK | |
| `name` | `text` NOT NULL，1–128 | 服务器名；大小写不敏感唯一 |
| `ip` | `inet` NOT NULL | 支持 IPv4/IPv6 |
| `port` | `integer` NOT NULL CHECK 1–65535 | |
| `owner_account_id` | `bigint` NOT NULL → `accounts(id)` | 负责人 |
| `api_key_hash` | `bytea` NOT NULL CHECK `octet_length >= 16` | Key 哈希，唯一 |
| `api_key_hash_algorithm` | `text` NOT NULL DEFAULT `sha256` | 为将来 Argon2id 预留 |
| `api_key_prefix` | `text` NOT NULL，4–16 字符 | 管理端识别用，非机密 |
| `api_key_created_at` | `timestamptz` NOT NULL DEFAULT now() | |
| `api_key_revoked_at` | `timestamptz` | 吊销时间（保留历史） |
| `plugin_version` | `text` | 最近上报的插件版本 |
| `current_map_id` | `bigint` → `maps(id)` ON DELETE SET NULL | 最近心跳中的地图 |
| `is_online` | `boolean` NOT NULL DEFAULT false | 心跳状态 |
| `last_heartbeat_at` | `timestamptz` | 最近心跳 |
| `approved_at` | `timestamptz` | 审核通过 |
| `created_at` / `updated_at` | `timestamptz` | |
| `deleted_at` | `timestamptz` | 软删除 |

索引：

| 索引 | 用途 |
| --- | --- |
| `uq_servers_name_active (lower(name)) WHERE deleted_at IS NULL` | 名称唯一 |
| `uq_servers_address_active (ip, port) WHERE deleted_at IS NULL` | 地址唯一 |
| `uq_servers_api_key_hash (api_key_hash)` | 认证查找（全局唯一，含已删除服务器，防止 Key 复用） |
| `idx_servers_online (is_online, last_heartbeat_at DESC) WHERE deleted_at IS NULL` | 在线列表 |
| `idx_servers_owner (owner_account_id)` | 我的服务器 |
| `idx_servers_current_map (current_map_id) WHERE current_map_id IS NOT NULL` | 按地图查服务器 |

## 3. API Key 设计

1. **生成**：服务端生成 32 字节随机值，Base64URL/十六进制编码后一次性返回给管理员（第三阶段）。
2. **存储**：仅存 `sha256(key)` 到 `api_key_hash`、前 8 位到 `api_key_prefix`；明文不落库、不写日志。
3. **认证**：插件请求携带 Key → 服务端对 Key 做 SHA-256 → 用 `uq_servers_api_key_hash` 查找 → 校验 `api_key_revoked_at IS NULL` 与 `deleted_at IS NULL`。
4. **轮换**：更新哈希并刷新前缀；旧 Key 立即失效。`api_key_created_at` 记录最近一次轮换时间。
5. **为什么不加盐/Argon2**：API Key 是 32 字节高熵随机值，暴力破解不可行，单轮 SHA-256 足够且便于等值索引（Argon2 无法索引）。若未来引入低熵 Key，需改为带前缀的哈希查找表（待确认）。

## 4. 在线状态与心跳

- 插件按固定间隔（建议 30–60 秒，第三阶段定）上报心跳：`plugin_version`、`current_map_id`、在线人数（本阶段不存人数）。
- 每次心跳执行：

```sql
UPDATE servers
SET is_online = true,
    last_heartbeat_at = now(),
    plugin_version = $2,
    current_map_id = $3
WHERE id = $1 AND deleted_at IS NULL;
```

- **离线判定由读取端完成**：`is_online AND last_heartbeat_at > now() - interval '2 minutes'`。`is_online` 只是缓存标记，避免依赖定时任务；第三阶段可加后台任务把超时服务器置为离线。
- 不为每次心跳写历史行，避免高频写入；需要在线率统计时再新增 `server_heartbeats` 分区表。

## 5. 生命周期

```
管理员创建服务器（owner + ip/port + 名称）
  → 生成 API Key（仅展示一次）
  → approved_at 设置
插件上线 → 心跳 → is_online=true, current_map_id, plugin_version
插件下线/失联 → 心跳超时 → 读取端显示离线（可选后台任务置 false）
停用 → api_key_revoked_at（立即拒绝提交）
删除 → deleted_at（软删除；成绩保留，server_id 仍可关联）
```

## 6. 典型查询

```sql
-- 认证（Key 的哈希由应用层计算）
SELECT id, name, plugin_version, current_map_id
FROM servers
WHERE api_key_hash = $1 AND api_key_revoked_at IS NULL AND deleted_at IS NULL;

-- 在线服务器（含心跳新鲜度）
SELECT id, name, ip, port, current_map_id, last_heartbeat_at
FROM servers
WHERE deleted_at IS NULL
  AND is_online
  AND last_heartbeat_at > now() - interval '2 minutes'
ORDER BY name;
```

## 7. 待确认事项

1. 心跳间隔与离线阈值（建议 30–60 秒心跳、2 倍超时判定离线）。
2. 是否需要在心跳中记录在线玩家数/地图 tickrate（当前不存）。
3. 服务器 owner 是否允许转移、是否需要多管理员。
4. 是否需要“服务器分组/地区/标签”（GlobalAPI 无此字段）。
5. API Key 的权限范围（scope）是否需要（例如只允许提交成绩、不允许 Jumpstats）。
