#!/usr/bin/env bash
set -euo pipefail

# OpenUSB Server Installer
# Usage: curl -fsSL https://get.openusb.dev | bash

REPO="kman1898/usb-passthrough"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/openusb"
LOG_DIR="/var/log/openusb"
DATA_DIR="/var/lib/openusb"

echo "==================================="
echo "  OpenUSB Server Installer"
echo "==================================="
echo

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    aarch64|arm64) ARCH="aarch64" ;;
    x86_64|amd64)  ARCH="x86_64" ;;
    armv7l)        ARCH="armv7" ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

echo "Detected architecture: $ARCH"

# Check for required kernel modules
if ! modinfo usbip_host &>/dev/null; then
    echo "Loading USB/IP kernel modules..."
    modprobe usbip_core
    modprobe usbip_host
    # Persist across reboots
    echo "usbip_core" >> /etc/modules-load.d/openusb.conf
    echo "usbip_host" >> /etc/modules-load.d/openusb.conf
fi

# Install usbip tools if not present
if ! command -v usbip &>/dev/null; then
    echo "Installing USB/IP tools..."
    if command -v apt-get &>/dev/null; then
        apt-get update -qq
        apt-get install -y -qq linux-tools-generic usbip 2>/dev/null || \
            apt-get install -y -qq usbutils
    elif command -v dnf &>/dev/null; then
        dnf install -y -q kmod-usbip usbip-utils
    else
        echo "Warning: Could not install usbip tools automatically."
        echo "Please install them manually for your distribution."
    fi
fi

# Create directories
mkdir -p "$CONFIG_DIR" "$LOG_DIR" "$DATA_DIR"

# Download latest release
echo "Downloading OpenUSB server..."
LATEST=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep tag_name | cut -d'"' -f4)

if [ -z "$LATEST" ]; then
    echo "No release found. Building from source..."
    echo "(Release downloads will be available once the project is published)"
    exit 1
fi

DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST/openusbd-linux-$ARCH"
curl -fsSL -o "$INSTALL_DIR/openusbd" "$DOWNLOAD_URL"
chmod +x "$INSTALL_DIR/openusbd"

# Install default config if not present
if [ ! -f "$CONFIG_DIR/openusb.toml" ]; then
    echo "Installing default configuration..."
    curl -fsSL "https://raw.githubusercontent.com/$REPO/main/server/config/openusb.toml.example" \
        -o "$CONFIG_DIR/openusb.toml"
fi

# Install systemd service
echo "Installing systemd service..."
curl -fsSL "https://raw.githubusercontent.com/$REPO/main/server/systemd/openusbd.service" \
    -o /etc/systemd/system/openusbd.service
systemctl daemon-reload
systemctl enable openusbd
systemctl start openusbd

echo
echo "==================================="
echo "  OpenUSB installed successfully!"
echo "==================================="
echo
echo "  Config: $CONFIG_DIR/openusb.toml"
echo "  Logs:   $LOG_DIR/openusb.log"
echo "  Status: systemctl status openusbd"
echo
echo "  Your USB devices are now shared on the network."
echo
