# 玩家积分与排行榜系统（第六阶段）

## 1. 审查结论与业务定义

| 项目 | 实际规则 |
| --- | --- |
| SteamID64 | `accounts.steam_id64 NUMERIC(20,0)`，uint64 CHECK；内部主键为 `accounts.id` |
| 模式 | `movement_modes` 有 vanilla/simple/kztimer/classic；本阶段 `scoring_modes` 配置 KZT→kztimer、SKZ→simple、VNL→vanilla，classic 不参与 |
| Style | 当前只有 `normal`；Filter 定义不含 Style，同 Filter 内所有 Style 取最快 PB |
| Course | `map_courses.stage`：0=Main，>0=Bonus；Bonus 是独立课程与独立 Filter |
| Tier | 新增权威表 `course_mode_tiers(map,course,mode,tier,completed,...)`；T1–T8；T9 仅是首通历史标签 |
| 成绩有效 | `status='approved' AND deleted_at IS NULL`（沿用既有规则，未修改） |
| PRO / NUB | DB CHECK 强制：PRO ⇔ teleports=0；NUB ⇔ teleports>0 |
| 时间 | 跑图时长 `records.time_ns`（纳秒）；上传时间 `created_at`；积分公式统一换算为秒 |
| PB | 每个玩家每个 Filter 取 `min(time_ns)`，记录 `pb_record_id` |
| WR | 重算时由该 Filter 有效 PB 集合的 `min(time_ns)` 得到，不直接信任旧展示指针 |
| 原积分字段 | `player_mode_stats.points/rating` 是阶段 2 预留，未用于本阶段；新总分写入独立表 `player_mode_points` |
| 模拟数据 | 无；所有积分来自有效成绩重算 |

## 2. Filter

```text
Filter = map_id + course_id + mode + record_type(NUB/PRO)
```

`scoring_filters` 唯一约束 `(map_id,course_id,mode_id,record_type)`。每个 Filter 独立计算 PB、WR、rank、成绩分布和积分；不同模式/课程/NUB-PRO 不混合。

## 3. 公式（纯 Rust 模块 `services/scoring/formula.rs`）

- 基础分：T1–T8 的 NUB/PRO 映射表；T9 等于 T8。
- 排名系数：`r=rank-1`，前100/前20/前5 三段；rank1=1.0，rank>=101=0。
- 分布：`x=2.1-0.25*min(tier,8)`，`y=1+exp(-0.5x)`，`z=1+exp(x*(time/wr-1.5))`，`dist=clamp(y/z,0,1)`。N 不参与第一版结果。
- 最终：`points=round(base + 0.125*(1000-base)*rank_factor + 0.875*(1000-base)*dist)`，clamp 0–1000。
- WR（rank=1、time=wr）强制 1000，不论 T1–T8 或 NUB/PRO。

## 4. T9 首通

- 仅 VNL（`scoring_modes.t9_enabled`）。
- `course_mode_tiers.completed=FALSE` 时首个有效成绩写入 `filter_first_completions` 历史标签，并立即把该 `(map,course,mode)` 置为 `tier=8, completed=TRUE`。
- KZT/SKZ 的等级与首通状态不受影响。
- T9 标签不绑定永久 1000 分；后续 WR 变化会重算原首通玩家积分。
- 首通成绩失效后的地图状态恢复：当前项目无写接口/规则，按需求列为待确认，不实现回滚。

## 5. 重算

