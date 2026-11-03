# Redis (infrastructure)

`redis.conf` is mounted into both the full stack and the development compose
files. Values:

- `appendonly yes` + `appendfsync everysec` — persistence is enabled because the
  named volume `cngokzapi-redis-data` is mounted at `/data`.
- `maxmemory 256mb` with `allkeys-lru` — phase 1 only caches data, eviction is
  safe. Increase this before storing anything that must survive eviction.
- No password: Redis is published on `127.0.0.1` and reachable only from the
  internal compose network. Add `requirepass` + `REDIS_URL` credentials if the
  network boundary changes.

The API opens one multiplexed connection (`redis::aio::ConnectionManager`) and
exposes `GET /health/ready` for a `PING` based probe.
