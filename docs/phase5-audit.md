# 第五阶段审查与验收记录

## 1. 审查范围

审查了 `apps/api`、`apps/web`（管理后台）、`apps/public`（公开门户）、`packages/*`、SQLx migrations、Redis 缓存、Docker/Compose、环境变量、OpenAPI、测试与文档。仓库实际结构与需求示例不同：没有 `apps/admin`、没有 `crates/`；管理后台为 `apps/web`，后端为 `apps/api`。两者均为既有命名，未重命名。

## 2. 审查结论

| 项目 | 结论 |
| --- | --- |
| 两个前端是否独立 | 是。各有 Vite/Router/布局/pages/构建产物/静态服务器；公开门户不依赖管理后台运行。 |
| 共享 API Client/类型 | 是。均引用 `@cngokz/api-client`、`@cngokz/shared-types`，类型源为 OpenAPI。 |
| 后端重复业务逻辑 | 未发现明显重复；公开只读逻辑分为 handlers/services/repositories。 |
| 数据库模型满足页面需求 | 满足当前公开页面；管理后台业务写入页面仍为占位，无写接口。 |
| 模拟/硬编码数据 | 未提交模拟玩家/地图/成绩；fixture 仅存在于测试与临时验证数据库。 |
| Redis 是否真正参与缓存 | 是。公开 API 18 个入口使用 `get_or_load`，Key 带版本命名空间；第五阶段增加失效端点、TTL 抖动、单飞与短 Key。 |
| Docker 完整启动 | 配置审查通过（5 服务、卷、健康检查、依赖、单入口）；**本环境无 docker CLI，未实际启动**。 |
| TODO/FIXME | 未发现 `TODO`/`FIXME`/`unimplemented!`；未实现功能以明确空状态与文档记录，不伪装成功。 |

## 3. 已修复问题

1. **统计口径不一致**：玩家详情同时存在全站汇总与当前范围数据，原先展示时易混淆。现在 API 返回 `overall(scoped=false)`、`scoped(scoped=true)` 与 `scope(mode/style/time_type/completion_definition)`；页面卡片显示当前范围，模式卡明确为分解视图。
2. **缓存缺陷**：Key 可直接由用户输入无限增长；无 TTL 抖动；并发 miss 会重复回源；无失效入口。已改为固定长度哈希 Key、0–15% TTL 抖动、per-key async 单飞、启动时 `SET NX` 初始化版本、受 `X-Maintenance-Token` 保护的失效/版本端点与首页统计预热。
3. **无限流**：公开 API 新增每 IP 每分钟 `PUBLIC_RATE_LIMIT_PER_MINUTE`（默认 120），超限返回 429 `rate_limited` + `Retry-After`；维护端点与健康检查不受该限流。单进程内存实现，多副本需改用 Redis/边缘限流。
4. **服务器状态**：新增 `online/offline/expired/unknown` 与 `heartbeat_age_seconds`；玩家数/最大人数/模式仍为 `null`（schema 未采集），不伪造实时状态。
5. **部署参数错误**：`docker-compose.yml` 的 `VITE_SITE_URL` 默认由 5174 改回 5173；`.env.example` 增加限流与维护令牌；CI 增加 `develop` 分支与 compose 校验。
6. **静态服务器路径**：`serveStatic` 增加解码异常处理与 `path.relative` 越界校验；`/assets/../../etc/passwd` 与 URL 编码穿越不再读取 dist 外文件（返回 SPA 回退或 403）。
7. **性能索引**：新增迁移 `20261104000001_phase5_indexes.sql`（pg_trgm、最近成绩、玩家/地图历史、进度查询、课程排名、WR 达成时间）。在 2 万条成绩临时库上用 `EXPLAIN (ANALYZE, BUFFERS)` 验证索引命中。
8. **文档/测试**：新增缓存、限流、维护令牌、玩家统计范围、服务器状态分类、缓存失效链路测试；OpenAPI 重新生成；README/部署/公开门户文档同步。

## 4. 未完成 / 需要产品确认

- 积分/Rating 算法仍未定义：`points`/`rating` 为 `null`，排行榜按完成地图数；**未擅自实现公式**。
- 管理后台无登录、Session/Token、角色权限与业务写接口；`/admin` 目前仅可浏览只读系统状态与空状态页面。生产部署应在反向代理或认证层保护 `/admin`。
- 服务器心跳上报接口、玩家数/最大人数/模式字段、WR 历史提升值、Replay 下载、动态 sitemap/SSR 尚未实现。
- 公开缓存失效端点仅用单一维护令牌，不是完整管理员认证；应在认证系统落地后收紧。
- Docker 启动/镜像构建、多副本限流、PostgreSQL 高可用与备份恢复演练未在当前环境执行。

## 5. 验收矩阵

| 验收项 | 结果 |
| --- | --- |
| Rust 编译（debug/release） | 通过 |
| SQLx migrations（全新库） | 通过（10 个迁移） |
| PostgreSQL / Redis 连接 | 通过（本地实例） |
| 管理后台构建 | 通过 |
| 公开门户构建 | 通过（sitemap/robots 生成） |
| Docker Compose 完整启动 | **未执行**：环境无 docker |
| 主要 API 响应 | 通过（10/10 端点 200） |
| 主要页面加载 | 通过（Playwright 公开门户 36/36、后台 38/38、统一入口 13/13、排行榜 8/8） |
| 前端页面级测试 | 通过（公开门户 14 个，覆盖页面渲染、API 成功/失败、空数据、玩家详情、地图详情、比较） |
| 数据统计一致性 | 通过（fixture 验证 overall/scoped/tier/map/records/WR/leaderboard） |
| 缓存失效 | 通过（版本 1→2，插入/删除记录后 invalidate 后统计同步变化） |
| 限流 | 通过（单元测试 + 集成 429 + Retry-After） |
| 安全基础 | 路径穿越修复、维护令牌保护、无密钥入库；完整认证仍缺失 |
| 浏览器控制台 | 0 严重错误 |
