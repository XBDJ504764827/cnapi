# 开发记录（Development Log）

本文件记录 CNGOKZAPI 已经发生的开发事实，供维护者追溯修改内容、修改原因、影响范围和验证结果。

记录规则：

1. 每完成一组边界明确的修改就追加记录，不等到阶段结束后统一补写。
2. 新条目插入到历史条目之前，**按完成时间倒序**排列。
3. 标题统一使用北京时间（UTC+8）：`## YYYY年MM月DD日 HH时mm分ss秒 主题`。
4. 时间取自可信系统时间（`date` 输出或文件系统修改时间）并换算为 UTC+8，不编造；无法确定精确秒数时先检查时间来源再记录。
5. 每条必须包含：背景、变更内容、涉及文件、数据库与 API 变化、测试与结果。
6. 文件列表以真实 Git 状态、diff 或实际文件操作为准；没有代码变更时如实说明。
7. 未执行的测试写“未执行”，不得把未验证内容写成通过。
8. 发现旧记录事实错误时，新增纠正记录并注明原记录位置，保留历史。
9. 文档修改若影响项目行为或开发规范，同样记录。

项目起点：`/home/xbdj/cnapi` 为空目录（除 `.`/`..` 外无文件），无既有 Git 历史。

---

## 2026年11月03日 19时43分15秒 第一阶段验收：构建、测试与运行时验证

**背景**

第一阶段功能与文档完成后，按验收标准逐项执行构建、测试与运行时验证，并如实记录通过情况与未验证项。

**变更内容**

- 无功能代码变更。本组只执行验证并追加本记录；同时补充 `infrastructure/redis/redis.conf` 的 `bind 0.0.0.0`（容器内需显式 bind 才能绕过 protected-mode 限制，否则 API 无法连接 Redis）与 Web 构建镜像的 `COREPACK_ENABLE_DOWNLOAD_PROMPT=0`，两处均为本次验证期间发现的问题修正。
- 新增空目录占位文件 `apps/web/src/assets/.gitkeep`，保证前端目录结构完整。

**涉及文件**

- 修改：`infrastructure/redis/redis.conf`
- 修改：`infrastructure/docker/web.Dockerfile`
- 新增：`apps/web/src/assets/.gitkeep`
- 修改：`docs/changelog.md`（本记录）

**数据库与 API 变化**

无。数据库结构、迁移版本与 API 契约均未改变。

**测试与结果**

| # | 验收项 | 结果 | 证据/说明 |
| --- | --- | --- | --- |
| 1 | `pnpm install` 成功 | 通过 | `pnpm install --frozen-lockfile` → “Already up to date”，`pnpm peers check` 无问题 |
| 2 | 前端可启动与构建 | 通过 | `vite` 开发服务器 5174 端口启动，`/`、`/health`、`/api/v1/system/status` 均 200；`tsc --noEmit && vite build` 通过 |
| 3 | Rust 后端可编译 | 通过 | `cargo build` 与 `cargo build --release --bin cngokz-api` 均通过（release 产物 13 MB） |
| 4 | PostgreSQL 可连接 | 通过 | 本地 PostgreSQL 18.6 实例，API 启动日志 `postgresql connection pool ready`，`/health/ready` 中 `postgresql=up` |
| 5 | Redis 可连接 | 通过 | 本地 Redis 8.0.5，`/health/ready` 中 `redis=up`，`redis connection ready` |
| 6 | 迁移可执行 | 通过 | 集成测试 `migrations_apply_to_a_fresh_database` 从零建库执行迁移并校验表；`_sqlx_migrations` 记录存在 |
| 7 | `GET /health` 正常 | 通过 | 200，`success=true`，`data.status=ok`，返回 `x-request-id` |
| 8 | `GET /health/ready` 反映真实状态 | 通过 | 依赖正常时 200；停止 Redis 后 503（`redis=down`，`detail=connection_failed`）；重启后恢复 200 |
| 9 | Swagger UI 可访问 | 通过 | `/swagger-ui/` 200 HTML（含 `swagger-ui.css`/`swagger-ui-bundle.js`），`/api-docs/openapi.json` 200（6 个 path） |
| 10 | 前端可访问后端 API | 通过 | Vite 代理（5174）与生产静态服务器代理（5173）下 `/health`、`/api/v1/system/*`、`/swagger-ui/` 均返回 200 |
| 11 | 仪表盘展示真实系统状态 | 部分验证 | 后端返回真实数据；Vitest 仪表盘用例验证状态/延迟/系统信息/空事件/错误态渲染。**未执行**浏览器人工视觉验收（无图形环境） |
| 12 | Docker Compose 一键启动 | **未验证** | 环境无 `docker` 命令，仅完成 YAML 解析；`docker compose up -d --build`、镜像构建与容器健康检查均未执行 |
| 13 | 重启后 PostgreSQL 数据不丢失 | 部分验证 | 本地 PostgreSQL 数据目录插入标记行 → 重启实例 → 数据仍在（容器命名卷持久化**未验证**） |
| 14 | 基础测试全部通过 | 通过 | Rust 34 个（21 单元 + 10 HTTP + 3 集成）；前端 13 个（3 文件）；`cargo clippy -D warnings`、`cargo fmt --check`、`pnpm -r lint/typecheck/format:check` 全部通过 |

