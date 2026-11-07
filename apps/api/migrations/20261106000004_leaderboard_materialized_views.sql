-- 20261106000004_leaderboard_materialized_views
--
-- 排行榜 / 玩家目录 / account_rank 的聚合从 records（160 万行）挪到物化视图。
--
-- 背景：2026-11 一次性导入 159 万条成绩后，排行榜与玩家页从"秒开"变成 3~21 秒，
-- 因为 stats_cte 原先对每个账号跑四个相关子查询，改成一次 GROUP BY 后仍要扫
-- 160 万行。玩家只有几百人，把"每人每模式每类型"的聚合物化下来，查询就只剩
-- 几百行。
--
-- 刷新时机：记录发生变化（导入、回填、成绩入库）之后调用
-- `REFRESH MATERIALIZED VIEW CONCURRENTLY`；积分重算 worker 在队列排空时会顺带
-- 刷新（带冷却），不需要额外的定时任务。

CREATE MATERIALIZED VIEW mv_player_board AS
SELECT r.account_id,
       r.mode_id,
       r.time_type,
       count(*)::bigint AS total_records,
       count(DISTINCT r.map_id) FILTER (WHERE mc.id IS NOT NULL)::bigint AS completed_maps,
       max(r.created_at) AS last_activity_at
  FROM records r
  LEFT JOIN map_courses mc ON mc.id = r.course_id AND mc.stage = 0
 WHERE r.status = 'approved'
   AND r.deleted_at IS NULL
 GROUP BY r.account_id, r.mode_id, r.time_type;

-- CONCURRENTLY 刷新需要唯一索引
CREATE UNIQUE INDEX uq_mv_player_board
    ON mv_player_board (account_id, mode_id, time_type);
CREATE INDEX idx_mv_player_board_mode
    ON mv_player_board (mode_id, time_type);

COMMENT ON MATERIALIZED VIEW mv_player_board IS 'Per-account/per-mode/per-time-type record aggregates used by leaderboards and the player directory';

CREATE MATERIALIZED VIEW mv_player_wr AS
SELECT wr.account_id,
       wr.mode_id,
       wr.time_type,
       count(*)::bigint AS wr_count
  FROM course_world_records w
  JOIN records wr ON wr.id = w.record_id
 WHERE wr.status = 'approved'
   AND wr.deleted_at IS NULL
 GROUP BY wr.account_id, wr.mode_id, wr.time_type;

CREATE UNIQUE INDEX uq_mv_player_wr
    ON mv_player_wr (account_id, mode_id, time_type);
CREATE INDEX idx_mv_player_wr_mode
    ON mv_player_wr (mode_id, time_type);

COMMENT ON MATERIALIZED VIEW mv_player_wr IS 'Per-account/per-mode/per-time-type world-record counts';
