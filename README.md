# CNGOKZAPI

面向 **CS:GO / CS2 KZ 社区** 的独立 API 与管理平台。当前为**第一阶段：项目基础架构**，包含可运行的前后端、数据库迁移、健康检查、OpenAPI 文档与 Docker 部署编排，业务模块（玩家、地图、成绩、排行榜、Jumpstats、Replay、服务器、封禁、插件）保留页面与扩展位置，尚未实现。

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

Docker（多阶段构建）、Docker Compose、PostgreSQL 18、Redis 8。

## 项目结构

```text
CNGOKZAPI/
├── apps/
│   ├── api/                     # Rust + Axum 服务
│   │   ├── migrations/          # SQLx 版本化迁移
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
│       ├── server/              # 生产静态服务器（含 API 反向代理）
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
│   ├── api-client/              # 基于 openapi-fetch 的类型化客户端
│   └── shared-types/            # OpenAPI 生成的类型 + 共享契约
├── infrastructure/
│   ├── docker/                  # api.Dockerfile / web.Dockerfile
│   ├── postgres/init/           # 数据库初始化脚本
│   └── redis/redis.conf
├── docs/                        # 架构、开发、部署、变更记录、OpenAPI JSON
├── scripts/                     # dev-up.sh / gen-api.sh / wait-for-api.sh
├── .github/workflows/ci.yml
├── docker-compose.yml
├── docker-compose.dev.yml
├── .env.example
├── Cargo.toml                   # Rust workspace
└── pnpm-workspace.yaml
```

## 快速开始（Docker）

```bash
cp .env.example .env          # 按需修改密码与端口
docker compose up -d --build  # 启动 PostgreSQL / Redis / API / Web
docker compose logs -f        # 查看日志
docker compose down           # 停止（保留数据卷）
```

启动后：

| 地址 | 说明 |
| --- | --- |
| <http://localhost:5173> | 管理后台 |
| <http://localhost:5173/swagger-ui> | Swagger UI（经 Web 静态服务器代理） |
| <http://localhost:8080/health> | API 健康检查（宿主机回环地址） |
| <http://localhost:8080/api-docs/openapi.json> | OpenAPI 文档 |

## 本地开发

前置：Node.js ≥ 22、pnpm 11、Rust 1.97、Docker（仅用于本地 PostgreSQL/Redis）。

```bash
pnpm install
cp .env.example .env
scripts/dev-up.sh                 # 只启动 PostgreSQL + Redis
cargo run -p cngokz-api           # API: http://localhost:8080
pnpm --filter @cngokz/web dev     # Web: http://localhost:5173
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
| `POSTGRES_USER` / `POSTGRES_PASSWORD` / `POSTGRES_DB` | `cngokz` / `cngokz` / `cngokzapi` | Docker 引导数据库 |
| `POSTGRES_PORT` / `REDIS_PORT` / `API_PORT` / `WEB_PORT` | `5432` / `6379` / `8080` / `5173` | 宿主机端口映射 |

生产环境请使用强密码，并只通过 `.env` 或编排平台的 secret 注入，**不要提交真实凭据**。

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

## 文档

- [架构说明](docs/architecture.md)
- [开发指南](docs/development.md)
- [部署指南](docs/deployment.md)
- [开发记录（变更日志）](docs/changelog.md)

## 阶段状态

第一阶段完成：项目骨架、管理后台 UI、健康检查、系统信息接口、数据库迁移、Redis 连接管理、OpenAPI 与类型化客户端、Docker 编排、基础测试与文档。

未实现：KZ 业务接口与页面数据、认证与 API 密钥、系统设置读写、插件接入、监控告警。详见 `docs/changelog.md` 的验收记录。

## 许可

MIT
