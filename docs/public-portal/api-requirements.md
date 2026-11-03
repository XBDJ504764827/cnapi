# 公开门户 · API 与数据方案

所有接口位于 `/api/v1/public`，统一响应信封，已纳入 OpenAPI（`/api-docs/openapi.json`、Swagger UI）。列表最多 100 条/页。

## 1. 端点

| 方法 | 路径 | 参数（query） | 返回 |
| --- | --- | --- | --- |
| GET | `/statistics` | — | 玩家/地图/课程/成绩/WR/在线服务器计数 |
| GET | `/modes` | — | movement_modes 引用数据 |
| GET | `/leaderboards` | `mode`、`time_type`（all/nub/pro）、`page`、`page_size` | 排名（完成地图数） |
| GET | `/players` | `search`、`mode`、`sort`（completion/wr/records/recent/name）、分页 | 玩家目录 |
| GET | `/players/active` | `days`（1-30）、`limit`（1-20） | 本周提交最多玩家 |
| GET | `/players/compare` | `left`、`right`、`mode`、`time_type`、`style`、`filter`、分页 | 双人对比 |
| GET | `/players/{steamid}` | `mode`、`time_type` | 资料/综合/模式/难度进度 |
| GET | `/players/{steamid}/maps` | `mode`、`time_type`、`style`、`status`、`search`、`tier`、`sort`、分页 | 全部/已完成/未完成地图 |
| GET | `/players/{steamid}/records` | `mode`、`time_type`、`style`、分页 | 成绩历史 |
| GET | `/maps` | `search`、`mode`、`time_type`、`tier`、`sort`、分页 | 地图列表 |
| GET | `/maps/{slug}` | `mode`、`time_type` | 地图详情 + 课程 + 统计 + 模式 |
| GET | `/maps/{slug}/leaderboard` | `mode`、`time_type`、`stage`、`style`、分页 | 课程排行榜 |
| GET | `/maps/{slug}/statistics` | `mode`、`time_type` | 完成统计 |
| GET | `/records` | `player`、`map`、`mode`、`style`、`time_type`、`tier`、`world_record`、`from`、`to`、`sort`、分页 | 成绩筛选 |
| GET | `/records/recent` | `limit`（1-50，默认 20）、`mode` | 最近上传 |
| GET | `/records/world-records` | `limit`、`mode`、`time_type` | 最近 WR 指针 |
| GET | `/records/{id}` | — | 成绩详情（含服务器、replay 标记） |
| GET | `/servers` | `search`、`online`、分页 | 服务器列表 |
| GET | `/servers/{id}` | — | 服务器详情 |
| GET | `/search` | `q`（2-64 字符） | 玩家 + 地图各最多 5 条 |

参数校验：分页 `page>=1`、`1<=page_size<=100`；`time_type ∈ {all,nub,pro}`；`tier ∈ {0,1..10}`；日期 RFC3339；`left != right`；模式/风格必须存在（404）；未知排序值返回 400。

## 2. 数据准确性与定义

| 概念 | 定义 | 实现要点 |
| --- | --- | --- |
| 完成地图 | 指定模式下主线课程（`stage=0`）存在 approved 成绩 | `count(DISTINCT map_id)`，不把 Bonus 当主线 |
| 完成率 | 完成地图 / 支持该模式的已上线地图 | 分母 `map_modes` + `status='approved'`；0 分母返回 0 |
| 未完成地图 | 已上线、模式支持、但玩家无主线成绩的地图 | `maps LEFT JOIN LATERAL best`，不是只查已有成绩 |
| 最佳成绩 | 同地图主线、mode/style/time_type 下最快 | `LEFT JOIN LATERAL ... ORDER BY time_ns LIMIT 1` |
| 地图排名 | 同一 course/mode/style/time_type 更快成绩数 + 1 | 相关子查询，命中 `idx_records_leaderboard` |
| PRO / NUB | `time_type` 字段（PRO ⇔ 0 传送，由 DB CHECK 保证） | 不根据 UI 推断 |
| 完成时间 | `time_ns` 跑图时长 | API 同时给纳秒与十进制秒字符串 |
| 上传时间 | `created_at` | 最近成绩排序字段 |
| WR | `course_world_records` 当前指针 | 历史 WR 状态未存 |
| 地图难度 | `course_tiers` 主线的 `nub_tier` / `pro_tier` | 按请求的 time_type 选择列；all → NUB 列 |
| 积分/Rating | **未定义** | 返回 `null` + `points_available=false` |

