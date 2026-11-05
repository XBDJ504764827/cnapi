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

## 2026年11月05日 22时44分05秒 部署架构切换为 Native：systemd + Nginx，生产不再使用 Docker

**背景**

生产服务器已有 PostgreSQL/Redis 与 Nginx。Docker 构建/部署会重复编译 Rust、拉取基础镜像，更新成本高。参考 LumiAdmin 的部署方式，改为本地构建 release 二进制与静态前端，服务器使用 systemd 运行 API、Nginx 提供域名与静态站点。

**变更内容**

- 新增 `scripts/build-native.sh`：构建 Rust release、管理后台/public 静态文件，生成可上传的 native bundle。
- 新增 `scripts/install-native.sh`：服务器安装二进制、public/admin 静态文件、systemd unit、Nginx 配置，备份旧二进制，保留已有 `.env`。
- 新增 `deploy/cngokz-api.service`：systemd 单元，`www-data` 运行，工作目录 `/mnt/1panel/apps/cnapi`，自动重启与 journald 日志。
- 新增 `deploy/nginx-ban.cngokz.com.conf`：公开门户/管理后台静态站点，`/api`、`/health` 反代 `127.0.0.1:8080`，保留已有证书路径。
- 重写 `.env.production.example` 为 Native 配置：API 只监听 `127.0.0.1:8080`，数据库/Redis 使用宿主机 `127.0.0.1`。
- 重写 `docs/deployment.md` 为 Native 部署步骤；README 生产入口改为 native bundle。
- 删除未采用的 Docker 生产 compose/镜像脚本；CI 移除 Docker 镜像构建 job。
- 新增 `native-bundles/` 到 `.gitignore`。

**涉及文件**

- 新增：`scripts/build-native.sh`、`scripts/install-native.sh`、`deploy/cngokz-api.service`、`deploy/nginx-ban.cngokz.com.conf`
- 修改：`.env.production.example`、`README.md`、`docs/deployment.md`、`.github/workflows/ci.yml`、`.gitignore`、`docs/changelog.md`
- 删除：`docker-compose.yml`、`docker-compose.dev.yml`、`.dockerignore`、`infrastructure/`、`apps/web/server/`、`apps/public/server/`、`scripts/dev-up.sh`
- 删除未提交原型：`docker-compose.prod.yml`、`docker-compose.images.yml`、`scripts/build-images.sh`

**数据库与 API 变化**

- 数据库/API 无变化；迁移仍由 API 启动时执行。
- 清理 Docker 应用部署文件：`docker-compose.yml`、`docker-compose.dev.yml`、`.dockerignore`、`infrastructure/docker/`、`infrastructure/postgres/`、`infrastructure/redis/`、`apps/{web,public}/server/`、`scripts/dev-up.sh`；不再保留应用 Docker 部署路径。

**测试与结果**

- 本地执行 `TAG=20261105-01 VITE_SITE_URL=https://ban.cngokz.com ./scripts/build-native.sh` 成功。
- 产物：`native-bundles/cngokzapi-native-20261105-01.tar.gz`，约 7.8 MB。
- 包含 release 二进制、public/admin 静态文件、systemd/nginx 配置、安装脚本、迁移文件。
- **未执行**：在真实生产服务器上安装 systemd/nginx 后的最终验收；需在目标服务器执行安装脚本并验证。

---

## 2026年11月05日 00时30分00秒 第八阶段：多数据源接入、原始成绩溯源与积分审计

**背景**

需要接入 GOKZ-Global 原始成绩，并由 CNGOKZ 独立计算积分；kzgo.eu 与自有插件作为补充/兜底来源，外部积分不得进入 CNGOKZ 最终积分。首要工作是调查真实接口，而不是假设存在 Webhook/全量 API。

**变更内容**

- 调查确认 GOKZ-Global 即官方 GlobalAPI v2（`kztimerglobal.com/api/v2.0`），有 records/maps/players/player_ranks/jumpstats 等公开 GET 接口，支持 offset/limit 与部分 `created_since` 增量；未发现官方 Webhook 或一次性全量 dump。当前执行环境连接该域名超时，未能在线验证。
- 真实拉取并验证 kzgo.eu：`/api/maps`、`/api/wrs/{mode}`、`/api/servers`；kzgo 自有 API 不提供完整 records，其前端成绩来自 GlobalAPI。
- 新增迁移 `20261105000002_phase8_sources_and_scoring_audit.sql`：`record_sources`、`source_sync_runs`、`raw_records`、`record_source_links`、`map_source_links`、`external_servers`、`records.source_server_name`、积分审计列。
- 新增统一外部成绩入口 `repositories::sources::ingest_external`：原始 payload、幂等、跨来源高置信度匹配、canonical 成绩、WR 条件更新、来源链接。
- 插件提交也写入 raw/provenance 链路，但仍使用同一 `records` canonical 表与同一积分公式。
- 新增 `KZGO Adapter`：真实同步地图、Course、Mode、Tier、当前 WR、服务器快照；公开服务器 API 在外部快照新鲜时优先使用 kzgo，否则回退自有心跳。
- 新增 `GOKZ-Global Adapter`（默认关闭）：`records/top/recent` 增量、Steam2→SteamID64、时间解析、来源审计；因网络不可达未在线验证。
- 新增数据源管理 API 与 `/admin/sources` 页面：状态、启停、手动同步、运行记录、cursor、错误。
- 积分为 `player_filter_points` 增加 `scoring_version/base_points/rank_factor/dist_points`，公式保持第六阶段不变。
- 更新 OpenAPI、生成类型、README、开发/数据库文档，并新增 `docs/phase8-multi-source-ingestion.md`。

**涉及文件**

- 新增：`apps/api/migrations/20261105000002_phase8_sources_and_scoring_audit.sql`、`apps/api/src/{dto,handlers,repositories,services}/source*.rs`、`apps/api/tests/{source_ingestion,source_live}.rs`
- 修改：`apps/api/src/config/mod.rs`、`apps/api/src/main.rs`、`apps/api/src/routes/*`、`apps/api/src/services/scoring/recalc.rs`、`apps/api/src/repositories/game_servers.rs`、`apps/api/src/services/public/mod.rs`、`apps/api/src/repositories/public/records.rs`、`apps/api/tests/schema_integration.rs`
- 新增：`apps/web/src/pages/sources-page.tsx`、`apps/web/src/services/sources-service.ts`
- 修改：`apps/web/src/router/index.tsx`、`apps/web/src/components/layout/nav-config.ts`、`.env.example`
- 修改：`docs/openapi.json`、`packages/shared-types/src/generated/api-schema.d.ts`、`README.md`、`docs/{development,database/overview,changelog}.md`
- 新增：`docs/phase8-multi-source-ingestion.md`
- 依赖：新增 `reqwest`（rustls/json）

**数据库与 API 变化**

- 数据库：新增 6 张表、2 个 `records`/积分审计列组；全部为增量迁移，不删除历史成绩。
- API：新增 `/api/v1/admin/sources*`、成绩来源审计、外部服务器快照；公开服务器在外部源新鲜时优先使用 kzgo 快照。

**测试与结果**

- `cargo test --all-targets`：79 个通过，0 失败；1 个 live test 默认忽略。
- 真实 kzgo live 测试：990 maps、4892 WRs seen、4884 canonical records、2 servers；8 条因地图/Course 不匹配跳过。
- 跨来源测试：KZGO + GOKZ-Global 同成绩 → 1 canonical + 2 raw + 2 links；超过 600 秒窗口的相同成绩保留为独立记录。
- 既有第六阶段积分、公开门户、服务器接入测试全部通过。
- **未验证**：GOKZ-Global 在线请求（当前环境连接超时）、完整历史回填、真实生产网络/授权/限流、GitHub Actions runner。

---

## 2026年11月04日 23时45分00秒 调整第七阶段方向：网站同步 KZGO 优先，插件延期

**背景**

产品方向确认：网站数据应优先同步 kzgo.eu（其成绩历史实际来自 KZTimerGlobal GlobalAPI），自有 CS:GO 插件仅在外部数据不可获取时作为兜底上传。当前单文件 SourceMod 原型目录不符合 `kzcharm/gokz-top-plugins` 规范，暂停继续开发。

**变更内容**

- 从工作区移除未被接受的单文件插件原型与 Replay uploader，不再作为当前交付物。
- 保留后端 `/api/v1/game-servers/*` 接入 API 与数据库模型，作为未来插件兜底链路。
- 记录 KZGO 数据源拆解：`kzgo.eu/api/{maps,servers,wrs,completions,steam}` + `kztimerglobal.com/api/v2.0/*`。
- 更新 README 与 `docs/phase7-game-server-integration.md`，明确插件延期及未来目录规范。
- KZGO 同步 worker、公开 API 源优先级与后台剩余页面尚未实现，列为下一步网站工作。

