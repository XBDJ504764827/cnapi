# 开发指南

## 1. 环境准备

| 工具 | 版本要求 | 说明 |
| --- | --- | --- |
| Node.js | ≥ 22（推荐 24） | 前端与脚本 |
| pnpm | 11.28.5（`packageManager` 固定） | 建议 `corepack enable` |
| Rust | 1.97.1（`rust-toolchain.toml` 固定） | 含 `rustfmt`、`clippy` |
| Docker + Compose | 最新稳定版 | 本地 PostgreSQL / Redis |
| sqlx-cli（可选） | 0.9 | 仅用于新增/回滚迁移 |

```bash
corepack enable
pnpm install
cp .env.example .env
```

> 本仓库不需要全局安装 `sqlx-cli` 即可运行服务：迁移在 API 启动时自动执行。

## 2. 启动本地依赖（PostgreSQL + Redis）

```bash
scripts/dev-up.sh
# 等价于
docker compose -f docker-compose.dev.yml up -d
```

默认端口：PostgreSQL `127.0.0.1:5432`、Redis `127.0.0.1:6379`，账号见 `.env`。

不使用 Docker 时，请自行准备 PostgreSQL 18 与 Redis 8，并在 `.env` 中调整 `DATABASE_URL` / `REDIS_URL`。

## 3. 启动后端

```bash
cargo run -p cngokz-api
```

- 读取根目录 `.env`（存在时），随后校验全部环境变量；缺失 `DATABASE_URL` / `REDIS_URL` 会直接报错退出。
- 启动时自动执行 `apps/api/migrations` 下的迁移。
- 监听 `APP_HOST:APP_PORT`，默认 <http://localhost:8080>。
- 日志级别由 `LOG_LEVEL` 控制（`info`、`debug`、`cngokz_api=debug,tower_http=debug`）。

常用端点：

```bash
curl -s localhost:8080/health | jq
curl -s localhost:8080/health/ready | jq
curl -s localhost:8080/api/v1/system/status | jq
open http://localhost:8080/swagger-ui
```