补充验证：

- OpenAPI 与代码同步：重新生成后 `diff` 无差异。
- 生产模式（`APP_ENV=production`）release 二进制启动成功，日志为 JSON 格式，`/health` 200。
- PostgreSQL 重启后连接池自愈，`/health/ready` 仍 200。
- **未执行**：GitHub Actions CI（无 runner）；`docker compose config`；容器网络与卷行为；浏览器交互与主题视觉检查。

**结论**：第一阶段的代码、构建、测试、运行时接口与前后端联调均验证通过；**Docker 相关验收（第 12 项、第 13 项容器部分）因当前环境缺少 Docker 尚未验证**，需在具备 Docker 的机器上执行 `docker compose up -d --build` 并复查健康检查与卷持久化。

---

## 2026年11月03日 19时40分54秒 第一阶段文档：README、架构、开发、部署与开发记录

**背景**

第一阶段代码与基础设施已落地，需要可追溯的文档：新维护者能在不了解上下文的情况下启动、开发与部署；同时本文件用于记录开发事实。

**变更内容**

- 新增根 `README.md`：项目介绍、技术栈与依赖版本、目录结构、Docker 与本地开发入口、环境变量表、常用命令、API 一览、文档索引、阶段状态与未实现清单。
- 新增 `docs/architecture.md`：整体架构图、前后端关系（同源代理 + OpenAPI 类型链路）、后端分层职责表、启动流程、API 契约与状态码语义、数据库/Redis 设计、OpenAPI 生成流程、第二阶段扩展步骤、非目标。
- 新增 `docs/development.md`：环境准备、本地依赖、前后端启动、迁移新增与执行、OpenAPI 类型生成、测试（含“无 DATABASE_URL 时跳过”的说明）、代码风格、新增页面检查清单、常见问题。
- 新增 `docs/deployment.md`：Compose 首次部署、端口与网络、环境变量、持久化与备份/恢复、更新与回滚、可选反向代理、安全基线、故障排查。
- 新增本文件 `docs/changelog.md`，并写入从项目初始化到文档阶段的历史条目。

**涉及文件**

- 新增：`README.md`
- 新增：`docs/architecture.md`
- 新增：`docs/development.md`
- 新增：`docs/deployment.md`
- 新增：`docs/changelog.md`

**数据库与 API 变化**

无。本组修改仅涉及文档，不改变服务行为、数据库结构或 API 契约。

**测试与结果**

未执行代码测试（仅文档变更，未修改代码）。本组修改前已完成的前端与后端测试结果见下方更早条目。

---

## 2026年11月03日 19时37分46秒 前端管理后台基础界面与后端 API 对接

**背景**

第一阶段需要可运行的管理后台外壳：固定侧边栏、顶部导航、响应式布局、深浅主题、侧边栏折叠，仪表盘展示后端真实健康状态；业务模块只需要页面结构与空状态，不允许伪造数据或按钮。

**变更内容**

