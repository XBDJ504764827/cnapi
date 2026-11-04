-- 0012: Phase 7 CS:GO game server ingestion
--
-- Extends the existing `servers` table instead of creating a parallel
-- `game_servers` table. Tokens are still stored hashed in `api_key_hash`; the
-- plaintext token is returned only once by the admin API and is never logged.
--
-- This migration is additive: existing records, servers and references are
-- preserved. No production data is deleted or reset.

-- ---------------------------------------------------------------------------
-- Server metadata required by the CS:GO / LinuxGSM integration.
-- ---------------------------------------------------------------------------
ALTER TABLE servers
    ADD COLUMN game TEXT NOT NULL DEFAULT 'csgo',
    ADD COLUMN region TEXT,
    ADD COLUMN server_type TEXT NOT NULL DEFAULT 'community',
    ADD COLUMN query_port INTEGER,
    ADD COLUMN max_players INTEGER NOT NULL DEFAULT 24,
    ADD COLUMN online_players INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN current_mode_id SMALLINT REFERENCES movement_modes (id) ON DELETE SET NULL,
    ADD COLUMN game_version TEXT,
    ADD COLUMN is_enabled BOOLEAN NOT NULL DEFAULT TRUE;

ALTER TABLE servers
    ADD CONSTRAINT ck_servers_game CHECK (game IN ('csgo')),
    ADD CONSTRAINT ck_servers_region CHECK (region IS NULL OR char_length(region) BETWEEN 1 AND 32),
    ADD CONSTRAINT ck_servers_type CHECK (server_type IN ('community', 'official', 'event', 'testing')),
    ADD CONSTRAINT ck_servers_query_port CHECK (query_port IS NULL OR query_port BETWEEN 1 AND 65535),
    ADD CONSTRAINT ck_servers_max_players CHECK (max_players BETWEEN 1 AND 128),
    ADD CONSTRAINT ck_servers_online_players CHECK (online_players >= 0 AND online_players <= max_players);

COMMENT ON COLUMN servers.game IS 'Game family; phase 7 only accepts csgo (Source 1 / GOKZ).';
COMMENT ON COLUMN servers.is_enabled IS 'Admin kill switch; disabled servers may authenticate but cannot submit data.';
COMMENT ON COLUMN servers.current_mode_id IS 'Last reported movement mode (vanilla/simple/kztimer).';
COMMENT ON COLUMN servers.online_players IS 'Denormalized count from the last heartbeat.';
COMMENT ON COLUMN servers.query_port IS 'Optional Source query port (A2S) for external monitoring.';
COMMENT ON COLUMN servers.game_version IS 'CS:GO dedicated server version reported by the plugin heartbeat.';

CREATE INDEX idx_servers_game_enabled
    ON servers (game, is_enabled, is_online, last_heartbeat_at DESC)
    WHERE deleted_at IS NULL;

-- Stable per-server record identifier from the SourceMod plugin. The existing
-- `submission_id UUID` remains for imports/other clients; plugin submissions use
-- `submission_key` because SourcePawn does not need to generate a UUID.
ALTER TABLE records ADD COLUMN submission_key TEXT;

CREATE UNIQUE INDEX uq_records_submission_key
    ON records (server_id, submission_key)
    WHERE submission_key IS NOT NULL AND server_id IS NOT NULL;

COMMENT ON COLUMN records.submission_key IS 'Idempotency key from cngokz_sync; unique per server.';

-- ---------------------------------------------------------------------------
-- Heartbeat history (bounded by a retention job; readers use servers.* cache).
-- ---------------------------------------------------------------------------
CREATE TABLE server_heartbeats (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    server_id BIGINT NOT NULL REFERENCES servers (id) ON DELETE CASCADE,
    received_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reported_at TIMESTAMPTZ,
    map_name TEXT,
    current_map_id BIGINT REFERENCES maps (id) ON DELETE SET NULL,
    mode_code TEXT,
    online_players INTEGER NOT NULL DEFAULT 0 CHECK (online_players >= 0),
    max_players INTEGER NOT NULL DEFAULT 24 CHECK (max_players BETWEEN 1 AND 128),
    plugin_version TEXT,
    game_version TEXT,
    source_ip INET,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT ck_server_heartbeats_metadata CHECK (jsonb_typeof(metadata) = 'object')
);

COMMENT ON TABLE server_heartbeats IS 'Append-only heartbeat history for server detail and diagnostics';

CREATE INDEX idx_server_heartbeats_server
    ON server_heartbeats (server_id, received_at DESC);
