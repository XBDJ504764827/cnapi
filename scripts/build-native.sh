#!/usr/bin/env bash
# Builds the native production bundle:
#   - Rust release binary
#   - public portal static files
#   - admin portal static files
#   - systemd unit / nginx config / env template
#
# No Docker is used.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

TAG="${TAG:-$(git rev-parse --short HEAD 2>/dev/null || date +%Y%m%d%H%M%S)}"
VITE_SITE_URL="${VITE_SITE_URL:-https://ban.cngokz.com}"
OUTPUT_ROOT="${OUTPUT_ROOT:-native-bundles}"
OUTPUT_DIR="${OUTPUT_ROOT}/${TAG}"
ARCHIVE="cngokzapi-native-${TAG}.tar.gz"

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
  "${OUTPUT_DIR}/static/admin" \
  "${OUTPUT_DIR}/static/public" \
  "${OUTPUT_DIR}/deploy" \
  "${OUTPUT_DIR}/migrations"

echo "==> building Rust release binary"
cargo build --release --locked --bin cngokz-api

echo "==> typechecking / building admin portal"
VITE_SITE_URL="${VITE_SITE_URL}" pnpm --filter @cngokz/web build

echo "==> typechecking / building public portal"
VITE_SITE_URL="${VITE_SITE_URL}" pnpm --filter @cngokz/public build

cp target/release/cngokz-api "${OUTPUT_DIR}/bin/cngokz-api"
chmod 0755 "${OUTPUT_DIR}/bin/cngokz-api"

cp -a apps/web/dist/. "${OUTPUT_DIR}/static/admin/"
cp -a apps/public/dist/. "${OUTPUT_DIR}/static/public/"

cp deploy/cngokz-api.service "${OUTPUT_DIR}/deploy/cngokz-api.service"
cp deploy/nginx-ban.cngokz.com.conf "${OUTPUT_DIR}/deploy/nginx-ban.cngokz.com.conf"
cp scripts/install-native.sh "${OUTPUT_DIR}/install-native.sh"
chmod 0755 "${OUTPUT_DIR}/install-native.sh"
cp .env.production.example "${OUTPUT_DIR}/.env.production.example"
cp -a apps/api/migrations/. "${OUTPUT_DIR}/migrations/"

cat > "${OUTPUT_DIR}/DEPLOY.txt" <<EOF
CNGOKZAPI native bundle ${TAG}

Server install (root):
  cd /path/to/bundle
  sudo APP_DIR=/mnt/1panel/apps/cnapi \
       STATIC_ROOT=/var/www/cngokzapi \
       bash install-native.sh

First install only:
  1. Create PostgreSQL database/user.
  2. Copy .env.production.example to /mnt/1panel/apps/cnapi/.env
  3. Fill DATABASE_URL / REDIS_URL / MAINTENANCE_TOKEN.
  4. Run install-native.sh again.
  5. Install /etc/nginx/conf.d/cngokzapi.conf from deploy/ and reload Nginx.

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