- 工程配置：Vite 8 + React 19 + TypeScript strict + Tailwind v4（`@tailwindcss/vite`）+ shadcn/ui 目录规范（`components.json`、`src/components/ui/*` 基于 `radix-ui`）+ ESLint flat config + Prettier（含 Tailwind class 排序）+ Vitest/jsdom。
- 应用外壳：`layouts/app-layout.tsx`（侧边栏 + 顶栏 + 主内容 + 页脚 + 跳转主内容链接 + ScrollRestoration），侧边栏折叠状态持久化；移动端使用 Radix Dialog 抽屉（`components/ui/sheet.tsx`）。
- 主题：`stores/ui-store.ts`（zustand persist，key `cngokz-ui`）+ `components/theme/theme-provider.tsx` + `hooks/use-theme.ts`；`index.html` 首屏前应用主题，避免闪烁；顶栏下拉菜单切换浅色/深色/跟随系统。
- 导航：`components/layout/nav-config.ts` 定义 6 个分组、14 个菜单项与路径，`findNavItem` 供顶栏标题使用。
- 仪表盘：`pages/dashboard-page.tsx` 使用 `useSystemStatus`（15s 轮询）、`useSystemInfo`（60s）、`useSystemEvents`；展示 API/PostgreSQL/Redis 真实状态与延迟、系统版本/环境/启动时间/运行时长、最近系统事件（空表显示空状态）、4 个统计卡片占位（值为 `—`，明确标注“统计接口将在后续阶段接入”），依赖降级时显示提示条。
- 其他页面：玩家、地图、成绩、排行榜、服务器列表、服务器状态、Jumpstats、Replay、封禁记录、API 密钥（预留）、系统设置均复用 `ModulePlaceholder`（标题/描述/规划能力/空状态），无假按钮；API 文档页展示真实 Swagger UI 与 OpenAPI JSON 入口，并列出已实现接口；系统状态页展示真实依赖探测与健康检查端点链接；新增 404 与路由错误页。
- 请求封装：`lib/env.ts`（Zod 校验环境变量）、`lib/api.ts`（单一 API 客户端实例）、`services/system-service.ts`（TanStack Query `queryOptions`）、`hooks/use-system-*`；页面不直接写请求。
- 生产静态服务：`apps/web/server/static-server.mjs`（零依赖 Node HTTP 服务）提供 `dist` 静态资源、SPA 回退、`/healthz`，并反向代理 `/api`、`/health`、`/swagger-ui`、`/api-docs` 到 `API_UPSTREAM`，使浏览器始终同源。
- 测试：`lib/format.test.ts`、`components/common/status-badge.test.tsx`、`pages/dashboard-page.test.tsx`（mock API，验证真实状态渲染、占位不造数、错误态）。

**涉及文件**

- 新增：`apps/web/package.json`、`apps/web/tsconfig.json`、`apps/web/vite.config.ts`、`apps/web/vitest.config.ts`、`apps/web/index.html`、`apps/web/components.json`、`apps/web/eslint.config.js`、`apps/web/.prettierrc.json`、`apps/web/.prettierignore`、`apps/web/.env.example`
- 新增：`apps/web/src/{main.tsx,vite-env.d.ts}`、`src/styles/index.css`、`src/assets/`
- 新增：`src/lib/{api.ts,env.ts,format.ts,format.test.ts,query-client.ts,status.ts,utils.ts}`
- 新增：`src/stores/ui-store.ts`、`src/types/index.ts`
- 新增：`src/components/theme/theme-provider.tsx`
- 新增：`src/components/layout/nav-config.ts`、`src/components/layout/sidebar.tsx`、`src/components/layout/topbar.tsx`
- 新增：`src/components/ui/{badge,button,card,dropdown-menu,input,label,separator,sheet,skeleton,table,tooltip}.tsx`
- 新增：`src/components/common/{empty-state,error-state,module-placeholder,page-header,page-loading,status-badge,status-badge.test}.tsx`
- 新增：`src/components/dashboard/{dependency-status-card,recent-events-card,stat-card,system-info-card}.tsx`
- 新增：`src/layouts/app-layout.tsx`、`src/router/index.tsx`
- 新增：`src/pages/` 下 16 个页面文件（含 `not-found-page.tsx`、`route-error-page.tsx`、`dashboard-page.test.tsx`）
- 新增：`src/hooks/{use-system-info,use-system-status,use-system-events,use-theme}.ts`、`src/services/system-service.ts`、`src/test/setup.ts`、`apps/web/server/static-server.mjs`
- 新增：`src/assets/.gitkeep`（占位，保持目录结构）