**涉及文件**

- 修改：`README.md`、`docs/phase7-game-server-integration.md`、`docs/changelog.md`
- 删除（未提交的工作区原型）：`addons/`、`tools/`

**数据库与 API 变化**

- 数据库/API 无新增变化；第七阶段既有服务器接入 API 保留。
- 网站公开数据源尚未切换到 KZGO，当前不能宣称已完成 KZGO 同步。

**测试与结果**

- 本次为方向调整与文档/原型清理，未新增功能测试。
- 既有后端 74 个测试之前已通过；KZGO 同步与网站改造尚未执行。

---

## 2026年11月04日 23时12分05秒 第七阶段：CS:GO KZ 游戏服务器接入与数据同步

**背景**

在第六阶段积分系统与公开门户之上，需要让真实 CS:GO（Source 1）KZ 服务器通过 LinuxGSM + MetaMod:Source + SourceMod + GOKZ 自动上报心跳、在线玩家、成绩和 Replay。审查确认现有 `servers`、`records`、`replays`、`filter_recalc_tasks` 可复用，缺口是服务器认证接口、CS:GO 元数据、在线会话、成绩幂等键、管理后台与 SourceMod 插件。

**变更内容**

- 新增迁移 `20261105000001_phase7_game_server_sync.sql`：扩展 `servers`（game/region/server_type/query_port/max_players/online_players/current_mode_id/game_version/is_enabled），新增 `server_heartbeats`、`server_player_sessions`、`server_sync_logs`、`server_sync_tasks`，并为 `records` 增加 `submission_key` + `(server_id, submission_key)` 唯一索引；全部为增量变更。
- 新增服务器侧 API：`/api/v1/game-servers/{heartbeat,players/connect,players/disconnect,players/sync,records,records/status,replays,config/sync,maps/sync}`，使用每服务器独立 Token（只存 SHA-256 哈希），支持轮换/撤销/禁用。
- 成绩提交按 `server_id + record_identifier` 幂等；自动写 `records`、`record_history`、条件更新 `course_world_records`，由既有 DB trigger 排入对应 Filter 重算；响应区分 duplicate、PB、WR、积分排队、Replay 状态。
- 新增管理 API：服务器列表/创建/详情/编辑/启停/Token 轮换与撤销/同步任务重试。
- 新增公开 API `/api/v1/public/players/online`；公开服务器列表改为展示真实人数、最大人数与模式。
- 管理后台服务器页从空壳改为真实列表与创建表单；详情页加入概览、在线玩家、成绩、Replay、心跳、同步日志/任务/重试。
- 公开门户服务器页加入当前在线玩家区块，并移除“人数/模式未入库”的旧提示。
- 新增 SourceMod 插件 `addons/sourcemod/scripting/cngokz_sync.sp`：使用 GOKZ `GOKZ_OnTimerEnd_Post`、`GOKZ_RP_OnTimerEnd_Post`、`GOKZ_GetCoreOption`，SteamWorks 异步 HTTP，本地成绩队列（上限 256、重启恢复、指数退避），Replay manifest 与配置文件。
- 新增独立 Replay uploader `tools/cngokz-replay-uploader.py`（rclone 上传 + SHA-256 + API 注册 + 失败保留）。
- 更新 OpenAPI、前端生成类型、README、开发/数据库文档，并新增 `docs/phase7-game-server-integration.md`。

**涉及文件**

- 新增：`apps/api/migrations/20261105000001_phase7_game_server_sync.sql`、`apps/api/src/{dto,handlers,repositories,services}/game_server*.rs`、`apps/api/tests/game_server_sync.rs`
- 修改：`apps/api/src/routes/*`、`apps/api/src/dto/public/servers.rs`、`apps/api/src/repositories/public/servers.rs`、`apps/api/src/services/public/mod.rs`、`apps/api/src/handlers/public.rs`、相关 `mod.rs`、`apps/api/tests/{public_api,schema_integration}.rs`
- 新增：`addons/sourcemod/scripting/cngokz_sync.sp`、`addons/sourcemod/configs/cngokz_sync.cfg`、`tools/cngokz-replay-uploader.py`
- 修改：`apps/web/src/pages/{servers-page,server-detail-page}.tsx`、`apps/web/src/services/game-servers-service.ts`、`apps/web/.env.example`
- 修改：`apps/public/src/pages/servers-page.tsx`、`apps/public/src/services/public-service.ts`
- 修改：`packages/api-client/src/{client,index}.ts`、`packages/shared-types/src/api-contract.ts`、`docs/openapi.json`、`packages/shared-types/src/generated/api-schema.d.ts`
- 修改：`README.md`、`docs/{development,database/overview,changelog}.md`
- 新增：`docs/phase7-game-server-integration.md`

**数据库与 API 变化**

- 数据库：新增 4 张服务器接入表与 `records.submission_key`；不删除已有成绩/服务器，不重置数据库。
- API：新增服务器 Token 认证接口与管理接口；公开新增在线玩家接口；公开服务器字段 `players/max_players/mode/region/server_type/game` 改为真实数据。积分公式与 Replay 保留规则未修改。

**测试与结果**

- `DATABASE_URL=postgres://cnapi@127.0.0.1:55432/cngokzapi_test REDIS_URL=redis://127.0.0.1:56379 cargo test --all-targets`：**74 个通过**，0 失败（含新增 3 个游戏服务器接入端到端用例）。
- `cargo fmt --all --check`、`cargo clippy --all-targets --all-features -- -D warnings`：通过。
- `pnpm -r lint`、`pnpm -r typecheck`、`pnpm -r test`：通过（管理后台 21 个、公开门户 17 个）。
- `pnpm --filter @cngokz/web build`、`pnpm --filter @cngokz/public build`：通过。
- SourceMod 插件使用 SourceMod 1.11 `spcomp` + GOKZ include 编译通过（0 errors）。
- **未执行**：真实 CS:GO Dedicated Server / LinuxGSM / GOKZ 在线联调；真实 R2/S3 Replay 上传；GitHub Actions 真实 runner 复跑。上述内容在文档中标记为待验收，不描述为真实环境通过。

---

## 2026年11月04日 22时43分32秒 修复 GitHub Actions CI：schema 集成测试不再依赖预先迁移的数据库

**背景**

推送后 GitHub Actions `api` job 的 `cargo test --all-targets` 在 `schema_integration` 用例失败，11 个用例全部报错：`phase 2 migrations are not applied; start the API once or run sqlx migrate`。

CI 的 PostgreSQL service 启动时只有空库，`cargo test` 之前没有启动 API 或执行迁移；而 `tests/schema_integration.rs` 直接连接 `DATABASE_URL` 并断言迁移已存在。其他真实依赖测试（`public_api`、`points_system`、`service_integration`）都会临时建库并执行 `sqlx::migrate!`，只有该文件仍依赖外部预置 schema。

**变更内容**

- `tests/schema_integration.rs` 的 `pool()` 改为临时创建数据库并执行 `sqlx::migrate!("./migrations")`，与其余集成测试保持一致。
- 新增 `TestDatabase` 守卫：为每个用例提供独立数据库；用例结束或断言失败回滚时，在独立线程中通过 `DROP DATABASE ... WITH (FORCE)` 清理，避免跨 Tokio runtime 使用数据库连接导致挂起。
- 缺少 `DATABASE_URL` 时仍按原约定跳过并打印原因。
- 更新 `docs/development.md` 中 schema 测试说明。

**涉及文件**

- 修改：`apps/api/tests/schema_integration.rs`、`docs/development.md`、`docs/changelog.md`

**数据库与 API 变化**

- 数据库：仅测试行为变化，用例临时建库/删库；生产 schema 无变化。
- API：无。

**测试与结果**

- `DATABASE_URL=postgres://cnapi@127.0.0.1:55432/cngokzapi_test cargo test --test schema_integration`：11/11 通过，临时库全部清理。
- `DATABASE_URL=... REDIS_URL=redis://127.0.0.1:56379 cargo test --all-targets`（与 CI 同命令）：**71 个通过**，0 失败。
- `cargo fmt --all`：通过。
- **未执行**：在 GitHub Actions 真实 runner 上复跑（本次为本地等价 PostgreSQL 18 + Redis 8 环境验证）。

---

## 2026年11月04日 22时25分18秒 修复 GitHub Actions CI：缓存维护端点在 Redis 不可用时返回 503

**背景**

