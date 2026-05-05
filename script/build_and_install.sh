#!/usr/bin/env bash
set -euo pipefail

# Build the SwiftUI app + Rust helper (bundled by the Xcode Run Script phase
# via bundle_heimdall_helper.sh) and the heimdall-hook binary, then drop
# Heimdall.app into $HEIMDALL_INSTALL_DIR (default /Applications). heimdall-hook
# is built separately because the app bundle only carries the dashboard helper.

CONFIGURATION="${1:-Release}"
APP_NAME="Heimdall"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT_DIR="$ROOT_DIR/macos/Heimdall"
PROJECT_FILE="$PROJECT_DIR/Heimdall.xcodeproj"
DERIVED_DATA="$PROJECT_DIR/.derived"
INSTALL_DIR="${HEIMDALL_INSTALL_DIR:-/Applications}"

case "$CONFIGURATION" in
  Debug|Release) ;;
  *)
    echo "usage: $0 [Debug|Release]" >&2
    exit 2
    ;;
esac

APP_BUNDLE="$DERIVED_DATA/Build/Products/$CONFIGURATION/$APP_NAME.app"

case "$(uname -m)" in
  arm64)
    DESTINATION_ARGS=(-destination "platform=macOS,arch=arm64")
    ;;
  x86_64)
    DESTINATION_ARGS=(-destination "platform=macOS,arch=x86_64")
    ;;
  *)
    DESTINATION_ARGS=(-destination "platform=macOS")
    ;;
esac

pkill -x "$APP_NAME" >/dev/null 2>&1 || true

(
  cd "$PROJECT_DIR"
  /opt/homebrew/bin/xcodegen generate --use-cache
)

xcodebuild \
  -project "$PROJECT_FILE" \
  -scheme HeimdallApp \
  -configuration "$CONFIGURATION" \
  -derivedDataPath "$DERIVED_DATA" \
  "${DESTINATION_ARGS[@]}" \
  CODE_SIGNING_ALLOWED=NO \
  build >/dev/null

CARGO_FLAGS=()
[[ "$CONFIGURATION" == "Release" ]] && CARGO_FLAGS+=(--release)
cargo build "${CARGO_FLAGS[@]}" \
  --manifest-path "$ROOT_DIR/Cargo.toml" \
  --bin heimdall-hook >/dev/null

[[ -d "$APP_BUNDLE" ]] || { echo "missing app bundle: $APP_BUNDLE" >&2; exit 1; }

mkdir -p "$INSTALL_DIR"
DEST="$INSTALL_DIR/$APP_NAME.app"
rm -rf "$DEST"
cp -R "$APP_BUNDLE" "$DEST"

echo "installed: $DEST"
