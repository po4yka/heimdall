#!/usr/bin/env bash
set -euo pipefail

# usage: build_and_install.sh [Debug|Release|uninstall]
#
# Builds the SwiftUI app + Rust helper (bundled by the Xcode Run Script phase
# via bundle_heimdall_helper.sh) and the heimdall-hook binary, removes any
# previously installed Heimdall surfaces (app bundle, launchd / cron / hook /
# statusline entries via heimdall's own uninstall subcommands), then drops
# Heimdall.app into $HEIMDALL_INSTALL_DIR (default /Applications). Pass
# `uninstall` to remove existing installs without rebuilding. User data under
# ~/.config/heimdall is preserved unless HEIMDALL_PURGE_DATA=1 is set.

MODE="${1:-Release}"
APP_NAME="Heimdall"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT_DIR="$ROOT_DIR/macos/Heimdall"
PROJECT_FILE="$PROJECT_DIR/Heimdall.xcodeproj"
DERIVED_DATA="$PROJECT_DIR/.derived"
INSTALL_DIR="${HEIMDALL_INSTALL_DIR:-/Applications}"

uninstall_existing() {
  pkill -x "$APP_NAME" >/dev/null 2>&1 || true

  # Use heimdall's own uninstall verbs so version-independent ownership
  # markers (HOOK_DESCRIPTION, STATUSLINE_VERSION_KEY, CRON_TAG, plist Label)
  # cleanly remove entries written by older versions.
  local bin
  for bin in "/Applications/$APP_NAME.app/Contents/Helpers/heimdall" \
             "$HOME/Applications/$APP_NAME.app/Contents/Helpers/heimdall" \
             "$(command -v heimdall 2>/dev/null || true)"; do
    [[ -n "$bin" && -x "$bin" ]] || continue
    "$bin" hook uninstall            >/dev/null 2>&1 || true
    "$bin" statusline-hook uninstall >/dev/null 2>&1 || true
    "$bin" scheduler uninstall       >/dev/null 2>&1 || true
    "$bin" daemon uninstall          >/dev/null 2>&1 || true
    break
  done

  local app
  for app in "/Applications/$APP_NAME.app" "$HOME/Applications/$APP_NAME.app"; do
    [[ -d "$app" ]] || continue
    rm -rf "$app"
    echo "removed: $app"
  done

  if [[ "${HEIMDALL_PURGE_DATA:-0}" == "1" ]]; then
    rm -rf "$HOME/.config/heimdall"
    echo "purged: $HOME/.config/heimdall"
  fi
}

case "$MODE" in
  Debug|Release) ;;
  uninstall)
    uninstall_existing
    exit 0
    ;;
  *)
    echo "usage: $0 [Debug|Release|uninstall]" >&2
    exit 2
    ;;
esac

CONFIGURATION="$MODE"
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

uninstall_existing

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
cp -R "$APP_BUNDLE" "$DEST"

echo "installed: $DEST"
