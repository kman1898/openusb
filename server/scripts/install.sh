#!/usr/bin/env bash
set -euo pipefail

# OpenUSB Server Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/kman1898/openusb/main/server/scripts/install.sh | sudo bash

REPO="kman1898/openusb"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/openusb"
LOG_DIR="/var/log/openusb"
DATA_DIR="/var/lib/openusb"
WEB_DIR="/usr/share/openusb/web"

echo "==================================="
echo "  OpenUSB Server Installer"
echo "==================================="
echo

# Must be root
if [ "$(id -u)" -ne 0 ]; then
    echo "Error: This script must be run as root (use sudo)"
    exit 1
fi

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    aarch64|arm64) RELEASE_ARCH="linux_arm64" ;;
    x86_64|amd64)  RELEASE_ARCH="linux_x86_64" ;;
    armv7l)        RELEASE_ARCH="linux_armv7" ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

echo "Detected architecture: $ARCH ($RELEASE_ARCH)"

# Check for required kernel modules
echo "Setting up USB/IP kernel modules..."
modprobe usbip_core 2>/dev/null || true
modprobe usbip_host 2>/dev/null || true

# Persist across reboots
mkdir -p /etc/modules-load.d
cat > /etc/modules-load.d/openusb.conf << 'MODULES'
usbip_core
usbip_host
MODULES

# Install usbip tools if not present
if ! command -v usbip &>/dev/null; then
    echo "Installing USB/IP tools..."
    if command -v apt-get &>/dev/null; then
        apt-get update -qq
        apt-get install -y -qq linux-tools-generic 2>/dev/null || \
            apt-get install -y -qq usbip 2>/dev/null || \
            apt-get install -y -qq usbutils
    elif command -v dnf &>/dev/null; then
        dnf install -y -q usbip-utils
    elif command -v pacman &>/dev/null; then
        pacman -S --noconfirm usbip
    else
        echo "Warning: Could not install usbip tools automatically."
        echo "Please install them manually for your distribution."
    fi
fi

# Create directories
mkdir -p "$CONFIG_DIR" "$LOG_DIR" "$DATA_DIR" "$WEB_DIR"

# Download latest release
echo "Fetching latest release..."
LATEST=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep tag_name | cut -d'"' -f4)

if [ -z "$LATEST" ]; then
    echo "Error: No release found on GitHub."
    echo "Visit https://github.com/$REPO/releases to check for releases."
    exit 1
fi

echo "Installing OpenUSB $LATEST..."

# Download and extract server + CLI + dashboard
VERSION="${LATEST#v}"
ARCHIVE_URL="https://github.com/$REPO/releases/download/$LATEST/openusb_v${VERSION}_${RELEASE_ARCH}.tar.gz"
echo "Downloading $ARCHIVE_URL ..."

TEMP_DIR=$(mktemp -d)
curl -fsSL "$ARCHIVE_URL" | tar xz -C "$TEMP_DIR"

# Install binaries
cp "$TEMP_DIR/openusbd" "$INSTALL_DIR/" 2>/dev/null || true
cp "$TEMP_DIR/openusb" "$INSTALL_DIR/" 2>/dev/null || true
chmod +x "$INSTALL_DIR/openusbd" "$INSTALL_DIR/openusb" 2>/dev/null || true

# Install web dashboard if included in archive
if [ -d "$TEMP_DIR/web" ]; then
    echo "Installing web dashboard..."
    cp -r "$TEMP_DIR/web/"* "$WEB_DIR/"
fi

rm -rf "$TEMP_DIR"

# Install default config if not present
if [ ! -f "$CONFIG_DIR/openusb.toml" ]; then
    echo "Installing default configuration..."
    curl -fsSL "https://raw.githubusercontent.com/$REPO/$LATEST/server/config/openusb.toml.example" \
        -o "$CONFIG_DIR/openusb.toml" 2>/dev/null || \
    curl -fsSL "https://raw.githubusercontent.com/$REPO/main/server/config/openusb.toml.example" \
        -o "$CONFIG_DIR/openusb.toml"
fi

# Install and start service
if command -v systemctl &>/dev/null; then
    echo "Installing systemd service..."
    curl -fsSL "https://raw.githubusercontent.com/$REPO/main/server/systemd/openusbd.service" \
        -o /etc/systemd/system/openusbd.service
    systemctl daemon-reload
    systemctl enable openusbd
    systemctl restart openusbd
    echo "Service started via systemd."
else
    echo "systemd not found. Starting server directly..."
    # Kill any existing instance
    pkill -f openusbd 2>/dev/null || true
    sleep 1
    # Start in background
    nohup "$INSTALL_DIR/openusbd" --config "$CONFIG_DIR/openusb.toml" \
        > "$LOG_DIR/openusb.log" 2>&1 &
    echo "Server started (PID: $!)."
    echo ""
    echo "  To auto-start on boot, add to /etc/rc.local:"
    echo "  $INSTALL_DIR/openusbd --config $CONFIG_DIR/openusb.toml &"
fi

# Get the IP address for the dashboard URL
IP_ADDR=$(hostname -I 2>/dev/null | awk '{print $1}' || echo "localhost")

echo
echo "==================================="
echo "  OpenUSB $LATEST installed!"
echo "==================================="
echo
echo "  Dashboard: http://${IP_ADDR}:8443"
echo "  Config:    $CONFIG_DIR/openusb.toml"
echo "  Logs:      $LOG_DIR/openusb.log"
echo "  Status:    ps aux | grep openusbd"
echo
echo "  Default login: admin / admin"
echo "  CHANGE THE DEFAULT PASSWORD IMMEDIATELY!"
echo