推送 `develop` 后 GitHub Actions `api` job 在 `cargo test --all-targets` 的 `phase5_cache_rate_limit_and_scope` 用例失败：断言 `assert_eq!(init.status(), StatusCode::OK)` 得到 503。原因有两点：
1. `apps/api/tests/public_api.rs` 把 Redis 地址硬编码为本地调试端口 `127.0.0.1:56379`，而 CI service 是 `localhost:6379`（workflow 提供的 `REDIS_URL`）；
2. `POST /api/v1/public/maintenance/cache/invalidate` 把 Redis `INCR` 失败直接映射为 `AppError::Redis`（503），但缓存是 best-effort，Redis 不可用时不应让数据变更维护请求失败。

**变更内容**

- `services/public/cache.rs::invalidate_all` 改为返回 `bool`：Redis 成功推进命名空间版本返回 `true`，失败记录 warn 并返回 `false`，不再产生 `AppError::Redis`。
- `services/public/mod.rs::invalidate_cache` 同步返回 `bool`（best-effort）。
- `handlers/public.rs::invalidate_cache` 返回 200 并显式携带 `cache_invalidated` / `warmed` 标志；Redis 不可用时不预热、标记降级，PostgreSQL 仍是真值来源。
- `tests/public_api.rs` 的 phase5 用例改为读取 workflow 注入的 `REDIS_URL`，不再写死本地端口；版本推进断言仅在 `cache_invalidated=true` 时执行；新增 `cache_invalidation_degrades_when_redis_is_unavailable` 回归测试（无 Redis 时必须 200 + `cache_invalidated=false`）。
- `.github/workflows/ci.yml` 的 `api` job 补齐 pnpm/action-setup、setup-node 和 `pnpm install --frozen-lockfile`；此前该 job 的 OpenAPI 校验步骤调用 `pnpm` 却没有 Node 工具链，会在测试通过后再次失败。
- 更新 OpenAPI 描述、部署文档与公开门户 API 文档；重新生成 OpenAPI 与 TypeScript 类型。

**涉及文件**

- 修改：`apps/api/src/services/public/cache.rs`、`apps/api/src/services/public/mod.rs`、`apps/api/src/handlers/public.rs`、`apps/api/tests/public_api.rs`
- 修改：`.github/workflows/ci.yml`、`docs/deployment.md`、`docs/public-portal/api-requirements.md`、`docs/openapi.json`、`packages/shared-types/src/generated/api-schema.d.ts`、`docs/changelog.md`

**数据库与 API 变化**

- 数据库：无。
- API：`POST /api/v1/public/maintenance/cache/invalidate` 响应新增 `cache_invalidated`、`warmed`；`invalidated` 保留兼容别名。Redis 不可用时由 503 改为 200 + `cache_invalidated=false`。

**测试与结果**

- `REDIS_URL=redis://127.0.0.1:1`（模拟 Redis 不可用）运行 `cargo test --test public_api`：3/3 通过。
- `REDIS_URL=redis://127.0.0.1:56379`（Redis 可用）运行同一测试：3/3 通过。
- `cargo test --all-targets`（与 CI 同命令）：**71 个通过**，0 失败。
- `cargo fmt --check`、`cargo clippy -D warnings`、`cargo test`、OpenAPI 同步检查：通过。
- 前端 `pnpm -r lint/typecheck/test/build`：通过（38 个单测、两个应用构建成功）。
- **未执行**：在 GitHub Actions 真实 runner 上复跑（本次为本地等价环境验证）；Docker 构建（环境无 docker）。

## 2026年11月04日 21时58分57秒 第六阶段：玩家积分与排行榜系统

**背景**

按需求先完成项目审查（见 `docs/phase6-audit.md`），输出实际数据结构与现有规则；仓库中未发现用户所说的历史积分规范文件，因此以本轮需求正文中的公式为最终准则。在不重建项目、不删除成绩、不修改既有成绩判定规则的前提下，实现 Filter 积分计算、持久化、重算、三模式独立总分、排行榜与前后端展示闭环。

**变更内容**

- 审查结论：PRO ⇔ `teleports=0` 由既有 CHECK 强制；有效成绩 = `approved` 且未软删除；等级原先按 `(course,mode)` 且 NUB/PRO 两列；`player_mode_stats.points/rating` 为未使用的预留列；Style 当前只有 `normal`；WR 指针表已存在但未关联积分。
- 新增迁移 `20261104000002_points_system.sql`：`scoring_modes`（KZT→kztimer / SKZ→simple / VNL→vanilla，classic 不参与）、`course_mode_tiers`（权威 tier[map][course][mode] + 首通状态，tier 1–8）、`scoring_filters`、`player_filter_points`、`player_mode_points`、`filter_recalc_tasks`、`filter_recalc_runs`、`filter_first_completions`，以及组合索引/partial 索引；未删除表或数据。
- 新增纯 Rust 模块 `services/scoring/{model,formula}.rs`：基础分表、排名系数、分布系数、最终积分；WR 恒为 1000；T9 用 T8 计算；非法输入拒绝；N 不参与第一版公式。
- 新增 `services/scoring/recalc.rs`：SQL CTE 聚合全部玩家 PB / WR / rank / pb_count，事务内批量替换积分并重建单模式总分；`FOR UPDATE SKIP LOCKED` 任务领取；按模式缓存失效；幂等；初始化端点不回填历史 T9。
- DB 触发器：成绩变更只标记受影响 Filter 并入队；VNL 未完成课程的首个有效成绩写 T9 历史标签并立即置 T8；等级变更触发同模式 Filter 重算。KZT/SKZ 不受 VNL 首通影响。
- 新增 API：玩家积分、Filter 积分明细、KZT/SKZ/VNL 积分榜、Filter 榜、积分状态、等级管理、重算任务/单 Filter/单模式/历史初始化；管理端点由 `X-Maintenance-Token` 保护。OpenAPI 重新生成。
- 前端：公开门户新增 `/points`、`/players/:steamid/points`，玩家详情加入三模式积分卡，页头/页脚加入口；管理后台新增积分概览、地图等级、重算管理、玩家积分明细页；API Client/共享类型新增 points 方法。
- 文档：`docs/points-system.md`、`docs/phase6-audit.md`，README 阶段状态同步；移除“积分算法尚未定义”的旧文案。

**涉及文件**

- 新增：`apps/api/migrations/20261104000002_points_system.sql`、`apps/api/src/services/scoring/{mod,model,formula,recalc}.rs`、`apps/api/src/repositories/points.rs`、`apps/api/src/services/points.rs`、`apps/api/src/handlers/points.rs`、`apps/api/src/dto/points.rs`、`apps/api/tests/points_system.rs`
- 修改：`apps/api/src/{main,services/mod,repositories/mod,dto/mod,handlers/mod,routes/mod,routes/openapi,services/public/cache}.rs`、`apps/api/src/dto/public/*` 部分类型
- 修改：`packages/api-client/src/{client,index}.ts`、`packages/shared-types/src/{api-contract,generated/api-schema.d}.ts`、`docs/openapi.json`
- 新增公开门户：`src/services/points-service.ts`、`src/components/points/*`、`src/pages/{points-leaderboards,player-points}-page.tsx`、`src/pages/points-pages.test.tsx`、`src/test/fixtures.ts` 扩展
- 新增管理后台：`src/services/points-service.ts`、`src/pages/{points,points-tiers,points-recalc,player-points}-page.tsx`，更新 nav-config/router/format/vite-env
- 修改：`README.md`、`docs/{changelog,points-system,phase6-audit}.md`

**数据库与 API 变化**

- 数据库：新增 8 张积分相关表、触发器与索引；`records`、`course_tiers`、`course_world_records`、`player_mode_stats` 均未删除或改结构；历史成绩初始化不伪造 T9 标签。
- API：新增 11 个积分相关端点；无现有公开接口被移除；管理端点需要维护令牌，未配置时 403。

**测试与结果**

- 后端：`cargo fmt --check`、`cargo clippy -D warnings` 通过；`cargo test` **70 个通过**（43 单元 + 10 HTTP + 1 points_system 集成 + 2 公开 API + 11 schema + 3 集成）；全新库 11 个迁移执行通过；算法测试覆盖 T1–T9、NUB/PRO、rank 边界、分布、非法输入、WR=1000。
- 集成测试 `points_system.rs`：PB/WR/rank、T9 首通、KZT/SKZ/VNL 隔离、总分=PRO+NUB、增量与全量重算一致、原 WR 失去 1000 分后按当前规则重算。
- 前端：`pnpm -r lint/typecheck/format:check` 通过；**38 个单测通过**（后台 21 + 门户 17，新增 3 个积分页面测试）；两个应用构建成功。
- 运行时：11 个积分/管理端点 200（无令牌管理接口 401）；真实 fixture 经触发器入队、worker 重算 6 个 Filter，状态 pending=0/failed=0；VNL 全量重算后 5 个 Filter 成功；积分榜/Filter 明细/玩家总分均返回真实 PostgreSQL 数据。
- Playwright 回归：公开门户 36/36、管理后台 38/38、统一入口 13/13；0 控制台错误（在第六阶段中间版本执行）。
- **未执行**：Docker 镜像构建/Compose 启动（环境无 docker）；完整管理员认证（仍为维护令牌）；生产规模（百万级 PB）性能压测；历史成绩 T9 回溯（需求未确认）。

