#!/usr/bin/env bash
# 本机直连部署：构建前后端 → 上传到服务器 → 重启后端并做健康检查。
#
# 用法：
#   scripts/deploy-local.sh              # 构建 + 上传 + 重启
#   SKIP_BUILD=1 scripts/deploy-local.sh # 只上传 + 重启（复用已有产物）
#
# 可用环境变量覆盖：
#   SERVER_HOST  默认 103.120.89.225
#   SERVER_USER  默认 root
#   SSH_KEY      默认 ~/.ssh/cngokz_deploy
#   APP_DIR      默认 1Panel OpenResty 站点目录
#   SERVICE      默认 cngokz-api
#   HEALTH_URL   默认 http://127.0.0.1:28080/health/ready
set -euo pipefail

SERVER_HOST="${SERVER_HOST:-103.120.89.225}"
SERVER_USER="${SERVER_USER:-root}"
SSH_KEY="${SSH_KEY:-$HOME/.ssh/cngokz_deploy}"
APP_DIR="${APP_DIR:-/mnt/1panel/apps/openresty/openresty/www/sites/test.cngokz.com/index/cnapi}"
SERVICE="${SERVICE:-cngokz-api}"
HEALTH_URL="${HEALTH_URL:-http://127.0.0.1:28080/health/ready}"
SITE_URL="${SITE_URL:-https://test.cngokz.com}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

TARGET="$SERVER_USER@$SERVER_HOST"
RSSH="ssh -i $SSH_KEY -o BatchMode=yes -o ConnectTimeout=20 -o StrictHostKeyChecking=accept-new"
STAGE_HOST="/tmp/cngokz-api.upstream"

log() { printf '\n\033[1m==> %s\033[0m\n' "$*"; }
die() { printf '\033[31mERROR: %s\033[0m\n' "$*" >&2; exit 1; }

# ---------------------------------------------------------------------------
# 0. 前置检查
# ---------------------------------------------------------------------------
log "检查连接与前置条件"
[ -f "$SSH_KEY" ] || die "找不到 SSH 私钥：$SSH_KEY"
$RSSH "$TARGET" "test -d '$APP_DIR' && test -f '$APP_DIR/.env'" \
  || die "服务器上 $APP_DIR 或 .env 不存在"
$RSSH "$TARGET" "systemctl list-unit-files | grep -q '^$SERVICE.service'" \
  || die "服务器上没有 $SERVICE.service"
echo "SSH 与目录检查通过（$TARGET:$APP_DIR）"

# ---------------------------------------------------------------------------
# 1. 构建
# ---------------------------------------------------------------------------
if [ "${SKIP_BUILD:-0}" != "1" ]; then
  log "构建前端"
  command -v pnpm >/dev/null || die "本机没有 pnpm"
  pnpm install --frozen-lockfile >/dev/null
  VITE_SITE_URL="$SITE_URL" pnpm --filter @cngokz/frontend build >/dev/null
  echo "前端产物：$(du -sh apps/frontend/dist | cut -f1)"

  log "构建后端（release）"
  command -v cargo >/dev/null || die "本机没有 cargo"
  cargo build --release --locked --bin cngokz-api 2>&1 | tail -2
  echo "后端产物：$(du -h target/release/cngokz-api | cut -f1)"
fi

# ---------------------------------------------------------------------------
# 2. 上传前端
# ---------------------------------------------------------------------------
log "上传前端（保留 bin/ 与 .env）"
rsync -az --delete \
  --exclude 'bin/' --exclude '.env' --exclude '.env.*' --exclude '.git*' \
  -e "$RSSH" apps/frontend/dist/ "$TARGET:$APP_DIR/"
echo "完成"

