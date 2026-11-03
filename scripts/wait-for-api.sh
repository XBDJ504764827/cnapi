#!/usr/bin/env bash
# Waits until the API reports ready. Useful for local scripts and CI smoke tests.
set -euo pipefail

url="${1:-http://localhost:8080/health/ready}"
attempts="${2:-30}"

for attempt in $(seq 1 "$attempts"); do
  status="$(curl -s -o /dev/null -w '%{http_code}' "$url" || true)"
  if [ "$status" = "200" ]; then
    echo "API is ready ($url)"
    exit 0
  fi
  echo "waiting for $url (attempt $attempt/$attempts, last status: ${status:-none})"
  sleep 2
done

echo "API did not become ready in time: $url" >&2
exit 1