## 3. SQL 与性能

- **批量聚合**：玩家列表/排行榜使用 `WITH stats ... RANK() OVER`；每个账号的完成数、WR、成绩数、最近活动通过子查询/`FILTER` 一次算出。
- **地图进度**：`map_base` 取全部地图，`LEFT JOIN LATERAL` 取玩家最佳主线成绩；`bonus_completed` 单独聚合。没有“先拉全部成绩再在 Rust 循环”的实现。
- **对比**：同一 `map_base` 上分别 LATERAL 两个玩家，SQL 侧计算筛选与计数，分页只取当前页。
- **避免 N+1**：列表/详情中的玩家、地图、模式均在 SQL JOIN；唯一的多查询是玩家各模式排名（模式数量固定且很小）与地图详情分区（courses/stats/support modes）。
- **索引依赖**：`idx_records_leaderboard`、`idx_records_account`、`idx_records_map`、`idx_records_server`、`uq_accounts_steam_id64_active`、`uq_maps_name_active`、`uq_map_courses_id_map`、`idx_jumpstats_*`（本阶段未使用 Jumpstats）。
- **分页**：所有列表使用 `LIMIT/OFFSET` + 独立 `count(*)`；最大 100/页。

## 4. Redis 缓存

TTL 表见 [architecture.md](architecture.md#7-redis-缓存)。要点：

- 缓存键包含 `public:version`（Redis `INCR` 全局失效），以及全部查询参数；不会把不同筛选结果混用。
- `statistics`、`servers` 等“实时性”数据 TTL 较短（15–30s）。
- 缓存失败不阻塞请求：Redis 连接 300ms 超时、0 重试、失败后 5 秒快速失败，直接查询 PostgreSQL。
- 阶段 5 的写入端点（成绩审核/导入、服务器心跳、封禁）必须在提交后调用 `public::invalidate_cache`。

## 5. 缺失字段 / 未实现能力

| 能力 | 现状 | 建议 |
| --- | --- | --- |
| 积分算法 | 未实现 | 先制定规则与数据来源；不要在前端硬编码 |
| 玩家头像 | 无字段 | 同步 Steam 头像 URL |
| 地图作者、封面 | 无表/字段 | `map_authors` + 对象存储字段 |
| 服务器玩家数/最大人数/模式 | 无字段 | 插件心跳增加字段 |
| WR 历史提升 | 无历史表 | `world_record_history` |
| Replay 下载 | 无公开端点 | 签名 URL + 权限/保留策略 |
| Jumpstats 公开页 | 本阶段未实现（需求允许后续） | 复用 `jumpstats` 表与索引 |
| 动态 sitemap | 仅核心路由 | 后端动态 sitemap 或 SSR |

## 6. 测试

- `apps/api/tests/public_api.rs`：临时数据库 + 迁移 + 真实 fixtures，覆盖统计、排行榜、玩家详情/地图进度、对比、地图列表/详情/榜/统计、成绩筛选（tier/WR/时间）、最近成绩排序、活跃玩家、服务器在线、搜索、成绩详情、参数校验与 404。
- `apps/public/src/lib/format.test.ts`、`tier-badge.test.tsx`：前端格式与难度标识。
- 运行：

```bash
DATABASE_URL=postgres://user:pass@localhost:5432/db \
REDIS_URL=redis://localhost:6379 \
cargo test --test public_api
```
