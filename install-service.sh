#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

echo "[1/4] Building release binary..."
cargo build --release

echo "[2/4] Installing systemd user unit..."
mkdir -p "$HOME/.config/systemd/user"
cp goguardian-docs.service "$HOME/.config/systemd/user/"

echo "[3/4] Reloading and enabling goguardian-docs.service..."
systemctl --user daemon-reload
systemctl --user enable --now goguardian-docs.service

echo "[4/4] Enabling systemd-logind linger for $USER (survives logout)..."
if ! loginctl enable-linger "$USER" 2>/dev/null; then
  echo "Unprivileged loginctl failed; trying with sudo..."
  sudo loginctl enable-linger "$USER"
fi

loginctl show-user "$USER" -p Linger
systemctl --user status goguardian-docs.service --no-pager
