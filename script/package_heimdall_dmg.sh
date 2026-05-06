#!/usr/bin/env bash
set -euo pipefail

# Build a signed, compressed `.dmg` from a signed Heimdall.app bundle.
#
# Usage:
#   script/package_heimdall_dmg.sh <app-bundle> <output-dmg> [volume-name]
#
# Inputs:
#   <app-bundle>   path to a Heimdall.app that is already signed (and ideally
#                  notarized + stapled by the upstream pipeline)
#   <output-dmg>   path where the final compressed DMG will be written
#   [volume-name]  optional Finder volume name; defaults to
#                  "Heimdall <CFBundleShortVersionString>"
#
# Environment:
#   HEIMDALL_CODESIGN_IDENTITY   "-" for ad-hoc (default), otherwise a
#                                Developer ID Application identity. Mirrors
#                                the convention in sign_heimdall_distribution.sh.
#   HEIMDALL_DMG_BACKGROUND      optional path to a background image for the
#                                Finder window. Default: assets/dmg/background.png
#                                if it exists. When absent the layout still
#                                applies (window size, icon positions, view).

if [[ $# -lt 2 || $# -gt 3 ]]; then
  echo "usage: $0 <app-bundle> <output-dmg> [volume-name]" >&2
  exit 2
fi

APP_BUNDLE="$1"
OUTPUT_DMG="$2"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

[[ -d "$APP_BUNDLE" ]] || { echo "missing app bundle: $APP_BUNDLE" >&2; exit 1; }

APP_VERSION="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' \
  "$APP_BUNDLE/Contents/Info.plist" 2>/dev/null || true)"
VOLUME_NAME="${3:-Heimdall ${APP_VERSION:-Installer}}"

IDENTITY="${HEIMDALL_CODESIGN_IDENTITY:--}"
BACKGROUND="${HEIMDALL_DMG_BACKGROUND:-$ROOT_DIR/assets/dmg/background.png}"
ICON="$APP_BUNDLE/Contents/Resources/AppIcon.icns"

STAGE="$(mktemp -d)"
DMG_ROOT="$STAGE/dmg-root"
TMP_DMG="$STAGE/heimdall.tmp.dmg"
MOUNT_POINT=""

cleanup() {
  if [[ -n "$MOUNT_POINT" ]] && mount | grep -F " on $MOUNT_POINT " >/dev/null 2>&1; then
    hdiutil detach "$MOUNT_POINT" -quiet -force >/dev/null 2>&1 || true
  fi
  rm -rf "$STAGE"
}
trap cleanup EXIT

mkdir -p "$DMG_ROOT"
cp -R "$APP_BUNDLE" "$DMG_ROOT/"
ln -s /Applications "$DMG_ROOT/Applications"

if [[ -f "$BACKGROUND" ]]; then
  mkdir -p "$DMG_ROOT/.background"
  cp "$BACKGROUND" "$DMG_ROOT/.background/background.png"
fi

if [[ -f "$ICON" ]]; then
  cp "$ICON" "$DMG_ROOT/.VolumeIcon.icns"
fi

hdiutil create \
  -volname "$VOLUME_NAME" \
  -srcfolder "$DMG_ROOT" \
  -ov \
  -format UDRW \
  -fs HFS+ \
  "$TMP_DMG" >/dev/null

# `hdiutil attach` text output is tab-separated; the mount path is always the
# last column on the row whose path begins with /Volumes/.
ATTACH_OUTPUT="$(hdiutil attach -readwrite -noverify -noautoopen "$TMP_DMG")"
MOUNT_POINT="$(echo "$ATTACH_OUTPUT" | awk -F'\t' '$NF ~ /^\/Volumes\// { print $NF; exit }')"
[[ -d "$MOUNT_POINT" ]] || { echo "could not determine DMG mount point" >&2; exit 1; }

# Let Finder register the volume before issuing AppleScript.
sleep 1

BG_LINE=""
if [[ -f "$DMG_ROOT/.background/background.png" ]]; then
  BG_LINE='set background picture of viewOptions to file ".background:background.png"'
fi

osascript <<APPLESCRIPT
tell application "Finder"
  tell disk "$VOLUME_NAME"
    open
    set current view of container window to icon view
    set toolbar visible of container window to false
    set statusbar visible of container window to false
    set the bounds of container window to {200, 200, 800, 600}
    set viewOptions to icon view options of container window
    set arrangement of viewOptions to not arranged
    set icon size of viewOptions to 128
    $BG_LINE
    set position of item "Heimdall.app" of container window to {160, 200}
    set position of item "Applications" of container window to {440, 200}
    update without registering applications
    delay 1
    close
  end tell
end tell
APPLESCRIPT

if [[ -f "$MOUNT_POINT/.VolumeIcon.icns" ]]; then
  SetFile -a C "$MOUNT_POINT"
fi

sync
hdiutil detach "$MOUNT_POINT" -quiet
MOUNT_POINT=""

mkdir -p "$(dirname "$OUTPUT_DMG")"
hdiutil convert "$TMP_DMG" \
  -format UDZO \
  -imagekey zlib-level=9 \
  -ov \
  -o "$OUTPUT_DMG" >/dev/null

if [[ "$IDENTITY" == "-" ]]; then
  codesign --force --sign - "$OUTPUT_DMG"
else
  codesign --force --sign "$IDENTITY" --timestamp "$OUTPUT_DMG"
fi

hdiutil verify "$OUTPUT_DMG" >/dev/null
codesign --verify --verbose=2 "$OUTPUT_DMG"

echo "created: $OUTPUT_DMG"
