#!/usr/bin/env bash
# Verifies that the shared Steam relay accepts this deployment's callback host.
#
# The relay (cngokz-steam-auth Worker) only echoes `return_to` back when its
# host is listed in ALLOWED_CALLBACK_HOSTS; otherwise it silently falls back to
# the forum callback and this API never receives the one-time token. That
# failure is invisible from the browser, so check it before deploying.
#
# Usage:
#   STEAM_RELAY_URL=https://cngokz-steam-auth.iquankz.cn \
#   CALLBACK_URL=https://test.cngokz.com/api/v1/auth/steam/callback \
#     scripts/check-steam-relay.sh
set -euo pipefail

RELAY="${STEAM_RELAY_URL:?set STEAM_RELAY_URL (e.g. https://cngokz-steam-auth.iquankz.cn)}"
CALLBACK="${CALLBACK_URL:?set CALLBACK_URL (e.g. https://test.cngokz.com/api/v1/auth/steam/callback)}"

RELAY="${RELAY%/}"

echo "==> relay  : ${RELAY}"
echo "==> callback: ${CALLBACK}"

health="$(curl -fsS -m 15 "${RELAY}/" || true)"
if [[ "${health}" != *'"status":"running"'* ]]; then
  echo "FAIL: relay health check did not return status=running (got: ${health:-<empty>})" >&2
  exit 1
fi
echo "OK: relay is reachable"

location="$(
  curl -fsS -m 15 -o /dev/null -D - \
    "${RELAY}/login?mode=login&state=preflight&return_to=$(python3 -c 'import sys,urllib.parse;print(urllib.parse.quote(sys.argv[1],safe=""))' "${CALLBACK}")" \
    | tr -d '\r' | sed -n 's/^[Ll]ocation: //p'
)"

if [[ -z "${location}" ]]; then
  echo "FAIL: /login did not answer with a redirect" >&2
  exit 1
fi

python3 - "$location" "$CALLBACK" <<'PY'
import sys
import urllib.parse

steam_url, expected = sys.argv[1], sys.argv[2]
query = dict(urllib.parse.parse_qsl(urllib.parse.urlparse(steam_url).query))
return_to = query.get("openid.return_to", "")
print(f"openid.realm     = {query.get('openid.realm')}")
print(f"openid.return_to = {return_to}")

expected_host = urllib.parse.urlparse(expected).netloc
inner = dict(urllib.parse.parse_qsl(urllib.parse.urlparse(return_to).query)).get("return_to", "")
inner_host = urllib.parse.urlparse(inner).netloc

if inner_host != expected_host:
    print(
        f"\nFAIL: the relay did not accept {expected_host}.\n"
        f"      It rewrote the callback to {inner_host or '<empty>'}.\n"
        "      Fix: set ALLOWED_CALLBACK_HOSTS on the Worker to include "
        f"{expected_host!r}, then redeploy the Worker.",
        file=sys.stderr,
    )
    sys.exit(1)

print(f"\nOK: {expected_host} is whitelisted; the token will come back to this API")
PY