## 2026年11月04日 20时59分12秒 第五阶段：全站联调、数据准确性、缓存/限流/性能与生产化

**背景**

按第五阶段要求先完成项目审查（结果见 `docs/phase5-audit.md`），再按最小修改方案修复：统一玩家统计范围、完善 Redis 缓存与限流、补充性能索引与执行计划、修正部署参数与静态服务器路径、补充测试与文档。不重建项目、不改技术栈、不改已有数据库表结构。

**变更内容**

- 新增审查文档 `docs/phase5-audit.md`，记录实际目录（管理后台为 `apps/web`、后端为 `apps/api`，无 `apps/admin`/`crates/`）、共享类型、模拟数据检查、Docker 与 TODO 审查结论。
- 数据准确性：`PlayerDetailResponse` 新增 `overall(scoped=false)`、`scoped(scoped=true)`、`scope(mode/style/time_type/completion_definition)`；页面明确区分全站汇总与当前范围；`scoped_overall` 统一按 mode/time_type/style 计算，`tier_progress` 支持 style 过滤；完成定义统一为主线课程（stage 0）approved 成绩，Bonus 单独统计。
- 积分/Rating：仍未定义算法，继续返回 `null` 并标注 `points_available=false`；**未擅自实现公式**。
- API 稳定性：新增 `rate_limited` 错误码与 429 + `Retry-After`；`middleware/rate_limit.rs` 实现每 IP 固定窗口限流（`PUBLIC_RATE_LIMIT_PER_MINUTE`，默认 120），排除健康检查与维护端点；维护端点使用 `X-Maintenance-Token`（`MAINTENANCE_TOKEN`，未配置则 403）。
- Redis：`services/public/cache.rs` 改为固定长度哈希 Key、TTL 抖动、per-key async 单飞、启动时 `SET NX` 初始化版本；新增 `POST /maintenance/cache/invalidate`（INCR + 首页统计预热）与 `GET /maintenance/cache/version`；`RedisClient` 增加 `set_nx`。
- 服务器状态：`ServerSummary` 新增 `status(online/offline/expired/unknown)` 与 `heartbeat_age_seconds`；前端展示四态；玩家数/最大人数/模式仍为 `null`。
- 性能：新增迁移 `20261104000001_phase5_indexes.sql`（pg_trgm、partial 索引：最近成绩、玩家/地图历史、进度查询、课程排名、WR 达成时间）；在 2 万条成绩临时库上用 `EXPLAIN (ANALYZE, BUFFERS)` 验证命中（0.09–0.30 ms 量级）。
- 部署：修正 compose `VITE_SITE_URL` 默认 5173；`.env.example` 增加限流/维护令牌；CI 覆盖 `develop` 与 compose 校验；部署文档补充缓存失效、备份恢复后的失效流程。
- 安全：两个静态服务器修复 decodeURIComponent 异常处理与路径穿越校验；日志/配置脱敏检查；无模拟数据写入。
- 测试：新增 `phase5_cache_rate_limit_and_scope` 集成测试（限流 429、维护令牌、缓存版本推进、统计范围、服务器四态），`handlers::public` 维护令牌单测，缓存 Key/TTL 抖动单测；更新 `public_api.rs` 断言。

**涉及文件**

- 新增：`docs/phase5-audit.md`、`apps/api/migrations/20261104000001_phase5_indexes.sql`、`apps/api/src/middleware/rate_limit.rs`
- 修改：`apps/api/src/{config,dto/response,errors,handlers/public,main,middleware/mod,routes/mod,routes/openapi,services/public/cache,services/public/mod,state/connections,state/redis_client,repositories/public/players}.rs`、`apps/api/src/dto/public/{players,servers}.rs`、`apps/api/tests/public_api.rs`
- 修改：`apps/public/src/pages/{player-detail,servers}.tsx`、`components/players/overall-stats.tsx`、`apps/public/server/static-server.mjs`、`apps/web/server/static-server.mjs`
- 修改：`.env.example`、`docker-compose.yml`、`.github/workflows/ci.yml`、`docs/{deployment,changelog,public-portal/*}.md`、`docs/openapi.json`、`packages/shared-types/src/generated/api-schema.d.ts`

**数据库与 API 变化**

- 数据库：仅新增索引与 `pg_trgm` 扩展，未修改表结构或业务数据。
- API：`/api/v1/public/players/{steamid}`、`/servers` 响应增加字段（向后兼容新增字段）；新增 2 个维护端点；OpenAPI 28 paths / 101 schemas。

**测试与结果**

- 后端：`cargo fmt --check`、`cargo clippy -D warnings`、`cargo test` **61 个通过**（35 单元 + 10 HTTP + 2 公开 API + 11 schema + 3 集成）；`cargo build --release` 通过。
- 前端：`pnpm -r lint/typecheck/format:check` 通过；**35 个单测通过**（管理后台 21 + 公开门户 14，新增 9 个页面级渲染/API 成功/失败/空数据/分页/搜索/玩家详情/地图详情/比较测试）；两个应用构建成功。
- 端到端：公开门户 36/36、管理后台 38/38、统一入口 13/13、排行榜 8/8；0 控制台错误。真实 fixture 验证玩家统计/完成地图/WR/地图榜/排行榜一致；缓存失效后统计随数据库写入/删除同步变化。
- 限流：集成测试与运行时 429 + `Retry-After` 均验证通过；维护端点无令牌 401。
- 路径穿越：编码/非编码 `..` 请求返回 403 或 SPA 回退，不读取 dist 外文件。
- **未执行**：Docker 镜像构建与 `docker compose up -d --build`（环境无 docker 命令）；多副本限流与 Redis 故障切换演练；生产认证/管理写接口（功能尚不存在）。

## 2026年11月04日 19时43分24秒 全球排行榜页面视觉与交互重构

**背景**

公开门户的 `/leaderboards` 原使用“大段说明 Callout + 两行 Tab + 宽数据表（含积分/国家/最近活动等列）”的后台式布局，信息层级弱、视觉拥挤，且积分列在算法未定义时长期显示 `—`。本次改为专门的排名控制台式布局，不修改 API 或数据库。

**变更内容**

- 新增 `components/leaderboards/`：`ModeSegment`（紧凑分段模式切换）、`SummaryStrip`（当前榜单事实条）、`RankRow`/`RankRowSkeleton`（排名行卡片化）。
- 页面头部保留标题与事实性说明，删除大块 `Callout`；把模式、TP/PRO、每页条数、重置统一收进一个“筛选控制台”卡片。
- 模式切换由下划线 Tabs 改为分段按钮；TP/PRO 改为紧凑按钮组；新增每页条数、重置与“排名说明”Tooltip。
- 榜单事实条显示当前模式/类型、上榜玩家、最高完成、页面 WR/成绩、最近活动，全部来自 API；空数据时为 0/`—`，不伪造。
- 排名列表从通用 `DataTable` 改为排名行：
  - 桌面端：名次徽块 + 玩家/头像 + 完成地图 + WR + 成绩/最近活动 + 详情箭头；
  - 前三名使用克制的金/蓝主色徽块与 TOP 标签，不使用夸张奖牌或霓虹；
  - 移动端：名次 + 玩家 + 关键指标（完成/WR）内联显示，其余信息不挤压；
  - 加载使用排名行骨架，空/错误状态仍完整显示。
- 分页面板保留在榜单卡片底部；所有筛选写入 URL，刷新/后退保持。
- `PublicLayout` 增加 `TooltipProvider`，修复页面首次使用 Tooltip 时 Radix 抛错导致路由错误页的问题。
- 更新 docs/public-portal 页面与设计文档中排行榜相关描述。

**涉及文件**

- 新增：`apps/public/src/components/leaderboards/{mode-segment,summary-strip,rank-row}.tsx`
- 重写：`apps/public/src/pages/leaderboards-page.tsx`
- 修改：`apps/public/src/layouts/public-layout.tsx`
- 修改：`docs/public-portal/pages.md`、`docs/public-portal/design-system.md`、`docs/changelog.md`

**数据库与 API 变化**

无。继续消费 `/api/v1/public/leaderboards` 与 `/api/v1/public/modes`，统计口径不变。

