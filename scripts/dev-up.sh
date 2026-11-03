#!/usr/bin/env bash
# Starts the development dependencies (PostgreSQL + Redis) and prints next steps.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

if ! command -v docker >/dev/null 2>&1; then
  echo "docker is required to run the development dependencies" >&2
  exit 1
fi

docker compose -f docker-compose.dev.yml up -d

echo
echo "PostgreSQL: localhost:${POSTGRES_PORT:-5432} (user ${POSTGRES_USER:-cngokz}, database ${POSTGRES_DB:-cngokzapi})"
echo "Redis:      localhost:${REDIS_PORT:-6379}"
echo
echo "Next:"
echo "  cp .env.example .env             # local defaults"
echo "  cargo run -p cngokz-api          # API on http://localhost:8080"
echo "  pnpm --filter @cngokz/web dev    # web on http://localhost:5173"
