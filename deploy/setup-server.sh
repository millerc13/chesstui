#!/usr/bin/env bash
#
# One-time server setup. Run this on the target machine as root.
#
# Usage:  sudo bash deploy/setup-server.sh
#
set -euo pipefail

info() { printf "\033[1;34m=>\033[0m %s\n" "$*"; }

info "Creating chesstui system user..."
id -u chesstui &>/dev/null || useradd --system --shell /usr/sbin/nologin --create-home chesstui

info "Creating state directory..."
mkdir -p /var/lib/chesstui
chown chesstui:chesstui /var/lib/chesstui

info "Installing systemd service..."
cp "$(dirname "$0")/chesstui-server.service" /etc/systemd/system/chesstui-server.service
systemctl daemon-reload
systemctl enable chesstui-server

info "Configuring firewall (ufw)..."
if command -v ufw > /dev/null 2>&1; then
  ufw allow 7600/tcp comment "chesstui server"
  info "Opened port 7600/tcp"
else
  info "ufw not found, ensure port 7600/tcp is open manually"
fi

info "Server setup complete."
info "Deploy a binary with: ./deploy/deploy.sh --host $(hostname -I | awk '{print $1}')"
