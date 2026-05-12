#!/bin/zsh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
BUILD_DIR="${AIO_DRIVE_FINDER_BUILD_DIR:-$REPO_ROOT/target/macos-finder-sync}"
APP_NAME="AIO Drive Finder.app"
APP_DIR="$BUILD_DIR/$APP_NAME"
APP_EXECUTABLE="AIODriveFinder"
EXTENSION_NAME="AIODriveFinderSync"
EXTENSION_ID="site.addzero.drive.findersync"
EXTENSION_DIR="$APP_DIR/Contents/PlugIns/$EXTENSION_NAME.appex"
APP_ENTITLEMENTS="$BUILD_DIR/app.entitlements"
EXTENSION_ENTITLEMENTS="$BUILD_DIR/extension.entitlements"
APP_ICON_NAME="AIO Drive Finder"
APP_ICONSET="$BUILD_DIR/AppIcon.iconset"
APP_ICON="$BUILD_DIR/AppIcon.icns"
BUILD_VERSION="${AIO_DRIVE_FINDER_BUILD_VERSION:-$(date +%Y%m%d%H%M%S)}"
BUILD_PROFILE="${AIO_DRIVE_FINDER_BUILD_PROFILE:-debug}"

if [[ "$BUILD_PROFILE" != "debug" && "$BUILD_PROFILE" != "release" ]]; then
  echo "AIO_DRIVE_FINDER_BUILD_PROFILE must be debug or release" >&2
  exit 2
fi

echo "Building az-drive-app $BUILD_PROFILE binary..."
if [[ "$BUILD_PROFILE" == "release" ]]; then
  cargo build -p az-drive-app --release
  DRIVE_BINARY="$REPO_ROOT/target/release/az-drive-app"
else
  cargo build -p az-drive-app
  DRIVE_BINARY="$REPO_ROOT/target/debug/az-drive-app"
fi

rm -rf "$APP_DIR" "$APP_ICONSET" "$APP_ICON"
mkdir -p "$APP_DIR/Contents/MacOS" \
  "$APP_DIR/Contents/PlugIns" \
  "$APP_DIR/Contents/Resources" \
  "$EXTENSION_DIR/Contents/MacOS" \
  "$EXTENSION_DIR/Contents/Resources"

python3 - "$APP_ICONSET" <<'PY'
import math
import os
import struct
import sys
import zlib

iconset = sys.argv[1]
os.makedirs(iconset, exist_ok=True)

def write_png(path, width, height, rows):
    raw = b"".join(b"\x00" + bytes(row) for row in rows)
    def chunk(kind, data):
        return (
            struct.pack(">I", len(data))
            + kind
            + data
            + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)
        )
    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )
    with open(path, "wb") as file:
        file.write(png)

def rounded_rect_mask(x, y, left, top, right, bottom, radius):
    if x < left or x >= right or y < top or y >= bottom:
        return 0.0
    cx = min(max(x, left + radius), right - radius - 1)
    cy = min(max(y, top + radius), bottom - radius - 1)
    dist = math.hypot(x - cx, y - cy)
    return 1.0 if dist <= radius else 0.0

def line_distance(px, py, ax, ay, bx, by):
    vx, vy = bx - ax, by - ay
    wx, wy = px - ax, py - ay
    denom = vx * vx + vy * vy
    t = 0.0 if denom == 0 else max(0.0, min(1.0, (wx * vx + wy * vy) / denom))
    qx, qy = ax + t * vx, ay + t * vy
    return math.hypot(px - qx, py - qy)

def draw(size):
    scale = size / 1024.0
    rows = []
    for y in range(size):
        row = bytearray()
        for x in range(size):
            px, py = x / scale, y / scale
            r = g = b = a = 0

            app = rounded_rect_mask(px, py, 72, 72, 952, 952, 210)
            if app:
                t = py / 1024.0
                r = int(20 + 18 * t)
                g = int(118 + 32 * t)
                b = int(235 + 16 * t)
                a = 255

            tab = rounded_rect_mask(px, py, 178, 230, 510, 390, 52)
            if tab:
                r, g, b, a = 240, 253, 247, 255

            body = rounded_rect_mask(px, py, 152, 330, 872, 760, 72)
            if body:
                t = (py - 330) / 430.0
                r = int(213 - 30 * t)
                g = int(249 - 34 * t)
                b = int(236 - 18 * t)
                a = 255

            cx, cy, rad = 290, 600, 126
            if math.hypot(px - cx, py - cy) <= rad:
                r, g, b, a = 19, 186, 84, 255

            check = min(
                line_distance(px, py, 230, 604, 278, 652),
                line_distance(px, py, 278, 652, 364, 532),
            )
            if check <= 20:
                r, g, b, a = 255, 255, 255, 255

            acx, acy, ar = 680, 560, 102
            ad = abs(math.hypot(px - acx, py - acy) - ar)
            if ad <= 17 and not (px < acx - 24 and py < acy - 38):
                r, g, b, a = 255, 255, 255, 255
            head = (
                px > 738 and px < 820 and py > 428 and py < 520
                and (py - 428) > abs(px - 779) * 0.75
            )
            if head:
                r, g, b, a = 255, 255, 255, 255

            row.extend([r, g, b, a])
        rows.append(row)
    return rows

