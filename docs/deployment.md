# 部署指南

使用 Docker Compose 部署五个服务：PostgreSQL、Redis、Rust API、管理后台（内部）、公开门户（唯一对外入口）。生产环境不需要额外的 Nginx 即可运行；如需 TLS 或统一域名，可在后续阶段前置反向代理。

## 1. 前置条件

- Docker Engine + Docker Compose v2
- 可写的持久化磁盘（`cngokzapi-postgres-data`、`cngokzapi-redis-data` 卷）
- 至少 2 vCPU / 2 GB 内存（Rust release 构建期间建议 4 GB）

## 2. 首次部署

```bash
git clone <repo> && cd CNGOKZAPI
cp .env.example .env
```

**必须修改** `.env` 中的：

| 变量 | 建议 |
| --- | --- |
| `POSTGRES_PASSWORD` | 强随机密码，勿用默认 `cngokz` |
| `APP_ENV` | `production` |
| `LOG_LEVEL` | `info`（排障时临时 `debug`） |
| `CORS_ALLOWED_ORIGINS` | 若浏览器只经 Web 访问 API，可留空；如直连 API 则填前端域名 |
| `POSTGRES_PORT` / `REDIS_PORT` | 生产环境可注释掉端口映射（见"安全基线"） |
| `PORTAL_PORT` | 对外暴露的唯一门户端口（默认 5173；`/` 公开站，`/admin` 管理后台） |
| `VITE_SITE_URL` | 公开门户 canonical/sitemap 的站点地址（默认 `http://localhost:5173`）；更改后需重建 public 镜像 |
| `ADMIN_PORT` | 仅本地开发的管理后台内部端口（默认 5174） |
| `PUBLIC_RATE_LIMIT_PER_MINUTE` | 公开 API 每 IP 每分钟请求上限（默认 120） |
| `MAINTENANCE_TOKEN` | 公开缓存失效端点的保护令牌；留空时端点禁用 |

> compose 默认把 PostgreSQL/Redis/API 端口绑定在 `127.0.0.1`，只有 Web 监听 `0.0.0.0`。

启动：

```bash
docker compose up -d --build
docker compose ps
docker compose logs -f
```

预期顺序：`postgres` / `redis` 健康 → `api` 构建并执行迁移、健康检查通过 → `web`（内部管理后台）健康 → `public`（唯一对外入口）启动。

验证：

```bash
curl -fsS http://localhost:5173/health | jq                       # 经公开门户同源代理
curl -fsS http://localhost:5173/api/v1/system/status | jq        # 依赖状态
curl -fsS http://localhost:5173/api/v1/public/statistics | jq    # 公开统计
curl -fsS http://localhost:5173/admin/ -o /dev/null -w '%{http_code}\n'  # 管理后台
curl -fsS http://localhost:8080/health/ready | jq               # 宿主机直连 API
```

浏览器访问 `http://<host>:5173`（公开门户，默认）与 `http://<host>:5173/admin`（管理后台）。两者共用同一后端；管理后台仪表盘显示真实的 API / PostgreSQL / Redis 状态，公开门户的统计、排行榜、成绩来自同一数据库。

## 3. 服务与端口

| 服务 | 容器端口 | 宿主机默认 | 暴露范围 |
| --- | --- | --- | --- |
| public（单入口） | 5173 | 5173 | `0.0.0.0` |
| web（内部，管理后台） | 5173 | — | 仅 compose 内部 |
| api | 8080 | 8080 | `127.0.0.1` |
| postgres | 5432 | 5432 | `127.0.0.1` |
| redis | 6379 | 6379 | `127.0.0.1` |

服务间通过 compose 网络 `cngokzapi-backend` 使用服务名通信：API 使用 `postgres:5432`、`redis:6379`，Web 使用 `http://api:8080`。

## 4. 环境变量

见 [`.env.example`](../.env.example)。容器内覆盖关系：

- `api` 的 `DATABASE_URL` / `REDIS_URL` 由 compose 使用 `POSTGRES_*` 变量拼装，**不要**在 `.env` 中手工写成 `localhost`，否则容器内解析失败。
- `APP_HOST` 固定 `0.0.0.0`，`APP_PORT` 固定 `8080`（与 compose 端口映射一致）。
- 生产环境推荐 `APP_ENV=production`：日志以 JSON 输出，便于采集。
- 不要提交 `.env`；使用编排平台的 secret 或受限权限的部署目录。

## 5. 数据持久化与备份

| 卷 | 内容 | 说明 |
| --- | --- | --- |
| `cngokzapi-postgres-data` | PostgreSQL 数据目录 | 挂载到 `/var/lib/postgresql`（PostgreSQL 18 的 PGDATA 在其子目录） |
| `cngokzapi-redis-data` | Redis AOF/RDB | AOF `everysec` |

- `docker compose down` **保留**卷；`docker compose down -v` 会删除所有数据，仅在确认可丢弃时使用。
- 服务重启后数据不丢失（迁移版本记录在 `_sqlx_migrations`，不会重复执行已应用迁移）。

备份：

```bash
docker compose exec -T postgres pg_dump -U cngokz -d cngokzapi -Fc > backup-$(date +%F).dump
```

恢复：

```bash
docker compose stop api
docker compose exec -T postgres pg_restore -U cngokz -d cngokzapi --clean --if-exists < backup-2026-11-03.dump
docker compose start api     # 启动时自动补齐迁移
```

