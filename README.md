# CNGOKZAPI

面向 **CS:GO KZ 社区** 的独立 API 与数据平台。当前进行**第八阶段：GOKZ-Global / kzgo.eu 多数据源接入与自有积分系统**。已实现统一原始成绩/来源溯源、跨来源去重、数据源健康监控、kzgo.eu 实时同步、GOKZ-Global 适配器、CNGOKZ 自有积分审计列与外部服务器兜底展示。完整调查、限制与验证结果见 [docs/phase8-multi-source-ingestion.md](docs/phase8-multi-source-ingestion.md)。

> 第一阶段不包含任何模拟业务数据：仪表盘上的所有状态都来自后端真实健康检查，未实现模块显示空状态。

## 技术栈

### 前端 `apps/web`

| 依赖 | 版本 |
| --- | --- |
| React / React DOM | 19.3.1 |
| TypeScript（strict） | 5.9.3 |
| Vite | 8.3.5 |
| Tailwind CSS | 4.3.4 |
| shadcn/ui 结构 + Radix UI | radix-ui 1.8.1 |
| React Router | 8.5.1 |
| TanStack Query | 5.105.3 |
| React Hook Form / Zod | 7.91.0 / 4.6.7 |
| Lucide Icons | 1.67.0 |
| Zustand | 5.0.16 |
| 字体 | Inter Variable（`@fontsource-variable/inter`，自托管） |
| 包管理 | pnpm 11.28.5 |

### 后端 `apps/api`

| 依赖 | 版本 |
| --- | --- |
| Rust（edition 2024） | 1.97.1（见 `rust-toolchain.toml`） |
| Tokio | 1.54 |
| Axum | 0.8.9 |
| SQLx | 0.9.0 |
| Redis | 1.8.1 |
| Utoipa / Swagger UI | 6.0.0 / 10.0.2 |
| tower-http | 0.7.1 |
| tracing / tracing-subscriber | 0.1 / 0.3 |
| Serde / thiserror / anyhow | 1 / 2 / 1 |

### 基础设施

原生 systemd + Nginx 部署；PostgreSQL 18、Redis 8。应用不再使用 Docker。

## 项目结构

```text
CNGOKZAPI/
├── apps/
│   ├── api/                     # Rust + Axum 服务
│   │   ├── build.rs             # 监听 migrations/ 目录触发重编译
│   │   ├── migrations/          # SQLx 版本化迁移（系统表 + KZ 数据模型）
│   │   ├── src/
│   │   │   ├── config/          # 环境变量读取、校验、脱敏 Debug
│   │   │   ├── state/           # AppState、PostgreSQL 连接池、Redis 客户端
│   │   │   ├── middleware/      # CORS、request-id、tracing
│   │   │   ├── routes/          # 路由装配 + OpenAPI 文档
│   │   │   ├── handlers/        # HTTP 输入输出
│   │   │   ├── services/        # 业务编排与依赖检查
│   │   │   ├── repositories/    # SQL 访问
│   │   │   ├── models/          # 内部领域模型
│   │   │   ├── dto/             # API 契约（serde + utoipa）
│   │   │   ├── errors/          # 统一错误与启动错误
│   │   │   ├── utils/           # redact、telemetry
│   │   │   ├── bin/dump_openapi.rs
│   │   │   ├── lib.rs
│   │   │   └── main.rs          # 仅负责启动/优雅关闭
│   │   └── tests/               # HTTP 与真实依赖集成测试
│   └── web/                     # React 管理后台
│       └── src/
│           ├── components/{ui,layout,dashboard,common,theme}
│           ├── layouts/         # 应用外壳（侧边栏 + 顶栏 + 内容区）
│           ├── pages/           # 路由页面
│           ├── hooks/           # TanStack Query 封装
│           ├── services/        # 查询选项 / 数据服务
│           ├── stores/          # Zustand UI 状态（主题、侧边栏）
│           ├── router/          # 路由表（按页面懒加载）
│           ├── lib/             # env、api client、格式化、工具
│           ├── types/           # UI 层类型
│           ├── styles/          # Tailwind v4 入口与设计令牌
│           └── assets/
├── packages/
│   ├── api-client/              # 基于 openapi-fetch 的类型化客户端（后台 + 门户）
│   ├── shared-types/            # OpenAPI 生成的类型 + 共享契约
│   └── ui/                      # 共享设计 Token（design-system.css）
├── deploy/                      # systemd unit / Nginx 站点配置
├── docs/                        # 架构、开发、部署、变更记录、OpenAPI JSON
│   ├── database/                # 数据模型文档（overview/模块/ER 图）
│   ├── design-system.md         # 管理后台设计系统
│   ├── public-portal/           # 公开门户架构/页面/API/设计文档
│   └── globalapi-compatibility.md
├── scripts/                     # build-native.sh / install-native.sh / gen-api.sh
├── .github/workflows/ci.yml
├── .env.example
├── .env.production.example
├── Cargo.toml                   # Rust workspace
└── pnpm-workspace.yaml
```

