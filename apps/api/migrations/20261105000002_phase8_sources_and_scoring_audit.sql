-- 0013: Phase 8 multi-source ingestion, raw record audit and scoring versioning
--
-- Design:
--   * `records` remains the canonical record table used by the existing scoring
--     engine. We do not create a second `canonical_records` table.
--   * Every incoming payload is preserved in `raw_records`; links keep the
--     provenance and matching method.
--   * External points/ranks are never stored in columns used by CNGOKZ scoring.
--   * All changes are additive.

-- ---------------------------------------------------------------------------
-- Data sources
-- ---------------------------------------------------------------------------
CREATE TABLE record_sources (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    source_type TEXT NOT NULL,
    display_name TEXT NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    priority INTEGER NOT NULL DEFAULT 100,
    status TEXT NOT NULL DEFAULT 'unknown',
    last_attempt_at TIMESTAMPTZ,
    last_success_at TIMESTAMPTZ,
    last_failure_at TIMESTAMPTZ,
    last_error TEXT,
    consecutive_failures INTEGER NOT NULL DEFAULT 0 CHECK (consecutive_failures >= 0),
    cursor JSONB NOT NULL DEFAULT '{}'::jsonb,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_record_sources_type CHECK (source_type IN ('gokz_global', 'kzgo', 'plugin', 'manual')),
    CONSTRAINT ck_record_sources_status CHECK (status IN ('healthy', 'degraded', 'offline', 'disabled', 'unknown')),
    CONSTRAINT ck_record_sources_cursor CHECK (jsonb_typeof(cursor) = 'object'),
    CONSTRAINT ck_record_sources_config CHECK (jsonb_typeof(config) = 'object')
);

COMMENT ON TABLE record_sources IS 'Multi-source registry; external points are never copied into CNGOKZ scoring columns';

CREATE TRIGGER trg_record_sources_updated_at
    BEFORE UPDATE ON record_sources
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

INSERT INTO record_sources (code, source_type, display_name, priority, status, config) VALUES
    ('gokz_global', 'gokz_global', 'GOKZ-Global / KZTimerGlobal', 10, 'unknown',
     '{"base_url":"https://kztimerglobal.com/api/v2.0"}'::jsonb),
    ('kzgo', 'kzgo', 'kzgo.eu', 20, 'unknown',
     '{"base_url":"https://kzgo.eu"}'::jsonb),
    ('cngokz_plugin', 'plugin', 'CNGOKZ SourceMod plugin', 100, 'unknown', '{}'::jsonb)
ON CONFLICT (code) DO NOTHING;

-- ---------------------------------------------------------------------------
-- One run per sync attempt (manual, incremental, backfill)
-- ---------------------------------------------------------------------------
CREATE TABLE source_sync_runs (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    source_id BIGINT NOT NULL REFERENCES record_sources (id) ON DELETE CASCADE,
    run_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'running',
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    stats JSONB NOT NULL DEFAULT '{}'::jsonb,
    error TEXT,
    CONSTRAINT ck_source_sync_runs_type CHECK (run_type IN ('manual', 'incremental', 'backfill', 'initial')),
    CONSTRAINT ck_source_sync_runs_status CHECK (status IN ('running', 'success', 'failed')),
    CONSTRAINT ck_source_sync_runs_stats CHECK (jsonb_typeof(stats) = 'object')
);

CREATE INDEX idx_source_sync_runs_source ON source_sync_runs (source_id, started_at DESC);

