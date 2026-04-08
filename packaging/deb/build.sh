#!/bin/bash
# Build a .deb package for OpenUSB server
# Usage: ./build.sh <target> [version]
# Example: ./build.sh x86_64-unknown-linux-gnu 0.1.0

set -e

TARGET="${1:-x86_64-unknown-linux-gnu}"
VERSION="${2:-0.1.0}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$(mktemp -d)"

case "$TARGET" in
    x86_64*) ARCH="amd64" ;;
    aarch64*) ARCH="arm64" ;;
    armv7*) ARCH="armhf" ;;
    *) echo "Unknown target: $TARGET"; exit 1 ;;
esac

PKG_NAME="openusb-server_${VERSION}_${ARCH}"
PKG_DIR="$BUILD_DIR/$PKG_NAME"

echo "Building $PKG_NAME..."

# Create package structure
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/local/bin"
mkdir -p "$PKG_DIR/usr/share/openusb/web"
mkdir -p "$PKG_DIR/etc/systemd/system"

# Control file
sed "s/ARCH/$ARCH/" "$SCRIPT_DIR/control" | sed "s/0.1.0/$VERSION/" > "$PKG_DIR/DEBIAN/control"
cp "$SCRIPT_DIR/postinst" "$PKG_DIR/DEBIAN/postinst"
cp "$SCRIPT_DIR/prerm" "$PKG_DIR/DEBIAN/prerm"
chmod 755 "$PKG_DIR/DEBIAN/postinst" "$PKG_DIR/DEBIAN/prerm"

# Binaries
cp "$PROJECT_ROOT/target/$TARGET/release/openusbd" "$PKG_DIR/usr/local/bin/"
cp "$PROJECT_ROOT/target/$TARGET/release/openusb" "$PKG_DIR/usr/local/bin/" 2>/dev/null || true

# Config
cp "$PROJECT_ROOT/server/config/openusb.toml.example" "$PKG_DIR/usr/share/openusb/"

# Systemd service
cp "$PROJECT_ROOT/server/systemd/openusbd.service" "$PKG_DIR/etc/systemd/system/"

# Web dashboard
if [ -d "$PROJECT_ROOT/web-dashboard/dist" ]; then
    cp -r "$PROJECT_ROOT/web-dashboard/dist/"* "$PKG_DIR/usr/share/openusb/web/"
fi

# Build the package
dpkg-deb --build "$PKG_DIR" "$BUILD_DIR/${PKG_NAME}.deb"
mv "$BUILD_DIR/${PKG_NAME}.deb" "$PROJECT_ROOT/"

echo "Package built: $PROJECT_ROOT/${PKG_NAME}.deb"
rm -rf "$BUILD_DIR"