**测试与结果**

- `pnpm --filter @cngokz/public typecheck`、`lint`、`test`（5 个）、`build`：通过。
- Playwright **8/8** 通过、0 控制台错误：标题、模式分段可见、模式切换、TP/PRO 切换、重置、Tooltip、1440/375 无横向溢出。
- 已生成 1440 与 375 截图人工检查：桌面信息密度与移动端单列布局正常。
- **未执行**：Docker 构建/Compose（环境无 docker）；真实数据下的前三名样式仅由 fixture 验证过（当前运行库为空，显示空状态）。

## 2026年11月04日 00时15分55秒 首页改版：移除营销式 Hero 与口号，改为数据优先布局

**背景**

公开门户首页使用了大段顶部留白（`pt-12 sm:pt-16`）、径向光晕 Hero、`CS:GO / CS2 · KZ DATA` 眉标与“KZ 数据，一目了然。”口号，视觉上偏营销模板，且与数据平台定位不符。需要改为更克制、数据优先的入口，并保留搜索与快捷入口。

**变更内容**

- 删除首页 `hero-panel` 整块（含径向渐变、`border-b` 分隔、超大上边距、眉标与口号）。
- 新的页面头部直接复用站点页头下方的常规内容区（`px-4 py-8 sm:px-6 lg:px-8`）：
  - 标题改为事实性的“数据概览”；
  - 说明改为“全球 KZ 排行榜、玩家进度、地图完成情况与最近上传成绩。数据来自公开数据库。”；
  - 右侧为玩家/地图搜索框与 `/`、`⌘K` 提示，搜索结果使用绝对定位下拉面板，不再撑开页面；
  - 操作按钮改为“排行榜 / 玩家目录 / 地图列表”。
- 全站统计改为移动端 2 列、桌面 4 列；在线服务器卡片提示改为“心跳 2 分钟内”。
- `RecordTable` 的空/错误/加载三态重构：空状态与错误态移到横向滚动容器之外，修复移动端“最近成绩”空状态被裁切的问题（成绩行仍可横向滚动）。
- 删除不再使用的 `hero-panel`、`mask-fade-bottom` CSS；更新公开门户设计系统/页面文档中关于 Hero 的描述。

**涉及文件**

- 修改：`apps/public/src/pages/home-page.tsx`
- 修改：`apps/public/src/components/records/record-table.tsx`
- 修改：`apps/public/src/styles/index.css`
- 修改：`docs/public-portal/design-system.md`、`docs/public-portal/pages.md`、`docs/changelog.md`

**数据库与 API 变化**

无。仅前端布局与空状态渲染调整。

**测试与结果**

- `pnpm -r lint`、`pnpm -r typecheck`、`pnpm -r format:check`：通过。
- `pnpm -r test`：管理后台 21 + 公开门户 5 = **26 个通过**。
- `pnpm -r build`：两个应用构建成功，sitemap/robots 正常生成。
- Playwright 最终检查 **9/9** 通过、0 控制台错误：首页标题为“数据概览”、页面不再包含“一目了然”、1440/375 无横向溢出、首页搜索面板可打开、移动端统计 2 列、成绩空状态完整可见、`/admin` 入口正常。
- **未执行**：Docker 构建/Compose（环境无 docker）。

## 2026年11月04日 00时07分26秒 统一入口：公开门户 `/` + 管理后台 `/admin`（单端口）

**背景**

第四阶段将公开门户与管理后台拆分为两个对外端口（5173/5174）。实际使用中希望**同一 IP、同一端口**：公开门户作为默认首页，管理后台通过后缀访问。本次调整统一入口，避免用户记忆/开放两个端口，同时保留两个独立应用与独立构建。

**变更内容**

- 入口约定：`http://<host>:5173/` 公开门户；`http://<host>:5173/admin` 管理后台；API/文档仍为同源 `/api`、`/health`、`/swagger-ui`、`/api-docs`。
- **管理后台**
  - Vite `base` 改为 `/admin/`；React Router 使用 `basename: '/admin'`，因此所有 `Link`/`Navigate`/面包屑自动带 `/admin` 前缀。
  - 开发端口改为 `ADMIN_PORT`（默认 5174），仅作为 public 开发服务器的内部代理目标，不对外暴露。
  - 管理后台静态服务器支持 `/admin` 前缀：去掉前缀后提供 `dist` 资源；`/admin` 308 重定向到 `/admin/`；SPA 回退正常。
  - 用户菜单新增“公开门户”链接（`href="/"`，位于 basename 之外）。
- **公开门户**
  - 作为默认站点运行在 5173（`PORTAL_PORT`），新增 `/admin` 开发代理（含 HMR WebSocket，`ws: true`）与生产代理。
  - 生产静态服务器升级为**唯一对外入口**：`/` 提供 public 构建，`/admin/*` 代理到内部 web 容器，`/api` 等仍代理到 API；`/healthz` 供容器健康检查。
  - 页脚新增“管理后台”链接。
- **开发与部署**
  - 根 `pnpm dev` 并行启动 web（5174，base `/admin/`）与 public（5173）；保留 `dev:admin` / `dev:public`。
  - `docker-compose.yml`：web 容器只 `expose 5173`，public 容器发布 `${PORTAL_PORT:-5173}:5173`，设置 `ADMIN_UPSTREAM=http://web:5173`，并依赖 web 健康。
  - `.env.example`：新增 `PORTAL_PORT`、`ADMIN_PORT`、`VITE_SITE_URL=http://localhost:5173`；移除对外 `WEB_PORT`/`PUBLIC_PORT` 语义；CORS 默认仅 5173。
  - `public.Dockerfile` 默认端口改为 5173，新增 `ADMIN_UPSTREAM`；sitemap/robots 默认站点 URL 改为 `http://localhost:5173`。
  - 文档更新：README、architecture、development、deployment、public-portal/architecture；新增 `apps/public/.env.example` 并更新 `apps/web/.env.example`（ADMIN_PORT）。
- **更正**：`2026年11月03日 23时54分18秒 第四阶段…` 记录中的“公开门户 5174 / 管理后台 5173 独立端口”安排已被本记录取代；历史条目保留原样。旧后台路径（如 `/dashboard`、`/players`）现在属于公开门户，后台需访问 `/admin/...`。

**涉及文件**

- 修改：`apps/web/vite.config.ts`、`apps/web/src/router/index.tsx`、`apps/web/server/static-server.mjs`、`apps/web/src/components/layout/user-menu.tsx`、`apps/web/.env.example`
- 修改：`apps/public/vite.config.ts`、`apps/public/server/static-server.mjs`、`apps/public/scripts/generate-seo.mjs`、`apps/public/src/components/layout/site-footer.tsx`
- 新增：`apps/public/.env.example`
- 修改：`package.json`（`dev` 并行启动）、`.env.example`、`docker-compose.yml`、`infrastructure/docker/public.Dockerfile`
- 修改：`README.md`、`docs/architecture.md`、`docs/development.md`、`docs/deployment.md`、`docs/public-portal/architecture.md`、`docs/changelog.md`

**数据库与 API 变化**

无。未修改数据库、迁移、公开 API 或管理 API；仅调整前端 base/路由、开发端口与容器入口。

**测试与结果**

- `pnpm -r lint`、`pnpm -r typecheck`、`pnpm -r test`（管理后台 21 + 门户 5 = 26 个）、`pnpm -r build` 全部通过。
- 开发入口（Playwright，1440/375）13/13 通过：`/` 公开门户、`/admin/` 自动跳转 `/admin/dashboard`、后台导航保持 `/admin` 前缀、深链接、公开/后台主题互不影响（独立 localStorage key）、后台移动抽屉、0 控制台错误（含 `/admin` HMR WebSocket 代理）。
- 生产静态入口验证：构建后在 4180（public 网关）+ 4181（内部 web）运行，`/`、`/players`、`/admin/`、`/admin/dashboard`、`/admin/players`、`/api/v1/public/statistics`、`/sitemap.xml`、`/robots.txt` 均 200；`/admin` 308 → `/admin/`；`/admin/assets/*.js` 与 `/assets/*.js` 均 200；Playwright 打开 `/admin/` 正常渲染仪表盘并访问真实健康状态，0 控制台错误。
- **未执行**：Docker 镜像构建/Compose 启动（环境无 `docker`）；`/admin` 在多副本/反向代理后的部署未验证。

## 2026年11月03日 23时54分18秒 第四阶段：公开玩家门户与 /api/v1/public 只读数据接口

**背景**

管理后台与核心数据模型已经就绪，但没有面向 KZ 玩家的公开查询入口。本阶段新增独立公开门户（排行榜、玩家资料/地图进度、比较、地图、成绩、服务器），并补齐真实只读 API、SQL 聚合与 Redis 缓存；不修改既有数据库结构与后台功能。

