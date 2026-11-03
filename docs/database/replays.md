# Replays 模块

> 表：`replays`
> 迁移：`20261103000008_replays.sql`

## 1. 研究依据

| 来源 | 行为 |
| --- | --- |
| GOKZ `gokz-replays` | 二进制 replay 文件，格式版本 1/2（`playback.sp` 读取文件头判断版本）；文件可由成绩触发保存并通过 GlobalAPI 上传 |
| GlobalAPI v2 | `POST /records/{recordId}/replay`（上传文件）、`GET /records/{recordId}/replay`、`GET /records/replay/{replayId}`、`GET /records/replay/list`；响应中没有持久化元数据字段 |
| cs2kz-api | 文件存入 S3 兼容存储，数据库只记录 `Records.replay_available`、`ReplayUploadKeys(record_id, upload_key, expires_at)` 与清理表；对象 Key/大小/校验值不在 MySQL 中 |

**结论**：两个上游都不保存完整的 Replay 元数据。CNGOKZAPI 显式保存对象元数据，以便删除/归档、完整性校验与按玩家/地图检索。

## 2. 表结构

| 列 | 类型 | 约束/说明 |
| --- | --- | --- |
| `id` | `bigint identity` PK | |
| `record_id` | `bigint` NOT NULL → `records(id)` ON DELETE CASCADE | 对应成绩 |
| `account_id` | `bigint` NOT NULL → `accounts(id)` | 上传时从成绩快照 |
| `map_id` | `bigint` NOT NULL → `maps(id)` | 上传时从成绩快照 |
| `storage_provider` | `text` DEFAULT `r2` | `r2` / `s3` / `local` |
| `bucket` | `text` NOT NULL，1–128 | 存储桶 |
| `object_key` | `text` NOT NULL，1–512 | R2 Object Key |
| `content_type` | `text` DEFAULT `application/octet-stream` | |
| `size_bytes` | `bigint` NOT NULL CHECK > 0 | |
| `checksum_sha256` | `bytea` NOT NULL CHECK 32 字节 | 文件完整性 |
| `replay_format_version` | `smallint` CHECK > 0 | GOKZ replay 格式版本 |
| `status` | `text` DEFAULT `pending` | `pending` / `available` / `failed` / `deleted` |
| `uploaded_at` | `timestamptz` | |
| `expires_at` | `timestamptz` | 保留截止时间，空 = 永久 |
| `deleted_at` | `timestamptz` | 逻辑删除 |
| `created_at` / `updated_at` | `timestamptz` | |

约束：

- `ck_replays_uploaded_at`：`uploaded_at >= created_at`。
- `ck_replays_expiry`：`expires_at > created_at`。

索引：

| 索引 | 用途 |
| --- | --- |
| `uq_replays_object (bucket, object_key)` | 对象唯一，防重复登记 |
| `idx_replays_record (record_id, created_at DESC)` | 成绩的 Replay 列表 |
| `idx_replays_account (account_id, created_at DESC)` | 玩家 Replay |
| `idx_replays_map (map_id, created_at DESC)` | 地图 Replay |
| `idx_replays_retention (expires_at) WHERE deleted_at IS NULL AND status='available' AND expires_at IS NOT NULL` | 清理任务扫描 |

## 3. 对象存储布局（建议）

```text
r2://cngokz-replays/
  records/{record_id}/{replay_format_version}/{checksum_prefix}.replay
```

- `object_key` 由服务端生成，包含 `record_id` 与校验值前缀，便于人工排查。
- 上传流程（第三阶段）：管理员/插件请求上传票据 → 服务端生成一次性上传地址 → 客户端直传 R2 → 服务端校验大小/SHA-256 → 写入 `replays` 行（`status=available`）。
- 一成绩多 Replay：允许历史版本/重传，通过 `created_at` 区分；展示时取最新 `available`。当前未加“每成绩唯一”约束（cs2kz 通过 `record_id` 主键限制为一份）。

## 4. 生命周期与保留

```
pending（票据已签发/上传中）
   ├─ 上传成功 + 校验通过 → available, uploaded_at, size/checksum
   └─ 上传失败/校验失败 → failed（可重试）
available
   ├─ 到期（expires_at）→ 清理任务删除对象 → status=deleted, deleted_at
   └─ 人工删除 → 同上
```

- **数据库不存文件内容**，只存元数据。
- 对象删除与元数据更新应由清理任务按 `idx_replays_retention` 扫描，先删对象再标记 `deleted`，失败可重试（幂等）。
- `expires_at` 的策略（例如：所有 replay 保留 180 天，WR 永久保留）在第三阶段与产品确认；表结构已支持按行设置。

## 5. 典型查询

```sql
-- 某成绩可用 Replay
SELECT id, bucket, object_key, size_bytes, uploaded_at
FROM replays
WHERE record_id = $1 AND status = 'available' AND deleted_at IS NULL
ORDER BY created_at DESC
LIMIT 1;

-- 待清理对象
SELECT id, bucket, object_key, expires_at
FROM replays
WHERE status = 'available' AND deleted_at IS NULL AND expires_at < now()
ORDER BY expires_at
LIMIT 500;
```

## 6. 待确认事项

1. Replay 保留期策略（全量 / 仅 WR / 按地图）。
2. 是否限制“每成绩一份 Replay”（当前允许多份历史）。
3. 是否需要在数据库中保存上传票据（cs2kz 有 `ReplayUploadKeys`）。
4. 是否需要把 Replay 与 Jumpstats 关联（当前只关联成绩）。
5. 大文件分片上传与内容类型校验规则。
