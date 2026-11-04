-- 00010: Phase 5 performance indexes
--
-- Only indexes that match actual public-portal query shapes are added. Every
-- index here was chosen against an EXPLAIN plan or an explicit query pattern;
-- see docs/public-portal/api-requirements.md (performance section).

-- SteamID64 exact lookup (login/search by id). The active unique index already
-- covers (steam_id64) for non-deleted rows; this expression index supports the
-- display-name ILIKE '%...%' fallback without a full table scan.
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX IF NOT EXISTS idx_player_profiles_display_name_trgm
    ON player_profiles USING gin (lower(display_name) gin_trgm_ops);

-- Map name ILIKE '%...%' search in /public/maps and /public/search.
CREATE INDEX IF NOT EXISTS idx_maps_name_trgm
    ON maps USING gin (lower(name) gin_trgm_ops)
    WHERE deleted_at IS NULL AND status = 'approved';

-- Recent uploads: (created_at DESC, id DESC) with the approved/not-deleted
-- predicate used by every public record query.
CREATE INDEX IF NOT EXISTS idx_records_recent_public
    ON records (created_at DESC, id DESC)
    WHERE status = 'approved' AND deleted_at IS NULL;

-- Per-player history and per-map history ordered by upload time.
CREATE INDEX IF NOT EXISTS idx_records_account_created
    ON records (account_id, created_at DESC, id DESC)
    WHERE status = 'approved' AND deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_records_map_created
    ON records (map_id, created_at DESC, id DESC)
    WHERE status = 'approved' AND deleted_at IS NULL;

-- Player completion checks repeatedly filter account_id + mode_id + style_id
-- and join to map_courses on stage=0.
CREATE INDEX IF NOT EXISTS idx_records_progress_lookup
    ON records (account_id, mode_id, style_id, course_id, time_ns)
    WHERE status = 'approved' AND deleted_at IS NULL;

-- Map/course leaderboards with optional time_type and stage filters.
CREATE INDEX IF NOT EXISTS idx_records_course_time
    ON records (course_id, mode_id, style_id, time_type, time_ns)
    WHERE status = 'approved' AND deleted_at IS NULL;

-- WR pointer lookups by achieved_at.
CREATE INDEX IF NOT EXISTS idx_course_world_records_achieved
    ON course_world_records (achieved_at DESC, updated_at DESC);

COMMENT ON INDEX idx_records_recent_public IS 'Recent uploads ordered by created_at DESC';
COMMENT ON INDEX idx_records_account_created IS 'Player record history ordered by upload time';
COMMENT ON INDEX idx_records_map_created IS 'Map record history ordered by upload time';
COMMENT ON INDEX idx_records_progress_lookup IS 'Player map progress best-run lookup';
COMMENT ON INDEX idx_records_course_time IS 'Course leaderboard ranking by exact nanosecond time';