**变更内容**

- **共享设计 Token**：将颜色/字体/圆角/动效/基础层抽取到 `packages/ui/src/design-system.css`（`@cngokz/ui/styles.css`），管理后台与公开门户共同引用；管理后台构建/测试保持通过。
- **公开门户 `apps/public`（独立 Vite 应用，端口 5174）**
  - 顶部横向导航（首页/排行榜/玩家/地图/成绩/服务器）、移动端抽屉、主题（默认深色、持久化）、Steam 登录占位 Dialog。
  - 全局搜索 Dialog（⌘K/Ctrl+K）：按玩家昵称、SteamID64、地图名搜索，结果分组并可跳转。
  - 首页：真实全站统计、模式 Tab Top10、最近 20 条上传成绩、最近 WR、本周活跃玩家、在线服务器、数据口径说明。
  - 排行榜：模式与 TP/PRO 过滤、分页；明确说明积分算法未定义，按完成地图数排序。
  - 玩家目录：搜索、模式/排序筛选、分页。
  - 玩家详情：资料头、8 项综合统计、模式卡片、动态难度进度、地图完成/未完成（主线完成定义 + Bonus 单独统计）、成绩历史、成绩详情 Dialog。
  - 玩家比较：双玩家搜索选择/交换/排行榜快速选择、综合表、模式与难度对比、逐地图对比（双方/单方/未完成/A 更快/B 更快）、分页。
  - 地图列表/详情：难度、课程、完成玩家/成绩数、WR、课程表、成绩分布、课程/模式/TP-PRO 排行榜、分页。
  - 成绩查询：玩家、地图、模式、TP/PRO、难度、WR、上传日期范围、排序、分页、成绩详情。
  - 服务器列表：在线状态（2 分钟心跳推导）、当前地图、插件版本；玩家数/模式等未采集字段显示 `—`。
  - SEO：每页独立 title/description/OG/Twitter/canonical；构建时生成 `sitemap.xml`、`robots.txt`；404 noindex；`VITE_SITE_URL` 配置站点地址。
  - 生产静态服务器：SPA 回退、`/api`、`/health`、`/swagger-ui`、`/api-docs` 同源代理、`/healthz`。
- **后端公开 API（`/api/v1/public`，20 个端点）**
  - 统计、模式、排行榜、玩家目录/活跃/比较、玩家详情/地图进度/成绩、地图列表/详情/排行榜/统计、成绩列表/最近/世界纪录/详情、服务器列表/详情、全局搜索。
  - 统一响应信封与错误结构；分页上限 100；参数校验（page/page_size/time_type/tier/日期/搜索长度/排序白名单/compare 双玩家不同）；全部纳入 OpenAPI（26 paths / 99 schemas）。
  - SQL 以聚合为主：`RANK() OVER` 排行榜；`LEFT JOIN LATERAL` 取每张地图/每个课程最佳成绩；`count(DISTINCT ...) FILTER` 统计完成与 PRO/NUB；对比页 SQL 侧筛选与计数；避免“先取全量再在 Rust 循环”。
  - **数据准确性**：完成地图 = 指定模式主线课程（stage 0）approved 成绩；Bonus 单独计数；完成率分母为支持该模式的已上线地图；完成时间（跑图时长）与上传时间（created_at）严格区分；WR 仅当前指针。
  - **积分缺口**：CNGOKZAPI 尚无积分算法，公开响应 `points/rating = null`、`points_available=false`，排行榜以 `completed_maps` 为指标；不伪造 Points/Rating。
- **Redis 缓存**
  - 统计 30s、排行榜 60s、最近成绩 15s、玩家/地图 30–120s、服务器 15s、搜索 30s；Key 包含 `public:version`，`INCR` 可一次性失效；阶段 5 写入路径可调用 `invalidate_cache`。
  - `RedisClient` 增加 get/set_ex/incr；连接配置 0 重试、300ms 超时、失败后 5 秒快速失败，Redis 故障时公开接口直接回源 PostgreSQL（不会让每个请求等待连接超时）。
- **部署**
  - `infrastructure/docker/public.Dockerfile`（Node 多阶段 + 静态服务器）；compose 新增 `public` 服务（5174，依赖 API 健康，`/healthz` 健康检查），`VITE_SITE_URL` 作为构建参数；`.env.example` 增加 `PUBLIC_PORT`、`VITE_SITE_URL`，CORS 默认加入 5174；CI 增加 public 构建与镜像构建；`scripts/gen-api.sh` 增加 public 类型检查。
- **文档**
  - 新增 `docs/public-portal/{architecture,pages,api-requirements,design-system}.md`；更新 README、architecture、development、deployment、design-system。

**涉及文件**

- 新增：`apps/public/**`（package/tsconfig/vite/vitest/eslint/prettier、src/{components,layouts,pages,hooks,services,stores,router,seo,styles,lib,types,test}、server/static-server.mjs、scripts/generate-seo.mjs）
- 新增：`apps/api/src/dto/public/**`、`repositories/public/**`、`services/public/**`、`handlers/public.rs`
- 修改：`apps/api/src/{dto,repositories,services,handlers}/mod.rs`、`routes/mod.rs`、`routes/openapi.rs`、`state/redis_client.rs`、`errors/mod.rs`
- 新增：`apps/api/tests/public_api.rs`
- 新增：`packages/ui/{package.json,src/design-system.css}`；修改 `apps/web/src/styles/index.css`、`apps/web/{package.json,vite.config.ts,vitest.config.ts}`
- 修改：`packages/api-client/src/{client,index}.ts`、`packages/shared-types/src/api-contract.ts`（公开 payload 别名 + TimeType）
- 修改：`pnpm-workspace.yaml`、`package.json`、`.env.example`、`docker-compose.yml`、`infrastructure/docker/{web,public}.Dockerfile`、`scripts/gen-api.sh`、`.github/workflows/ci.yml`
- 新增：`docs/public-portal/{architecture,pages,api-requirements,design-system}.md`；修改 README、`docs/{architecture,development,deployment,design-system,changelog}.md`
- 生成物更新：`docs/openapi.json`、`packages/shared-types/src/generated/api-schema.d.ts`

**数据库与 API 变化**

- 数据库：**无迁移、无表结构变更**；公开接口只读。
- API：新增 `/api/v1/public/*` 20 个端点；管理后台接口与行为不变；OpenAPI 从 6 个 path 扩展到 26 个 path（含既有 health/system）。
- 缓存：新增公开缓存命名空间与 TTL；未改变现有 health/system 接口语义。
- 第一阶段 Docker 服务保持不变；新增 `public` 服务。

**测试与结果**

- 后端：`cargo fmt --check` / `cargo clippy -D warnings` 通过；`cargo test` **54 个通过**（29 单元 + 10 HTTP + 1 公开 API 端到端 + 11 schema + 3 真实依赖集成），0 失败。公开 API 测试使用临时数据库 + migrations + 真实 fixtures，覆盖统计/排行榜/玩家详情/地图完成与未完成/比较/地图详情/榜/统计/成绩筛选/最近排序/活跃/服务器/搜索/详情/参数校验/404。
- 前端：`pnpm -r lint`、`pnpm -r typecheck`、`pnpm -r format:check` 通过；管理后台 6 文件/21 用例、公开门户 2 文件/5 用例，共 **26 个通过**；`pnpm -r build` 两个应用均构建成功（公开门户构建时生成 sitemap/robots）。
- OpenAPI：重新生成后与提交文件一致（26 paths / 99 schemas）。
- 浏览器端到端（Playwright headless Chromium）：
  - 公开门户 **36/36** 断言通过：首页真实统计/排行榜/最近成绩、模式 Tab、全局搜索、玩家详情地图进度与未完成筛选、玩家搜索、地图与地图榜、成绩与 WR 筛选、服务器、比较、主题持久化、1440/375 无横向溢出、移动菜单与导航；**0 控制台错误**。
  - 管理后台回归 **38/38** 断言通过（真实健康状态、主题、侧边栏、导航、弹窗、筛选、响应式），0 控制台错误。
- **未执行**：Docker 构建/Compose 一键启动（当前环境无 `docker` 命令）；SSR/预渲染（当前采用 SPA + 动态 meta，已记录限制）；Firefox/Safari 实机。
- 空数据库验证：公开门户首页/玩家/地图/排行榜/成绩/服务器/比较 7/7 页面显示明确空状态，0 控制台错误；管理后台在空数据库下仍为 38/38 通过。
- 说明：验证使用临时 fixture 数据库（仅测试/截图用，验证后已删除），未向项目提交任何模拟玩家/地图/成绩数据；当前运行环境已切回干净的 `cngokzapi_phase2` 数据库。