**数据库与 API 变化**

无后端改动。前端消费已有接口：`GET /health`、`GET /health/ready`、`GET /api/v1/system/info`、`GET /api/v1/system/status`、`GET /api/v1/system/events`、Swagger UI 与 OpenAPI JSON。

**测试与结果**

- `pnpm --filter @cngokz/web typecheck`：通过。
- `pnpm --filter @cngokz/web lint`（`--max-warnings 0`）：通过。
- `pnpm --filter @cngokz/web test`：3 个测试文件 / 13 个用例全部通过（`vitest run`）。
- `pnpm --filter @cngokz/web build`：通过（`tsc --noEmit` + `vite build`），产出 `dist/`，存在 >500 kB 主 chunk 的构建警告（未处理）。
- `pnpm -r format:check`：通过。
- 集成验证（curl）：Vite 开发服务器（5174 端口，代理指向 127.0.0.1:8080）返回 `GET /health` 与 `GET /api/v1/system/info` 正常；`node server/static-server.mjs`（5173）返回首页 200、`/dashboard` SPA 回退 200、`/health` 代理正常、`/api/v1/system/status` 200、`/swagger-ui/` 200、`/healthz` 正常。
- **未执行**：图形浏览器中的视觉/交互验收（当前环境无浏览器），因此布局、主题切换、侧边栏折叠的视觉效果未经人工确认。

---

## 2026年11月03日 19时37分01秒 Docker Compose 部署架构

**背景**

需要一条命令启动 PostgreSQL、Redis、Rust API 与 React Web，并配置健康检查、启动依赖、持久化、内部网络与多阶段构建；生产/开发环境配置需分离；数据库不得无条件暴露公网。

**变更内容**

- `docker-compose.yml`（完整栈）：
  - `postgres:18-alpine`，命名卷 `cngokzapi-postgres-data` 挂载 `/var/lib/postgresql`（PG18 的 PGDATA 在其子目录），端口仅绑定 `127.0.0.1`，`pg_isready` 健康检查。
  - `redis:8-alpine`，挂载 `redis.conf` 与 `cngokzapi-redis-data`，`redis-cli ping` 健康检查，端口仅绑定 `127.0.0.1`。
  - `api`：多阶段构建，`depends_on` 两个依赖的 `service_healthy`，容器内使用 `postgres:5432` / `redis:6379`，`/health/ready` 作为健康检查，端口绑定 `127.0.0.1`。
  - `web`：多阶段构建，`API_UPSTREAM=http://api:8080`，依赖 api 健康，`/healthz` 健康检查，对外发布 `WEB_PORT`。
  - 统一 `json-file` 日志轮转（10m × 3）、`restart: unless-stopped`、独立网络 `cngokzapi-backend`。
- `docker-compose.dev.yml`：只启动 PostgreSQL + Redis（本地开发依赖），便于原生运行 API/Web。
- `infrastructure/docker/api.Dockerfile`：`rust:1.97-slim-bookworm` 构建（先占位 target 预热依赖缓存，再复制源码 `cargo build --release --locked`），`debian:bookworm-slim` 运行，非 root 用户 uid 10001，仅安装 `ca-certificates`/`curl`。
- `infrastructure/docker/web.Dockerfile`：`node:24-alpine` 构建（corepack + `pnpm install --frozen-lockfile` + `pnpm --filter @cngokz/web build`），运行阶段仅复制 `dist` 与静态服务器脚本，`USER node`。
- `infrastructure/postgres/init/00-init.sql`：容器首次初始化时设置数据库时区为 UTC（不建表，schema 由迁移管理）。
- `infrastructure/redis/redis.conf`：AOF `everysec`、快照、`maxmemory 256mb` + `allkeys-lru`。
- `infrastructure/postgres/README.md`、`infrastructure/redis/README.md`：迁移归属、持久化与配置说明。
- `.dockerignore`：排除 `target/`、`node_modules/`、`dist/`、`.env`、测试与文档等。
- `scripts/dev-up.sh`、`scripts/wait-for-api.sh`：本地依赖启动与就绪等待脚本（可执行权限）。
- `Cargo.toml`：`rust-version` 调整到 `1.94`（SQLx 0.9 的 MSRV）。

