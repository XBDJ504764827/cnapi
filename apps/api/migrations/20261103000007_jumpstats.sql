-- 0007_jumpstats
--
-- Jumpstat submissions.
--
-- Source findings:
--   * GOKZ jumpstats (include/gokz/jumpstats.inc): Jump struct with type,
--     distance (float game units), strafes, sync (%), pre/max speed, block,
--     duration (ticks), deadair, overlap, deviation, height, width.
--     GOKZ local DB Jumpstats stores Distance/Strafes/Sync/Pre/Max/Airtime as
--     integers and IsBlockJump/Block.
--   * GlobalAPI v2 POST /jumpstats parameters: steamId, jump_type (int),
--     distance (float), jump_json_info (string blob), tickrate, msl_count,
--     is_crouch_bind, is_forward_bind, is_crouch_boost, strafe_count.
--     APIJumpstat response: id, server_id, steamid64, steam_id, player_name,
--     jump_type (int), distance, tickrate, msl_count, strafe_count,
--     is_crouch_bind, is_forward_bind, is_crouch_boost, updated_by_id,
--     created_on, updated_on.
--   * cs2kz Jumps: jump type, mode, styles bitmask, time (seconds), strafes,
--     distance, sync, pre, max, overlap, bad_angles, dead_air, height, airpath,
--     deviation, average_width (all floats).
--
-- Design decisions:
--   * Only metrics with agreed semantics across sources are promoted to columns
--     (distance, strafe count, sync %, airtime). The remaining source-specific
--     metrics (pre/max/overlap/bad_angles/height/...) are kept verbatim in
--     `raw_metrics` until their units are confirmed. This avoids inventing
--     columns whose meaning would differ between GOKZ and cs2kz.
--   * `distance_units` and percentage columns are NUMERIC, not float, so ranking
--     and filtering are exact.

CREATE TABLE jumpstats (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts (id),
    map_id BIGINT NOT NULL REFERENCES maps (id),
    server_id BIGINT REFERENCES servers (id) ON DELETE SET NULL,
    mode_id SMALLINT NOT NULL REFERENCES movement_modes (id),
    style_id SMALLINT NOT NULL REFERENCES movement_styles (id),
    jump_type_id SMALLINT NOT NULL REFERENCES jump_types (id),
    distance_units NUMERIC(12, 4) NOT NULL,
    strafe_count SMALLINT,
    sync_percent NUMERIC(6, 3),
    airtime_ns BIGINT,
    block INTEGER,
    is_block_jump BOOLEAN NOT NULL DEFAULT FALSE,
    msl_count SMALLINT,
    is_crouch_bind BOOLEAN NOT NULL DEFAULT FALSE,
    is_forward_bind BOOLEAN NOT NULL DEFAULT FALSE,
    is_crouch_boost BOOLEAN NOT NULL DEFAULT FALSE,
    tickrate SMALLINT NOT NULL,
    source TEXT NOT NULL DEFAULT 'plugin',
    submission_id UUID,
    external_ref TEXT,
    raw_metrics JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT ck_jumpstats_distance CHECK (distance_units >= 0),
    CONSTRAINT ck_jumpstats_strafe_count CHECK (strafe_count IS NULL OR strafe_count >= 0),
    CONSTRAINT ck_jumpstats_sync CHECK (sync_percent IS NULL OR sync_percent BETWEEN 0 AND 100),
    CONSTRAINT ck_jumpstats_airtime CHECK (airtime_ns IS NULL OR airtime_ns > 0),
    CONSTRAINT ck_jumpstats_block CHECK (block IS NULL OR block >= 0),
    CONSTRAINT ck_jumpstats_msl CHECK (msl_count IS NULL OR msl_count >= 0),
    CONSTRAINT ck_jumpstats_tickrate CHECK (tickrate BETWEEN 1 AND 1000),
    CONSTRAINT ck_jumpstats_source CHECK (source IN ('plugin', 'admin', 'globalapi_import', 'cs2kz_import', 'legacy_import')),
    CONSTRAINT ck_jumpstats_raw_metrics CHECK (jsonb_typeof(raw_metrics) = 'object')
);

COMMENT ON TABLE jumpstats IS 'Jumpstat measurements submitted by servers or imported from GlobalAPI/cs2kz';
COMMENT ON COLUMN jumpstats.distance_units IS 'Jump distance in game units (float in sources, stored as exact NUMERIC)';
COMMENT ON COLUMN jumpstats.sync_percent IS 'Strafe sync percentage (GOKZ sync / cs2kz sync), 0-100';
COMMENT ON COLUMN jumpstats.airtime_ns IS 'Air time in nanoseconds when the source unit is known; NULL if it cannot be converted';
COMMENT ON COLUMN jumpstats.block IS 'Block number for block jumps (GOKZ Jump.block)';
COMMENT ON COLUMN jumpstats.raw_metrics IS 'Verbatim source metrics (GlobalAPI jump_json_info / cs2kz jump fields) pending unit confirmation';
COMMENT ON COLUMN jumpstats.submission_id IS 'Idempotency key for CNGOKZ plugin retries; unique per server';
COMMENT ON COLUMN jumpstats.external_ref IS 'Upstream jumpstat id when imported';

CREATE UNIQUE INDEX uq_jumpstats_submission
    ON jumpstats (server_id, submission_id)
    WHERE submission_id IS NOT NULL AND server_id IS NOT NULL;

CREATE UNIQUE INDEX uq_jumpstats_external_ref
    ON jumpstats (source, external_ref)
    WHERE external_ref IS NOT NULL;

-- Top lists: per jump type and per map/jump type.
CREATE INDEX idx_jumpstats_top
    ON jumpstats (jump_type_id, distance_units DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_jumpstats_map_top
    ON jumpstats (map_id, jump_type_id, distance_units DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_jumpstats_account
    ON jumpstats (account_id, created_at DESC)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_jumpstats_server
    ON jumpstats (server_id, created_at DESC)
    WHERE deleted_at IS NULL;

CREATE TRIGGER trg_jumpstats_updated_at
    BEFORE UPDATE ON jumpstats
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
