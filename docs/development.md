# 开发指南

## 1. 环境准备

| 工具 | 版本要求 | 说明 |
| --- | --- | --- |
| Node.js | ≥ 22（推荐 24） | 前端与脚本 |
| pnpm | 11.28.5（`packageManager` 固定） | 建议 `corepack enable` |
| Rust | 1.97.1（`rust-toolchain.toml` 固定） | 含 `rustfmt`、`clippy` |
| PostgreSQL | 18（或兼容版本） | 本地数据库 |
| Redis | 8（或兼容版本） | 本地缓存 |
| sqlx-cli（可选） | 0.9 | 仅用于新增/回滚迁移 |

```bash
corepack enable
pnpm install
cp .env.example .env
```

> 本仓库不需要全局安装 `sqlx-cli` 即可运行服务：迁移在 API 启动时自动执行。

## 2. 准备本地依赖（PostgreSQL + Redis）

使用本机已安装的 PostgreSQL 18 与 Redis 8，或连接你现有的实例。

`.env` 示例：

```env
DATABASE_URL=postgres://cngokz:cngokz@127.0.0.1:5432/cngokzapi
REDIS_URL=redis://127.0.0.1:6379/3
```

创建数据库和用户：

```bash
sudo -u postgres psql
```

```sql
CREATE ROLE cngokz LOGIN PASSWORD 'cngokz';
CREATE DATABASE cngokzapi OWNER cngokz;
\c cngokzapi
CREATE EXTENSION IF NOT EXISTS pg_trgm;
```

Redis 使用独立 DB index 或配置密码即可，不需要改动代码。

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
pnpm --filter @cngokz/frontend dev
```

- 公开门户（默认站点）：<http://localhost:5173>，代理 `/api`、`/health`、`/swagger-ui`、`/api-docs`，并把 `/admin` 代理到管理后台开发服务器。
- 管理后台：<http://localhost:5173/admin>（内部开发端口 5174，base `/admin/`）。
- 一键同时启动：`pnpm dev`（并行启动 web + public）；单独启动：`pnpm dev:admin` / `pnpm dev:public`。
- 开发服务器把 `/api`、`/health`、`/swagger-ui`、`/api-docs` 代理到 `VITE_API_PROXY_TARGET`（默认 `http://localhost:8080`），因此前端代码使用相对路径即可。
- 如需直连其他 API 地址，在 `apps/frontend/.env.local` 中设置 `VITE_API_BASE_URL=http://host:port`（此时后端需在 `CORS_ALLOWED_ORIGINS` 中放行前端来源）。

前端环境变量见 `apps/frontend/.env.example`：

| 变量 | 默认值 | 说明 |
| --- | --- | --- |
| `VITE_API_BASE_URL` | 空 | 空 = 同源；末尾 `/` 会被移除 |
| `VITE_API_PROXY_TARGET` | `http://localhost:8080` | 仅开发/预览服务器使用 |
| `VITE_APP_NAME` | `CNGOKZAPI` | 侧边栏显示名 |
| `VITE_API_DOCS_PATH` | `/swagger-ui` | API 文档入口 |
| `ADMIN_PORT` | `5174` | 管理后台开发服务器内部端口（base `/admin/`） |

生产构建：

```bash
pnpm --filter @cngokz/frontend build     # 先 tsc --noEmit，再 vite build → apps/frontend/dist
pnpm --filter @cngokz/frontend preview   # 本地预览（同样带代理）
```

## 5. 数据库迁移

### 自动执行

API 启动时运行 `sqlx::migrate!("./migrations")`，已应用版本记录在 `_sqlx_migrations` 表中，重复启动安全。

`apps/api/build.rs` 注册了 `cargo:rerun-if-changed=migrations`：新增/修改迁移文件会触发 crate 重新编译。没有它时 Cargo 不会感知新文件，`sqlx::migrate!` 会继续使用旧的编译期内嵌列表。

### 新增迁移

```bash
cargo install sqlx-cli --no-default-features --features postgres,rustls   # 仅一次
cd apps/api
sqlx migrate add <description>           # 生成 <ts>_<description>.sql（本项目使用 up-only 迁移）
# 编辑 SQL，注意：单条迁移内不要写 BEGIN/COMMIT（由 sqlx 管理事务）
cd ../..
cargo run -p cngokz-api                 # 启动即应用，或：
cd apps/api && sqlx migrate run         # 需要 DATABASE_URL
```

要求：

- 已合并的迁移不可修改，修正必须新增迁移。
- 迁移需可重复部署：使用 `CREATE TABLE`（而非依赖 `IF NOT EXISTS` 掩盖错误）、显式索引名、`timestamptz`。
- 第二阶段的迁移清单与数据模型说明见 [docs/database/overview.md](database/overview.md)。
- 回滚：本项目未提供 down 迁移，生产回滚采用“新迁移 + 备份恢复”；开发环境可重建数据库。

### 在测试中重建 schema

```bash
DATABASE_URL=postgres://cngokz:cngokz@localhost:5432/cngokzapi \
cargo test --test schema_integration
```

`tests/schema_integration.rs` 会为每个用例临时创建数据库并执行全部迁移，结束后（包括失败用例）删除临时库，因此不要求 `DATABASE_URL` 指向已迁移的数据库；数据库用户需要 `CREATEDB` 权限。