**涉及文件**

- 新增：`docker-compose.yml`、`docker-compose.dev.yml`、`.dockerignore`
- 新增：`infrastructure/docker/api.Dockerfile`、`infrastructure/docker/web.Dockerfile`
- 新增：`infrastructure/postgres/init/00-init.sql`、`infrastructure/postgres/README.md`
- 新增：`infrastructure/redis/redis.conf`、`infrastructure/redis/README.md`
- 新增：`scripts/dev-up.sh`（可执行）、`scripts/wait-for-api.sh`（可执行）
- 修改：`Cargo.toml`

**数据库与 API 变化**

- 数据库结构无变化（仍由 `apps/api/migrations` 管理）。
- 容器网络内 API 使用 `postgres:5432`、`redis:6379`；宿主机默认仅通过回环地址访问 `8080/5432/6379`，Web 通过 `5173` 对外。
- 健康检查依赖：web → api → (postgres, redis)。

**测试与结果**

- YAML 语法解析通过（Python `yaml.safe_load`，两个 compose 文件；CI 工作流 YAML 同样解析通过）。
- **未执行**：`docker compose config`、镜像构建、`docker compose up -d --build` 一键启动、容器健康检查与卷持久化验证。原因：当前环境没有 `docker` 命令（`docker: command not found`），无法在本机执行容器验证。
- 替代验证（非容器）：使用本地解包的 PostgreSQL 18.6 与 Redis 8.0.5 进程验证了 API 连通、迁移执行、健康检查与依赖中断恢复；容器相关行为仍待有 Docker 的环境复验。

---

## 2026年11月03日 19时34分00秒 后端测试套件：HTTP 契约与真实依赖集成

**背景**

第一阶段验收要求“基础测试全部通过”，需要覆盖统一响应结构、状态码、依赖不可用时的就绪行为、迁移可执行性与事件查询，而不是只验证能编译。

**变更内容**

- 新增 `apps/api/tests/http_api.rs`（10 个用例，不依赖外部服务）：依赖指向 `127.0.0.1:1`，验证：
  - `/health` 统一成功结构、版本、`x-request-id` 响应头；
  - `/health/live` 运行时长；
  - `/health/ready` 依赖不可达时返回 503、`error.code = not_ready`、两项 `down`，且响应不泄漏地址/SQL；
  - `/api/v1/system/info`、`/api/v1/system/status`（degraded、200）；
  - `?limit=0` 在查询数据库前返回 400 `validation_error`；
  - 未知路由 404 统一错误结构；
  - `/api-docs/openapi.json` 可访问且包含关键 path；
  - `/swagger-ui` 重定向与 `/swagger-ui/` HTML；
  - 允许来源的 CORS 预检。
- 新增 `apps/api/tests/service_integration.rs`（3 个用例，需真实依赖；缺失环境变量时打印原因并跳过）：
  - 临时创建数据库 → `sqlx::migrate!` 从零执行 → 校验 `_sqlx_migrations`、`system_events`、`app_settings` 存在 → 删除临时库；
  - 真实 PostgreSQL + Redis 下 `/health/ready` 200、`/api/v1/system/status` `healthy`；
  - `system_events` 空表返回 `count=0`，插入事件后接口返回该事件，测试后清理。
- 修正测试细节：SQLx 0.9 要求动态 SQL 使用 `AssertSqlSafe`（仅测试内的 `CREATE/DROP DATABASE`）；Swagger UI 断言改为先验证重定向再验证 HTML。

**涉及文件**

- 新增：`apps/api/tests/http_api.rs`
- 新增：`apps/api/tests/service_integration.rs`

**数据库与 API 变化**

无生产代码/接口变更；测试会读取 `system_events`、`_sqlx_migrations`，并在集成用例中临时创建/删除测试数据库与临时事件行。

**测试与结果**

