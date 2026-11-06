-- 20261106000001_auth_steam_openid
--
-- Steam OpenID 2.0 authentication, opaque server-side sessions and the
-- administrator role used to protect the management console.
--
-- Design decisions:
--   * Steam stays the only identity provider. OpenID 2.0 only proves "this
--     browser controls SteamID64 X"; it returns no profile data, so the
--     persona name and avatar are filled in from the Steam Web API when an
--     API key is configured and stay NULL otherwise.
--   * Sessions are opaque 256-bit random tokens. Only the SHA-256 digest is
--     stored, so a database dump cannot be replayed as a login cookie.
--   * `account_roles` is a table (not a boolean on `accounts`) so moderators
--     and other privileged roles can be added without a breaking migration.
--   * `steam_login_states` holds the single-use CSRF nonce of an in-flight
--     OpenID round trip. Rows are consumed exactly once and expire quickly.

-- ---------------------------------------------------------------------------
-- player_profiles.avatar_url
-- ---------------------------------------------------------------------------
-- Steam avatar URL propagated from the Steam Web API at login time. Public
-- pages fall back to an initial-based avatar when this is NULL.
ALTER TABLE player_profiles
    ADD COLUMN avatar_url TEXT,
    ADD CONSTRAINT ck_player_profiles_avatar_url
        CHECK (avatar_url IS NULL OR (char_length(avatar_url) BETWEEN 8 AND 512));

COMMENT ON COLUMN player_profiles.avatar_url IS 'Steam avatar URL refreshed from ISteamUser/GetPlayerSummaries at login; NULL when no API key is configured';

-- ---------------------------------------------------------------------------
-- account_roles
-- ---------------------------------------------------------------------------
CREATE TABLE account_roles (
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    granted_by BIGINT REFERENCES accounts (id) ON DELETE SET NULL,
    PRIMARY KEY (account_id, role),
    CONSTRAINT ck_account_roles_role CHECK (role IN ('admin'))
);

COMMENT ON TABLE account_roles IS 'Privileged roles granted to an account; only `admin` may access /api/v1/admin/* and /admin/';
COMMENT ON COLUMN account_roles.granted_by IS 'Account that granted the role (NULL for bootstrap via ADMIN_STEAM_IDS)';

CREATE INDEX idx_account_roles_role ON account_roles (role);

-- ---------------------------------------------------------------------------
-- auth_sessions
-- ---------------------------------------------------------------------------
CREATE TABLE auth_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id BIGINT NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    -- SHA-256 of the opaque session token; the token itself is never stored.
    token_hash BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    user_agent TEXT,
    ip INET,
    CONSTRAINT uq_auth_sessions_token_hash UNIQUE (token_hash),
    CONSTRAINT ck_auth_sessions_token_hash_length CHECK (octet_length(token_hash) = 32),
    CONSTRAINT ck_auth_sessions_expiry CHECK (expires_at > created_at),
    CONSTRAINT ck_auth_sessions_user_agent_length CHECK (user_agent IS NULL OR char_length(user_agent) <= 512)
);

COMMENT ON TABLE auth_sessions IS 'Opaque browser sessions; the cookie carries the token, the database only the SHA-256 digest';
COMMENT ON COLUMN auth_sessions.revoked_at IS 'Set on logout; a revoked session is rejected immediately even if expires_at is in the future';

CREATE INDEX idx_auth_sessions_account ON auth_sessions (account_id);
CREATE INDEX idx_auth_sessions_expires_at ON auth_sessions (expires_at);
CREATE INDEX idx_auth_sessions_active_account
    ON auth_sessions (account_id)
    WHERE revoked_at IS NULL;

-- ---------------------------------------------------------------------------
-- steam_login_states
-- ---------------------------------------------------------------------------
CREATE TABLE steam_login_states (
    -- Random nonce echoed through `openid.return_to`; the only value an
    -- attacker cannot forge without a valid state row.
    state TEXT PRIMARY KEY,
    -- Same-origin path the browser returns to after the round trip.
    redirect_path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,
    CONSTRAINT ck_steam_login_states_state CHECK (state ~ '^[A-Za-z0-9_-]{22,128}$'),
    CONSTRAINT ck_steam_login_states_redirect CHECK (redirect_path LIKE '/%' AND redirect_path NOT LIKE '//%'),
    CONSTRAINT ck_steam_login_states_expiry CHECK (expires_at > created_at)
);

COMMENT ON TABLE steam_login_states IS 'Single-use OpenID login nonces (CSRF protection); consumed at most once within 10 minutes';
COMMENT ON COLUMN steam_login_states.redirect_path IS 'Validated same-origin path the SPA navigates to after a successful login';

CREATE INDEX idx_steam_login_states_expires_at ON steam_login_states (expires_at);
