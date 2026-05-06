#!/usr/bin/env bash
set -euo pipefail

# Notarize a signed `.dmg` and staple the ticket onto it.
#
# Usage:
#   script/dmg_notarize.sh <dmg-path>
#
# Required environment (script errors if any are missing — gating is the
# caller's responsibility, mirroring release.yml's existing notarize step):
#   HEIMDALL_NOTARY_APPLE_ID
#   HEIMDALL_NOTARY_TEAM_ID
#   HEIMDALL_NOTARY_APP_PASSWORD

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <dmg-path>" >&2
  exit 2
fi

DMG_PATH="$1"

[[ -f "$DMG_PATH" ]]                         || { echo "missing dmg: $DMG_PATH" >&2; exit 1; }
[[ -n "${HEIMDALL_NOTARY_APPLE_ID:-}" ]]     || { echo "missing HEIMDALL_NOTARY_APPLE_ID" >&2; exit 1; }
[[ -n "${HEIMDALL_NOTARY_TEAM_ID:-}" ]]      || { echo "missing HEIMDALL_NOTARY_TEAM_ID" >&2; exit 1; }
[[ -n "${HEIMDALL_NOTARY_APP_PASSWORD:-}" ]] || { echo "missing HEIMDALL_NOTARY_APP_PASSWORD" >&2; exit 1; }

xcrun notarytool submit "$DMG_PATH" \
  --apple-id "$HEIMDALL_NOTARY_APPLE_ID" \
  --team-id "$HEIMDALL_NOTARY_TEAM_ID" \
  --password "$HEIMDALL_NOTARY_APP_PASSWORD" \
  --wait

xcrun stapler staple "$DMG_PATH"
