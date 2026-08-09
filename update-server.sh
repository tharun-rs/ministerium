#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SERVICE_NAME="${MINISTERIUM_SERVICE_NAME:-ministerium}"
INSTALL_DIR="${MINISTERIUM_INSTALL_DIR:-/opt/ministerium}"
INSTALL_USER="${MINISTERIUM_INSTALL_USER:-rst}"
INSTALL_GROUP="${MINISTERIUM_INSTALL_GROUP:-$INSTALL_USER}"

echo "==> Pulling latest code"
git -C "$SCRIPT_DIR" pull --ff-only

echo "==> Building release binary"
cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"

echo "==> Stopping ministerium service"
sudo systemctl stop "$SERVICE_NAME"

echo "==> Updating binary"
sudo install -o "$INSTALL_USER" -g "$INSTALL_GROUP" -m 0755 \
    "$SCRIPT_DIR/target/release/ministerium" \
    "$INSTALL_DIR/ministerium"

echo "==> Starting ministerium service"
sudo systemctl start "$SERVICE_NAME"

echo "==> Deployment complete"
sudo systemctl status "$SERVICE_NAME" --no-pager