## 2026年11月03日 22时46分45秒 第三阶段：Design System 建立与管理后台 UI/UX 重构

**背景**

前两阶段完成了基础架构与核心数据模型，但前端仍是最小可用后台：视觉语言不统一、导航与页面结构分散、缺少表格/分页/弹窗等标准交互，也没有明确的设计规范。第三阶段目标是在**不重建项目、不修改后端/数据库/Docker** 的前提下，建立完整设计系统并把全部页面升级为统一、专业的管理平台。

**变更内容**

- **设计系统与主题**
  - 重写 `styles/index.css`：新增 `surface / surface-elevated / surface-hover / border-strong / info / danger` 等语义 token，深色改为默认主题（深灰 `oklch(0.168)`，避免纯黑），浅色完整定义。
  - 自托管 Inter Variable（`@fontsource-variable/inter`），统一字号层级（页面标题 22px、卡片 14px、正文 13px、表格头 11px、数字 26px + tabular-nums）。
  - 统一圆角（8px 基础）、间距、阴影、滚动条与焦点环；新增 `page-enter`、`zoom-in`、`slide-in-left`、`pulse-soft` 动效并支持 `prefers-reduced-motion`。
  - `index.html` 首屏前应用主题且默认深色；`ui-store` 默认主题改为 `dark`，主题与侧边栏折叠持久化。
- **基础组件**
  - 新增：`Select`、`Dialog`、`Tabs`、`Popover`、`Avatar`、`Switch`、`Pagination`、`StatusIndicator`、`SearchInput`、`Callout`、`Kbd`、`DataTable`。
  - 重构：`Button`（6 变体/5 尺寸/loading/asChild）、`Card`、`Badge`（语义色）、`Input`、`Table`（表头/行高）、`Tooltip`、`DropdownMenu`（Check 指示）、`Sheet`、`Skeleton`。
  - 组合组件：`ListPanel` + `FilterBar`（列表页统一工具栏/分页）、`CodeBlock`（真实复制）、`ServerCard`、`ServiceStatusRow`、`DependencyStatusCard`、`StatCard`。
- **布局与导航**
  - 导航重构为 7 个分组 / 15 个入口：总览、玩家管理、KZ 数据、服务器管理、安全管理、开发者、系统；`nav-config` 同时驱动侧边栏、面包屑与 ⌘K 搜索。
  - 侧边栏：240px ↔ 64px 折叠、激活态左侧色条、分组分隔、图标 tooltip、底部显示**真实**系统状态（`useSystemStatus`）与环境/版本（`useSystemInfo`）。
  - 顶栏：面包屑、全局页面搜索（⌘K/Ctrl+K，键盘上下/回车/ESC）、通知 Popover（空状态）、主题菜单、账号菜单与"关于"Dialog。
  - 移动端抽屉、跳过主内容链接、页面切换淡入动画；内容区最大宽度 1500px。
- **页面（全部重写）**
  - 仪表盘：真实 API/PG/Redis 状态与延迟、运行时长、系统事件；4 个统计卡片明确标注"接口未接入"；API 请求趋势与服务器在线状态为空状态；时间范围选择禁用并注明需统计接口。
  - 系统状态：整体状态、依赖卡片、系统信息、健康检查端点外链、刷新；降级/错误提示。
  - 玩家列表 + 玩家详情（Tabs：资料/成绩/Jumpstats/封禁）；地图、成绩（工具条排序 + 5 个筛选）、排行榜（Tabs + 模式/Style/TP-PRO）、Jumpstats、Replay、服务器列表（列表/卡片视图切换）+ 服务器详情、封禁、操作日志。
  - API 密钥：创建 Dialog（含一次性密钥安全说明，提交禁用并注明接口未接入）、请求统计空状态；API 文档：真实端点表 + 复制示例；API 请求日志。
  - 系统设置：主题卡片（真实生效并可持久化）、折叠侧边栏 Switch、平台信息（真实 `/system/info`）、快捷入口、认证未启用提示。
  - 404 与路由错误页重设计。
- **路由**
  - 新路径：`/players`、`/maps`、`/records`、`/leaderboard`、`/jumpstats`、`/replays`、`/bans`、`/audit-logs`、`/servers/:serverId`、`/players/:playerId`、`/developer/request-logs` 等。
  - 旧路径兼容重定向：`/kz/*`、`/data/*`、`/servers/status`。
- **测试与文档**
  - 新增前端测试：分页交互、导航搜索/面包屑、默认深色主题与侧边栏切换；更新仪表盘测试（真实状态、占位、错误态）。
  - 新增 `docs/design-system.md`（token/字体/间距/组件/布局/响应式/动效/可访问性/文件索引）；更新 README、architecture、development。
- **明确不做假数据**：没有读取接口的页面统一显示 `Callout` + `EmptyState`；分页/筛选/排序状态可交互但结果为 0；弹窗中的"创建"按钮禁用并说明接口未接入。

**涉及文件**

- 修改：`apps/web/package.json`、`pnpm-lock.yaml`（新增 Inter 字体）
- 修改：`apps/web/index.html`、`src/main.tsx`、`src/styles/index.css`、`src/stores/ui-store.ts`
- 新增：`src/components/ui/{select,dialog,tabs,popover,avatar,switch,pagination,status-indicator,search-input,callout,kbd,data-table}.tsx`
- 修改：`src/components/ui/{button,card,badge,input,table,tooltip,dropdown-menu,sheet,skeleton}.tsx`
- 新增：`src/components/common/{list-panel,code-block}.tsx`；删除：`src/components/common/module-placeholder.tsx`
- 新增：`src/components/layout/{command-menu,notifications-menu,user-menu}.tsx`；修改：`sidebar.tsx`、`topbar.tsx`、`nav-config.ts`
- 修改：`src/layouts/app-layout.tsx`、`src/types/index.ts`、`src/router/index.tsx`
- 新增：`src/hooks/use-table-state.ts`、`src/lib/kz-constants.ts`、`src/components/servers/server-card.tsx`、`src/components/dashboard/service-status-row.tsx`
- 修改：`src/components/dashboard/{stat-card,dependency-status-card,system-info-card,recent-events-card}.tsx`、`src/components/common/{empty-state,error-state,page-header,page-loading,status-badge}.tsx`
- 重写页面：`dashboard-page`、`system-status-page`、`players-page`、`maps-page`、`records-page`、`leaderboard-page`、`jumpstats-page`、`replays-page`、`servers-page`、`bans-page`、`api-keys-page`、`api-docs-page`、`settings-page`、`not-found-page`、`route-error-page`
- 新增页面：`player-detail-page`、`server-detail-page`、`audit-logs-page`、`request-logs-page`；删除：`server-status-page`
- 新增测试：`src/components/ui/pagination.test.tsx`、`src/components/layout/nav-config.test.ts`、`src/stores/ui-store.test.ts`
- 修改测试：`src/pages/dashboard-page.test.tsx`
- 新增文档：`docs/design-system.md`；修改：`README.md`、`docs/architecture.md`、`docs/development.md`、`docs/changelog.md`

**数据库与 API 变化**

- 无。未修改 `apps/api`、迁移、Dockerfile、compose 文件；`docs/openapi.json` 重新生成后与原文件一致。
- 前端仅消费既有接口：`/health`、`/api/v1/system/info`、`/status`、`/events`。

**测试与结果**

- `pnpm install --frozen-lockfile`：通过。
- `pnpm -r lint`（`--max-warnings 0`）：通过；`pnpm -r typecheck`：通过。
- `pnpm -r test`：6 个测试文件 / **21 个用例通过**（格式化、状态徽章、仪表盘、分页、导航搜索、主题 store）。
- `pnpm --filter @cngokz/web build`：通过（仍有主 chunk >500 kB 的提示性警告，未处理）。
- **Playwright（Chromium 156，playwright 1.64.1）**：
  - 主流程 38 项断言全部通过：默认深色、真实 PG/Redis 状态、主题切换与刷新持久化、侧边栏 240→64px、导航、⌘K 搜索跳转、通知空状态、关于弹窗、搜索/重置、分页禁用、API Key 弹窗、服务器卡片视图、设置页主题切换、1920/1440/1280/768/375 无横向溢出、移动抽屉开合与跳转。
  - 20 组视口/页面组合（1920/1440/1280/768/375 × dashboard/records/servers/settings）：**横向溢出 0**。
  - 交互补充 9 项通过：Tooltip 显示、Select 筛选、排序 Select、15 个侧边栏链接全部可解析、4 个旧路径重定向、API 故障时降级到错误状态。
  - 正常流程 **0 个控制台错误**；仅在"模拟 API 断网"用例中出现 2 条预期的 `net::ERR_FAILED` 资源错误。
