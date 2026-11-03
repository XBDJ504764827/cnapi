-- 0009_moderation
--
-- Bans and ban revocations.
--
-- Source findings:
--   * cs2kz Bans: player_id, player_ip, banned_by (Server(server_id) |
--     Admin(user_id) encoded in one column), reason, plugin_version_id,
--     created_at, expires_at (CHECK >= created_at). Unbans: ban_id PK,
--     admin_id, reason, created_at.
--   * GlobalAPI v1 POST /bans body: {steam_id (Steam2), ban_type, notes, stats,
--     ip}; v2 APIBan: id, steamid64, steam_id, player_name, notes, stats,
--     ban_type, expires_on, server_id, created_on, updated_on, updated_by_id.
--   * GOKZ local Players.Cheater is a separate boolean flag, not a ban record.
--
-- Design decisions:
--   * The issuer is split into admin account / server instead of cs2kz's single
--     tagged column: both foreign keys are explicit and indexable, and a CHECK
--     guarantees at least one issuer.
--   * Expiry is derived from `expires_at` instead of storing an "expired"
--     status, so no background job is needed to keep rows consistent. Only
--     `active` and `revoked` are stored.
--   * Bans are append-only audit data: no soft delete. Revocation is a separate
--     row (`ban_revocations`), which keeps the original decision intact.
--   * `ban_type` and `stats` are free-form text because GlobalAPI does not
--     define an enum for them and the legacy plugin forwards arbitrary strings.

CREATE TABLE bans (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts (id),
    ip INET,
    ban_type TEXT NOT NULL DEFAULT 'global',
    reason TEXT NOT NULL,
    stats TEXT,
    issued_by_account_id BIGINT REFERENCES accounts (id) ON DELETE SET NULL,
    issued_by_server_id BIGINT REFERENCES servers (id) ON DELETE SET NULL,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ,
    status TEXT NOT NULL DEFAULT 'active',
    source TEXT NOT NULL DEFAULT 'admin',
    external_ref TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_bans_type CHECK (char_length(ban_type) BETWEEN 1 AND 64),
    CONSTRAINT ck_bans_reason CHECK (char_length(reason) BETWEEN 1 AND 1024),
    CONSTRAINT ck_bans_status CHECK (status IN ('active', 'revoked')),
    CONSTRAINT ck_bans_source CHECK (source IN ('admin', 'server', 'globalapi_import')),
    CONSTRAINT ck_bans_period CHECK (expires_at IS NULL OR expires_at > issued_at),
    CONSTRAINT ck_bans_issuer CHECK (issued_by_account_id IS NOT NULL OR issued_by_server_id IS NOT NULL)
);

COMMENT ON TABLE bans IS 'Player bans; append-only, revocation recorded in ban_revocations';
COMMENT ON COLUMN bans.ban_type IS 'Free-form GlobalAPI ban_type (e.g. global/cheat); upstream does not define an enum';
COMMENT ON COLUMN bans.stats IS 'Free-form GlobalAPI `stats` text captured with the ban';
COMMENT ON COLUMN bans.expires_at IS 'NULL = permanent ban; expiration is derived at read time';
COMMENT ON COLUMN bans.status IS 'active|revoked only; expired bans are computed from expires_at';
COMMENT ON COLUMN bans.external_ref IS 'Upstream ban id for GlobalAPI imports';

CREATE UNIQUE INDEX uq_bans_external_ref ON bans (external_ref) WHERE external_ref IS NOT NULL;
CREATE INDEX idx_bans_account_active ON bans (account_id, issued_at DESC)
    WHERE status = 'active';
CREATE INDEX idx_bans_expiry ON bans (expires_at)
    WHERE status = 'active' AND expires_at IS NOT NULL;
CREATE INDEX idx_bans_issued_by_admin ON bans (issued_by_account_id, issued_at DESC);
CREATE INDEX idx_bans_issued_by_server ON bans (issued_by_server_id, issued_at DESC);

CREATE TRIGGER trg_bans_updated_at
    BEFORE UPDATE ON bans
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ---------------------------------------------------------------------------
-- ban_revocations
-- ---------------------------------------------------------------------------
CREATE TABLE ban_revocations (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    ban_id BIGINT NOT NULL UNIQUE REFERENCES bans (id) ON DELETE CASCADE,
    revoked_by_account_id BIGINT REFERENCES accounts (id) ON DELETE SET NULL,
    revoked_by_server_id BIGINT REFERENCES servers (id) ON DELETE SET NULL,
    reason TEXT NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_ban_revocations_reason CHECK (char_length(reason) BETWEEN 1 AND 1024),
    CONSTRAINT ck_ban_revocations_actor CHECK (
        revoked_by_account_id IS NOT NULL OR revoked_by_server_id IS NOT NULL
    )
);

COMMENT ON TABLE ban_revocations IS 'Unban records; one revocation per ban, original ban row is preserved';
COMMENT ON COLUMN ban_revocations.revoked_by_server_id IS 'Set when a server lifts its own ban (cs2kz BannedBy::Server equivalent)';
