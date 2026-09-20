#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SERVICE_NAME="${MINISTERIUM_SERVICE_NAME:-ministerium}"
INSTALL_DIR="${MINISTERIUM_INSTALL_DIR:-/opt/ministerium}"
INSTALL_USER="${MINISTERIUM_INSTALL_USER:-rst}"
INSTALL_GROUP="${MINISTERIUM_INSTALL_GROUP:-$INSTALL_USER}"
SOURCE_ENV_FILE="${MINISTERIUM_ENV_FILE:-$SCRIPT_DIR/.env}"
TARGET_ENV_FILE="${MINISTERIUM_ENV_INSTALL_FILE:-/etc/ministerium/ministerium.env}"

echo "==> Pulling latest code"
git -C "$SCRIPT_DIR" pull --ff-only

echo "==> Building release binary"
cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"

if [[ ! -f "$SOURCE_ENV_FILE" ]]; then
    echo "ERROR: environment file not found: $SOURCE_ENV_FILE" >&2
    exit 1
fi

echo "==> Installing environment file"
sudo install -d -m 0755 "$(dirname "$TARGET_ENV_FILE")"
sudo install -o root -g root -m 0600 "$SOURCE_ENV_FILE" "$TARGET_ENV_FILE"

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
