#!/usr/bin/env bash
# Installs a CNGOKZAPI native bundle on the server.
#
# Final production layout (host path):
#   /mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi/
#     ├── .env
#     ├── bin/cngokz-api
#     ├── index.html
#     ├── assets/
#     ├── robots.txt
#     └── sitemap.xml
#
# OpenResty container sees this directory as:
#   /www/sites/test.cngokz.com/index/cnapi/
#
# Run from the extracted bundle directory as root:
#   sudo bash install-native.sh
set -euo pipefail

BUNDLE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="${APP_DIR:-/mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi}"
SERVICE_NAME="${SERVICE_NAME:-cngokz-api}"
SERVICE_USER="${SERVICE_USER:-www-data}"
SERVICE_GROUP="${SERVICE_GROUP:-www-data}"

if [[ "$(id -u)" -ne 0 ]]; then
  echo "run as root (or with sudo)" >&2
  exit 1
fi

if [[ ! -x "${BUNDLE_DIR}/bin/cngokz-api" ]]; then
  echo "bundle is missing bin/cngokz-api: ${BUNDLE_DIR}" >&2
  exit 1
fi
if [[ ! -d "${BUNDLE_DIR}/static" ]]; then
  echo "bundle is missing static/: ${BUNDLE_DIR}" >&2
  exit 1
fi

if ! getent group "${SERVICE_GROUP}" >/dev/null 2>&1; then
  echo "service group does not exist: ${SERVICE_GROUP}" >&2
  exit 1
fi
if ! id "${SERVICE_USER}" >/dev/null 2>&1; then
  echo "service user does not exist: ${SERVICE_USER}" >&2
  exit 1
fi

echo "==> target directory: ${APP_DIR}"
mkdir -p "${APP_DIR}/bin"

echo "==> installing backend binary"
if [[ -f "${APP_DIR}/bin/cngokz-api" ]]; then
  cp -a "${APP_DIR}/bin/cngokz-api" "${APP_DIR}/bin/cngokz-api.prev"
fi
install -m 0755 "${BUNDLE_DIR}/bin/cngokz-api" "${APP_DIR}/bin/cngokz-api"
chmod 0755 "${APP_DIR}" "${APP_DIR}/bin" "${APP_DIR}/bin/cngokz-api"

echo "==> installing unified frontend static files"
# Remove only old frontend outputs. Never remove APP_DIR itself: it also
# contains .env and bin/.
rm -rf "${APP_DIR}/public" "${APP_DIR}/admin" "${APP_DIR}/assets"
rm -f "${APP_DIR}/index.html" "${APP_DIR}/robots.txt" "${APP_DIR}/sitemap.xml"
cp -a "${BUNDLE_DIR}/static/." "${APP_DIR}/"

# Make public static files readable by OpenResty, but keep .env restricted.
chmod 0755 "${APP_DIR}"
chmod 0755 "${APP_DIR}/bin"
chmod 0755 "${APP_DIR}/bin/cngokz-api"
if [[ -d "${APP_DIR}/assets" ]]; then
  chmod -R a+rX "${APP_DIR}/assets"
fi
for file in index.html robots.txt sitemap.xml; do
  if [[ -f "${APP_DIR}/${file}" ]]; then
    chmod 0644 "${APP_DIR}/${file}"
  fi
done

CONFIG_MISSING=0
if [[ ! -f "${APP_DIR}/.env" ]]; then
  echo "==> creating ${APP_DIR}/.env from template"
  install -m 0640 -o "${SERVICE_USER}" -g "${SERVICE_GROUP}" \
    "${BUNDLE_DIR}/.env.production.example" "${APP_DIR}/.env"
  CONFIG_MISSING=1
else
  chown "${SERVICE_USER}:${SERVICE_GROUP}" "${APP_DIR}/.env"
  chmod 0640 "${APP_DIR}/.env"
fi

echo "==> installing systemd unit"
service_tmp="$(mktemp)"
sed \
  -e "s|/mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi|${APP_DIR}|g" \
  -e "s|^User=.*|User=${SERVICE_USER}|" \
  -e "s|^Group=.*|Group=${SERVICE_GROUP}|" \
  "${BUNDLE_DIR}/deploy/cngokz-api.service" > "${service_tmp}"
install -m 0644 "${service_tmp}" "/etc/systemd/system/${SERVICE_NAME}.service"
rm -f "${service_tmp}"

echo "==> nginx site config is managed by 1Panel/OpenResty; not overwritten"
echo "    reference file: ${BUNDLE_DIR}/deploy/nginx-test.cngokz.com.conf"

systemctl daemon-reload

if [[ "${CONFIG_MISSING}" -eq 1 ]]; then
  echo
  echo "IMPORTANT: edit ${APP_DIR}/.env before starting the service:"
  echo "  DATABASE_URL=postgres://...@127.0.0.1:5432/cngokzapi"
  echo "  REDIS_URL=redis://127.0.0.1:6379/3"
  echo "  MAINTENANCE_TOKEN=openssl rand -hex 32"
  echo
  echo "Then run: systemctl enable --now ${SERVICE_NAME}"
  exit 0
fi

systemctl enable "${SERVICE_NAME}"
systemctl restart "${SERVICE_NAME}"

if command -v nginx >/dev/null 2>&1; then
  nginx -t
  systemctl reload nginx
fi

echo
echo "Installed ${SERVICE_NAME}."
echo "  systemctl status ${SERVICE_NAME}"
echo "  journalctl -u ${SERVICE_NAME} -f"
echo "  curl -fsS http://127.0.0.1:28080/health/ready | jq"
