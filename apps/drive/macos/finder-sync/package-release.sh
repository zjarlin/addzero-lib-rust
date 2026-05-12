#!/bin/zsh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
BUILD_DIR="${AIO_DRIVE_FINDER_BUILD_DIR:-$REPO_ROOT/target/macos-finder-sync-release}"
RELEASE_DIR="${AIO_DRIVE_RELEASE_DIR:-$REPO_ROOT/target/aio-drive-release}"
APP_NAME="AIO Drive Finder.app"
APP_DIR="$BUILD_DIR/$APP_NAME"
ARCH_RAW="$(uname -m)"
ARCH="$ARCH_RAW"
if [[ "$ARCH_RAW" == "arm64" ]]; then
  ARCH="aarch64"
fi
DMG_PATH="$RELEASE_DIR/aio-drive-finder-macos-$ARCH.dmg"
STAGE_DIR="$RELEASE_DIR/dmg-root"

export AIO_DRIVE_FINDER_BUILD_PROFILE="${AIO_DRIVE_FINDER_BUILD_PROFILE:-release}"
export AIO_DRIVE_FINDER_BUILD_DIR="$BUILD_DIR"
"$SCRIPT_DIR/build-app.sh"

rm -rf "$RELEASE_DIR"
mkdir -p "$STAGE_DIR"
cp -R "$APP_DIR" "$STAGE_DIR/"
ln -s /Applications "$STAGE_DIR/Applications"

cat > "$STAGE_DIR/README.txt" <<'TXT'
AIO Drive Finder for macOS

1. Drag "AIO Drive Finder.app" into Applications.
2. Open "AIO Drive Finder.app" once to install Finder integration.
3. The app will register the Finder extension, install Quick Actions, and restart Finder.
4. If Finder still has no "AIO Drive 托管", open the app again and click "打开扩展设置".
5. Full Disk Access does not pop up automatically on macOS. For protected paths, open the app again and click "打开完全磁盘访问".
6. The bundled app contains az-drive-app for Finder integration.
7. The aio CLI is released separately as a tar.gz asset.
TXT

hdiutil create \
  -volname "AIO Drive Finder" \
  -srcfolder "$STAGE_DIR" \
  -ov \
  -format UDZO \
  "$DMG_PATH" >/dev/null

echo "Packaged $DMG_PATH"
