#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-run}"
APP_NAME="HeimdallBar"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT_DIR="$ROOT_DIR/macos/HeimdallBar"
PROJECT_FILE="$PROJECT_DIR/HeimdallBar.xcodeproj"
DERIVED_DATA="$PROJECT_DIR/.derived"
APP_BUNDLE="$DERIVED_DATA/Build/Products/Debug/$APP_NAME.app"
APP_BINARY="$APP_BUNDLE/Contents/MacOS/$APP_NAME"

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
  -scheme HeimdallBarApp \
  -configuration Debug \
  -derivedDataPath "$DERIVED_DATA" \
  "${DESTINATION_ARGS[@]}" \
  CODE_SIGNING_ALLOWED=NO \
  build >/dev/null

open_app() {
  /usr/bin/open -n "$APP_BUNDLE"
}

case "$MODE" in
  run)
    open_app
    ;;
  --debug|debug)
    lldb -- "$APP_BINARY"
    ;;
  --logs|logs)
    open_app
    /usr/bin/log stream --info --style compact --predicate "process == \"$APP_NAME\""
    ;;
  --telemetry|telemetry)
    open_app
    /usr/bin/log stream --info --style compact --predicate "subsystem == \"dev.heimdall.HeimdallBar\""
    ;;
  --verify|verify)
    open_app
    sleep 1
    pgrep -x "$APP_NAME" >/dev/null
    ;;
  *)
    echo "usage: $0 [run|--debug|--logs|--telemetry|--verify]" >&2
    exit 2
    ;;
esac
