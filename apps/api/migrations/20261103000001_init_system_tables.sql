-- 0001_init_system_tables
--
-- Phase 1 only ships system level tables. KZ business tables (players, maps,
-- records, jumpstats, replays, servers, bans) are added in later phases with
-- their own migrations.
--
-- Migrations are applied by sqlx::migrate at startup; applied versions are
-- tracked in `_sqlx_migrations`, so `migrate run` is idempotent.

-- Recent system events shown on the dashboard. Phase 1 reads this table only.
CREATE TABLE system_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'info' CHECK (severity IN ('info', 'warning', 'error')),
    message TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE system_events IS 'Audit trail of system level events (deployments, dependency failures, ...)';
COMMENT ON COLUMN system_events.event_type IS 'Dotted event name, e.g. api.started';

CREATE INDEX idx_system_events_created_at ON system_events (created_at DESC);
CREATE INDEX idx_system_events_event_type ON system_events (event_type);

-- Key/value store for platform settings edited from the admin UI.
-- Reserved for the "System settings" page; no reader/writer is implemented yet.
CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value JSONB NOT NULL DEFAULT '{}'::jsonb,
    description TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE app_settings IS 'Platform settings key/value store (admin UI)';