- `DATABASE_URL=postgres://cnapi@127.0.0.1:55432/cngokzapi_test REDIS_URL=redis://127.0.0.1:56379 cargo test --all-targets`：
  - lib 单元测试 21 个通过；
  - `http_api` 10 个通过；
  - `service_integration` 3 个通过（全部真实执行，未跳过）；
  - 合计 34 个测试通过，0 失败。
- `cargo clippy --all-targets --all-features -- -D warnings`：通过。
- `cargo fmt --all --check`：通过。

---

## 2026年11月03日 19时28分00秒 共享类型包与类型化 API 客户端（OpenAPI 驱动）

**背景**

要求“使用 OpenAPI 生成的类型或客户端，避免手动维护重复 API 类型”。需要把后端 OpenAPI 文档转成 TypeScript 类型，并让前端只通过一个类型化客户端访问 API。

**变更内容**

- `packages/shared-types`：`pnpm gen:api` 使用 `openapi-typescript` 从 `docs/openapi.json` 生成 `src/generated/api-schema.d.ts`；`src/api-contract.ts` 在生成类型之上定义统一响应包络（`ApiEnvelope`/`ApiErrorEnvelope`）与业务别名（`SystemStatusPayload`、`DependencyStatusPayload`、`ErrorCode` 等），避免手写重复类型。
- `packages/api-client`：基于 `openapi-fetch` 的 `createApiClient({ baseUrl, fetch? })`，暴露 `health.*`、`system.*` 方法；`request`/`unwrapResponse` 统一解包 `{ success, data }` 或将 `{ success:false, error }` 转为 `ApiClientError`（含 `code`、`status`、`details`），网络错误映射为 `network_error`。
- 两个包均以 TypeScript 源码形式被工作区消费（`exports` 指向 `src/index.ts`），`vite.config.ts`/`tsconfig.json` 配置别名。
- 工具链修正：TypeScript 固定 `5.9.3`（`typescript-eslint` 与 `openapi-typescript` 的 peer 兼容范围），`@eslint/js` 固定 `10.0.1`（与 ESLint 10 对应）。

**涉及文件**

- 新增：`packages/shared-types/{package.json,tsconfig.json,.prettierignore}`
- 新增：`packages/shared-types/src/{index.ts,api-contract.ts,generated/api-schema.d.ts}`
- 新增：`packages/api-client/{package.json,tsconfig.json,.prettierignore}`
- 新增：`packages/api-client/src/{index.ts,client.ts,request.ts,errors.ts}`
- 新增：`scripts/gen-api.sh`（生成 OpenAPI → schema → 三个包类型检查）

**数据库与 API 变化**

无后端行为变化。生成物对应后端 6 个 path 与全部组件 schema（`ApiResponse_*`、`ApiErrorBody`、`DependencyStatusDto` 等）。

**测试与结果**

- `pnpm install --frozen-lockfile`：通过，`pnpm peers check` 无 peer 依赖问题。
- `pnpm --filter @cngokz/shared-types gen:api`：生成成功（105 ms）。
- `pnpm --filter @cngokz/shared-types typecheck`、`pnpm --filter @cngokz/api-client typecheck`：通过。
- 修正记录：初次生成后 `ApiClientError` 类型断言失败，原因是生成 schema 中 `success: boolean`（而非字面量 `false`）；已将 `ApiErrorEnvelope.success` 调整为 `boolean` 并注明原因。

---

## 2026年11月03日 19时26分44秒 后端基础服务：Axum、配置、PostgreSQL、Redis、迁移与 OpenAPI

**背景**

需要可运行的后端骨架：异步 Axum 服务、统一响应与错误、模块化分层、配置管理、SQLx 连接池与版本化迁移、Redis 连接管理、健康检查、Utoipa 文档、优雅关闭，且 `main.rs` 不堆积业务逻辑。

**变更内容**

