#!/usr/bin/env bash
# Visual regression gate for heimdall logo assets.
#
# Run from the repo root. Intended for CI; safe to run locally.
#
# Flow:
#   1. Python grammar validator (scripts/validate_svg.py --strict)
#   2. svglint (optional — skipped if the CI node_modules is not installed)
#   3. Re-rasterize every variant SVG at 128px + 16px into a tempdir
#   4. odiff each regenerated PNG vs. the committed PNG
#   5. Fail if any diff exceeds $MAX_DIFF_PIXELS (default 2 pixels for 16x16,
#      40 pixels for 128x128). Anti-alias noise below this threshold is normal.
#
# Usage:
#   bash .claude/skills/logo-generator/ci/visual_regression.sh
#   MAX_DIFF_PIXELS_SMALL=4 bash ...ci/visual_regression.sh    # custom threshold

set -euo pipefail

SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$(cd "${SKILL_DIR}/../../.." && pwd)"
VARIANTS_DIR="${REPO_ROOT}/assets/icons/variants"
CI_DIR="${SKILL_DIR}/ci"

MAX_DIFF_PIXELS_SMALL="${MAX_DIFF_PIXELS_SMALL:-4}"    # 16x16 threshold (of 256 pixels)
MAX_DIFF_PIXELS_LARGE="${MAX_DIFF_PIXELS_LARGE:-80}"   # 128x128 threshold (of 16384 pixels)

echo "=== heimdall logo CI ==="
echo "  repo root   : ${REPO_ROOT}"
echo "  variants    : ${VARIANTS_DIR}"
echo "  thresholds  : 16px<=${MAX_DIFF_PIXELS_SMALL}, 128px<=${MAX_DIFF_PIXELS_LARGE}"
echo ""

# -----------------------------------------------------------------------------
# Gate 1: grammar contract (Python validator — always runs)
# -----------------------------------------------------------------------------

echo "--- [1/3] grammar contract ---"
if ! python3 "${SKILL_DIR}/scripts/validate_svg.py" --strict "${VARIANTS_DIR}"/v*.svg; then
    echo ""
    echo "FAIL: one or more variants violate the grammar contract."
    exit 1
fi
echo ""

# -----------------------------------------------------------------------------
# Gate 2: svglint (optional — skipped if ci/node_modules is missing)
# -----------------------------------------------------------------------------

echo "--- [2/3] svglint ---"
if [ -d "${CI_DIR}/node_modules" ] && [ -x "${CI_DIR}/node_modules/.bin/svglint" ]; then
    (
        cd "${CI_DIR}"
        # --ci exits nonzero on any violation; --stdin false forces file mode
        ./node_modules/.bin/svglint --ci "${VARIANTS_DIR}"/v*.svg
    )
    echo ""
else
    echo "  skipped (run \`cd ${CI_DIR} && npm install\` to enable)"
    echo ""
fi

# -----------------------------------------------------------------------------
# Gate 3: visual regression (odiff against committed previews)
# -----------------------------------------------------------------------------

echo "--- [3/3] visual regression ---"

if [ ! -d "${CI_DIR}/node_modules" ] || [ ! -x "${CI_DIR}/node_modules/.bin/odiff" ]; then
    echo "  skipped (run \`cd ${CI_DIR} && npm install\` to enable)"
    echo ""
    echo "[OK] logo CI gates passed (visual regression skipped)"
    exit 0
fi

ODIFF="${CI_DIR}/node_modules/.bin/odiff"
SVG_TO_PNG="${SKILL_DIR}/scripts/svg_to_png.py"

tmp=$(mktemp -d)
trap 'rm -rf "${tmp}"' EXIT

fails=0
regenerated=0

for svg in "${VARIANTS_DIR}"/v*.svg; do
    base=$(basename "${svg}" .svg)
    for size in 16 128; do
        committed="${VARIANTS_DIR}/${base}-${size}.png"
        if [ ! -f "${committed}" ]; then
            echo "  skip ${base}-${size} (no committed preview)"
            continue
        fi
        fresh="${tmp}/${base}-${size}.png"
        diff_png="${tmp}/${base}-${size}-diff.png"

        python3 "${SVG_TO_PNG}" "${svg}" --output "${fresh}" \
            --width "${size}" --height "${size}" > /dev/null
        regenerated=$((regenerated + 1))

        # odiff exit codes:
        #   0 = identical, 21 = different, 22 = input size mismatch, 23 = layout fail
        set +e
        out=$("${ODIFF}" "${committed}" "${fresh}" "${diff_png}" --threshold=0.1 2>&1)
        rc=$?
        set -e

        if [ ${rc} -eq 0 ]; then
            continue
        fi

        # odiff reports "Different pixels: N" on a diff; parse and threshold.
        diff_pixels=$(printf '%s' "${out}" | grep -oE '[0-9]+' | head -1 || echo 0)
        threshold=${MAX_DIFF_PIXELS_SMALL}
        if [ "${size}" = "128" ]; then
            threshold=${MAX_DIFF_PIXELS_LARGE}
        fi

        if [ "${diff_pixels}" -le "${threshold}" ]; then
            echo "  OK    ${base}-${size}  (${diff_pixels}px diff, threshold ${threshold})"
        else
            echo "  FAIL  ${base}-${size}  (${diff_pixels}px diff > threshold ${threshold})"
            echo "        diff written to ${diff_png}"
            fails=$((fails + 1))
        fi
    done
done

echo ""
echo "  regenerated ${regenerated} PNG(s); ${fails} regression(s)"
if [ "${fails}" -gt 0 ]; then
    echo ""
    echo "FAIL: visual regression. Commit the new PNGs if the SVG changes are intentional."
    exit 1
fi

echo ""
echo "[OK] logo CI gates passed"
