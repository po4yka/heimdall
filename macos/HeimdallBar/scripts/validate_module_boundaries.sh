#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <module-dir> <module-name>" >&2
  exit 64
fi

module_dir="$1"
module_name="$2"
forbidden_pattern='^import (AppKit|Darwin|Security|WebKit|WidgetKit)$'

if matches="$(rg -n --glob '*.swift' "$forbidden_pattern" "$module_dir")" && [[ -n "$matches" ]]; then
  echo "Forbidden platform imports detected in ${module_name}:"
  echo "$matches"
  exit 1
fi