- Rust workspace（`Cargo.toml` + `rust-toolchain.toml` 固定 1.97.1，edition 2024），`apps/api` 拆分为 `lib.rs`（模块导出）与 `main.rs`（仅启动/优雅关闭）。
- 配置：`config/` 从环境变量读取并校验 `APP_ENV/APP_HOST/APP_PORT/APP_VERSION/LOG_LEVEL/CORS_ALLOWED_ORIGINS/DATABASE_URL/REDIS_URL/REDIS_KEY_PREFIX/DATABASE_MAX_CONNECTIONS/DATABASE_MIN_CONNECTIONS/DATABASE_ACQUIRE_TIMEOUT_SECS/HEALTH_CHECK_TIMEOUT_MS`；URL scheme、端口范围、CORS 格式、连接数关系校验；手写 `Debug` 脱敏密码；`from_pairs` 便于测试。
- 状态：`state/` 中 `AppState`（`Arc` 共享，含配置、连接池、Redis 客户端、启动时间）、`connections::connect`（连接池 → 迁移 → Redis 客户端 → 启动探测）、`redis_client.rs`（`ConnectionManager` 懒连接、`PING` 失败丢弃缓存连接、`key_prefix` 命名空间）。
- 中间件：CORS（按配置来源，支持 `*`）、`x-request-id`（生成/透传）、Tracing（请求 span、延迟、失败 ERROR）。
- 分层：`handlers/health.rs`、`handlers/system.rs`、`services/{health,system}.rs`、`repositories/{health_repository,system_event_repository}.rs`、`models/{dependency,system_event}.rs`、`dto/{response,health,system}.rs`、`errors/{mod,startup}.rs`。
- 统一契约：`ApiResponse<T> { success, data, message }` 与 `ApiErrorBody { success, error: { code, details }, message }`；`ErrorCode` 覆盖 validation/unauthorized/forbidden/not_found/database/redis/dependency_unavailable/not_ready/internal，HTTP 状态码按语义映射（400/401/403/404/500/503）；内部错误只进日志，客户端仅见安全文案；`NotFound` 兜底路由返回统一错误。
- 接口：`GET /health`、`GET /health/live`、`GET /health/ready`（并发探测 PG/Redis，超时受控，失败 503 + 脱敏 checks）、`GET /api/v1/system/info`、`GET /api/v1/system/status`、`GET /api/v1/system/events?limit=1..100`。
- 数据库：`migrations/20261103000001_init_system_tables.sql` 创建 `system_events`（UUID 主键、severity CHECK、metadata JSONB、created_at 索引、event_type 索引）与 `app_settings`（key/value JSONB）；启动时 `sqlx::migrate!("./migrations")`。
- OpenAPI：`routes/openapi.rs` 汇总 paths/schemas/tags，`SwaggerUi`（`vendored`，无需构建期下载前端资源）挂载 `/swagger-ui` 与 `/api-docs/openapi.json`；新增 `bin/dump_openapi.rs` 输出 `docs/openapi.json`。
- 日志：`utils/telemetry.rs` 开发环境 compact、生产环境 JSON；`utils/redact.rs` 脱敏 URL。
- 优雅关闭：SIGINT/SIGTERM → 停止接收请求 → 关闭数据库连接池 → 退出。

**涉及文件**

- 新增：`apps/api/Cargo.toml`
- 新增：`apps/api/src/{lib.rs,main.rs}`
- 新增：`apps/api/src/bin/dump_openapi.rs`
- 新增：`apps/api/src/config/mod.rs`
- 新增：`apps/api/src/state/{mod.rs,connections.rs,redis_client.rs}`
- 新增：`apps/api/src/middleware/{mod.rs,cors.rs,request_id.rs,trace.rs}`
- 新增：`apps/api/src/routes/{mod.rs,openapi.rs}`
- 新增：`apps/api/src/handlers/{mod.rs,health.rs,system.rs}`
- 新增：`apps/api/src/services/{mod.rs,health.rs,system.rs}`
- 新增：`apps/api/src/repositories/{mod.rs,health_repository.rs,system_event_repository.rs}`
- 新增：`apps/api/src/models/{mod.rs,dependency.rs,system_event.rs}`
- 新增：`apps/api/src/dto/{mod.rs,response.rs,health.rs,system.rs}`
- 新增：`apps/api/src/errors/{mod.rs,startup.rs}`
- 新增：`apps/api/src/utils/{mod.rs,redact.rs,telemetry.rs}`
- 新增：`apps/api/migrations/20261103000001_init_system_tables.sql`
- 新增：`docs/openapi.json`（生成物，已提交）
- 修改：`Cargo.toml`（workspace、依赖版本、release `strip`）

**数据库与 API 变化**

