#!/usr/bin/env bash
# Installs a CNGOKZAPI native bundle on the server.
# Run from the extracted bundle directory as root:
#   sudo APP_DIR=/mnt/1panel/apps/cnapi STATIC_ROOT=/var/www/cngokzapi bash install-native.sh
set -euo pipefail

BUNDLE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="${APP_DIR:-/mnt/1panel/apps/cnapi}"
STATIC_ROOT="${STATIC_ROOT:-/var/www/cngokzapi}"
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

if ! getent group "${SERVICE_GROUP}" >/dev/null 2>&1; then
  echo "service group does not exist: ${SERVICE_GROUP}" >&2
  exit 1
fi
if ! id "${SERVICE_USER}" >/dev/null 2>&1; then
  echo "service user does not exist: ${SERVICE_USER}" >&2
  exit 1
fi

echo "==> installing backend binary"
mkdir -p "${APP_DIR}/bin"
if [[ -f "${APP_DIR}/bin/cngokz-api" ]]; then
  cp -a "${APP_DIR}/bin/cngokz-api" "${APP_DIR}/bin/cngokz-api.prev"
fi
install -m 0755 "${BUNDLE_DIR}/bin/cngokz-api" "${APP_DIR}/bin/cngokz-api"
chmod 0755 "${APP_DIR}" "${APP_DIR}/bin"

echo "==> installing public/admin static files"
mkdir -p "${STATIC_ROOT}"
for portal in public admin; do
  staging="${STATIC_ROOT}/.${portal}.new"
  rm -rf "${staging}"
  cp -a "${BUNDLE_DIR}/static/${portal}" "${staging}"
  rm -rf "${STATIC_ROOT:?}/${portal}"
  mv "${staging}" "${STATIC_ROOT}/${portal}"
done
chmod -R a+rX "${STATIC_ROOT}"

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
  -e "s|/mnt/1panel/apps/cnapi|${APP_DIR}|g" \
  -e "s|^User=.*|User=${SERVICE_USER}|" \
  -e "s|^Group=.*|Group=${SERVICE_GROUP}|" \
  "${BUNDLE_DIR}/deploy/cngokz-api.service" > "${service_tmp}"
install -m 0644 "${service_tmp}" "/etc/systemd/system/${SERVICE_NAME}.service"
rm -f "${service_tmp}"

echo "==> installing nginx site (only if absent)"
NGINX_CONF="/etc/nginx/conf.d/cngokzapi.conf"
if [[ ! -f "${NGINX_CONF}" ]]; then
  nginx_tmp="$(mktemp)"
  sed "s|/var/www/cngokzapi|${STATIC_ROOT}|g" \
    "${BUNDLE_DIR}/deploy/nginx-ban.cngokz.com.conf" > "${nginx_tmp}"
  install -m 0644 "${nginx_tmp}" "${NGINX_CONF}"
  rm -f "${nginx_tmp}"
  echo "    installed ${NGINX_CONF}"
else
  echo "    ${NGINX_CONF} already exists; left unchanged"
fi

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
echo "  curl -fsS http://127.0.0.1:8080/health/ready | jq"