## 生产部署（Native / 无 Docker）

生产环境使用 **Rust release 二进制 + systemd + Nginx + 静态前端**。后端和前端不通过 Docker 运行。

发布链路：

```text
本地 git push develop
        ↓
不会自动部署
        ↓
手动运行 GitHub Actions: Deploy production (manual)
        ↓
构建前后端包 → Cloudflare R2 → SSH 生产服务器从 R2 下载安装
```

只有手动触发工作流、且分支必须是 `main` 或 `develop` 才会部署。生产服务器不访问 GitHub，也不需要保留拉取脚本。

部署结构：

```text
Nginx
  ├── /        → /var/www/cngokzapi/public
  ├── /admin/  → /var/www/cngokzapi/admin
  └── /api     → 127.0.0.1:8080
                    │
                    ▼
              systemd: cngokz-api
```

详见 [部署指南](docs/deployment.md)。

## 本地开发

前置：Node.js ≥ 22、pnpm 11、Rust 1.97、PostgreSQL、Redis。

```bash
pnpm install
cp .env.example .env
# 启动现有的 PostgreSQL / Redis，或按开发文档准备依赖
cargo run -p cngokz-api           # API: http://localhost:8080
pnpm dev                          # 公开门户(5173) + 管理后台(内部 5174)
```

Web 开发服务器会把 `/api`、`/health`、`/swagger-ui`、`/api-docs` 代理到 `VITE_API_PROXY_TARGET`（默认 `http://localhost:8080`），因此浏览器只访问同源地址，无需配置 CORS。

详细步骤见 [docs/development.md](docs/development.md)。

## 环境变量

完整清单与说明见 [`.env.example`](.env.example)。

| 变量 | 默认值 | 说明 |
| --- | --- | --- |
| `APP_ENV` | `development` | `development` / `test` / `production` |
| `APP_HOST` / `APP_PORT` | `0.0.0.0` / `8080` | API 监听地址 |
| `APP_VERSION` | `0.1.0` | 返回给 `/api/v1/system/info` |
| `LOG_LEVEL` | `info` | tracing EnvFilter |
| `CORS_ALLOWED_ORIGINS` | 空 | 逗号分隔，仅浏览器直连 API 时需要 |
| `HEALTH_CHECK_TIMEOUT_MS` | `2000` | 单个依赖探测超时 |
| `DATABASE_URL` | — | 必填，`postgres://...` |
| `DATABASE_MAX_CONNECTIONS` / `DATABASE_MIN_CONNECTIONS` | `10` / `1` | 连接池 |
| `REDIS_URL` | — | 必填，`redis://...` |
| `REDIS_KEY_PREFIX` | `cngokz` | 预留缓存命名空间 |
| `PORTAL_PORT` / `ADMIN_PORT` | `5173` / `5174` | 仅前端开发服务器端口 |
| `KZGO_BASE_URL` / `KZGO_SYNC_ENABLED` | `https://kzgo.eu` / `true` | kzgo.eu 数据源 |
| `GOKZ_GLOBAL_BASE_URL` / `GOKZ_GLOBAL_SYNC_ENABLED` | `https://kztimerglobal.com/api/v2.0` / `false` | GOKZ-Global 数据源 |

生产环境请使用强密码，将 `.env` 放在服务器受限目录并设置 `600/640` 权限，**不要提交真实凭据**。

## 常用命令

```bash
# 质量检查
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
pnpm -r lint
pnpm -r typecheck
pnpm -r format:check

# 测试
cargo test --all-targets                      # 需 DATABASE_URL/REDIS_URL 才执行集成用例
# 带真实依赖：
# DATABASE_URL=... REDIS_URL=... cargo test --all-targets
# 缓存失效维护端点（需 MAINTENANCE_TOKEN）：
# curl -X POST localhost:8080/api/v1/public/maintenance/cache/invalidate -H "X-Maintenance-Token: ..." 
pnpm -r test

# 构建
cargo build --release --bin cngokz-api
pnpm --filter @cngokz/web build

# OpenAPI → TypeScript 类型
scripts/gen-api.sh
```

