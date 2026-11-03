-- Runs once, when the PostgreSQL container initializes an empty data directory.
-- Schema changes are NOT made here: the API applies versioned migrations from
-- apps/api/migrations at startup (sqlx::migrate).

-- Keep all timestamps in UTC regardless of the client time zone.
DO $$
BEGIN
    EXECUTE format('ALTER DATABASE %I SET timezone TO ''UTC''', current_database());
END
$$;
