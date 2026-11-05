#!/usr/bin/env bash
# Regenerates the OpenAPI document, the TypeScript schema and the API client
# types. Run after changing backend routes, DTOs or schemas.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

echo "==> Generating docs/openapi.json from the Rust handlers"
cargo run --quiet --bin dump-openapi -- docs/openapi.json

echo "==> Generating TypeScript schema (packages/shared-types)"
pnpm --filter @cngokz/shared-types gen:api

echo "==> Type checking generated packages and the web app"
pnpm --filter @cngokz/shared-types typecheck
pnpm --filter @cngokz/api-client typecheck
pnpm --filter @cngokz/frontend typecheck

echo
echo "Done. Review docs/openapi.json and packages/shared-types/src/generated/api-schema.d.ts"