## API 概览

所有业务接口使用 `/api/v1` 前缀，统一响应结构：

```json
{ "success": true, "data": { }, "message": "ok" }
```

```json
{ "success": false, "error": { "code": "not_ready", "details": { } }, "message": "Service is not ready" }
```

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/health` | API 运行状态（不探测依赖） |
| GET | `/health/live` | 进程存活 |
| GET | `/health/ready` | PostgreSQL + Redis 就绪检查（异常返回 503） |
| GET | `/api/v1/system/info` | 版本、环境、启动时间、运行时长 |
| GET | `/api/v1/system/status` | 依赖实时状态（200，含 degraded） |
| GET | `/api/v1/system/events?limit=` | 最近系统事件（1–100，默认 20） |
| GET | `/swagger-ui` / `/api-docs/openapi.json` | Swagger UI / OpenAPI 3.1 文档 |
| POST | `/api/v1/game-servers/*` | CS:GO 插件心跳/玩家/成绩/Replay（服务器 Token） |
| GET/POST | `/api/v1/admin/game-servers/*` | 服务器创建、启停、Token 轮换与同步详情（维护 Token） |
| GET | `/api/v1/public/players/online` | 心跳正常服务器的真实在线玩家 |
| GET/PATCH/POST | `/api/v1/admin/sources*` | 数据源状态、启停、手动同步、运行记录（维护 Token） |
| GET | `/api/v1/admin/records/{id}/sources` | 成绩原始来源与去重审计（维护 Token） |

## 公开玩家门户（第四阶段）

- 独立应用 `apps/public` 作为默认站点（`/`）；管理后台挂载在同一端口下 ` /admin`（同一 IP、同一端口，无需分开部署）。两者共用 API、数据库、API Client 与设计 Token，但保持独立页面布局与独立构建。
- 页面：首页（统计/模式 Top10/最近 20 条成绩/最近 WR/活跃玩家/在线服务器/全局搜索）、排行榜、玩家目录、玩家详情（综合/模式/难度进度/地图完成与未完成/成绩历史）、玩家比较（综合/模式/难度/逐地图）、地图列表与详情（课程/统计/排行榜）、成绩查询（多维筛选/排序/详情）、服务器列表。
- 后端新增 `/api/v1/public/*` 只读接口（统计、模式、排行榜、玩家、地图、成绩、服务器、搜索），分页上限 100，统一响应结构并纳入 OpenAPI；SQL 聚合（`LATERAL` 最佳成绩、`RANK()`、`count(DISTINCT)`），避免 N+1。
- Redis 缓存：统计 30s、排行榜 60s、最近成绩 15s、玩家/地图 30–120s；`public:version` 计数器可一次性失效；Redis 故障时 300ms 超时快速回源。
- 数据准确性：完成地图以主线课程（stage 0）为准，Bonus 单独统计；完成时间（跑图时长）与上传时间严格区分。积分在第六阶段按 Filter 公式实现，详见 [docs/points-system.md](docs/points-system.md)。
- 已知缺口：玩家头像、地图作者/封面、服务器玩家数/模式、WR 历史提升值、Replay 公开下载、动态 sitemap。详见 [公开门户架构](docs/public-portal/architecture.md#10-已知缺口与阶段-5-建议)。

## 界面设计（第三阶段）

- 默认深色主题，支持浅色与跟随系统；主题、侧边栏折叠状态持久化在浏览器。
- 设计 token 集中在 `apps/web/src/styles/index.css`，组件在 `apps/web/src/components/ui/`，页面不写一次性样式。
- 布局：240/64px 可折叠侧边栏 + 56px 顶栏（面包屑、页面搜索 ⌘K、通知、主题、账号）+ 1500px 内容区。
- 已实现 15 个导航入口 + 2 个详情页、7 个导航分组；旧路径（`/kz/*`、`/data/*`）自动重定向。
- 所有未实现接口的页面使用真实空状态与"接口未接入"说明，**不伪造统计或业务数据**；系统健康数据来自 `/api/v1/system/status` 与 `/api/v1/system/info`。
- 详细规范见 [docs/design-system.md](docs/design-system.md)。

## 数据模型（第二阶段）

- 20 张新表（17 张业务表 + 3 张引用表）：`accounts`、`player_profiles`、`player_mode_stats`、`player_identities`、`maps`、`map_versions`、`map_courses`、`map_modes`、`course_tiers`、`servers`、`records`、`record_history`、`course_world_records`、`jumpstats`、`replays`、`bans`、`ban_revocations`，以及引用表 `movement_modes`、`movement_styles`、`jump_types`。
- SteamID64 使用 `NUMERIC(20,0)`（避免 `BIGINT` 溢出）；成绩时间使用整数纳秒；TP/PRO 由传送次数通过 CHECK 强制推导；重复提交与 WR 并发更新均有数据库级约束。
- 研究依据：GOKZ 3.8.0、cs2kz-api、csgo-legacy-globalapi-plugin 的真实源码与迁移（详见数据模型文档）。

## 文档

- [架构说明](docs/architecture.md)
- [设计系统](docs/design-system.md)
- 公开门户：[架构](docs/public-portal/architecture.md) · [页面](docs/public-portal/pages.md) · [API 与数据方案](docs/public-portal/api-requirements.md) · [设计系统](docs/public-portal/design-system.md)
- [开发指南](docs/development.md)
- [部署指南](docs/deployment.md)
- [数据库总览](docs/database/overview.md) / [ER 图](docs/database/er-diagram.md)
- [Accounts](docs/database/accounts.md) · [Maps](docs/database/maps.md) · [Records](docs/database/records.md) · [Servers](docs/database/servers.md) · [Jumpstats](docs/database/jumpstats.md) · [Replays](docs/database/replays.md) · [Moderation](docs/database/moderation.md)
- [GlobalAPI 兼容性分析](docs/globalapi-compatibility.md)
- [开发记录（变更日志）](docs/changelog.md)

## 第六阶段状态

- 新增 Filter 积分系统：KZT / SKZ / VNL 完全独立，权威 `(map,course,mode)` 等级与 VNL T9 首通历史标签；积分公式、PB/WR/排名重算、玩家模式总分、Filter/模式积分榜、管理端点与前后端页面已实现。
- 详见 [docs/points-system.md](docs/points-system.md) 与 [docs/phase6-audit.md](docs/phase6-audit.md)。
- 积分/Rating 已按第六阶段规则实现；历史成绩 T9 回溯、封禁成绩排除、同成绩 tie-break 等仍为待确认事项。

## 第五阶段状态

- 完成审查（[docs/phase5-audit.md](docs/phase5-audit.md)）、统计口径统一、Redis 缓存硬化、公开接口限流、服务器四态、性能索引与 EXPLAIN 验证、静态路径安全、部署参数修正。
- 积分算法已在第六阶段实现；管理后台认证仍只有维护令牌，服务器心跳上报、Replay 下载、动态 sitemap 尚未实现。
- Docker 启动因环境无 `docker` 命令未实际执行；其余构建、测试与端到端验证通过。

## 阶段状态

**第一阶段（完成）**：项目骨架、管理后台 UI、健康检查、系统信息接口、数据库迁移机制、Redis 连接管理、OpenAPI 与类型化客户端、Docker 编排、基础测试与文档。

**第二阶段（完成）**：核心数据库架构（20 张新表 + 索引/约束）、Rust model 与 DTO 契约、GlobalAPI 字段映射与兼容性分析、数据库文档与 Mermaid ER 图、schema 集成测试（53 个后端测试全部通过）。

**第三阶段（完成）**：完整设计系统（token/字体/间距/组件/动效/可访问性）、15 个路由页面与 7 个导航分组、可折叠侧边栏与移动抽屉、主题持久化、⌘K 页面搜索、命令行/弹窗/下拉/提示/分页/筛选等交互、真实系统健康状态展示、响应式与视觉回归验证。

**第四阶段（完成）**：独立公开门户（首页/排行榜/玩家/玩家详情/比较/地图/地图详情/成绩/服务器）、`/api/v1/public/*` 只读 API（含 OpenAPI）、SQL 聚合与 Redis 缓存、SEO 基础配置（动态 meta、sitemap、robots）、响应式与真实数据端到端测试（54 个后端测试 + 26 个前端测试通过）。

未实现（后续阶段）：积分算法与积分榜、管理员认证/API Key、成绩导入与审核、服务器心跳采集（玩家数/模式）、Replay 公开下载、动态 sitemap/SSR、通知与监控。详见 `docs/changelog.md`。

## 许可

MIT
