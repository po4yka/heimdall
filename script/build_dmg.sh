#!/usr/bin/env bash
set -euo pipefail

# Build Heimdall.app, the CLI, sign the staged distribution, and produce a
# DMG suitable for sharing. Output: target/dist/heimdall-v<version>-macos-app.dmg
#
# Usage:
#   script/build_dmg.sh [Debug|Release]   (default: Release)
#
# Environment (all optional):
#   HEIMDALL_CODESIGN_IDENTITY     "-" for ad-hoc (default), otherwise a
#                                  Developer ID Application identity.
#   HEIMDALL_NOTARY_APPLE_ID       full notary credentials trigger an
#   HEIMDALL_NOTARY_TEAM_ID        automatic xcrun notarytool submit + staple
#   HEIMDALL_NOTARY_APP_PASSWORD   on the produced DMG.

CONFIGURATION="${1:-Release}"
case "$CONFIGURATION" in
  Debug|Release) ;;
  *) echo "usage: $0 [Debug|Release]" >&2; exit 2 ;;
esac

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT_DIR="$ROOT_DIR/macos/Heimdall"
PROJECT_FILE="$PROJECT_DIR/Heimdall.xcodeproj"
DERIVED_DATA="$PROJECT_DIR/.derived"
PRODUCTS_DIR="$DERIVED_DATA/Build/Products/$CONFIGURATION"

case "$(uname -m)" in
  arm64)  DESTINATION_ARGS=(-destination "platform=macOS,arch=arm64") ;;
  x86_64) DESTINATION_ARGS=(-destination "platform=macOS,arch=x86_64") ;;
  *)      DESTINATION_ARGS=(-destination "platform=macOS") ;;
esac

VERSION="$(awk -F'"' '/^version *=/ { print $2; exit }' "$ROOT_DIR/Cargo.toml")"
[[ -n "$VERSION" ]] || { echo "could not read package version from Cargo.toml" >&2; exit 1; }

DIST_DIR="$ROOT_DIR/target/dist"
STAGING_DIR="$DIST_DIR/heimdall-v${VERSION}-macos-app"
DMG_PATH="$DIST_DIR/heimdall-v${VERSION}-macos-app.dmg"

rm -rf "$STAGING_DIR"
mkdir -p "$DIST_DIR"

(
  cd "$PROJECT_DIR"
  xcodegen generate --use-cache
)

xcodebuild \
  -project "$PROJECT_FILE" \
  -scheme HeimdallApp \
  -configuration "$CONFIGURATION" \
  -derivedDataPath "$DERIVED_DATA" \
  "${DESTINATION_ARGS[@]}" \
  CODE_SIGNING_ALLOWED=NO \
  build >/dev/null

xcodebuild \
  -project "$PROJECT_FILE" \
  -scheme HeimdallTool \
  -configuration "$CONFIGURATION" \
  -derivedDataPath "$DERIVED_DATA" \
  "${DESTINATION_ARGS[@]}" \
  CODE_SIGNING_ALLOWED=NO \
  build >/dev/null

"$ROOT_DIR/script/package_heimdall_distribution.sh" "$PRODUCTS_DIR" "$STAGING_DIR"

IDENTITY="${HEIMDALL_CODESIGN_IDENTITY:--}"
"$ROOT_DIR/script/sign_heimdall_distribution.sh" \
  "$STAGING_DIR/Heimdall.app" \
  "$STAGING_DIR/bin/heimdall" \
  "$IDENTITY"

"$ROOT_DIR/script/package_heimdall_dmg.sh" "$STAGING_DIR/Heimdall.app" "$DMG_PATH"

if [[ -n "${HEIMDALL_NOTARY_APPLE_ID:-}" \
   && -n "${HEIMDALL_NOTARY_TEAM_ID:-}" \
   && -n "${HEIMDALL_NOTARY_APP_PASSWORD:-}" \
   && "$IDENTITY" != "-" ]]; then
  "$ROOT_DIR/script/dmg_notarize.sh" "$DMG_PATH"
fi

echo "DMG ready at: $DMG_PATH"
