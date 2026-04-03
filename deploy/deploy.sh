#!/usr/bin/env bash
#
# Deploy chesstui server to a remote host.
#
# Usage:
#   ./deploy/deploy.sh                          # build + deploy
#   ./deploy/deploy.sh --binary ./chesstui      # deploy a pre-built binary
#   DEPLOY_HOST=1.2.3.4 ./deploy/deploy.sh      # override host
#
set -euo pipefail

DEPLOY_HOST="${DEPLOY_HOST:?Set DEPLOY_HOST to the server IP or hostname}"
DEPLOY_USER="${DEPLOY_USER:-chesstui}"
SSH_KEY="${SSH_KEY:-}"
BINARY=""
TARGET="x86_64-unknown-linux-musl"

# Parse args
while [ $# -gt 0 ]; do
  case "$1" in
    --binary) BINARY="$2"; shift 2 ;;
    --host)   DEPLOY_HOST="$2"; shift 2 ;;
    --user)   DEPLOY_USER="$2"; shift 2 ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

SSH_OPTS="-o StrictHostKeyChecking=no -o ConnectTimeout=10"
[ -n "$SSH_KEY" ] && SSH_OPTS="$SSH_OPTS -i $SSH_KEY"

info() { printf "\033[1;34m=>\033[0m %s\n" "$*"; }

# ---- Build if no binary provided ----
if [ -z "$BINARY" ]; then
  info "Building for ${TARGET}..."

  # Check if cross is needed (macOS building for Linux)
  if [ "$(uname -s)" != "Linux" ] || [ "$(uname -m)" != "x86_64" ]; then
    if ! command -v cross > /dev/null 2>&1; then
      info "Installing cross for cross-compilation..."
      cargo install cross --git https://github.com/cross-rs/cross
    fi
    cross build --release --locked --target "$TARGET"
  else
    # Native Linux x86_64 build
    cargo build --release --locked --target "$TARGET"
  fi

  BINARY="target/${TARGET}/release/chesstui"
fi

[ -f "$BINARY" ] || { echo "Binary not found: $BINARY" >&2; exit 1; }

# ---- Deploy ----
info "Deploying to ${DEPLOY_USER}@${DEPLOY_HOST}..."

scp $SSH_OPTS "$BINARY" "${DEPLOY_USER}@${DEPLOY_HOST}:/tmp/chesstui-new"

ssh $SSH_OPTS "${DEPLOY_USER}@${DEPLOY_HOST}" << 'REMOTE'
  set -euo pipefail

  echo "=> Stopping service..."
  sudo systemctl stop chesstui-server || true

  echo "=> Installing binary..."
  sudo mv /tmp/chesstui-new /usr/local/bin/chesstui
  sudo chmod +x /usr/local/bin/chesstui

  echo "=> Verifying binary..."
  /usr/local/bin/chesstui --version

  echo "=> Starting service..."
  sudo systemctl start chesstui-server
  sleep 2

  if sudo systemctl is-active --quiet chesstui-server; then
    echo "=> Server is running"
    sudo systemctl status chesstui-server --no-pager
  else
    echo "=> FAILED to start server"
    sudo journalctl -u chesstui-server -n 20 --no-pager
    exit 1
  fi
REMOTE

info "Deploy complete"
