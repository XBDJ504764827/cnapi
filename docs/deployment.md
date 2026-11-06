# 部署指南（Native / 无 Docker）

当前生产部署采用 **原生二进制 + systemd + Nginx + 静态前端**，不使用 Docker 运行 CNGOKZAPI。

PostgreSQL / Redis 可以是：

1. 现有 Docker 容器（只作为基础设施服务），API 通过 `127.0.0.1:5432/6379` 连接；
2. 或直接安装在宿主机上的 PostgreSQL / Redis。

CNGOKZAPI 的 API、管理后台、公开门户**不再使用 Docker**。

---

## 1. 架构

```text
Nginx :443
  ├── /              → /www/sites/test.cngokz.com/index/cnapi
  ├── /admin/        → /www/sites/test.cngokz.com/index/cnapi
  └── /api, /health  → http://127.0.0.1:28080
                              │
                              ▼
                    systemd: cngokz-api
                    /mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi/bin/cngokz-api
                              │
                ┌─────────────┴─────────────┐
                ▼                           ▼
          PostgreSQL                    Redis
     127.0.0.1:5432                127.0.0.1:6379
```

---

## 2. 目录约定

```text
/mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi/
├── bin/cngokz-api
├── .env
└── migrations/              # 仅参考；迁移已编译进二进制

/www/sites/test.cngokz.com/index/cnapi/
├── public/                  # 公开门户
└── admin/                   # 管理后台
```

---

## 3. 前置条件

服务器安装：

```bash
rustc --version
cargo --version
node --version
pnpm --version
nginx -v
```

生产服务器**不需要** Rust / Node。它们只需要在本地构建机安装。

服务器需要：

- PostgreSQL 18（或兼容版本）
- Redis 8
- Nginx
- systemd

---

## 4. 创建数据库

如果继续使用现有 Docker PostgreSQL：

```bash
docker exec -it 1Panel-postgresql-I65c psql -U postgres
```

执行：

```sql
CREATE ROLE cngokz LOGIN PASSWORD '替换为强密码';
CREATE DATABASE cngokzapi OWNER cngokz;
```

进入数据库：

```bash
docker exec -it 1Panel-postgresql-I65c psql -U postgres -d cngokzapi
```

执行：

```sql
CREATE EXTENSION IF NOT EXISTS pg_trgm;
-- 如果 sha256(bytea) 不存在，再执行：
-- CREATE EXTENSION IF NOT EXISTS pgcrypto;
```

验证：

```bash
docker exec -it 1Panel-postgresql-I65c psql -U postgres -d cngokzapi \
  -c "SELECT version();"

docker exec -it 1Panel-postgresql-I65c psql -U postgres -d cngokzapi \
  -c "SELECT sha256('abc'::bytea);"
```

API 在宿主机运行，因此 `.env` 里使用：

```env
DATABASE_URL=postgres://cngokz:密码@127.0.0.1:5432/cngokzapi
```

不要写 Docker 容器名，因为 API 不在 Docker 网络里。

---

## 5. 本地构建原生包

在本地或构建机：

```bash
cd /path/to/cnapi

TAG=20261105-01 \
VITE_SITE_URL=https://test.cngokz.com \
./scripts/build-native.sh
```

输出：

```text
native-bundles/20261105-01/
├── bin/cngokz-api
├── index.html
├── assets/
├── deploy/cngokz-api.service
├── deploy/nginx-test.cngokz.com.conf
├── .env.production.example
├── install-native.sh
└── migrations/

native-bundles/cngokzapi-native-20261105-01.tar.gz
native-bundles/cngokzapi-native-20261105-01.tar.gz.sha256
```

构建脚本会执行：

```bash
cargo build --release --locked --bin cngokz-api
pnpm --filter @cngokz/frontend build
pnpm --filter @cngokz/frontend build
```

---

## 6. 上传并安装

上传：

```bash
scp native-bundles/cngokzapi-native-20261105-01.tar.gz \
    native-bundles/cngokzapi-native-20261105-01.tar.gz.sha256 \
    root@server:/tmp/
```

服务器：

```bash
cd /tmp
sha256sum -c cngokzapi-native-20261105-01.tar.gz.sha256
mkdir -p cngokzapi-native-20261105-01
tar -xzf cngokzapi-native-20261105-01.tar.gz \
  -C cngokzapi-native-20261105-01
cd cngokzapi-native-20261105-01
```

首次安装：

```bash
sudo APP_DIR=/mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi \
     STATIC_ROOT=/www/sites/test.cngokz.com/index/cnapi \
     bash install-native.sh
```

首次执行会：