- 数据库：新增 `system_events`、`app_settings` 两张表及索引/约束；迁移版本由 `_sqlx_migrations` 管理。
- API：新增 6 个接口（见上）；统一响应/错误结构；`/api/v1` 版本前缀；OpenAPI 3.1 文档与 Swagger UI。

**测试与结果**

- `cargo build`（debug）、`cargo build --release --bin cngokz-api`：通过。
- `cargo fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`：通过。
- 使用本地 PostgreSQL 18.6 + Redis 8.0.5 实例启动服务后 curl 验证：
  - `/health` → 200，`data.status=ok`，返回 `x-request-id`；
  - `/health/ready` → 200，`postgresql`/`redis` 均 `up`；
  - `/api/v1/system/info` → 200，版本/环境/启动时间/运行时长正确；
  - `/api/v1/system/status` → 200，`healthy`；
  - `/api/v1/system/events?limit=5` → `items=[] count=0`（空表，未造数据）；
  - `?limit=0` → 400 `validation_error`；
  - `/api-docs/openapi.json` → 6 个 path；`/swagger-ui` → 重定向，`/swagger-ui/` → 200 HTML；
  - 停止 Redis 后 `/health/ready` → 503，`redis.status=down`、`detail=connection_failed`；`/api/v1/system/status` → `degraded`；重启 Redis 后自动恢复 200；
  - 重启 PostgreSQL 后 API 连接池自愈，`/health/ready` 仍 200。
- **未执行**：Docker 相关验证（环境无 docker）；浏览器端到端验证（无图形浏览器）。

---

## 2026年11月03日 19时21分07秒 Monorepo 初始化与工具链配置

**背景**

项目为空目录，需要先确定仓库形态、技术栈版本与统一开发规范，为前后端与基础设施提供入口。

**变更内容**

- 采用 pnpm workspace + Cargo workspace 的 monorepo；目录骨架：`apps/{web,api}`、`packages/{api-client,shared-types}`、`infrastructure/{docker,postgres,redis}`、`docs/`、`scripts/`、`.github/workflows/`。
- 根 `package.json`：固定 `packageManager: pnpm@11.28.5`，提供 `dev/build/lint/format/typecheck/test/gen:openapi/docker:*` 脚本；根 devDependency 为 Prettier 与 Tailwind class 排序插件。
- `pnpm-workspace.yaml`（`apps/web`、`packages/*`）；`Cargo.toml` workspace（`members = ["apps/api"]`，resolver 3，统一依赖版本与 release profile）；`rust-toolchain.toml` 固定 1.97.1 + rustfmt/clippy。
- `.gitignore` 覆盖 Node/Rust 构建产物、`.env`、编辑器与日志；`.env.example` 记录后端、数据库、Redis、Compose 与 Web 端口等全部环境变量（不含真实凭据）。
- 代码风格：根 `.prettierrc.json`/`.prettierignore`（Rust 与生成物不参与）、各工作区独立 Prettier 配置。
- CI：`.github/workflows/ci.yml` 三个 job —— API（fmt/clippy/test + OpenAPI 是否最新）、Web（lint/typecheck/test/build）、Docker（`docker compose config` + 两个镜像构建）。

**涉及文件**

- 新增：`package.json`、`pnpm-workspace.yaml`、`Cargo.toml`、`rust-toolchain.toml`
- 新增：`.gitignore`、`.env.example`、`.dockerignore`（后于 Docker 记录补充）
- 新增：`.prettierrc.json`、`.prettierignore`
- 新增：`.github/workflows/ci.yml`
- 新增空目录：`apps/api/migrations`、`apps/api/tests`、`apps/web/src/*`、`packages/{api-client,shared-types}/src`、`infrastructure/*`、`docs`、`scripts`

**数据库与 API 变化**

无（本组仅初始化工程结构与工具链）。

**测试与结果**

- `pnpm install`：通过（首次因 `@eslint/js@10.14.0` 不存在失败，改为 `10.0.1` 后成功）。
- `pnpm install --frozen-lockfile`：通过。
- `cargo build`：通过（依赖首次编译，使用本地 crates.io 缓存/网络）。
- CI 工作流：**未执行**（本环境无 GitHub Actions runner）。