Redis 仅存缓存/临时数据，第一阶段无需备份；如需保留可备份 `cngokzapi-redis-data` 卷。

## 6. 更新与重启

```bash
git pull
docker compose build api web
docker compose up -d
docker compose ps
docker compose logs -f api
```

- 迁移由新版本 API 启动时自动执行，属于部署的一部分；请先评估破坏性迁移。
- 只改前端时可 `docker compose build web && docker compose up -d web`。
- 回滚：检出上一个 tag/commit，重新 `docker compose up -d --build`。数据库迁移默认只向前（down 迁移仅用于开发），必要时按备份恢复。

## 7. 可选：反向代理与 TLS

第一阶段不强制 Nginx。若需要统一域名，可只代理 Web 容器的 5173 端口，API 通过 Web 内置代理访问（`/api`、`/health`、`/swagger-ui`、`/api-docs`）：

```nginx
server {
    listen 443 ssl http2;
    server_name kz.example.com;

    location / {
        proxy_pass http://127.0.0.1:5173;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

此时应清空 `CORS_ALLOWED_ORIGINS`（同源访问），并将 `API_PORT` 映射移除。

## 8. 安全基线

- 数据库使用强密码，且不发布到公网（默认已绑定回环地址）。
- 生产环境 `CORS_ALLOWED_ORIGINS` 不使用 `*`。
- 不向公网暴露 `/swagger-ui` 与 `/api-docs`（可仅经内网访问，或后续加认证）。
- 及时更新基础镜像（`postgres:18-alpine`、`redis:8-alpine`、`node:24-alpine`、`debian:bookworm-slim`）。
- API 容器以非 root 用户（uid 10001）运行。
- 密钥/密码不进入镜像与 Git；`.dockerignore` 已排除 `.env`。
- 定期备份并验证恢复流程。

## 9. 故障排查

| 现象 | 排查步骤 |
| --- | --- |
| `api` 一直 `unhealthy` | `docker compose logs api`；通常是 `DATABASE_URL` 错误或迁移失败 |
| `web` 容器启动但页面 502 | 检查 `API_UPSTREAM` 是否为 `http://api:8080`；`docker compose ps api` |
| `/health/ready` 返回 503 | 响应体 `error.details.checks` 指出是 `postgresql` 还是 `redis`；`docker compose logs postgres redis` |
| PostgreSQL 健康检查失败 | 首次启动数据目录权限、卷残留都可能导致；确认卷属于空目录初始化（必要时先备份） |
| 构建 Rust 镜像极慢/内存不足 | 构建阶段需要编译全部依赖；增加内存或使用 CI 缓存 |
| 端口被占用 | 修改 `.env` 中对应 `*_PORT`，或 `docker compose down` 释放 |
| 页面能开但数据不更新 | 浏览器请求的 `/api` 是否命中代理；查看 Web/Public 容器日志中的 `[proxy] ... failed` |
| 公开门户站点 URL 错误 | 修改 `.env` 的 `VITE_SITE_URL` 后执行 `docker compose build public && docker compose up -d public` |
| `/admin` 502 | public 容器依赖 web 容器健康；检查 `docker compose ps web` 与 `ADMIN_UPSTREAM` |
| 重启后数据丢失 | 是否使用了 `docker compose down -v`；检查卷 `docker volume ls | grep cngokzapi` |

查看容器内 API 的实时状态：

```bash
docker compose exec api curl -fsS http://127.0.0.1:8080/api/v1/system/status | jq
```

## 10. 资源与扩展

- 单实例足以支撑第一阶段；后续可按模块拆出只读副本或独立 Worker，但**不要**提前拆分微服务。
- 提高 `DATABASE_MAX_CONNECTIONS` 时同步评估 PostgreSQL `max_connections`。
- Redis `maxmemory` 为 256MB，接入真实缓存前按容量调整 `infrastructure/redis/redis.conf`。

## 11. 缓存失效与运维（第五阶段）

公开 API 使用 Redis 命名空间版本 `public:version`：数据变更后调用一次受保护端点即可让全部公开缓存失效，并自动预热首页统计。

```bash
# 需要 MAINTENANCE_TOKEN；未配置时返回 403
curl -X POST http://localhost:8080/api/v1/public/maintenance/cache/invalidate \
  -H "X-Maintenance-Token: $MAINTENANCE_TOKEN"

# 查询当前缓存命名空间版本
curl -H "X-Maintenance-Token: $MAINTENANCE_TOKEN" \
  http://localhost:8080/api/v1/public/maintenance/cache/version
```

公开 API 每 IP 每分钟限流由 `PUBLIC_RATE_LIMIT_PER_MINUTE` 控制；维护端点不受该限流，但仍然需要令牌。限流器为单进程内存实现，多副本部署应改用 Redis 或边缘网关限流。

## 12. 数据库备份与恢复（补充）

```bash
# 备份（仅数据卷容器，不停止服务）
docker compose exec -T postgres pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" -Fc > backup-$(date +%F).dump

# 恢复（会覆盖目标库，执行前请确认）
docker compose stop api
docker compose exec -T postgres pg_restore -U "$POSTGRES_USER" -d "$POSTGRES_DB" --clean --if-exists < backup-2026-11-04.dump
docker compose start api
curl -X POST http://localhost:8080/api/v1/public/maintenance/cache/invalidate \
  -H "X-Maintenance-Token: $MAINTENANCE_TOKEN"
```

> 恢复后必须使缓存失效，否则公开页面可能继续展示旧数据直到 TTL 到期。