CREATE INDEX idx_server_heartbeats_received
    ON server_heartbeats (received_at DESC);

-- ---------------------------------------------------------------------------
-- Online player sessions. One active row per (server, account); a player may
-- simultaneously have active sessions on different servers.
-- ---------------------------------------------------------------------------
CREATE TABLE server_player_sessions (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    server_id BIGINT NOT NULL REFERENCES servers (id) ON DELETE CASCADE,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    current_map_id BIGINT REFERENCES maps (id) ON DELETE SET NULL,
    current_mode_id SMALLINT REFERENCES movement_modes (id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'connected',
    connected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    disconnected_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_server_player_sessions_name CHECK (char_length(display_name) BETWEEN 1 AND 64),
    CONSTRAINT ck_server_player_sessions_status CHECK (status IN ('connected', 'spectating', 'afk', 'disconnected')),
    CONSTRAINT ck_server_player_sessions_disconnect CHECK (
        (disconnected_at IS NULL AND status <> 'disconnected')
        OR (disconnected_at IS NOT NULL AND status = 'disconnected')
    )
);

COMMENT ON TABLE server_player_sessions IS 'Real CS:GO online sessions; active = disconnected_at IS NULL';
COMMENT ON COLUMN server_player_sessions.status IS 'connected|spectating|afk while active; disconnected after leaving';

CREATE UNIQUE INDEX uq_server_player_active
    ON server_player_sessions (server_id, account_id)
    WHERE disconnected_at IS NULL;
CREATE INDEX idx_server_player_active_server
    ON server_player_sessions (server_id, last_seen_at DESC)
    WHERE disconnected_at IS NULL;
CREATE INDEX idx_server_player_active_account
    ON server_player_sessions (account_id, last_seen_at DESC)
    WHERE disconnected_at IS NULL;

CREATE TRIGGER trg_server_player_sessions_updated_at
    BEFORE UPDATE ON server_player_sessions
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ---------------------------------------------------------------------------
-- Ingestion audit log. Never stores tokens or replay bytes.
-- ---------------------------------------------------------------------------
CREATE TABLE server_sync_logs (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    server_id BIGINT REFERENCES servers (id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    status TEXT NOT NULL,
    message TEXT,
    record_id BIGINT REFERENCES records (id) ON DELETE SET NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_server_sync_logs_kind CHECK (kind IN ('auth', 'heartbeat', 'players', 'record', 'replay', 'config', 'other')),
    CONSTRAINT ck_server_sync_logs_status CHECK (status IN ('success', 'duplicate', 'rejected', 'failed')),
    CONSTRAINT ck_server_sync_logs_metadata CHECK (jsonb_typeof(metadata) = 'object')
);

COMMENT ON TABLE server_sync_logs IS 'Server ingestion audit trail; sanitized, no credentials or payload secrets';

CREATE INDEX idx_server_sync_logs_server
    ON server_sync_logs (server_id, created_at DESC);
CREATE INDEX idx_server_sync_logs_status
    ON server_sync_logs (status, created_at DESC);

-- ---------------------------------------------------------------------------
-- Replay compensation / sync tasks. Records are accepted independently of a
-- replay upload; a failed or pending replay creates one retryable task.
-- ---------------------------------------------------------------------------
CREATE TABLE server_sync_tasks (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    server_id BIGINT NOT NULL REFERENCES servers (id) ON DELETE CASCADE,
    record_id BIGINT REFERENCES records (id) ON DELETE CASCADE,
    task_type TEXT NOT NULL DEFAULT 'replay_upload',
    status TEXT NOT NULL DEFAULT 'pending',
    payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    CONSTRAINT ck_server_sync_tasks_type CHECK (task_type IN ('replay_upload', 'record_retry', 'config_sync')),
    CONSTRAINT ck_server_sync_tasks_status CHECK (status IN ('pending', 'running', 'done', 'failed')),
    CONSTRAINT ck_server_sync_tasks_payload CHECK (jsonb_typeof(payload) = 'object')
);

COMMENT ON TABLE server_sync_tasks IS 'Retryable server-side jobs, primarily replay upload compensation';

CREATE INDEX idx_server_sync_tasks_pending
    ON server_sync_tasks (status, created_at)
    WHERE status IN ('pending', 'failed');
CREATE INDEX idx_server_sync_tasks_server
    ON server_sync_tasks (server_id, created_at DESC);

CREATE TRIGGER trg_server_sync_tasks_updated_at
    BEFORE UPDATE ON server_sync_tasks
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
