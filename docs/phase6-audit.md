# 第六阶段审查与验收记录

## 审查范围与结论

审查 `apps/api/migrations`、`records`、`movement_modes`、`movement_styles`、`map_courses`、`course_tiers`、`course_world_records`、`player_mode_stats`、公开/管理 API、Redis 缓存与两个前端。实际目录是 `apps/api`、`apps/web`、`apps/public`，无 `crates/`。

- PRO 严格 `teleports=0`（DB CHECK）；NUB `teleports>0`。
- 有效成绩 `approved` 且未软删除；封禁成绩无排除规则。
- 等级原先按 `(course,mode)` 且 NUB/PRO 两列，无首通状态；本阶段新增权威 `(map,course,mode)` 表。
- 无积分算法/历史积分数据；阶段 5 的 `points/rating` 预留列保持不用。
- 未在仓库发现“历史积分规范文件”；以本轮需求正文公式为准。

## 通过 / 未执行

见 `docs/changelog.md` 第六阶段记录与最终报告的测试章节。Docker 启动因环境无 docker 命令未执行；管理后台认证仍只有维护令牌，不是完整登录系统。