- DB 触发器 `trg_records_enqueue_recalc` 在成绩插入/有效性/时间/维度变更时只标记受影响 Filter 并入队；等级变更触发器标记同 `(map,course,mode)` 的 Filter。
- Tokio worker 用 `FOR UPDATE SKIP LOCKED` 领取任务，避免同一 Filter 并发重复重算。
- `recalc_filter` 在一个事务内：锁 Filter → SQL CTE 聚合所有玩家 PB / WR / rank / pb_count → 纯公式逐行计算 → 批量删除旧积分并单条 INSERT 批量写入 → 重建该模式 `player_mode_points` → 更新 Filter 版本/时间 → 提交。
- 只更新同模式的玩家总分；KZT 重算不触碰 SKZ/VNL。
- 任务状态：pending/running/failed（done 行完成后删除，保留 `filter_recalc_runs` 审计）。失败可重新入队，重算幂等。
- 非 PB 成绩不会触发重算（触发器按受影响维度入队，worker 按当前 PB 集合重算；同 Filter 重复入队由唯一 pending 约束合并）。
- 历史成绩初始化：受保护端点 `POST /api/v1/admin/points/initialize`，不回填 T9 历史标签，把现有完成状态仅作为审计锚点。

## 6. 数据库（迁移 `20261104000002_points_system.sql`）

新增表：`scoring_modes`、`course_mode_tiers`、`scoring_filters`、`player_filter_points`、`player_mode_points`、`filter_recalc_tasks`、`filter_recalc_runs`、`filter_first_completions`。
索引：dirty 任务、模式/Tier、Filter 唯一、玩家积分、Filter rank、模式总分排名等。

## 7. API

| 路径 | 说明 |
| --- | --- |
| `GET /api/v1/players/{steamid}/points` | KZT/SKZ/VNL 的 PRO/NUB/总分 |
| `GET /api/v1/players/{steamid}/points/filters` | 明细，支持 mode/record_type/tier/course/map/排序/分页 |
| `GET /api/v1/leaderboards/points/{mode}` | 单模式积分榜 |
| `GET /api/v1/leaderboards/filters` | 单 Filter 积分榜 |
| `GET /api/v1/admin/points/status` | 任务状态（维护令牌） |
| `GET /api/v1/admin/points/tiers` | 等级/首通（维护令牌） |
| `PATCH /api/v1/admin/points/tiers/{map}/{course}/{mode}` | 改等级，触发重算（维护令牌） |
| `GET /api/v1/admin/points/recalc-runs` | 重算记录（维护令牌） |
| `POST /api/v1/admin/points/recalc/filter/{id}` | 单 Filter 入队（维护令牌） |
| `POST /api/v1/admin/points/recalc/mode/{mode}` | 单模式全量入队（维护令牌） |
| `POST /api/v1/admin/points/initialize` | 历史成绩初始化（维护令牌） |

未配置 `MAINTENANCE_TOKEN` 时管理接口 403；令牌错误 401。公开只读接口不需要令牌。

## 8. 前端

- 公开门户：新增 `/points`（KZT/SKZ/VNL 独立积分榜）、`/players/:steamid/points`（三模式总分与 Filter 明细），玩家详情增加三模式积分卡，页头/页脚加入积分榜入口。
- 管理后台：新增 `/admin/points`（积分概览 + 三模式榜单 + 最近重算）、`/admin/points/tiers`（等级管理）、`/admin/points/recalc`（任务管理）、`/admin/players/:id/points`（玩家明细）。
- 前台不计算积分；所有数值来自后端 API。

## 9. 缓存

按模式维护 `points:mode:{mode_id}:version`。Filter 重算成功后仅 `INCR` 该模式版本；不会清除其他模式的积分缓存。Redis 故障时仍从 PostgreSQL 读取。

## 10. 待确认

1. 同成绩时间的排名：沿用现有 SQL `RANK()` 竞争排名；未新增 tie-break 规则。
2. 首通成绩失效后 `completed` 是否恢复为 FALSE：当前无成绩删除/无效化写接口，需求明确不默认新增回滚；待确认。
3. 历史成绩首次初始化是否追溯 T9 标签：本阶段默认不追溯，避免伪造历史首通。
4. 封禁玩家成绩是否排除：当前无规则，不自行发明。
5. Style 未来多值后是否纳入 Filter：当前只有 normal，待产品确认。
6. N>50 拟合分布：需求明确禁止，第一版不实现。