- 安装后端二进制
- 安装 unified frontend 静态文件
- 安装 systemd unit
- 生成 `/mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi/.env`
- 安装 Nginx 配置（如果不存在）

脚本会提示先编辑 `.env`。

---

## 7. 配置 `.env`

```bash
nano /mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi/.env
chmod 640 /mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi/.env
chown www-data:www-data /mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi/.env
```

必填：

```env
APP_ENV=production
APP_HOST=127.0.0.1
APP_PORT=28080

DATABASE_URL=postgres://cngokz:密码@127.0.0.1:5432/cngokzapi
REDIS_URL=redis://127.0.0.1:6379/3

# 生成：openssl rand -hex 32
MAINTENANCE_TOKEN=...

KZGO_BASE_URL=https://kzgo.eu
KZGO_SYNC_ENABLED=true
GOKZ_GLOBAL_BASE_URL=https://kztimerglobal.com/api/v2.0
GOKZ_GLOBAL_SYNC_ENABLED=false
```

重新执行安装脚本：

```bash
sudo APP_DIR=/mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi \
     STATIC_ROOT=/www/sites/test.cngokz.com/index/cnapi \
     bash install-native.sh
```

或者手动：

```bash
systemctl daemon-reload
systemctl enable --now cngokz-api
```

---

## 8. systemd

安装后的 unit：

```bash
systemctl status cngokz-api
journalctl -u cngokz-api -f
```

启动 / 重启 / 停止：

```bash
systemctl start cngokz-api
systemctl restart cngokz-api
systemctl stop cngokz-api
```

API 启动时自动执行数据库迁移。

健康检查：

```bash
curl -fsS http://127.0.0.1:28080/health/ready | jq
curl -fsS http://127.0.0.1:28080/api/v1/public/statistics | jq
```

---

## 9. Nginx

配置文件：

```text
deploy/nginx-test.cngokz.com.conf
```

安装：

```bash
sudo cp deploy/nginx-test.cngokz.com.conf \
  /etc/nginx/conf.d/cngokzapi.conf
sudo nginx -t
sudo systemctl reload nginx
```

访问：

```text
https://test.cngokz.com/
https://test.cngokz.com/admin/
```

如果你的证书路径不是 `/www/sites/test.cngokz.com/ssl/`，修改 Nginx 配置里的证书路径。

---

## 10. 更新

本地构建新版本：

```bash
TAG=20261106-01 \
VITE_SITE_URL=https://test.cngokz.com \
./scripts/build-native.sh
```

上传新 tar 包，服务器解压后执行同样的安装脚本：

```bash
sudo APP_DIR=/mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi \
     STATIC_ROOT=/www/sites/test.cngokz.com/index/cnapi \
     bash install-native.sh
```

安装脚本会：

- 备份旧二进制为 `bin/cngokz-api.prev`
- 替换二进制
- 替换 unified frontend 静态文件
- 重启 `cngokz-api`
- 不覆盖已有 `.env`
- 不覆盖已有 Nginx 配置

---

## 10.1 GitHub Actions 手动发布（直接 SSH 上传）

按照 LumiAdmin 的方式：GitHub Actions 构建单一前端和后端二进制，直接通过 SSH/rsync 上传到生产服务器指定目录。

工作流：

```text
.github/workflows/deploy.yml
```

只能手动触发：

```text
Actions
→ Deploy production (manual)
→ Run workflow
→ component: all / frontend / backend
```

允许从 `main` 或 `develop` 分支运行；其他分支会立即失败。

### 统一目标目录

宿主机：

```text
/mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi
```

OpenResty 容器内：

```text
/www/sites/test.cngokz.com/index/cnapi
```

目标目录内容：

```text
cnapi/
├── .env
├── bin/
│   └── cngokz-api
├── index.html
├── assets/
├── robots.txt
└── sitemap.xml
```

前端：

1. 构建 `apps/frontend`；
2. 上传 `dist/` 内容到 `cnapi/` 根目录；
3. 删除旧 `assets/`、`index.html`、`robots.txt`、`sitemap.xml` 以及历史 `public/`、`admin/`；
4. **不会删除** `bin/` 和 `.env`；
5. `chmod -R a+rX`；
6. reload OpenResty。

后端：

1. `cargo build --release --locked --bin cngokz-api`；
2. scp 到 `/tmp`；
3. 备份当前 `bin/cngokz-api` 为 `bin/cngokz-api.prev`；
4. 安装到 `cnapi/bin/cngokz-api`；
5. `systemctl restart cngokz-api`；
6. 轮询 `http://127.0.0.1:28080/health/ready`。

数据库迁移由后端启动时自动执行。

### GitHub Secrets

