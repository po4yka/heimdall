#!/usr/bin/env python3
"""Generate the Heimdall DMG installer background image.

Produces a 600x400 PNG matching DESIGN.md tokens:
- canvas: #0A0A0A (refined-dark)
- accent guide: #4A7FA5 (blue-gray, the project's interactive accent)
- caption: #999999 (text-secondary)

The script is the source of truth; assets/dmg/background.png is regenerated
from it whenever the design changes. CI does not run this script — the PNG
is committed alongside.

Requires Pillow (PIL). On a developer Mac:
    pip3 install --user Pillow

Usage:
    python3 script/generate_dmg_background.py
    python3 script/generate_dmg_background.py --output /tmp/bg.png
"""

import argparse
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

CANVAS_W, CANVAS_H = 600, 400
CANVAS_BG = (10, 10, 10, 255)            # #0A0A0A
ACCENT_BG = (74, 127, 165, 255)          # #4A7FA5 (--accent-interactive)
TEXT_SECONDARY = (153, 153, 153, 255)    # #999999

# Icon center positions. Must match the AppleScript layout in
# script/package_heimdall_dmg.sh — Heimdall.app at (160, 200), Applications
# symlink at (440, 200), each rendered at icon size 128.
ICON_LEFT_X = 160
ICON_RIGHT_X = 440
ICON_Y = 200
ICON_SIZE = 128

# Arrow guide: stretches between the inner edges of the two icon slots, with
# 16px breathing room either side.
ARROW_X_START = ICON_LEFT_X + ICON_SIZE // 2 + 16   # 240
ARROW_X_END = ICON_RIGHT_X - ICON_SIZE // 2 - 16    # 360
ARROW_Y = ICON_Y
ARROW_STROKE = 2
ARROWHEAD_LENGTH = 12
ARROWHEAD_WIDTH = 10

WORDMARK = "Heimdall"
WORDMARK_Y = 56

CAPTION = "Drag Heimdall to Applications"
CAPTION_Y = 312

# Helvetica Neue ships on every macOS install and renders cleanly through
# Pillow (unlike the SFNS.ttf variable font, which exposes kerning glitches
# in Pillow's truetype layer). Arial is the path used on Linux dev machines.
SYSTEM_FONTS = (
    "/System/Library/Fonts/HelveticaNeue.ttc",
    "/Library/Fonts/HelveticaNeue.ttc",
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
)


def load_font(size: int) -> ImageFont.ImageFont:
    for path in SYSTEM_FONTS:
        if Path(path).exists():
            try:
                return ImageFont.truetype(path, size)
            except OSError:
                continue
    return ImageFont.load_default()


def draw_arrow(draw: ImageDraw.ImageDraw) -> None:
    shaft_end = ARROW_X_END - ARROWHEAD_LENGTH
    draw.line(
        [(ARROW_X_START, ARROW_Y), (shaft_end, ARROW_Y)],
        fill=ACCENT_BG,
        width=ARROW_STROKE,
    )
    draw.polygon(
        [
            (ARROW_X_END, ARROW_Y),
            (shaft_end, ARROW_Y - ARROWHEAD_WIDTH // 2),
            (shaft_end, ARROW_Y + ARROWHEAD_WIDTH // 2),
        ],
        fill=ACCENT_BG,
    )


def draw_centered_text(
    draw: ImageDraw.ImageDraw,
    text: str,
    y: int,
    size: int,
    fill: tuple[int, int, int, int],
) -> None:
    font = load_font(size)
    bbox = draw.textbbox((0, 0), text, font=font)
    text_w = bbox[2] - bbox[0]
    draw.text(((CANVAS_W - text_w) / 2, y), text, font=font, fill=fill)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    default_output = (
        Path(__file__).resolve().parent.parent / "assets" / "dmg" / "background.png"
    )
    parser.add_argument("--output", default=str(default_output))
    args = parser.parse_args()

    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)

    img = Image.new("RGBA", (CANVAS_W, CANVAS_H), CANVAS_BG)
    draw = ImageDraw.Draw(img)

    draw_centered_text(draw, WORDMARK, WORDMARK_Y, 22, TEXT_SECONDARY)
    draw_arrow(draw)
    draw_centered_text(draw, CAPTION, CAPTION_Y, 13, TEXT_SECONDARY)

    img.save(output, "PNG")
    print(f"wrote {output} ({CANVAS_W}x{CANVAS_H})")


if __name__ == "__main__":
    main()
