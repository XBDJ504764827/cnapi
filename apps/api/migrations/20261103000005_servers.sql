-- 0005_servers
--
-- Game servers that submit records and jumpstats.
--
-- Source findings:
--   * GlobalAPI v2 APIServer: name, ip, port, owner_steamid64 (no server creation
--     endpoint in the studied plugin; keys are provisioned out of band).
--     Record submissions are authenticated with an `X-ApiKey` header in the
--     legacy plugin (GlobalAPI-Core/server.sp).
--   * cs2kz Servers: name UNIQUE, host, port, owner_id, access_key BINARY(16)
--     UNIQUE, approved_at, last_connected_at, UNIQUE(host, port).
--
-- Design decisions:
--   * Only a hash of the API key is stored. `api_key_prefix` keeps the first
--     characters for identification in the admin UI; the full key is shown once
--     at creation time (phase 3).
--   * Online state is denormalized from heartbeats (`is_online`,
--     `last_heartbeat_at`) so dashboards do not need an event table. A stale
--     heartbeat (> 2 * interval) must be treated as offline by readers; a future
--     `server_heartbeats` history table can be added without changing this one.

CREATE TABLE servers (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL,
    ip INET NOT NULL,
    port INTEGER NOT NULL,
    owner_account_id BIGINT NOT NULL REFERENCES accounts (id),
    api_key_hash BYTEA NOT NULL,
    api_key_hash_algorithm TEXT NOT NULL DEFAULT 'sha256',
    api_key_prefix TEXT NOT NULL,
    api_key_created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    api_key_revoked_at TIMESTAMPTZ,
    plugin_version TEXT,
    current_map_id BIGINT REFERENCES maps (id) ON DELETE SET NULL,
    is_online BOOLEAN NOT NULL DEFAULT FALSE,
    last_heartbeat_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT ck_servers_name CHECK (char_length(name) BETWEEN 1 AND 128),
    CONSTRAINT ck_servers_port CHECK (port BETWEEN 1 AND 65535),
    CONSTRAINT ck_servers_api_key_hash CHECK (octet_length(api_key_hash) >= 16),
    CONSTRAINT ck_servers_api_key_prefix CHECK (char_length(api_key_prefix) BETWEEN 4 AND 16)
);

COMMENT ON TABLE servers IS 'Registered game servers; API keys are stored hashed only';
COMMENT ON COLUMN servers.api_key_hash IS 'Hash of the server API key (sha256 of 32 random bytes in phase 2); never expose';
COMMENT ON COLUMN servers.api_key_hash_algorithm IS 'Algorithm identifier; sha256 is the phase 2 default, argon2id may be added later';
COMMENT ON COLUMN servers.api_key_prefix IS 'Non-secret prefix of the key for identification/rotation in the admin UI';
COMMENT ON COLUMN servers.is_online IS 'Denormalized heartbeat state; readers must also check last_heartbeat_at freshness';
COMMENT ON COLUMN servers.last_heartbeat_at IS 'Last heartbeat received from the plugin (UTC)';

CREATE UNIQUE INDEX uq_servers_name_active ON servers (lower(name)) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX uq_servers_address_active ON servers (ip, port) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX uq_servers_api_key_hash ON servers (api_key_hash);
CREATE INDEX idx_servers_online ON servers (is_online, last_heartbeat_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_servers_owner ON servers (owner_account_id);
CREATE INDEX idx_servers_current_map ON servers (current_map_id) WHERE current_map_id IS NOT NULL;

CREATE TRIGGER trg_servers_updated_at
    BEFORE UPDATE ON servers
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();
