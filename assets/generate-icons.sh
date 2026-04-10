#!/bin/bash
# Generate platform-specific icons from the SVG source
# Requires: rsvg-convert (librsvg) or Inkscape, and ImageMagick
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SVG="$SCRIPT_DIR/icon.svg"

echo "Generating icons from $SVG..."

# Check for rsvg-convert or inkscape
if command -v rsvg-convert &>/dev/null; then
    CONVERT_CMD="rsvg-convert"
elif command -v inkscape &>/dev/null; then
    CONVERT_CMD="inkscape"
else
    echo "Error: Need rsvg-convert (librsvg) or inkscape to convert SVG to PNG"
    echo "  macOS: brew install librsvg"
    echo "  Ubuntu: sudo apt install librsvg2-bin"
    exit 1
fi

# Generate PNGs at various sizes
for size in 16 32 48 64 128 256 512 1024; do
    if [ "$CONVERT_CMD" = "rsvg-convert" ]; then
        rsvg-convert -w $size -h $size "$SVG" > "$SCRIPT_DIR/icon-${size}.png"
    else
        inkscape -w $size -h $size "$SVG" -o "$SCRIPT_DIR/icon-${size}.png"
    fi
    echo "  icon-${size}.png"
done

# Generate .ico for Windows (multiple sizes in one file)
if command -v convert &>/dev/null; then
    convert "$SCRIPT_DIR/icon-16.png" "$SCRIPT_DIR/icon-32.png" "$SCRIPT_DIR/icon-48.png" "$SCRIPT_DIR/icon-256.png" "$SCRIPT_DIR/openusb.ico"
    echo "  openusb.ico"
elif command -v magick &>/dev/null; then
    magick "$SCRIPT_DIR/icon-16.png" "$SCRIPT_DIR/icon-32.png" "$SCRIPT_DIR/icon-48.png" "$SCRIPT_DIR/icon-256.png" "$SCRIPT_DIR/openusb.ico"
    echo "  openusb.ico"
else
    echo "  Skipping .ico (need ImageMagick)"
fi

# Generate macOS .icns
if command -v iconutil &>/dev/null; then
    ICONSET="$SCRIPT_DIR/AppIcon.iconset"
    mkdir -p "$ICONSET"
    cp "$SCRIPT_DIR/icon-16.png" "$ICONSET/icon_16x16.png"
    cp "$SCRIPT_DIR/icon-32.png" "$ICONSET/icon_16x16@2x.png"
    cp "$SCRIPT_DIR/icon-32.png" "$ICONSET/icon_32x32.png"
    cp "$SCRIPT_DIR/icon-64.png" "$ICONSET/icon_32x32@2x.png"
    cp "$SCRIPT_DIR/icon-128.png" "$ICONSET/icon_128x128.png"
    cp "$SCRIPT_DIR/icon-256.png" "$ICONSET/icon_128x128@2x.png"
    cp "$SCRIPT_DIR/icon-256.png" "$ICONSET/icon_256x256.png"
    cp "$SCRIPT_DIR/icon-512.png" "$ICONSET/icon_256x256@2x.png"
    cp "$SCRIPT_DIR/icon-512.png" "$ICONSET/icon_512x512.png"
    cp "$SCRIPT_DIR/icon-1024.png" "$ICONSET/icon_512x512@2x.png"
    iconutil -c icns "$ICONSET" -o "$SCRIPT_DIR/AppIcon.icns"
    rm -rf "$ICONSET"
    echo "  AppIcon.icns"
else
    echo "  Skipping .icns (need iconutil, macOS only)"
fi

echo "Done!"
