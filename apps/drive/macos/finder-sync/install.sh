#!/bin/zsh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
BUILD_DIR="${AIO_DRIVE_FINDER_BUILD_DIR:-$REPO_ROOT/target/macos-finder-sync}"
APP_NAME="AIO Drive Finder.app"
APP_DIR="$BUILD_DIR/$APP_NAME"
if [[ -w "/Applications" ]]; then
  INSTALL_DIR="/Applications"
else
  INSTALL_DIR="$HOME/Applications"
fi
INSTALLED_APP="$INSTALL_DIR/$APP_NAME"
EXTENSION_NAME="AIODriveFinderSync"
EXTENSION_ID="site.addzero.drive.findersync"

"$SCRIPT_DIR/build-app.sh"

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
