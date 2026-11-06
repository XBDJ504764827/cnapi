#!/usr/bin/env bash
# Builds the native production bundle:
#   - Rust release binary
#   - unified frontend static files
#   - systemd unit / nginx config / env template
#
# No Docker is used. Final layout mixes binary and static files into one
# production directory:
#   /mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi/
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

TAG="${TAG:-$(git rev-parse --short HEAD 2>/dev/null || date +%Y%m%d%H%M%S)}"
VITE_SITE_URL="${VITE_SITE_URL:-https://test.cngokz.com}"
OUTPUT_ROOT="${OUTPUT_ROOT:-native-bundles}"
OUTPUT_DIR="${OUTPUT_ROOT}/${TAG}"
ARCHIVE="cngokzapi-native-${TAG}.tar.gz"
PROD_DIR="/mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi"

for cmd in cargo pnpm; do
  command -v "$cmd" >/dev/null 2>&1 || {
    echo "missing required command: $cmd" >&2
    exit 1
  }
done

echo "==> tag: ${TAG}"
echo "==> site: ${VITE_SITE_URL}"
echo "==> output: ${OUTPUT_ROOT}/${TAG}"

rm -rf "${OUTPUT_DIR}"
mkdir -p "${OUTPUT_DIR}/bin" \
  "${OUTPUT_DIR}/static" \
  "${OUTPUT_DIR}/deploy" \
  "${OUTPUT_DIR}/migrations"

echo "==> building Rust release binary"
cargo build --release --locked --bin cngokz-api

echo "==> typechecking / building unified frontend"
VITE_SITE_URL="${VITE_SITE_URL}" pnpm --filter @cngokz/frontend build

cp target/release/cngokz-api "${OUTPUT_DIR}/bin/cngokz-api"
chmod 0755 "${OUTPUT_DIR}/bin/cngokz-api"

cp -a apps/frontend/dist/. "${OUTPUT_DIR}/static/"

cp deploy/cngokz-api.service "${OUTPUT_DIR}/deploy/cngokz-api.service"
cp deploy/nginx-test.cngokz.com.conf "${OUTPUT_DIR}/deploy/nginx-test.cngokz.com.conf"
cp scripts/install-native.sh "${OUTPUT_DIR}/install-native.sh"
chmod 0755 "${OUTPUT_DIR}/install-native.sh"
cp .env.production.example "${OUTPUT_DIR}/.env.production.example"
cp -a apps/api/migrations/. "${OUTPUT_DIR}/migrations/"

cat > "${OUTPUT_DIR}/DEPLOY.txt" <<EOF
CNGOKZAPI unified native bundle ${TAG}

Final production directory:
  ${PROD_DIR}

OpenResty container path:
  /www/sites/test.cngokz.com/index/cnapi/

Server install (root), from this extracted bundle directory:
  sudo bash install-native.sh

First install only:
  1. Create PostgreSQL database/user.
  2. Edit ${PROD_DIR}/.env
  3. Fill DATABASE_URL / REDIS_URL / MAINTENANCE_TOKEN.
  4. Run install-native.sh again.
  5. Configure OpenResty using deploy/nginx-test.cngokz.com.conf as reference.

The install script never deletes bin/ or .env when updating the frontend.
The server does not run Docker, cargo or pnpm.
EOF

tar -czf "${OUTPUT_ROOT}/${ARCHIVE}" -C "${OUTPUT_DIR}" .

(
  cd "${OUTPUT_ROOT}"
  sha256sum "${ARCHIVE}" > "${ARCHIVE}.sha256"
)

echo
echo "Done: ${OUTPUT_ROOT}/${ARCHIVE}"
cat "${OUTPUT_ROOT}/${ARCHIVE}.sha256"
