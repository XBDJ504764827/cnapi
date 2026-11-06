-- 20261106000002_player_avatar_sync
--
-- Steam 头像 / 昵称的后台增量同步。
--
-- 依据：生产服务器无法直连 api.steampowered.com，头像只能经由中转 Worker 的
-- `/profiles`（一次最多 100 个 SteamID）批量拉取。为避免每个请求都去问一次，
-- 这里记录「最后一次成功拉取 Steam 资料的时间」，worker 只处理缺失或过期的行。

ALTER TABLE player_profiles
    ADD COLUMN avatar_synced_at TIMESTAMPTZ;

COMMENT ON COLUMN player_profiles.avatar_synced_at IS 'Last successful Steam profile fetch (persona + avatar); NULL = never synced';

-- 后台扫描条件是「NULL 或早于 N 天前」，NULLS FIRST 与 worker 的排序一致。
CREATE INDEX idx_player_profiles_avatar_synced_at
    ON player_profiles (avatar_synced_at NULLS FIRST);
