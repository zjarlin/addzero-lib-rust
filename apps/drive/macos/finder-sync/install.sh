#!/bin/zsh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
BUILD_DIR="$REPO_ROOT/target/macos-finder-sync"
APP_NAME="AIO Drive Finder.app"
APP_DIR="$BUILD_DIR/$APP_NAME"
if [[ -w "/Applications" ]]; then
  INSTALL_DIR="/Applications"
else
  INSTALL_DIR="$HOME/Applications"
fi
INSTALLED_APP="$INSTALL_DIR/$APP_NAME"
APP_EXECUTABLE="AIODriveFinder"
EXTENSION_NAME="AIODriveFinderSync"
EXTENSION_ID="site.addzero.drive.findersync"
EXTENSION_DIR="$APP_DIR/Contents/PlugIns/$EXTENSION_NAME.appex"
APP_ENTITLEMENTS="$BUILD_DIR/app.entitlements"
EXTENSION_ENTITLEMENTS="$BUILD_DIR/extension.entitlements"
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

rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/PlugIns" "$EXTENSION_DIR/Contents/MacOS"

cat > "$APP_DIR/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key><string>en</string>
  <key>CFBundleDisplayName</key><string>AIO Drive Finder</string>
  <key>CFBundleExecutable</key><string>$APP_EXECUTABLE</string>
  <key>CFBundleIdentifier</key><string>site.addzero.drive.finder</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>CFBundleName</key><string>AIO Drive Finder</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>1.0</string>
  <key>CFBundleSupportedPlatforms</key><array><string>MacOSX</string></array>
  <key>CFBundleVersion</key><string>1</string>
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
  <key>CFBundleIdentifier</key><string>$EXTENSION_ID</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>CFBundleName</key><string>AIO Drive Finder Sync</string>
  <key>CFBundlePackageType</key><string>XPC!</string>
  <key>CFBundleShortVersionString</key><string>1.0</string>
  <key>CFBundleSupportedPlatforms</key><array><string>MacOSX</string></array>
  <key>CFBundleVersion</key><string>1</string>
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

mkdir -p "$INSTALL_DIR"
pluginkit -r "$INSTALLED_APP/Contents/PlugIns/$EXTENSION_NAME.appex" >/dev/null 2>&1 || true
pluginkit -r "$HOME/Applications/$APP_NAME/Contents/PlugIns/$EXTENSION_NAME.appex" >/dev/null 2>&1 || true
pluginkit -r "/Applications/$APP_NAME/Contents/PlugIns/$EXTENSION_NAME.appex" >/dev/null 2>&1 || true
rm -rf "$INSTALLED_APP"
cp -R "$APP_DIR" "$INSTALLED_APP"

LSREGISTER="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
"$LSREGISTER" -f "$INSTALLED_APP"
pluginkit -a "$INSTALLED_APP/Contents/PlugIns/$EXTENSION_NAME.appex"
pluginkit -e use -i "$EXTENSION_ID"
killall Finder >/dev/null 2>&1 || true
sleep 1
pluginkit -a "$INSTALLED_APP/Contents/PlugIns/$EXTENSION_NAME.appex"
pluginkit -e use -i "$EXTENSION_ID"

echo "Installed $INSTALLED_APP"
echo "Finder Sync status:"
pluginkit -m -p com.apple.FinderSync -A -i "$EXTENSION_ID" || true
echo "If macOS still marks it disabled, run: open '$INSTALLED_APP'"