- 后端回归：`cargo fmt --check`、`cargo clippy -D warnings` 通过；`cargo test` **53 个通过**；`/health`、`/health/ready`、`/api/v1/system/status` 均 200。
- **未执行**：Docker 重建/Compose 启动（本阶段未修改基础设施）、Firefox/Safari 实机、人工视觉走查（使用 headless Chromium 截图与断言替代）、业务读取接口联调（接口尚不存在）。

## 2026年11月03日 22时17分28秒 第二阶段：核心数据库架构、Rust 模型与 GlobalAPI 兼容性分析

**背景**

第一阶段只有系统表；第二阶段需要研究 GOKZ / cs2kz / GlobalAPI 的真实数据结构，设计 CNGOKZAPI 自己的核心数据模型，为玩家、地图、成绩、排行榜、服务器、Jumpstats、Replay 和封禁打基础。要求保留第一阶段代码与 Docker 配置，不实现完整业务接口，不提前引入复杂分区/微服务。

**研究过程（基于真实源码，未修改上游）**

- 克隆并阅读 `KZGlobalTeam/gokz`（tag 3.8.0 / `3a601e1`）、`KZGlobalTeam/cs2kz-api`（`c938925`）、`KZGlobalTeam/csgo-legacy-globalapi-plugin`（`bec212c`）。
- 依据：GOKZ `gokz-localdb/db/sql.sp`、`include/gokz/{core,jumpstats,global}.inc`、`gokz-replays`、`include/GlobalAPI.inc` + `responses.inc`；cs2kz `0001_initial.up.sql` 及 0002–0006、`mode.rs`/`styles.rs`/`jumpstats/jump_type.rs`/`maps/state.rs`/`records.rs`；legacy `GlobalAPI-Core.sp`、`records.sp`、`bans.sp`、`server.sp`。
- 明确记录了研究局限：GlobalAPI v2 服务端实现不在仓库中（POST 字段由 native 参数反推）、KZTimer 源码未获取、GOKZ 只有 `Style_Normal` 而 cs2kz 的 `styles` 是服务器设置位掩码而非移动风格。

**变更内容**

- 新增 8 个迁移（`0002`–`0009`），创建 20 张新表（17 张业务表 + 3 张引用表）：
  - 引用数据：`movement_modes`、`movement_styles`、`jump_types`（含 GOKZ/cs2kz 枚举映射列 + 种子数据）。
  - Accounts：`accounts`（SteamID64 `NUMERIC(20,0)` + uint64 CHECK、软删除部分唯一索引）、`player_profiles`、`player_mode_stats`、`player_identities`。
  - Maps：`maps`、`map_versions`、`map_courses`（stage 0 主线 / 1.. bonus，复合唯一 `(id, map_id)`）、`map_modes`、`course_tiers`（NUB/PRO 1–10 + ranked 状态）。
  - Servers：`servers`（API Key 哈希/前缀/轮换时间、插件版本、当前地图、在线状态、心跳）。
  - Records：`records`（`time_ns BIGINT` 纳秒、TP/PRO CHECK、tickrate、来源、审核、`submission_id`/`external_ref` 幂等键、复合外键保证 map/course 一致）、`record_history`、`course_world_records`（WR 条件 upsert）。
  - Jumpstats：`jumpstats`（距离 `NUMERIC`、sync%、airtime、block、msl、绑定标记、`raw_metrics` JSON 保留未确认指标）+ 幂等索引。
  - Replays：`replays`（R2 bucket/object_key、size、SHA-256、格式版本、状态、保留期）。
  - Moderation：`bans`（执行者可为管理员或服务器、过期时间推导、不可软删除）、`ban_revocations`。
- 新增共享 `set_updated_at()` 触发器函数，并在所有带 `updated_at` 的表上安装触发器。
- 新增 Rust models：`models/{account,map,server,movement,record,jumpstat,replay,moderation}.rs`；状态/来源枚举与 SQL CHECK 一一对应（`sqlx::Type`）。
- 新增第三阶段读取契约（DTO，暂未注册到 OpenAPI）：`dto/{account,map,record,server,jumpstat,replay,moderation}.rs`。
- 新增 `apps/api/build.rs`：`cargo:rerun-if-changed=migrations`，解决新增迁移文件不会触发 `sqlx::migrate!` 重新编译的问题。
- 新增 `apps/api/tests/schema_integration.rs`（11 个事务回滚测试）。
- 扩展 `tests/service_integration.rs` 的全新数据库迁移断言（新表 + 引用数据种子）。
- 新增文档：`docs/database/` 下 9 个文件（overview + 7 个模块 + ER 图）、`docs/globalapi-compatibility.md`；更新 `README.md`、`docs/architecture.md`、`docs/development.md`。
- `Cargo.toml` 为 SQLx 启用 `bigdecimal`、`ipnetwork` feature（SteamID64 用 `BigDecimal`，IP 用 `INET`）。

**涉及文件**

- 新增：`apps/api/migrations/20261103000002_kz_reference_data.sql`
- 新增：`apps/api/migrations/20261103000003_accounts.sql`
- 新增：`apps/api/migrations/20261103000004_maps.sql`
- 新增：`apps/api/migrations/20261103000005_servers.sql`
- 新增：`apps/api/migrations/20261103000006_records.sql`
- 新增：`apps/api/migrations/20261103000007_jumpstats.sql`
- 新增：`apps/api/migrations/20261103000008_replays.sql`
- 新增：`apps/api/migrations/20261103000009_moderation.sql`
- 新增：`apps/api/build.rs`
- 新增：`apps/api/src/models/{account,map,server,movement,record,jumpstat,replay,moderation}.rs`
- 修改：`apps/api/src/models/mod.rs`（模块/重导出/enum `IngestionSource`）
- 新增：`apps/api/src/dto/{account,map,record,server,jumpstat,replay,moderation}.rs`
- 修改：`apps/api/src/dto/mod.rs`
- 新增：`apps/api/tests/schema_integration.rs`
- 修改：`apps/api/tests/service_integration.rs`
- 修改：`Cargo.toml`、`Cargo.lock`
- 新增：`docs/database/{overview,accounts,maps,records,servers,jumpstats,replays,moderation,er-diagram}.md`
- 新增：`docs/globalapi-compatibility.md`
- 修改：`README.md`、`docs/architecture.md`、`docs/development.md`、`docs/changelog.md`（本记录）

**数据库与 API 变化**

- 数据库：新增 20 张表、引用数据种子（4 个移动模式、1 个风格、12 个跳跃类型）、多个部分唯一索引/CHECK/触发器。阶段 1 的 `system_events`、`app_settings` 未修改。
- API：**无变化**。未新增 HTTP 路由，`docs/openapi.json` 重新生成后与提交版本完全一致。
- 兼容性：GlobalAPI 字段映射、类型差异、枚举差异、转换规则和兼容接口建议记录在 `docs/globalapi-compatibility.md`；CNGOKZAPI 未复制 GlobalAPI 表结构。

**测试与结果**

- `cargo fmt --all --check`：通过。
- `cargo clippy --all-targets --all-features -- -D warnings`：通过（新增 `build.rs`、模型、DTO 无告警）。
- `DATABASE_URL=...cngokzapi_phase2 REDIS_URL=... cargo test`：**53 个测试通过**（29 单元 + 10 HTTP + 11 schema 集成 + 3 真实依赖集成），0 失败。
- 迁移在本地 PostgreSQL 18.6 上从零执行成功；重启 API 后 `_sqlx_migrations` 保持 9 条（幂等）。
- 手工验证：uint64 上限 SteamID64 写入成功、超上限被 CHECK 拒绝；PRO/NUB 与 teleports 不一致被拒绝；重复 `submission_id` 被唯一索引拒绝；慢成绩不能覆盖 WR；软删除后可用同一 SteamID64 重新注册。
- 阶段 1 回归：`/health`、`/health/ready`（PG+Redis 正常时 200）、`/api/v1/system/info`、`/status`、`/events` 均 200，OpenAPI 文档无差异。
- 前端回归：`pnpm -r typecheck`、`pnpm -r test`（13 个）、`pnpm -r format:check` 全部通过（无前端代码改动）。
- Mermaid ER 图：使用 `mermaid@11` + jsdom 调用 `mermaid.parse()` 解析通过（diagram type: `er`）。
- **未执行**：Docker 构建/Compose 启动（本环境无 `docker` 命令）；浏览器页面验证（本阶段无 UI 改动）；GlobalAPI 真实接口联调（兼容层尚未实现）。

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