工程结构约定见 [architecture.md](architecture.md#3-后端分层)。

## 4. 启动前端

```bash
pnpm --filter @cngokz/web dev
```

- 默认 <http://localhost:5173>。
- 开发服务器把 `/api`、`/health`、`/swagger-ui`、`/api-docs` 代理到 `VITE_API_PROXY_TARGET`（默认 `http://localhost:8080`），因此前端代码使用相对路径即可。
- 如需直连其他 API 地址，在 `apps/web/.env.local` 中设置 `VITE_API_BASE_URL=http://host:port`（此时后端需在 `CORS_ALLOWED_ORIGINS` 中放行前端来源）。

前端环境变量见 `apps/web/.env.example`：

| 变量 | 默认值 | 说明 |
| --- | --- | --- |
| `VITE_API_BASE_URL` | 空 | 空 = 同源；末尾 `/` 会被移除 |
| `VITE_API_PROXY_TARGET` | `http://localhost:8080` | 仅开发/预览服务器使用 |
| `VITE_APP_NAME` | `CNGOKZAPI` | 侧边栏显示名 |
| `VITE_API_DOCS_PATH` | `/swagger-ui` | API 文档入口 |
| `WEB_PORT` | `5173` | 开发服务器端口 |

生产构建：

```bash
pnpm --filter @cngokz/web build     # 先 tsc --noEmit，再 vite build → apps/web/dist
pnpm --filter @cngokz/web preview   # 本地预览（同样带代理）
```

## 5. 数据库迁移

### 自动执行

API 启动时运行 `sqlx::migrate!("./migrations")`，已应用版本记录在 `_sqlx_migrations` 表中，重复启动安全。

### 新增迁移

```bash
cargo install sqlx-cli --no-default-features --features postgres,rustls   # 仅一次
cd apps/api
sqlx migrate add -r create_players      # 生成 <ts>_create_players.up.sql / .down.sql
# 编辑 SQL，注意：单条迁移内不要写 BEGIN/COMMIT（由 sqlx 管理事务）
cd ../..
cargo run -p cngokz-api                 # 启动即应用，或：
cd apps/api && sqlx migrate run         # 需要 DATABASE_URL
```

要求：

- 已合并的迁移不可修改，修正必须新增迁移。
- 迁移需可重复部署：使用 `CREATE TABLE`（而非依赖 `IF NOT EXISTS` 掩盖错误）、显式索引名、`timestamptz`。
- 回滚：`sqlx migrate revert`（仅开发环境使用）。

## 6. OpenAPI 与前端类型

后端 `handlers` / `dto` 变更后：

```bash
scripts/gen-api.sh
# 等价于：
cargo run --bin dump-openapi -- docs/openapi.json
pnpm --filter @cngokz/shared-types gen:api
pnpm -r typecheck
```

- `docs/openapi.json` 与 `packages/shared-types/src/generated/api-schema.d.ts` 均需提交，CI 会校验 OpenAPI 是否最新。
- 不要手改生成文件；需要新增字段时修改 Rust DTO。
- 新增 handler 时必须在 `apps/api/src/routes/openapi.rs` 的 `paths(...)` 与 `components(schemas(...))` 中登记。

## 7. 测试

### 后端

```bash
cargo test --all-targets
```

- `tests/http_api.rs`：不依赖外部服务（依赖指向关闭的端口），验证响应结构、状态码、CORS、OpenAPI、Swagger UI、404。
- `tests/service_integration.rs`：需要真实服务；未设置 `DATABASE_URL` 时**跳过并打印原因**：

```bash
DATABASE_URL=postgres://cngokz:cngokz@localhost:5432/cngokzapi \
REDIS_URL=redis://localhost:6379 \
cargo test --all-targets
```

其中 `migrations_apply_to_a_fresh_database` 会临时创建并删除一个数据库，验证从零迁移（需要 `CREATEDB` 权限）。

### 前端

```bash
pnpm --filter @cngokz/web test        # vitest（jsdom）
```

覆盖：格式化工具、状态映射、状态徽章渲染、仪表盘数据/错误/占位渲染。

### 全部检查

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
pnpm -r lint
pnpm -r typecheck
pnpm -r test
pnpm -r format:check
```

## 8. 代码风格

- Rust：`cargo fmt`、`cargo clippy -D warnings`；错误使用 `thiserror`，禁止 `unwrap()` 出现在请求路径（测试与启动阶段除外）。
- TypeScript：strict + `noUncheckedIndexedAccess` + `verbatimModuleSyntax`；类型导入使用 `import type`。
- ESLint flat config（`apps/web/eslint.config.js`）；Prettier 配置在 `apps/web/.prettierrc.json`（含 Tailwind class 排序）。
- 统一使用 `@/` 路径别名访问 `apps/web/src`。
- 页面不放请求逻辑：请求写在 `services/`，用 `hooks/` 暴露给组件。

## 9. 新增一个页面（示例）

1. `apps/web/src/pages/xyz-page.tsx`：默认导出命名组件 `XyzPage`，使用 `PageHeader` + `EmptyState`（未接入接口前不要放模拟数据）。
2. `apps/web/src/components/layout/nav-config.ts`：在对应分组添加 `{ label, to, icon }`。
3. `apps/web/src/router/index.tsx`：添加 `lazy` 导入与路由项。
4. 若需要接口：`services/xyz-service.ts`（`queryOptions`）→ `hooks/use-xyz.ts` → 页面组件。
5. 若后端无接口：先按 [architecture.md](architecture.md#8-扩展指引第二阶段起) 完成后端分层，再 `scripts/gen-api.sh`。

## 10. 常见问题

| 现象 | 排查 |
| --- | --- |
| API 启动报 `missing required environment variable DATABASE_URL` | 未复制 `.env`，或未在容器中注入变量 |
| `failed to connect to PostgreSQL` | 依赖未启动 / 端口冲突 / 密码不一致（`docker compose -f docker-compose.dev.yml ps`） |
| `/health/ready` 返回 503 且 redis `down` | Redis 未启动或 `REDIS_URL` 错误；恢复后下一次请求自动重连 |
| 前端请求 404/502 | 开发代理目标不对，检查 `VITE_API_PROXY_TARGET`；后端是否监听 8080 |
| 前端类型报 `ApiResponse_*` 不存在 | 未执行 `scripts/gen-api.sh`，或 `docs/openapi.json` 过期 |
| Swagger UI 打开是空白/跳转 | `/swagger-ui` 会重定向到 `/swagger-ui/`；请访问后一个地址 |
| 端口被占用 | 修改 `.env` 中 `APP_PORT` / `WEB_PORT` / `POSTGRES_PORT` / `REDIS_PORT` |
