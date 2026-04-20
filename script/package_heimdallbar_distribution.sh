#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <products-dir> <staging-dir>" >&2
  exit 2
fi

PRODUCTS_DIR="$1"
STAGING_DIR="$2"
APP_BUNDLE="$PRODUCTS_DIR/HeimdallBar.app"
CLI_BINARY="$PRODUCTS_DIR/heimdallbar"
SHARED_FRAMEWORK="$PRODUCTS_DIR/HeimdallBarShared.framework"

[[ -d "$APP_BUNDLE" ]] || { echo "missing app bundle: $APP_BUNDLE" >&2; exit 1; }
[[ -x "$CLI_BINARY" ]] || { echo "missing CLI binary: $CLI_BINARY" >&2; exit 1; }
[[ -d "$SHARED_FRAMEWORK" ]] || { echo "missing shared framework: $SHARED_FRAMEWORK" >&2; exit 1; }

rm -rf "$STAGING_DIR"
mkdir -p "$STAGING_DIR/bin" "$STAGING_DIR/Frameworks"

cp -R "$APP_BUNDLE" "$STAGING_DIR/"
install -m 755 "$CLI_BINARY" "$STAGING_DIR/bin/heimdallbar"
cp -R "$SHARED_FRAMEWORK" "$STAGING_DIR/Frameworks/"
