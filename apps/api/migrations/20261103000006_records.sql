-- 0006_records
--
-- Runs (records), world-record pointers and the record audit trail.
--
-- Source findings:
--   * GOKZ local DB Times: SteamID32, MapCourseID, Mode, Style, RunTime (integer),
--     Teleports, Created. `RunTime` is stored as an integer; GOKZ runtime values
--     are float seconds in the plugin, so the local table is a lossy legacy copy.
--     GOKZ_GetTimeTypeEx(): teleports == 0 => TimeType_Pro, otherwise TimeType_Nub.
--   * GlobalAPI v1 POST /records body: {steam_id (Steam2), map_id, mode (string),
--     stage, tickrate, teleports, time (float seconds)}. Response:
--     {place, top_100, top_100_overall, record_id}.
--     GlobalAPI v2 APIRecord: id, steamid64, steam_id, player_name, server_id,
--     map_id, stage, mode, tickrate, time (float), teleports, created_on,
--     updated_on, updated_by, server_name, map_name.
--   * cs2kz Records: id (UUID), player_id, server_id, filter_id, styles (bitmask),
--     teleports, time FLOAT8 seconds, plugin_version_id. NUB/PRO leaderboards are
--     separate tables (BestNubRecords / BestProRecords) and CourseFilters encode
--     mode + tickrate + has_teleports.
--
-- Design decisions:
--   * Time is stored as `time_ns BIGINT` (nanoseconds). Source data is float
--     seconds; converting to an integer avoids float comparison and ordering
--     errors. 128-tick boundaries (7.8125 ms) are exactly representable in ns.
--   * NUB/PRO is stored explicitly as `time_type` and cross-checked against
--     `teleports` by a CHECK constraint, because both ecosystems derive it that
--     way (GOKZ TimeType, GlobalAPI has_teleports).
--   * `time_type` and `style_id` are part of the leaderboard key instead of
--     materialising a "filter" row per combination (cs2kz approach). This keeps
--     the model smaller while still allowing per-filter tiers and state.
--   * map_id is denormalized on records for query performance, but enforced
--     against map_courses through a composite foreign key so it cannot drift.

CREATE TABLE records (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts (id),
    map_id BIGINT NOT NULL REFERENCES maps (id),
    course_id BIGINT NOT NULL,
    mode_id SMALLINT NOT NULL REFERENCES movement_modes (id),
    style_id SMALLINT NOT NULL REFERENCES movement_styles (id),
    server_id BIGINT REFERENCES servers (id) ON DELETE SET NULL,
    server_plugin_version TEXT,
    tickrate SMALLINT NOT NULL,
    teleports INTEGER NOT NULL,
    time_type TEXT NOT NULL,
    time_ns BIGINT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    source TEXT NOT NULL DEFAULT 'plugin',
    submission_id UUID,
    external_ref TEXT,
    reviewed_by_account_id BIGINT REFERENCES accounts (id) ON DELETE SET NULL,
    reviewed_at TIMESTAMPTZ,
    review_note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT fk_records_course_map FOREIGN KEY (course_id, map_id)
        REFERENCES map_courses (id, map_id) ON DELETE RESTRICT,
    CONSTRAINT ck_records_tickrate CHECK (tickrate BETWEEN 1 AND 1000),
    CONSTRAINT ck_records_teleports CHECK (teleports >= 0),
    CONSTRAINT ck_records_time_type CHECK (time_type IN ('nub', 'pro')),
    CONSTRAINT ck_records_time_ns CHECK (time_ns > 0),
    CONSTRAINT ck_records_status CHECK (status IN ('pending', 'approved', 'rejected')),
    CONSTRAINT ck_records_source CHECK (source IN ('plugin', 'admin', 'globalapi_import', 'cs2kz_import', 'legacy_import')),
    -- GOKZ: 0 teleports => PRO, anything else => NUB (GOKZ_GetTimeTypeEx).
    CONSTRAINT ck_records_time_type_matches_teleports CHECK (
        (time_type = 'pro' AND teleports = 0)
        OR (time_type = 'nub' AND teleports > 0)
    )
);

