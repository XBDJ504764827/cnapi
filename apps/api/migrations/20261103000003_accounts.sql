-- 0003_accounts
--
-- Steam identity, player profile, derived statistics and external identities.
--
-- Source findings:
--   * GOKZ local DB `Players`: SteamID32 (MySQL INTEGER UNSIGNED), Alias,
--     Country, IP, Cheater, LastPlayed, Created.
--   * GlobalAPI v2 APIPlayer: steamid64, steam_id (Steam2), is_banned,
--     total_records, name. APIPlayerRank: points, points_overall, average,
--     rating, finishes.
--   * cs2kz Players: id (steamid64, INT8 UNSIGNED), name, ip_address, preferences,
--     vnl_rating, ckz_rating, first_joined_at, last_joined_at, prime_verified.
--
-- Design decisions:
--   * SteamID64 is stored as NUMERIC(20,0) (0 .. 18446744073709551615) because
--     PostgreSQL BIGINT is signed and cannot hold the upper half of the uint64
--     SteamID64 space. The CHECK constraint enforces the uint64 range.
--   * Steam2 / Steam3 are derived representations and are stored as optional
--     text for legacy compatibility imports; they are not the source of truth.

CREATE TABLE accounts (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    steam_id64 NUMERIC(20, 0) NOT NULL,
    -- Legacy "STEAM_X:Y:Z" representation, as used by GlobalAPI v1/v2 and GOKZ localdb.
    steam_id2 TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT ck_accounts_steam_id64_range
        CHECK (steam_id64 >= 0 AND steam_id64 <= 18446744073709551615),
    CONSTRAINT ck_accounts_steam_id2_format
        CHECK (steam_id2 IS NULL OR steam_id2 ~ '^STEAM_[0-5]:[01]:[0-9]{1,10}$')
);

COMMENT ON TABLE accounts IS 'Canonical Steam account identity (one row per SteamID64)';
COMMENT ON COLUMN accounts.steam_id64 IS 'Unsigned 64-bit SteamID; NUMERIC(20,0) to avoid BIGINT overflow';
COMMENT ON COLUMN accounts.steam_id2 IS 'Optional legacy STEAM_X:Y:Z form used by GlobalAPI/GOKZ imports';
COMMENT ON COLUMN accounts.deleted_at IS 'Soft delete marker; unique indexes are partial on deleted_at IS NULL';

-- Partial unique indexes allow a new account row after a soft delete while still
-- guaranteeing one active row per Steam identity / Steam2 string.
CREATE UNIQUE INDEX uq_accounts_steam_id64_active
    ON accounts (steam_id64)
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX uq_accounts_steam_id2_active
    ON accounts (steam_id2)
    WHERE deleted_at IS NULL AND steam_id2 IS NOT NULL;

CREATE TRIGGER trg_accounts_updated_at
    BEFORE UPDATE ON accounts
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ---------------------------------------------------------------------------
-- player_profiles
-- ---------------------------------------------------------------------------
CREATE TABLE player_profiles (
    account_id BIGINT PRIMARY KEY REFERENCES accounts (id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    country TEXT,
    last_ip INET,
    preferences JSONB NOT NULL DEFAULT '{}'::jsonb,
    prime_verified BOOLEAN NOT NULL DEFAULT FALSE,
    first_joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_player_profiles_display_name CHECK (char_length(display_name) BETWEEN 1 AND 64),
    CONSTRAINT ck_player_profiles_preferences_object CHECK (jsonb_typeof(preferences) = 'object')
);

COMMENT ON TABLE player_profiles IS 'Mutable player profile data (name, country, IP, plugin preferences)';
COMMENT ON COLUMN player_profiles.country IS 'Free-form country string as reported by the game/plugin (GOKZ Players.Country); no ISO normalization yet';
COMMENT ON COLUMN player_profiles.preferences IS 'Arbitrary JSON preferences set by the game server (cs2kz Preferences)';
COMMENT ON COLUMN player_profiles.prime_verified IS 'cs2kz Players.prime_verified';

CREATE INDEX idx_player_profiles_display_name
    ON player_profiles (lower(display_name));

CREATE TRIGGER trg_player_profiles_updated_at
    BEFORE UPDATE ON player_profiles
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ---------------------------------------------------------------------------
-- player_mode_stats
-- ---------------------------------------------------------------------------
-- Cached aggregates per account and movement mode. These numbers are derived
-- from records and MUST be rebuildable; the source of truth are the record
-- tables. Field semantics follow GlobalAPI v2 APIPlayerRank.
CREATE TABLE player_mode_stats (
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    mode_id SMALLINT NOT NULL REFERENCES movement_modes (id),
    points INTEGER NOT NULL DEFAULT 0 CHECK (points >= 0),
    points_overall INTEGER NOT NULL DEFAULT 0 CHECK (points_overall >= 0),
    rating NUMERIC(10, 4) NOT NULL DEFAULT 0,
    average NUMERIC(10, 4) NOT NULL DEFAULT 0,
    finishes INTEGER NOT NULL DEFAULT 0 CHECK (finishes >= 0),
    total_records INTEGER NOT NULL DEFAULT 0 CHECK (total_records >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, mode_id)
);

COMMENT ON TABLE player_mode_stats IS 'Derived per-mode player aggregates (GlobalAPI PlayerRank / cs2kz rating); rebuilt from records';
COMMENT ON COLUMN player_mode_stats.points_overall IS 'GlobalAPI APIPlayerRank.points_overall';
COMMENT ON COLUMN player_mode_stats.average IS 'GlobalAPI APIPlayerRank.average';
COMMENT ON COLUMN player_mode_stats.rating IS 'GlobalAPI APIPlayerRank.rating / cs2kz vnl_rating,ckz_rating depending on mode';

CREATE INDEX idx_player_mode_stats_points ON player_mode_stats (mode_id, points DESC);

CREATE TRIGGER trg_player_mode_stats_updated_at
    BEFORE UPDATE ON player_mode_stats
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ---------------------------------------------------------------------------
-- player_identities
-- ---------------------------------------------------------------------------
CREATE TABLE player_identities (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    identifier TEXT NOT NULL,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ck_player_identities_provider CHECK (provider ~ '^[a-z][a-z0-9_-]*$')
);

COMMENT ON TABLE player_identities IS 'Additional identities linked to an account (discord, forum, plugin-specific ids); Steam stays canonical in accounts';
COMMENT ON COLUMN player_identities.provider IS 'Stable provider key, e.g. discord, forum, qq';

CREATE UNIQUE INDEX uq_player_identities_provider_identifier
    ON player_identities (provider, identifier);

CREATE INDEX idx_player_identities_account ON player_identities (account_id);
