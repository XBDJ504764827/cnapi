# 架构说明

## 1. 总体架构

项目采用**前后端分离 + 模块化单体**结构，同一个仓库（monorepo）内维护：

```text
                     ┌──────────────────────────────┐
  浏览器 ──────────▶ │ apps/web (管理后台 SPA)       │
                     │ 静态资源 + 同源反向代理       │
                     └──────────────┬───────────────┘
                     ┌──────────────────────────────┐
  浏览器 ──────────▶ │ apps/public (公开门户 SPA)    │
                     │ `/` 公开站 + `/admin` 后台     │
                     │ 独立布局 + 同源反向代理       │
                     └──────────────┬───────────────┘
                                    │ /api, /health, /swagger-ui, /api-docs
                                    ▼
                     ┌──────────────────────────────┐
                     │  apps/api (Axum)             │
                     │  routes → handlers → services│
                     │            → repositories    │
                     └───────┬──────────────┬───────┘
                             │              │
                    SQLx pool│              │ConnectionManager
                             ▼              ▼
                     ┌──────────────┐  ┌──────────────┐
                     │ PostgreSQL   │  │ Redis        │
                     │ 持久化数据    │  │ 缓存/临时状态 │
                     └──────────────┘  └──────────────┘
```

设计约束：

- **不拆分微服务**。KZ 业务按模块（players、maps、records、jumpstats、replays、servers、bans、auth）在同一进程内扩展。
- 前后端各自独立构建：API 由 systemd 运行，public/admin 静态文件由 Nginx 提供，Nginx 反向代理 `/api`、`/health`。
- API 只监听 `127.0.0.1:8080`；数据库与 Redis 通过宿主机回环地址访问，不直接暴露到公网。

## 2. 前后端关系

| 关注点 | 做法 |
| --- | --- |
| 请求链路 | 页面 → hooks → `services/*` → `@cngokz/api-client` → HTTP |
| 地址配置 | 浏览器默认访问同源（`VITE_API_BASE_URL` 为空）；开发环境由 Vite `server.proxy` 转发，生产环境由 Nginx 转发到 `127.0.0.1:8080` |
| 类型来源 | 后端 `Utoipa` 生成 `docs/openapi.json` → `openapi-typescript` 生成 `packages/shared-types/src/generated/api-schema.d.ts` → `openapi-fetch` 提供类型化请求 |
| 数据获取 | TanStack Query 统一缓存、轮询、错误重试；页面不直接调用 `fetch` |
| 公开门户 | `apps/public` 独立 Vite 应用，默认站点 `/`，同一端口下 `/admin` 反向代理管理后台；共享 API Client、shared-types 与设计 Token（`packages/ui`）；详见 [public-portal/architecture.md](public-portal/architecture.md) |
| UI 状态 | Zustand（主题、侧边栏折叠），持久化到 `localStorage` |
| 主题 | Tailwind v4 CSS 变量 + `<html class="dark">`；`index.html` 首屏前应用持久化主题，避免闪烁；默认深色 |
| 设计系统 | Token 在 `styles/index.css`，基础组件在 `components/ui`，页面只组合组件；规范见 [design-system.md](design-system.md) |

跨域：默认同源，不需要 CORS。直接访问 API（例如 `localhost:8080`）时由 `CORS_ALLOWED_ORIGINS` 控制允许的来源，`*` 仅用于本地调试。

## 3. 后端分层

```text
请求
 │
 ├─ middleware   CORS、x-request-id、tracing span
 ├─ routes       路径与 OpenAPI 装配（/health, /api/v1/*, /swagger-ui）
 ├─ handlers     提取参数、校验、调用 service、包装统一响应
 ├─ services     业务编排（健康检查、系统信息、事件查询）
 ├─ repositories SQL / Redis 访问，不含业务规则
 └─ models       内部领域类型（DependencyCheck、SystemEvent…）
```

| 目录 | 责任 | 禁止事项 |
| --- | --- | --- |
| `config/` | 读取并校验环境变量，只此一处读 `std::env` | 业务逻辑 |
| `state/` | `AppState`（`Arc` 共享）、连接池、Redis 客户端 | 直接处理 HTTP |
| `middleware/` | 横切关注点 | 业务逻辑 |
| `routes/` | 路由与 OpenAPI 文档装配 | 业务逻辑 |
| `handlers/` | 输入输出、状态码 | SQL、复杂分支 |
| `services/` | 编排、依赖检查、错误映射 | 直接写 SQL |
| `repositories/` | SQL 查询 | 返回 HTTP 类型 |
| `models/` | 领域模型 | 序列化给客户端 |
| `dto/` | API 契约（serde + utoipa） | 数据库访问 |
| `errors/` | `AppError`（HTTP 错误）、`StartupError`（启动错误） | 吞掉内部错误 |
| `utils/` | 脱敏、日志初始化等无状态工具 | 与请求上下文耦合 |

### 启动流程（`main.rs`）

