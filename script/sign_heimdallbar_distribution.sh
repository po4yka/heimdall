#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 || $# -gt 5 ]]; then
  echo "usage: $0 <app-bundle> <cli-binary> [identity] [app-entitlements] [widget-entitlements]" >&2
  exit 2
fi

APP_BUNDLE="$1"
CLI_BINARY="$2"
IDENTITY="${3:-${HEIMDALL_CODESIGN_IDENTITY:-}}"
APP_ENTITLEMENTS="${4:-${HEIMDALL_APP_ENTITLEMENTS:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/macos/HeimdallBar/App/HeimdallBar.entitlements}}"
WIDGET_ENTITLEMENTS="${5:-${HEIMDALL_WIDGET_ENTITLEMENTS:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/macos/HeimdallBar/Widget/HeimdallBarWidget.entitlements}}"
CLI_FRAMEWORKS_DIR="$(cd "$(dirname "$CLI_BINARY")/../Frameworks" && pwd 2>/dev/null || true)"
AD_HOC_SIGNING=0

if [[ "$IDENTITY" == "-" ]]; then
  AD_HOC_SIGNING=1
fi

[[ -n "$IDENTITY" ]] || { echo "missing codesign identity" >&2; exit 1; }
[[ -d "$APP_BUNDLE" ]] || { echo "missing app bundle: $APP_BUNDLE" >&2; exit 1; }
[[ -x "$CLI_BINARY" ]] || { echo "missing CLI binary: $CLI_BINARY" >&2; exit 1; }
[[ -f "$APP_ENTITLEMENTS" ]] || { echo "missing app entitlements: $APP_ENTITLEMENTS" >&2; exit 1; }
[[ -f "$WIDGET_ENTITLEMENTS" ]] || { echo "missing widget entitlements: $WIDGET_ENTITLEMENTS" >&2; exit 1; }

codesign_runtime() {
  if [[ "$AD_HOC_SIGNING" -eq 1 ]]; then
    codesign --force --sign "$IDENTITY" "$1"
  else
    codesign --force --sign "$IDENTITY" --timestamp --options runtime "$1"
  fi
}

codesign_with_entitlements() {
  if [[ "$AD_HOC_SIGNING" -eq 1 ]]; then
    codesign --force --sign "$IDENTITY" --entitlements "$2" "$1"
  else
    codesign --force --sign "$IDENTITY" --timestamp --options runtime --entitlements "$2" "$1"
  fi
}

sign_frameworks_in_tree() {
  local root="$1"
  [[ -d "$root" ]] || return 0
  while IFS= read -r framework; do
    codesign_runtime "$framework"
  done < <(find "$root" -type d -name '*.framework' | awk '{ print length, $0 }' | sort -rn | cut -d" " -f2-)
}

sign_frameworks_in_tree "$APP_BUNDLE"
sign_frameworks_in_tree "$CLI_FRAMEWORKS_DIR"

if [[ -d "$APP_BUNDLE/Contents/Helpers" ]]; then
  while IFS= read -r helper; do
    codesign_runtime "$helper"
  done < <(find "$APP_BUNDLE/Contents/Helpers" -type f -perm -111 | sort)
fi

if [[ -d "$APP_BUNDLE/Contents/PlugIns" ]]; then
  while IFS= read -r appex; do
    sign_frameworks_in_tree "$appex"
    codesign_with_entitlements "$appex" "$WIDGET_ENTITLEMENTS"
  done < <(find "$APP_BUNDLE/Contents/PlugIns" -type d -name '*.appex' | awk '{ print length, $0 }' | sort -rn | cut -d" " -f2-)
fi

codesign_with_entitlements "$APP_BUNDLE" "$APP_ENTITLEMENTS"
codesign_runtime "$CLI_BINARY"