files = [
    ("icon_16x16.png", 16),
    ("icon_16x16@2x.png", 32),
    ("icon_32x32.png", 32),
    ("icon_32x32@2x.png", 64),
    ("icon_128x128.png", 128),
    ("icon_128x128@2x.png", 256),
    ("icon_256x256.png", 256),
    ("icon_256x256@2x.png", 512),
    ("icon_512x512.png", 512),
    ("icon_512x512@2x.png", 1024),
]
for name, size in files:
    write_png(os.path.join(iconset, name), size, size, draw(size))
PY

iconutil -c icns "$APP_ICONSET" -o "$APP_ICON"
cp "$APP_ICON" "$APP_DIR/Contents/Resources/$APP_ICON_NAME.icns"
cp "$APP_ICON" "$EXTENSION_DIR/Contents/Resources/$APP_ICON_NAME.icns"

cat > "$APP_DIR/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key><string>en</string>
  <key>CFBundleDisplayName</key><string>AIO Drive Finder</string>
  <key>CFBundleExecutable</key><string>$APP_EXECUTABLE</string>
  <key>CFBundleIconFile</key><string>$APP_ICON_NAME</string>
  <key>CFBundleIdentifier</key><string>site.addzero.drive.finder</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>CFBundleName</key><string>AIO Drive Finder</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>1.0</string>
  <key>CFBundleSupportedPlatforms</key><array><string>MacOSX</string></array>
  <key>CFBundleVersion</key><string>$BUILD_VERSION</string>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
  <key>LSUIElement</key><true/>
  <key>NSPrincipalClass</key><string>NSApplication</string>
</dict>
</plist>
PLIST

cat > "$EXTENSION_DIR/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key><string>en</string>
  <key>CFBundleDisplayName</key><string>AIO Drive Finder Sync</string>
  <key>CFBundleExecutable</key><string>$EXTENSION_NAME</string>
  <key>CFBundleIconFile</key><string>$APP_ICON_NAME</string>
  <key>CFBundleIdentifier</key><string>$EXTENSION_ID</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>CFBundleName</key><string>AIO Drive Finder Sync</string>
  <key>CFBundlePackageType</key><string>XPC!</string>
  <key>CFBundleShortVersionString</key><string>1.0</string>
  <key>CFBundleSupportedPlatforms</key><array><string>MacOSX</string></array>
  <key>CFBundleVersion</key><string>$BUILD_VERSION</string>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
  <key>LSUIElement</key><true/>
  <key>NSExtension</key>
  <dict>
    <key>NSExtensionAttributes</key>
    <dict>
      <key>NSExtensionVersion</key><string>1.0</string>
    </dict>
    <key>NSExtensionPointIdentifier</key><string>com.apple.FinderSync</string>
    <key>NSExtensionPrincipalClass</key><string>FinderSync</string>
  </dict>
  <key>NSPrincipalClass</key><string>NSApplication</string>
</dict>
</plist>
PLIST

xcrun clang \
  -fobjc-arc \
  -mmacosx-version-min=12.0 \
  -framework Cocoa \
  -framework FinderSync \
  "$SCRIPT_DIR/App/ContainerApp.m" \
  -o "$APP_DIR/Contents/MacOS/$APP_EXECUTABLE"

xcrun clang \
  -fobjc-arc \
  -mmacosx-version-min=12.0 \
  -framework Cocoa \
  -framework FinderSync \
  "$SCRIPT_DIR/Extension/FinderSync.m" \
  -o "$EXTENSION_DIR/Contents/MacOS/$EXTENSION_NAME"

cp "$DRIVE_BINARY" "$APP_DIR/Contents/MacOS/az-drive-app"

cat > "$APP_ENTITLEMENTS" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>com.apple.security.app-sandbox</key><true/>
  <key>com.apple.security.files.user-selected.read-write</key><true/>
</dict>
</plist>
PLIST

cat > "$EXTENSION_ENTITLEMENTS" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>com.apple.security.app-sandbox</key><true/>
  <key>com.apple.security.files.user-selected.read-write</key><true/>
  <key>com.apple.security.network.client</key><true/>
  <key>com.apple.security.network.server</key><true/>
  <key>com.apple.security.temporary-exception.files.home-relative-path.read-write</key>
  <array>
    <string>/</string>
    <string>/.config/aio/</string>
    <string>/.config/az-drive/</string>
    <string>/Library/Logs/</string>
  </array>
  <key>com.apple.security.temporary-exception.files.absolute-path.read-write</key>
  <array>
    <string>/Volumes/</string>
  </array>
</dict>
</plist>
PLIST

/usr/bin/codesign --force --sign - --entitlements "$EXTENSION_ENTITLEMENTS" "$EXTENSION_DIR" >/dev/null
/usr/bin/codesign --force --sign - "$APP_DIR/Contents/MacOS/az-drive-app" >/dev/null
/usr/bin/codesign --force --sign - --entitlements "$APP_ENTITLEMENTS" "$APP_DIR" >/dev/null

echo "Built $APP_DIR"
