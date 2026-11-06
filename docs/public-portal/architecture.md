# 公开玩家门户 · 架构说明

## 1. 目标与边界

公开门户 `apps/frontend` 面向所有 KZ 玩家，是一个**独立的 React 应用**，与 `apps/frontend`（管理后台）共用同一个 Rust API、PostgreSQL、Redis、API Client 和设计 Token，但拥有独立的布局与导航。

- 公开门户不依赖管理后台运行。
- 管理后台不受影响，原有路由、组件保持不变。
- 本阶段只提供**公开只读数据**；登录、写操作与管理员接口不暴露。

## 2. 目录结构

```text
apps/
├── web/        # 管理后台（SaaS 控制台布局）
├── public/     # 公开玩家门户（顶部导航 + 社区数据布局）
└── api/        # 共享 Rust API
packages/
├── api-client/   # 管理后台 + 公开门户共享的类型化客户端
├── shared-types/ # OpenAPI 生成的类型与公开 payload 别名
└── ui/           # 共享设计 Token（design-system.css）
```

公开门户是独立的 Vite 应用，作为默认站点运行在 **5173**；管理后台挂载在同端口的 `/admin`：

```text
http://host:5173/          → apps/frontend（公开门户，默认）
http://host:5173/admin/    → apps/frontend（管理后台，basename=/admin）
http://host:5173/api/...   → Rust API
```

开发时公开门户 5173 把 `/admin`（含 HMR WebSocket）代理到管理后台开发服务器 5174；生产时 Nginx 直接提供 `/admin/` 静态文件，`/api`、`/health` 反向代理到 `127.0.0.1:8080`。对外只暴露 443 端口。

```text
浏览器 → 5173（Vite dev proxy / 公开门户静态服务器）
        ├─ /                 → public 构建产物
        ├─ /admin/*          → web（管理后台；dev: 5174，prod: web:5173）
        └─ /api, /health, /swagger-ui, /api-docs → api:8080
```

因此浏览器不需要 CORS，也不需要构建时写死 API 域名。

## 3. 前端数据链路

```text
页面
 └─ hooks / TanStack Query（publicQueries）
     └─ @cngokz/api-client（openapi-fetch，类型来自 OpenAPI）
         └─ /api/v1/public/*
             └─ handlers → services（Redis 缓存）→ repositories（SQL）
                 └─ PostgreSQL
```

- API 类型来自 `docs/openapi.json`；后端 DTO / 路由变更后运行 `scripts/gen-api.sh`。
- 所有列表状态（筛选、排序、分页）写入 URL 查询参数，可分享、可后退。
- 缓存失效时（Redis 不可用）服务降级到 PostgreSQL，不影响正确性。

## 4. 后端模块

```text
apps/api/src/
├── dto/public/*          # 公开 API 契约（serde + utoipa）
├── repositories/public/  # 只读 SQL（统计、玩家、地图、成绩、服务器、搜索）
├── services/public/      # 编排、校验、缓存（cache.rs）
└── handlers/public.rs    # /api/v1/public/* 路由与 OpenAPI 注解
```

公开接口全部挂在 `/api/v1/public`，使用统一响应信封与错误结构；路由登记在 `routes/openapi.rs`，Swagger UI 自动包含。

## 5. 排行指标与积分缺口（重要）

CNGOKZAPI **尚未实现积分算法**。为避免编造数据：

- 公开 API 的 `points` / `rating` 字段为 `null`，响应包含 `points_available: false`。
- 排行榜以 **完成主线地图数** 为指标（`ranking_metric = "completed_maps"`），并列时依次比较 WR 数、成绩数。
- 页面明确显示“积分算法未定义”，不以 0 或伪造数值占位。
- 未来实现积分时，应：
  1. 在文档中定义公式、数据来源（tier、排名分布、距离分布）；
  2. 写入 `player_mode_stats`（或新的缓存表）；
  3. 更新公开 API 的 metric 与前端文案，而不是修改既有历史口径。

## 6. 统计口径

| 指标 | 定义 | 实现 |
| --- | --- | --- |
| 玩家总数 | 未软删除的 `accounts` | `count(*)` |
| 地图总数 | `status='approved'` 且未软删除的地图 | `count(*)` |
| 成绩总数 | 已通过、未删除、玩家与地图均有效 | `count(*)` |
| 在线服务器 | `is_online` 且心跳在 2 分钟内 | 读取时推导，不使用缓存误报实时状态（缓存 TTL 15s） |
| 完成地图 | 该模式下主线课程（stage 0）存在 approved 成绩 | `count(DISTINCT map_id)` + `stage=0` |
| 完成率 | 完成地图 / 支持该模式的已上线地图（`map_modes`） | 0 分母返回 0 |
| WR 数 | 当前 `course_world_records` 指向该玩家的成绩数 | join records |
| PRO / NUB | `time_type` 计数（PRO=0 传送） | `count(*) FILTER` |
| 地图完成玩家 | 该地图主线课程（任意 TP/PRO）的 distinct 玩家 | 地图统计 |
| 地图 PRO/NUB 完成人数 | 分别按 time_type 过滤 | 地图统计 |
| 成绩排名 | 同一（course, mode, style, time_type）中更快成绩数 + 1 | 相关子查询，走榜单索引 |
| 完成时间 | 跑图时长（`time_ns` → 秒） | 不是上传时间 |
| 上传时间 | `records.created_at` | 最近成绩按此倒序 |