1. `dotenvy::dotenv()`（可选；生产也可由 systemd 环境变量提供）
2. `AppConfig::from_env()` 校验配置，失败立即退出
3. 初始化 tracing：开发环境 compact 文本，生产环境 JSON
4. `state::connect()`：建立 PostgreSQL 连接池 → 执行 `sqlx::migrate!` → 创建 Redis 客户端并做一次 best-effort `PING`
5. 绑定 `TcpListener`，`axum::serve(...).with_graceful_shutdown(...)`
6. 收到 SIGINT/SIGTERM 后停止接收新请求，关闭连接池后退出

> PostgreSQL 是硬依赖（迁移需要），启动失败即退出；Redis 允许启动时不可用，`/health/ready` 会返回 503，恢复后自动重连。

### 关键实现细节

- **Redis 连接管理**：`state/redis_client.rs` 封装 `redis::aio::ConnectionManager`，懒连接、`PING` 失败时丢弃缓存连接并在下次请求重建，避免进程重启。
- **超时**：所有依赖探测受 `HEALTH_CHECK_TIMEOUT_MS` 限制，使用 `tokio::time::timeout`。
- **请求 ID**：`tower-http` 的 `SetRequestIdLayer` / `PropagateRequestIdLayer`，响应头返回 `x-request-id`，便于日志关联。
- **脱敏**：`AppConfig` 手写 `Debug`，连接 URL 密码替换为 `***`；`AppError` 只向客户端返回安全文案，SQL/连接串/堆栈只进服务端日志。

## 4. API 契约

### 成功响应

```json
{ "success": true, "data": { }, "message": "ok" }
```

### 失败响应

```json
{
  "success": false,
  "error": { "code": "validation_error", "details": null },
  "message": "`limit` must be between 1 and 100"
}
```

HTTP 状态码语义：

| 状态码 | `error.code` | 场景 |
| --- | --- | --- |
| 400 | `validation_error` | 参数校验失败 |
| 401 | `unauthorized` | 认证（第二阶段预留，当前无受保护路由） |
| 403 | `forbidden` | 授权（预留） |
| 404 | `not_found` | 路由或资源不存在 |
| 500 | `database_error` / `internal_error` | 服务端错误（不暴露细节） |
| 503 | `not_ready` / `dependency_unavailable` / `redis_error` | 依赖不可用 |

健康检查语义：

- `/health`：进程能处理 HTTP 即 200，不探测依赖。
- `/health/live`：同上，返回进程运行时长，供编排器 liveness 探针使用。
- `/health/ready`：并发探测 PostgreSQL 与 Redis；任一失败返回 **503**，`error.details.checks` 包含脱敏后的探测结果（`status`、`latency_ms`、`detail`）。
- `/api/v1/system/status`：即使降级也返回 **200**，`data.status` 为 `healthy` / `degraded`，供管理后台展示。

## 5. 数据库设计

### 迁移

- 位置：`apps/api/migrations/`，文件名 `<UTC 时间戳>_<描述>.sql`。
- 执行：启动时 `sqlx::migrate!("./migrations").run(&pool)`。
- 版本表：`_sqlx_migrations`，重复执行安全；修改已应用迁移会导致校验失败（应新增迁移）。
- `apps/api/build.rs` 让 Cargo 监听 `migrations/` 目录：新增迁移会触发重新编译（`sqlx::migrate!` 在编译期内嵌迁移列表，否则新文件不会被加载）。
- 迁移清单：`20261103000001`（系统表）、`0002`（KZ 引用数据）、`0003`（accounts）、`0004`（maps）、`0005`（servers）、`0006`（records）、`0007`（jumpstats）、`0008`（replays）、`0009`（moderation）。

### 表结构

`system_events`：系统级事件（部署、依赖故障、后续插件事件），仪表盘"最近系统事件"来源。

| 列 | 类型 | 说明 |
| --- | --- | --- |
| `id` | `uuid` PK，默认 `gen_random_uuid()` | 事件 ID |
| `event_type` | `text` | 点分事件名，如 `api.started` |
| `severity` | `text` | `info` / `warning` / `error`（CHECK 约束） |
| `message` | `text` | 可读描述 |
| `metadata` | `jsonb` | 结构化附加信息 |
| `created_at` | `timestamptz` | 默认 `now()` |

索引：`created_at DESC`、`event_type`。

`app_settings`：平台设置 key/value（`key` 主键、`value jsonb`、`description`、`updated_at`）。为"系统设置"页面预留，第一阶段**没有读写接口**。

### 业务数据模型（第二阶段）

第二阶段新增 20 张新表（17 张业务表 + 3 张引用表），覆盖 accounts / maps / records / servers / jumpstats / replays / moderation，并建立引用数据、索引、幂等键与世界纪录并发更新约束。完整设计见：

- [数据库总览](database/overview.md)（原则、表清单、迁移、待确认项）
- [Accounts](database/accounts.md) · [Maps](database/maps.md) · [Records](database/records.md) · [Servers](database/servers.md) · [Jumpstats](database/jumpstats.md) · [Replays](database/replays.md) · [Moderation](database/moderation.md)
- [ER 图](database/er-diagram.md)
- [GlobalAPI 兼容性分析](globalapi-compatibility.md)