# ---------------------------------------------------------------------------
# 3. 上传后端（压缩传输 + 校验）
# ---------------------------------------------------------------------------
log "上传后端二进制（压缩后上传并校验）"
gzip -9 -c target/release/cngokz-api > /tmp/cngokz-api.deploy.gz
echo "压缩后：$(du -h /tmp/cngokz-api.deploy.gz | cut -f1)"
cat /tmp/cngokz-api.deploy.gz | $RSSH "$TARGET" "cat > $STAGE_HOST.gz"
$RSSH "$TARGET" "gunzip -c $STAGE_HOST.gz > $STAGE_HOST && chmod 0755 $STAGE_HOST && rm -f $STAGE_HOST.gz"

LOCAL_SUM=$(sha256sum target/release/cngokz-api | awk '{print $1}')
REMOTE_SUM=$($RSSH "$TARGET" "sha256sum $STAGE_HOST | awk '{print \$1}'")
[ "$LOCAL_SUM" = "$REMOTE_SUM" ] || { $RSSH "$TARGET" "rm -f $STAGE_HOST"; die "校验和不一致"; }
echo "校验通过：${LOCAL_SUM:0:16}…"

# ---------------------------------------------------------------------------
# 4. 安装并重启（失败自动回滚）
# ---------------------------------------------------------------------------
log "安装并重启 $SERVICE"
$RSSH "$TARGET" "APP_DIR='$APP_DIR' SERVICE='$SERVICE' HEALTH='$HEALTH_URL' STAGED='$STAGE_HOST' bash -s" <<'REMOTE'
set -euo pipefail

# 保留当前版本用于回滚（不会覆盖更早的 .prev，那是再上一版）
if [ -x "$APP_DIR/bin/cngokz-api" ]; then
  cp -a "$APP_DIR/bin/cngokz-api" "$APP_DIR/bin/cngokz-api.prev"
fi

install -m 0755 "$STAGED" "$APP_DIR/bin/cngokz-api"
chown root:root "$APP_DIR/bin/cngokz-api"
rm -f "$STAGED"

systemctl restart "$SERVICE"

for i in $(seq 1 30); do
  if curl -fsS --max-time 2 "$HEALTH" >/dev/null 2>&1; then
    echo "健康检查通过（${i}s）"
    exit 0
  fi
  sleep 1
done

echo "健康检查失败，回滚到 bin/cngokz-api.prev" >&2
journalctl -u "$SERVICE" -n 40 --no-pager >&2 || true
if [ -f "$APP_DIR/bin/cngokz-api.prev" ]; then
  cp -a "$APP_DIR/bin/cngokz-api.prev" "$APP_DIR/bin/cngokz-api"
  systemctl restart "$SERVICE" || true
fi
exit 1
REMOTE

# 确认落盘的确实是本次产物（健康检查可能被 systemd 反复拉起的旧进程骗过）。
INSTALLED_SUM=$($RSSH "$TARGET" "sha256sum $APP_DIR/bin/cngokz-api | awk '{print \$1}'")
if [ "$INSTALLED_SUM" != "$LOCAL_SUM" ]; then
  die "部署后二进制校验不一致，服务可能仍在运行旧版本（检查 journalctl -u $SERVICE）"
fi

# ---------------------------------------------------------------------------
# 5. 冒烟检查
# ---------------------------------------------------------------------------
log "冒烟检查"
$RSSH "$TARGET" "curl -fsS --max-time 3 '$HEALTH_URL' >/dev/null"
echo "health/ready OK"
if [ -n "$SITE_URL" ]; then
  curl -fsS --max-time 10 "$SITE_URL/api/v1/auth/me" >/dev/null && echo "公网 /api/v1/auth/me OK"
  curl -fsS --max-time 10 -o /dev/null -w "/admin/login -> %{http_code}\n" "$SITE_URL/admin/login"
fi

log "部署完成"
echo "回滚命令：ssh -i $SSH_KEY $TARGET 'cp -a $APP_DIR/bin/cngokz-api.prev $APP_DIR/bin/cngokz-api && systemctl restart $SERVICE'"
