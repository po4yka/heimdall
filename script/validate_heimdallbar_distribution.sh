#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <staging-dir>" >&2
  exit 2
fi

STAGING_DIR="$1"
APP_BUNDLE="$STAGING_DIR/HeimdallBar.app"
APP_BINARY="$APP_BUNDLE/Contents/MacOS/HeimdallBar"
HELPER_BINARY="$APP_BUNDLE/Contents/Helpers/claude-usage-tracker"
WIDGET_BUNDLE="$APP_BUNDLE/Contents/PlugIns/HeimdallBarWidget.appex"
CLI_BINARY="$STAGING_DIR/bin/heimdallbar"
FRAMEWORK_DIR="$STAGING_DIR/Frameworks"
SHARED_FRAMEWORK="$FRAMEWORK_DIR/HeimdallBarShared.framework"
WIDGET_INFO="$WIDGET_BUNDLE/Contents/Info.plist"

[[ -d "$APP_BUNDLE" ]] || { echo "missing app bundle: $APP_BUNDLE" >&2; exit 1; }
[[ -x "$APP_BINARY" ]] || { echo "missing app executable: $APP_BINARY" >&2; exit 1; }
[[ -x "$HELPER_BINARY" ]] || { echo "missing helper binary: $HELPER_BINARY" >&2; exit 1; }
[[ -d "$WIDGET_BUNDLE" ]] || { echo "missing widget extension: $WIDGET_BUNDLE" >&2; exit 1; }
[[ -x "$CLI_BINARY" ]] || { echo "missing CLI binary: $CLI_BINARY" >&2; exit 1; }
[[ -d "$SHARED_FRAMEWORK" ]] || { echo "missing shared framework: $SHARED_FRAMEWORK" >&2; exit 1; }
[[ -f "$WIDGET_INFO" ]] || { echo "missing widget Info.plist: $WIDGET_INFO" >&2; exit 1; }

EXPECTED_WIDGET_ID="dev.heimdall.HeimdallBar.widget"
ACTUAL_WIDGET_ID="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$WIDGET_INFO")"
[[ "$ACTUAL_WIDGET_ID" == "$EXPECTED_WIDGET_ID" ]] || {
  echo "unexpected widget bundle id: $ACTUAL_WIDGET_ID" >&2
  exit 1
}

codesign --verify --deep --strict --verbose=2 "$APP_BUNDLE"
codesign --verify --strict --verbose=2 "$CLI_BINARY"
codesign --verify --strict --verbose=2 "$SHARED_FRAMEWORK"

if [[ "${HEIMDALL_REQUIRE_STAPLE:-0}" == "1" ]]; then
  xcrun stapler validate "$APP_BUNDLE"
fi

DYLD_FRAMEWORK_PATH="$FRAMEWORK_DIR" "$CLI_BINARY" config dump >/dev/null