关键约束：

- 业务主键统一 `BIGINT GENERATED ALWAYS AS IDENTITY`；SteamID64 使用 `NUMERIC(20,0)`（避免有符号 bigint 溢出）。
- 成绩时长以 `BIGINT` 纳秒存储，避免浮点排序问题；TP/PRO 由 `teleports` 通过 CHECK 强制推导。
- `records` 以 `(server_id, submission_id)`、`(source, external_ref)` 两个部分唯一索引保证重复提交幂等。
- 世界纪录 `course_world_records` 使用条件 upsert（`WHERE EXCLUDED.time_ns < current`），并发下慢成绩不会覆盖快成绩。

> 第二阶段只创建表、Rust model/DTO 与文档，不注册新的 HTTP 路由；`docs/openapi.json` 未变化。

## 6. Redis 设计

- 单一 `RedisClient` 在 `AppState` 中共享，内部为 `ConnectionManager` 多路复用连接。
- 键命名空间：`REDIS_KEY_PREFIX`（默认 `cngokz`），通过 `RedisClient::key("...")` 生成，例如 `cngokz:health`。
- 第一阶段只用于健康检查（`PING`）与后续缓存；不存储业务真值数据。
- 持久化由实际部署的 Redis 配置决定（建议 AOF `everysec` + 快照）；CNGOKZAPI 只使用缓存，不存业务真值。

## 7. OpenAPI 与类型同步

```text
Rust handler/dto
   │ #[utoipa::path] / #[derive(ToSchema)]
   ▼
cargo run --bin dump-openapi  ──▶ docs/openapi.json
   │
   ▼
pnpm --filter @cngokz/shared-types gen:api
   ▼
packages/shared-types/src/generated/api-schema.d.ts
   │
   ▼
@cngokz/api-client（openapi-fetch）──▶ apps/web
```

- `routes/openapi.rs` 显式登记所有 path 与 schema；新增接口必须同时登记，否则文档缺失（CI 会校验 `docs/openapi.json` 是否与代码一致）。
- 统一响应用泛型 `ApiResponse<T>`，`utoipa` 生成 `ApiResponse_HealthResponse` 等具体 schema。
- Swagger UI 资源和 OpenAPI 文档分别位于 `/swagger-ui`、`/api-docs/openapi.json`；`utoipa-swagger-ui` 启用 `vendored`，构建时无需联网下载前端资源。

## 8. 扩展指引（第三阶段起）

第二阶段已经完成 schema、model 与 DTO 契约，第三阶段以“读取接口优先”的方式接入。以玩家列表为例：

1. `repositories/player_repository.rs` — 基于已有表编写查询（分页、排序、过滤），返回 `models::account::*`。
2. `services/player_service.rs` — 组装 DTO（例如 `dto::account::AccountSummary`），处理不存在/分页参数。
3. `handlers/player.rs` — `router()` + `#[utoipa::path]`，响应使用 `ApiResponse<T>`。
4. `routes/mod.rs` — 挂载 `handlers::player::router()`；`routes/openapi.rs` — 登记 path/schema（`dto` 中的类型已实现 `ToSchema`）。
5. `scripts/gen-api.sh` — 重新生成 `docs/openapi.json` 与 TypeScript 类型。
6. `apps/web/src/{services,hooks,pages}` — 用真实接口替换占位页面。

新增表时：

1. `migrations/<ts>_<description>.sql`（不修改已应用的迁移）。
2. 在 `models/` 增加 `FromRow` 结构体，在 `dto/` 增加 `ToSchema` 契约。
3. 若涉及状态/枚举，使用 `TEXT + CHECK` + Rust `sqlx::Type`（rename_all）保持一致。
4. 更新 `docs/database/` 对应模块文档与 ER 图，并在 `docs/changelog.md` 追加记录。

认证与授权（服务器 API Key、管理员会话）在第三阶段实现：

- 服务器认证：`X-ApiKey` → SHA-256 → `servers.api_key_hash` 等值查找（见 [servers.md](database/servers.md)）。
- 管理员认证：预留在 `AppError::Unauthorized`/`Forbidden`，在 OpenAPI 中登记 security scheme。

## 9. 非目标（当前阶段）

- 不实现业务 CRUD、认证、插件协议、Replay 上传/下载、排行榜与积分计算。
- 不引入消息队列、服务网格、分布式缓存一致性、Nginx 强制依赖。
- 不做多租户与水平分片；暂不引入表分区，但 schema 已为按时间/维度查询预留索引。
- GlobalAPI 兼容接口尚未实现，仅完成字段级映射与差异分析。
- 第三阶段只做 UI/UX 与设计系统；第四阶段新增公开只读 API 与独立公开门户。
- 公开门户不包含登录、写操作与管理员能力；未定义积分算法前不展示 Points/Rating。
