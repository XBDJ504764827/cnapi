# 公开门户 · 页面清单

所有页面默认深色、支持浅色/跟随系统，采用顶部横向导航（移动端抽屉）。所有未实现字段显示 `—` 并附说明，不生成模拟数据。

| 路由 | 页面 | 主要区块 | 数据来源 | 交互 |
| --- | --- | --- | --- | --- |
| `/` | 首页 | 页面标题 + 全局搜索、全站统计、模式 Top10、最近 20 条成绩、最近 WR、本周活跃、在线服务器、口径说明 | `statistics`、`modes`、`leaderboards`、`records/recent`、`records/world-records`、`players/active`、`servers` | 模式 Tab、内联搜索、跳转详情、主题、登录占位 |
| `/leaderboards` | 全球排行榜 | 筛选控制台（模式分段/TP-PRO/每页/重置）、榜单事实条、排名行卡片（前三名强调）、分页 | `modes`、`leaderboards` | 模式/类型切换、页大小、重置、排名说明 Tooltip、分页 |
| `/players` | 玩家目录 | 搜索、模式/排序筛选、分页玩家表 | `players`、`modes` | 搜索提交、筛选、排序、分页、清空/重置 |
| `/players/:steamid` | 玩家详情 | 资料头、综合统计、模式卡、难度进度、地图完成/未完成、成绩历史 | `player`、`player/maps`、`player/records`、`modes` | 模式 Tab、TP/PRO Tab、地图状态 Tab、搜索/难度/排序、分页、成绩详情 Dialog、跳转比较 |
| `/compare` | 玩家比较 | 双玩家选择、综合对比、模式/难度对比、地图对比 | `search`、`players`、`compare`、`modes` | 玩家搜索选择、交换、快速选人、模式/TP-PRO/地图筛选、分页 |
| `/maps` | 地图列表 | 搜索、模式、难度类型/等级、排序、分页 | `maps`、`modes` | 模式 Tab、NUB/PRO、难度、排序、搜索、分页 |
| `/maps/:slug` | 地图详情 | 地图信息、统计卡、课程表、成绩分布、地图排行榜 | `map`、`map/leaderboard`、`modes` | 模式 Tab、TP/PRO、课程切换、分页 |
| `/records` | 成绩查询 | 玩家/地图/模式/TP-PRO/难度/WR/日期筛选、排序、分页、详情 Dialog | `records`、`modes`、`record/{id}` | 文本搜索、下拉筛选、日期范围、排序、分页、详情 |
| `/servers` | 服务器列表 | 搜索、在线筛选、服务器表 | `servers` | 搜索、在线状态筛选、分页 |
| `*` | 404 | 返回首页/玩家目录 | 无 | 路由 |

## 玩家详情统计范围（第五阶段）

- `overall`：全部模式、全部成绩类型、全部 Style 的账号级汇总，标记 `scoped=false`。
- `scoped` 与难度进度、地图进度：统一使用当前 `mode + time_type + 全部 Style`，标记 `scoped=true`；完成定义为「主线课程（stage 0）存在 approved 成绩」。
- 模式卡片：按模式汇总，明确属于全类型/全 Style 的分解视图，不与 `scoped` 混用。
- 排行榜与玩家排名：按 `mode + time_type` 且全部 Style 计算完成地图数；`style` 仅在地图进度/比较/地图榜中作为可选细化维度。
- 积分/Rating：算法未定义，所有页面为 `null`/`—`；不得伪造。

## 状态处理

- **Loading**：表格骨架、卡片骨架、玩家/地图详情整体骨架；不使用全屏转圈。
- **Empty**：带图标 + 标题 + 说明；未实现字段明确写出（如“积分算法未定义”）。
- **Error**：显示 API 安全错误消息 + 重试按钮；404 单独展示“未找到玩家/地图”。
- **URL 状态**：列表筛选/排序/分页写入查询参数；刷新/分享/后退保持。

## 数据来源约束

| 功能 | 来源 | 备注 |
| --- | --- | --- |
| 玩家头像 | 无 | schema 无 avatar；显示昵称首字母 + Steam 主页链接 |
| 地图封面/作者 | 无 | 显示占位与“未提供”，不伪造 |
| 服务器玩家数/最大人数/模式 | 无 | 显示 `—`，文档记录待采集 |
| 积分/Rating | 无算法 | `null`，页面显示 `—`；排名按完成地图数 |
| 完成时间 | `records.time_ns` | 跑图时长，不是上传时间 |
| 上传时间 | `records.created_at` | 最近成绩按此倒序 |
| WR 标记 | `course_world_records` | 仅当前 WR，历史状态未存储 |
| Replay | `replays.status='available'` | 仅标记可用，公开下载未开放 |

## 三模式与 TP/PRO

- 模式来自数据库 `movement_modes`（当前 seed：Vanilla、Simple KZ、KZTimer、Classic），前端**动态渲染**，不硬编码三种；这样新增模式无需改前端。
- NUB / PRO 是成绩类型（`records.time_type`），与模式是两个维度。UI 在不同页面分别提供模式 Tab 和 TP/PRO 过滤。
