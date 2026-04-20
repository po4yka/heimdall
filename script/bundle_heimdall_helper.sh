#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 <app-bundle> [Debug|Release]" >&2
  exit 2
fi

APP_BUNDLE="$1"
CONFIGURATION="${2:-Debug}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HELPERS_DIR="$APP_BUNDLE/Contents/Helpers"

case "$CONFIGURATION" in
  Release)
    CARGO_ARGS=(build --release --bin claude-usage-tracker)
    HELPER_SOURCE="$ROOT_DIR/target/release/claude-usage-tracker"
    ;;
  *)
    CARGO_ARGS=(build --bin claude-usage-tracker)
    HELPER_SOURCE="$ROOT_DIR/target/debug/claude-usage-tracker"
    ;;
esac

cargo "${CARGO_ARGS[@]}" --manifest-path "$ROOT_DIR/Cargo.toml" >/dev/null

mkdir -p "$HELPERS_DIR"
install -m 755 "$HELPER_SOURCE" "$HELPERS_DIR/claude-usage-tracker"
