#!/bin/bash
# Build a macOS .app bundle from the compiled binary
# Usage: ./bundle.sh <path-to-binary> <output-dir>
set -e

BINARY="${1:?Usage: bundle.sh <binary-path> <output-dir>}"
OUTPUT_DIR="${2:-.}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_NAME="OpenUSB.app"
APP_DIR="$OUTPUT_DIR/$APP_NAME"

echo "Creating $APP_NAME..."

# Create .app structure
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# Copy binary
cp "$BINARY" "$APP_DIR/Contents/MacOS/openusb-client"
chmod +x "$APP_DIR/Contents/MacOS/openusb-client"

# Copy Info.plist
cp "$SCRIPT_DIR/Info.plist" "$APP_DIR/Contents/"

# Generate icon (simple green circle as .icns)
# If iconutil is available, create a proper .icns from the generated PNGs
if command -v sips &>/dev/null && command -v iconutil &>/dev/null; then
    ICONSET_DIR=$(mktemp -d)/AppIcon.iconset
    mkdir -p "$ICONSET_DIR"

    # Create a simple green circle PNG using sips-compatible approach
    # We'll create the icon from a base 1024x1024 PNG
    python3 -c "
import struct, zlib

def create_png(size):
    raw = []
    center = size / 2
    radius = size * 0.375
    for y in range(size):
        raw.append(b'\\x00')  # filter byte
        for x in range(size):
            dx, dy = x - center, y - center
            dist = (dx*dx + dy*dy) ** 0.5
            if dist <= radius:
                raw.append(bytes([0x22, 0xc5, 0x5e, 0xff]))
            elif dist <= radius + 1:
                alpha = max(0, min(255, int(255 * (radius + 1 - dist))))
                raw.append(bytes([0x22, 0xc5, 0x5e, alpha]))
            else:
                raw.append(bytes([0, 0, 0, 0]))

    raw_data = b''.join(raw)

    def chunk(ctype, data):
        c = ctype + data
        return struct.pack('>I', len(data)) + c + struct.pack('>I', zlib.crc32(c) & 0xffffffff)

    ihdr = struct.pack('>IIBBBBB', size, size, 8, 6, 0, 0, 0)
    idat = zlib.compress(raw_data)

    return b'\\x89PNG\\r\\n\\x1a\\n' + chunk(b'IHDR', ihdr) + chunk(b'IDAT', idat) + chunk(b'IEND', b'')

import sys
with open(sys.argv[1], 'wb') as f:
    f.write(create_png(1024))
" "$ICONSET_DIR/icon_512x512@2x.png"

    # Create all required sizes
    for size in 16 32 64 128 256 512; do
        sips -z $size $size "$ICONSET_DIR/icon_512x512@2x.png" --out "$ICONSET_DIR/icon_${size}x${size}.png" &>/dev/null
    done
    cp "$ICONSET_DIR/icon_32x32.png" "$ICONSET_DIR/icon_16x16@2x.png"
    cp "$ICONSET_DIR/icon_64x64.png" "$ICONSET_DIR/icon_32x32@2x.png"
    cp "$ICONSET_DIR/icon_256x256.png" "$ICONSET_DIR/icon_128x128@2x.png"
    cp "$ICONSET_DIR/icon_512x512.png" "$ICONSET_DIR/icon_256x256@2x.png"
    rm -f "$ICONSET_DIR/icon_64x64.png"

    iconutil -c icns "$ICONSET_DIR" -o "$APP_DIR/Contents/Resources/AppIcon.icns" 2>/dev/null || true
    rm -rf "$(dirname "$ICONSET_DIR")"
fi

echo "Created $APP_DIR"

# Create DMG if hdiutil is available
if command -v hdiutil &>/dev/null; then
    DMG_PATH="$OUTPUT_DIR/OpenUSB.dmg"
    DMG_TEMP=$(mktemp -d)
    cp -R "$APP_DIR" "$DMG_TEMP/"
    ln -s /Applications "$DMG_TEMP/Applications"
    hdiutil create -volname "OpenUSB" -srcfolder "$DMG_TEMP" -ov -format UDZO "$DMG_PATH" &>/dev/null
    rm -rf "$DMG_TEMP"
    echo "Created $DMG_PATH"
fi