COMMENT ON TABLE records IS 'Accepted/submitted KZ runs; one row per run, best runs are derived';
COMMENT ON COLUMN records.time_ns IS 'Run duration in nanoseconds; sources send float seconds (GlobalAPI time, cs2kz Seconds)';
COMMENT ON COLUMN records.time_type IS 'NUB (teleports > 0) or PRO (teleports = 0), derived per GOKZ_GetTimeTypeEx';
COMMENT ON COLUMN records.tickrate IS 'Server tickrate at submission time; GlobalAPI sends it per record, GOKZ plugin hardcodes 128';
COMMENT ON COLUMN records.status IS 'Review state: pending -> approved|rejected. Soft delete uses deleted_at instead of a status';
COMMENT ON COLUMN records.source IS 'Ingestion channel; used to resolve external_ref uniqueness and conversions';
COMMENT ON COLUMN records.submission_id IS 'CNGOKZ plugin submission id for idempotent retries; unique per server';
COMMENT ON COLUMN records.external_ref IS 'Upstream identifier (e.g. GlobalAPI record id or cs2kz UUID) for imports';
COMMENT ON COLUMN records.reviewed_by_account_id IS 'Staff account that manually reviewed the run; approved imports may have no reviewer';

-- Idempotency: a retried plugin submission with the same id must not create a
-- second record. Imports use (source, external_ref) instead.
CREATE UNIQUE INDEX uq_records_submission
    ON records (server_id, submission_id)
    WHERE submission_id IS NOT NULL AND server_id IS NOT NULL;

CREATE UNIQUE INDEX uq_records_external_ref
    ON records (source, external_ref)
    WHERE external_ref IS NOT NULL;

-- Leaderboard / WR lookup: exact integer ordering on time_ns.
CREATE INDEX idx_records_leaderboard
    ON records (course_id, mode_id, style_id, time_type, time_ns)
    WHERE status = 'approved' AND deleted_at IS NULL;

CREATE INDEX idx_records_account
    ON records (account_id, created_at DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_records_map
    ON records (map_id, created_at DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_records_server
    ON records (server_id, created_at DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_records_review_queue
    ON records (created_at)
    WHERE status = 'pending' AND deleted_at IS NULL;

CREATE TRIGGER trg_records_updated_at
    BEFORE UPDATE ON records
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ---------------------------------------------------------------------------
-- record_history
-- ---------------------------------------------------------------------------
CREATE TABLE record_history (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    record_id BIGINT NOT NULL REFERENCES records (id) ON DELETE CASCADE,
    action TEXT NOT NULL,
    old_status TEXT,
    new_status TEXT,
    old_time_ns BIGINT,
    new_time_ns BIGINT,
    changed_by_account_id BIGINT REFERENCES accounts (id) ON DELETE SET NULL,
    reason TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_record_history_action CHECK (
        action IN ('created', 'status_changed', 'time_corrected', 'reviewed', 'soft_deleted', 'restored', 'other')
    ),
    CONSTRAINT ck_record_history_metadata CHECK (jsonb_typeof(metadata) = 'object')
);

COMMENT ON TABLE record_history IS 'Append-only audit trail for record changes (review, corrections, soft delete)';

CREATE INDEX idx_record_history_record ON record_history (record_id, created_at DESC);
CREATE INDEX idx_record_history_actor ON record_history (changed_by_account_id, created_at DESC);

-- ---------------------------------------------------------------------------
-- course_world_records
-- ---------------------------------------------------------------------------
-- Current world record pointer per (course, mode, style, time type).
--
-- Concurrent updates must use a conditional upsert so a slower run can never
-- overwrite a faster one:
--
--   INSERT INTO course_world_records (...)
--   VALUES (...)
--   ON CONFLICT (course_id, mode_id, style_id, time_type) DO UPDATE
--   SET record_id = EXCLUDED.record_id,
--       time_ns   = EXCLUDED.time_ns,
--       achieved_at = EXCLUDED.achieved_at
--   WHERE EXCLUDED.time_ns < course_world_records.time_ns;
CREATE TABLE course_world_records (
    course_id BIGINT NOT NULL REFERENCES map_courses (id) ON DELETE CASCADE,
    mode_id SMALLINT NOT NULL REFERENCES movement_modes (id) ON DELETE CASCADE,
    style_id SMALLINT NOT NULL REFERENCES movement_styles (id) ON DELETE CASCADE,
    time_type TEXT NOT NULL,
    record_id BIGINT NOT NULL REFERENCES records (id) ON DELETE CASCADE,
    time_ns BIGINT NOT NULL,
    achieved_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (course_id, mode_id, style_id, time_type),
    CONSTRAINT ck_course_world_records_time_type CHECK (time_type IN ('nub', 'pro')),
    CONSTRAINT ck_course_world_records_time_ns CHECK (time_ns > 0)
);

COMMENT ON TABLE course_world_records IS 'World record pointer per course/mode/style/time type; update with conditional upsert';
COMMENT ON COLUMN course_world_records.time_ns IS 'Cached winning time; must match the referenced record time (application invariant)';

CREATE INDEX idx_course_world_records_record ON course_world_records (record_id);

CREATE TRIGGER trg_course_world_records_updated_at
    BEFORE UPDATE ON course_world_records
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