**地图完成 vs Bonus**：完成状态只取主线课程（stage 0）；Bonus 完成数量单独统计（`bonus_completed / bonus_total`），不会把 Bonus 成绩误认为主线完成。

## 7. Redis 缓存

| 数据 | Key 前缀 | TTL |
| --- | --- | --- |
| 全站统计 | `public:v{ver}:statistics` | 30s |
| 模式列表 | `public:v{ver}:modes` | 300s |
| 排行榜 | `public:v{ver}:leaderboard:*` | 60s |
| 玩家列表 | `public:v{ver}:players:*` | 30s |
| 玩家详情 / 进度 | `public:v{ver}:player*` | 30s |
| 成绩列表 | `public:v{ver}:records:*` | 20s |
| 最近成绩 | `public:v{ver}:recent-records:*` | 15s |
| 最近 WR | `public:v{ver}:world-records:*` | 30s |
| 活跃玩家 | `public:v{ver}:active-players:*` | 60s |
| 地图列表 / 详情 | `public:v{ver}:maps*` / `map:*` | 60s / 120s |
| 服务器 | `public:v{ver}:servers:*` | 15s |
| 搜索 | `public:v{ver}:search:*` | 30s |

- 命名空间版本 `public:version`：`INCR` 一次即可让全部公开缓存失效（无需 SCAN/DEL）。`services::public::invalidate_cache` 暴露给阶段 5 的写入路径调用。
- Redis 不可用时：连接配置为 300ms 超时、0 重试，并在失败后 5 秒内快速失败，请求直接回源 PostgreSQL；不会让每个请求等待连接超时。
- 服务器在线状态缓存仅 15 秒，避免把过期缓存当作实时状态；前端每 30 秒轮询。

## 8. SEO

已实现：

- 每个页面通过 `usePageMeta` 设置独立 `title`、`description`、Open Graph、Twitter、canonical。
- 玩家/地图详情页使用稳定 URL：`/players/:steamid64`、`/maps/:slug`（slug = 地图名小写化）。
- `sitemap.xml`、`robots.txt` 在构建时由 `apps/frontend/scripts/generate-seo.mjs` 生成（核心页面），`VITE_SITE_URL` 控制绝对地址。
- 404 页面、`robots` noindex。

已知限制：

- 当前为 SPA，玩家/地图的动态 meta 由浏览器执行 JS 后写入；不执行 JS 的社交爬虫可能读取到默认 meta。
- 玩家/地图的 sitemap 条目需要数据快照，本阶段未生成；阶段 5 可增加 `/sitemap-players.xml` 等后端动态端点，或引入 SSR/预渲染。
- **未引入 SSR**：评估后认为当前数据规模与页面结构下重写为 SSR 成本高于收益；优先完善动态 meta 与动态 sitemap。

## 9. 部署

- 开发：`pnpm dev` 同时启动门户（5173）与后台（内部 5174）；`/admin` 由门户代理。
- 生产：`scripts/build-native.sh` 构建 unified frontend 静态文件；Nginx 从 `/www/sites/test.cngokz.com/index/cnapi` 与 `/www/sites/test.cngokz.com/index/cnapi` 提供服务，并把 `/api`、`/health` 反向代理到 `127.0.0.1:8080`。
- 站点 URL 是构建参数 `VITE_SITE_URL`，更换域名后需重新执行 native 构建并发布静态文件。

## 10. 已知缺口与阶段 5 建议

| 缺口 | 影响 | 建议 |
| --- | --- | --- |
| 积分算法未定义 | 排行榜只按完成数，积分列显示 — | 先定义公式与数据来源，再写入 `player_mode_stats` |
| 玩家头像 | 数据库无 avatar 字段，使用昵称首字母 | 玩家资料同步时写入 Steam avatar URL（新增列） |
| 地图作者/封面 | schema 无 mapper/cover 字段 | 新增 `map_authors`、`map_cover_url` 或对象存储 |
| 服务器玩家数/最大人数/模式 | schema 未采集 | 心跳上报增加字段（players/max_players/mode） |
| WR 历史提升值 | 未保存 WR 变更历史 | 新增 `world_record_history` 表，记录 prev/next |
| 公开 Replay 下载 | 无公开下载端点 | 增加签名 URL 下载接口与权限校验 |
| 动态 sitemap / 非 JS 爬虫 | SEO 覆盖有限 | 动态 sitemap + 预渲染/SSR 评估 |
| 服务器模式 | 未采集 | 心跳上报或按 current map 的 map_modes 推导（需确认口径） |