| Secret | 说明 |
|---|---|
| `DEPLOY_SSH_KEY` | 生产服务器 SSH 私钥 |

创建专用 SSH 密钥：

```bash
ssh-keygen -t ed25519 -a 100 -C "github-actions-cngokz" \
  -f ~/.ssh/cngokz_deploy

ssh-copy-id -i ~/.ssh/cngokz_deploy.pub root@103.120.89.225
```

把私钥内容填入 `DEPLOY_SSH_KEY`：

```bash
cat ~/.ssh/cngokz_deploy
```

### GitHub Variables

| Variable | 示例 |
|---|---|
| `DEPLOY_HOST` | `103.120.89.225` |
| `DEPLOY_USER` | `root` |
| `DEPLOY_PORT` | `22` |

路径、服务名和站点域名已固定在 workflow 中：

```text
ROOT=/mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi
SERVICE_NAME=cngokz-api
VITE_SITE_URL=https://test.cngokz.com
APP_PORT=28080
```

### 服务器前置条件

首次仍需：

```text
cnapi/.env
cnapi/bin/cngokz-api
/etc/systemd/system/cngokz-api.service
OpenResty 站点配置
```

服务器需要：

```bash
rsync
curl
nginx
systemctl
```

如果使用 `root`，不需要 sudo 配置。

## 11. 回滚

查看旧二进制：

```bash
ls -l /mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi/bin/
```

回滚：

```bash
sudo systemctl stop cngokz-api
sudo cp -a /mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi/bin/cngokz-api.prev \
           /mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi/bin/cngokz-api
sudo systemctl start cngokz-api
```

静态文件回滚需要保留上一版本 bundle，重新覆盖 `/www/sites/test.cngokz.com/index/cnapi`。

数据库迁移只向前；回滚代码前确认迁移兼容性，必要时使用数据库备份。

---

## 12. 备份

PostgreSQL：

```bash
docker exec -t 1Panel-postgresql-I65c \
  pg_dump -U postgres -d cngokzapi -Fc \
  > cngokzapi-$(date +%F).dump
```

或宿主机 PostgreSQL：

```bash
pg_dump -U cngokz -d cngokzapi -Fc > cngokzapi-$(date +%F).dump
```

Redis 只存缓存，不要求备份。

---

## 13. Docker 文件清理

仓库中的 API/前端 Docker 部署文件已删除：

```text
docker-compose.yml
docker-compose.dev.yml
.dockerignore
infrastructure/docker/
infrastructure/postgres/
infrastructure/redis/
apps/frontend/server/
apps/frontend/server/
scripts/dev-up.sh
```

生产只使用：

```text
scripts/build-native.sh
scripts/install-native.sh
deploy/cngokz-api.service
deploy/nginx-test.cngokz.com.conf
```

如果你仍然用 Docker 运行 PostgreSQL/Redis，那只是外部基础设施，不属于 CNGOKZAPI 应用部署。

---

## 14. 故障排查

| 现象 | 排查 |
|---|---|
| `cngokz-api` 启动失败 | `journalctl -u cngokz-api -n 100 --no-pager` |
| `health/ready` 返回 503 | 检查 PostgreSQL/Redis 是否监听 `127.0.0.1`，`.env` 是否正确 |
| 页面 404 | 检查 `/www/sites/test.cngokz.com/index/cnapi`、`/www/sites/test.cngokz.com/index/cnapi` 是否存在 |
| `/admin` 空白 | 检查 Nginx `/admin/` 配置和 Vite `base=/admin/` |
| API 502 | 检查 `cngokz-api` 是否监听 `127.0.0.1:28080` |
| 数据库迁移失败 | 查看 journald 日志；不要手工删表 |
| KZGO 同步失败 | 查看 `journalctl` 与 `/api/v1/admin/sources` |
| 前端操作 401/403 | 检查 `MAINTENANCE_TOKEN`；当前前端受保护操作还需要运行时 Token 改造 |

---

## 15. 验证清单

```bash
systemctl status cngokz-api
curl -fsS http://127.0.0.1:28080/health/ready | jq
curl -fsS http://127.0.0.1:28080/api/v1/system/status | jq
curl -fsS http://127.0.0.1:28080/api/v1/public/statistics | jq
curl -fsS https://test.cngokz.com/health | jq
curl -fsS https://test.cngokz.com/api/v1/public/statistics | jq
curl -fsS https://test.cngokz.com/admin/ -o /dev/null -w '%{http_code}\n'
```

管理 API：

```bash
curl -fsS \
  -H "X-Maintenance-Token: $MAINTENANCE_TOKEN" \
  http://127.0.0.1:28080/api/v1/admin/sources | jq
```
