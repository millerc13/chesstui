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

info "Setting up PostgreSQL..."
if command -v psql > /dev/null 2>&1; then
  sudo -u postgres psql -tc "SELECT 1 FROM pg_roles WHERE rolname='chesstui'" | grep -q 1 || \
    sudo -u postgres createuser chesstui
  sudo -u postgres psql -tc "SELECT 1 FROM pg_database WHERE datname='chesstui'" | grep -q 1 || \
    sudo -u postgres createdb -O chesstui chesstui
  info "PostgreSQL database 'chesstui' ready"
else
  info "PostgreSQL not found — install it and create the 'chesstui' database manually"
fi

info "Creating config directory..."
mkdir -p /etc/chesstui
if [ ! -f /etc/chesstui/server.env ]; then
  cp "$(dirname "$0")/server.env.example" /etc/chesstui/server.env
  chmod 600 /etc/chesstui/server.env
  info "Created /etc/chesstui/server.env — edit with your DATABASE_URL and RESEND_API_KEY"
fi

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