缺少 `DATABASE_URL` 时 schema 集成用例会跳过并打印原因。

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
- `tests/schema_integration.rs`：第二阶段 schema 测试（表/索引/约束/枚举/触发器/SteamID64/WR 并发/软删除唯一性），每个用例临时建库并执行迁移、结束后删库；事务回滚，可重复执行，不依赖预先迁移的数据库。
- `tests/source_ingestion.rs`：第八阶段多来源原始成绩、跨来源去重、来源溯源、数据源健康与外部服务器快照测试。
- `tests/source_live.rs`：真实 kzgo.eu 同步测试，默认忽略，需显式 `--ignored`。
- `tests/game_server_sync.rs`：第七阶段 CS:GO 接入端到端测试（服务器 Token、心跳、玩家多服务器隔离、成绩幂等/PB/WR、Replay、配置版本、禁用服务器），临时建库并执行全部迁移。
- `tests/service_integration.rs`：需要真实服务；未设置 `DATABASE_URL` 时**跳过并打印原因**：

```bash
DATABASE_URL=postgres://cngokz:cngokz@localhost:5432/cngokzapi \
REDIS_URL=redis://localhost:6379 \
cargo test --all-targets
```

其中 `migrations_apply_to_a_fresh_database` 会临时创建并删除一个数据库，验证从零迁移（需要 `CREATEDB` 权限）。

### 前端

```bash
pnpm --filter @cngokz/frontend test        # 管理后台 vitest（jsdom）
pnpm --filter @cngokz/frontend test     # 公开门户 vitest（jsdom）
```

覆盖：格式化工具、状态映射、状态徽章渲染、仪表盘数据/错误/占位渲染、分页交互、导航搜索、主题默认值；公开门户的秒数/百分比格式化与难度标识。

前端测试规模（第四阶段结束）：管理后台 6 文件 / 21 用例；公开门户 2 文件 / 5 用例。
> 管理后台现在以 `/admin` 为 basename，独立开发时地址是 `http://localhost:5174/admin/`。

后端测试规模（第八阶段）：79 个测试（46 单元 + 3 游戏服务器接入 + 2 数据源接入 + 10 HTTP 契约 + 1 积分 + 3 公开 API + 11 schema + 3 真实依赖）；另有 1 个需要真实 kzgo.eu 网络的 `source_live` 忽略测试。

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
- ESLint flat config（`apps/frontend/eslint.config.js`）；Prettier 配置在 `apps/frontend/.prettierrc.json`（含 Tailwind class 排序）。
- 统一使用 `@/` 路径别名访问 `apps/frontend/src`。
- 页面不放请求逻辑：请求写在 `services/`，用 `hooks/` 暴露给组件。
- UI 必须复用设计系统：颜色/圆角/字体使用 token，基础组件来自 `components/ui`，列表页使用 `ListPanel` + `FilterBar` + `DataTable` + `Pagination`。规范见 [design-system.md](design-system.md)。
- 没有后端接口的页面使用 `Callout` + `EmptyState` 明确说明，不写模拟数据或假成功操作。
- 公开门户：筛选/排序/分页状态写入 URL（`useListParams`）；新增公开查询时在 `services/public-service.ts` 定义 `queryOptions`，并扩展 `api-client`。
- 公开数据的统计口径（完成地图/完成率/TP-PRO/积分缺口）见 [public-portal/api-requirements.md](public-portal/api-requirements.md)。

## 9. 新增一个页面（示例）

1. `apps/frontend/src/pages/xyz-page.tsx`：命名导出 `XyzPage`，使用 `PageHeader`；列表页复用 `ListPanel` / `FilterBar` / `DataTable` / `Pagination`。
2. `apps/frontend/src/components/layout/nav-config.ts`：在对应分组添加 `{ label, to, icon, description, keywords }`（同时驱动侧边栏、面包屑与 ⌘K 搜索）。
3. `apps/frontend/src/router/index.tsx`：添加 `lazy` 导入与路由项；旧路径如需兼容用 `<Navigate replace />`。
4. 若需要接口：`services/xyz-service.ts`（`queryOptions`）→ `hooks/use-xyz.ts` → 页面组件。
5. 若后端无接口：先按 [architecture.md](architecture.md#8-扩展指引第二阶段起) 完成后端分层，再 `scripts/gen-api.sh`。

## 10. 常见问题

| 现象 | 排查 |
| --- | --- |
| API 启动报 `missing required environment variable DATABASE_URL` | 未复制 `.env`，或运行目录不是项目根目录 |
| `failed to connect to PostgreSQL` | PostgreSQL 未启动、端口冲突、数据库/用户/密码与 `DATABASE_URL` 不一致 |
| `/health/ready` 返回 503 且 redis `down` | Redis 未启动或 `REDIS_URL` 错误；恢复后下一次请求自动重连 |
| 前端请求 404/502 | 开发代理目标不对，检查 `VITE_API_PROXY_TARGET`；后端是否监听 8080 |
| 前端类型报 `ApiResponse_*` 不存在 | 未执行 `scripts/gen-api.sh`，或 `docs/openapi.json` 过期 |
| Swagger UI 打开是空白/跳转 | `/swagger-ui` 会重定向到 `/swagger-ui/`；请访问后一个地址 |
| 端口被占用 | 修改 `.env` 中 `APP_PORT` / `PORTAL_PORT` / `ADMIN_PORT`，或停止占用端口的进程 |