-- ---------------------------------------------------------------------------
-- Raw payloads. Never overwritten; duplicates are detected by source + id.
-- ---------------------------------------------------------------------------
CREATE TABLE raw_records (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    source_id BIGINT NOT NULL REFERENCES record_sources (id) ON DELETE CASCADE,
    external_record_id TEXT NOT NULL,
    payload_hash BYTEA NOT NULL,
    raw_payload JSONB NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    processing_status TEXT NOT NULL DEFAULT 'pending',
    processing_error TEXT,
    canonical_record_id BIGINT REFERENCES records (id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_raw_records_status CHECK (processing_status IN ('pending', 'processed', 'duplicate', 'rejected', 'failed')),
    CONSTRAINT ck_raw_records_payload CHECK (jsonb_typeof(raw_payload) = 'object'),
    CONSTRAINT uq_raw_records_source_external UNIQUE (source_id, external_record_id)
);

CREATE INDEX idx_raw_records_canonical ON raw_records (canonical_record_id);
CREATE INDEX idx_raw_records_status ON raw_records (processing_status, received_at);

CREATE TRIGGER trg_raw_records_updated_at
    BEFORE UPDATE ON raw_records
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ---------------------------------------------------------------------------
-- Provenance links. A canonical record can have one link per source.
-- ---------------------------------------------------------------------------
CREATE TABLE record_source_links (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    canonical_record_id BIGINT NOT NULL REFERENCES records (id) ON DELETE CASCADE,
    raw_record_id BIGINT NOT NULL REFERENCES raw_records (id) ON DELETE CASCADE,
    source_id BIGINT NOT NULL REFERENCES record_sources (id) ON DELETE CASCADE,
    external_record_id TEXT NOT NULL,
    source_recorded_at TIMESTAMPTZ,
    received_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    match_method TEXT NOT NULL,
    match_confidence NUMERIC(5, 4) NOT NULL DEFAULT 1.0,
    field_conflicts JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_record_source_links_method CHECK (match_method IN (
        'source_id', 'high_confidence_fields_time', 'fuzzy_pending_review', 'plugin_submission'
    )),
    CONSTRAINT ck_record_source_links_confidence CHECK (match_confidence >= 0 AND match_confidence <= 1),
    CONSTRAINT ck_record_source_links_conflicts CHECK (jsonb_typeof(field_conflicts) = 'object'),
    CONSTRAINT uq_record_source_links_raw UNIQUE (raw_record_id)
);

CREATE INDEX idx_record_source_links_canonical ON record_source_links (canonical_record_id);
CREATE INDEX idx_record_source_links_source ON record_source_links (source_id, external_record_id);
CREATE INDEX idx_record_source_links_recorded ON record_source_links (source_recorded_at);

-- ---------------------------------------------------------------------------
-- External map id mapping (KZGO/GlobalAPI ids -> canonical maps)
-- ---------------------------------------------------------------------------
CREATE TABLE map_source_links (
    source_id BIGINT NOT NULL REFERENCES record_sources (id) ON DELETE CASCADE,
    external_map_id TEXT NOT NULL,
    map_id BIGINT NOT NULL REFERENCES maps (id) ON DELETE CASCADE,
    external_map_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (source_id, external_map_id)
);

CREATE INDEX idx_map_source_links_map ON map_source_links (map_id);

CREATE TRIGGER trg_map_source_links_updated_at
    BEFORE UPDATE ON map_source_links
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ---------------------------------------------------------------------------
-- Snapshot of public kzgo.eu server states (display fallback source).
-- ---------------------------------------------------------------------------
CREATE TABLE external_servers (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    source_id BIGINT NOT NULL REFERENCES record_sources (id) ON DELETE CASCADE,
    external_id TEXT NOT NULL,
    name TEXT NOT NULL,
    label TEXT,
    host TEXT NOT NULL,
    port INTEGER NOT NULL CHECK (port BETWEEN 1 AND 65535),
    map_name TEXT,
    map_tier SMALLINT,
    online_players INTEGER NOT NULL DEFAULT 0 CHECK (online_players >= 0),
    max_players INTEGER NOT NULL DEFAULT 0 CHECK (max_players >= 0),
    is_online BOOLEAN NOT NULL DEFAULT FALSE,
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    raw_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_external_servers_tags CHECK (jsonb_typeof(tags) = 'array'),
    CONSTRAINT ck_external_servers_payload CHECK (jsonb_typeof(raw_payload) = 'object'),
    CONSTRAINT uq_external_servers_source_external UNIQUE (source_id, external_id)
);

CREATE INDEX idx_external_servers_online ON external_servers (source_id, is_online, last_seen_at DESC);

CREATE TRIGGER trg_external_servers_updated_at
    BEFORE UPDATE ON external_servers
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ---------------------------------------------------------------------------
-- Canonical record display source and scoring audit.
-- ---------------------------------------------------------------------------
ALTER TABLE records ADD COLUMN source_server_name TEXT;

COMMENT ON COLUMN records.source_server_name IS 'Server name snapshot from an external source; displayed when server_id is NULL';

CREATE INDEX idx_records_canonical_match
    ON records (account_id, map_id, course_id, mode_id, style_id, time_type, teleports, time_ns);

ALTER TABLE player_filter_points
    ADD COLUMN scoring_version TEXT NOT NULL DEFAULT 'phase6-v1',
    ADD COLUMN base_points DOUBLE PRECISION,
    ADD COLUMN rank_factor DOUBLE PRECISION,
    ADD COLUMN dist_points DOUBLE PRECISION;

COMMENT ON COLUMN player_filter_points.scoring_version IS 'CNGOKZ scoring formula version used for this row';
COMMENT ON COLUMN player_filter_points.base_points IS 'Audit: base points from the CNGOKZ formula';
COMMENT ON COLUMN player_filter_points.rank_factor IS 'Audit: rank factor from the CNGOKZ formula';
COMMENT ON COLUMN player_filter_points.dist_points IS 'Audit: distribution coefficient from the CNGOKZ formula';
