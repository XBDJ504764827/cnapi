# PostgreSQL (infrastructure)

| Path | Purpose |
| --- | --- |
| `init/00-init.sql` | Runs once when the container creates an empty data directory (timezone only). |

## Migration ownership

Schema changes are **not** managed by docker-entrypoint scripts. They live in
`apps/api/migrations/` and are applied by the API process on startup using
`sqlx::migrate!`, which records applied versions in `_sqlx_migrations` and makes
re-running safe.

To add a schema change:

```bash
sqlx migrate add -r <description>   # from apps/api, needs sqlx-cli
# edit the generated .up.sql / .down.sql
```

## Persistence

`docker-compose.yml` mounts the named volume `cngokzapi-postgres-data` at
`/var/lib/postgresql` (PostgreSQL 18 keeps `PGDATA` below that directory).
`docker compose down` keeps the volume; `docker compose down -v` deletes it.
