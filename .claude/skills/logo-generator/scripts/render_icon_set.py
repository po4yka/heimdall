#!/usr/bin/env python3
"""
Render a complete heimdall platform icon set from a master SVG.

Produces:
  macos/AppIcon.appiconset/   10 PNGs + Contents.json
  macos/heimdall.icns         compiled via `iconutil` (macOS only; skipped elsewhere)
  macos/menu-bar/             icon_template.png (16x16) + icon_template@2x.png (32x32)
  linux/hicolor/<size>/apps/heimdall.png  (16,32,48,64,128,256,512)
  windows/heimdall.ico        multi-size (16,32,48,64,128,256)
  web/favicon.ico             16+32
  web/favicon.svg             copy of master

The menu-bar template icon strips the #D71921 accent pip so the result is a pure
ink-on-transparent silhouette, matching NSImage's template-image convention.

Rasterization: resvg CLI (Rust, tiny-skia-backed). Install: `cargo install resvg`.
Packaging: Pillow for .ico writing; macOS `iconutil` for .icns compilation.

Optional: pass --use-tauri to delegate the .icns and .ico packaging to the
`tauri icon` subcommand (requires `cargo install tauri-cli`). The rest of the
pipeline (AppIcon.appiconset layout, menu-bar template, Linux hicolor tree,
favicon) still runs on resvg+Pillow since tauri icon's output layout does
not cover those surfaces.
"""

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Iterable

try:
    from PIL import Image
except ImportError:
    print("Error: Pillow not installed. Install with: pip install Pillow")
    sys.exit(1)


def _find_resvg() -> str:
    """Locate the resvg binary on PATH, or exit with a clear install hint."""
    path = shutil.which("resvg")
    if not path:
        print("Error: resvg binary not found on PATH.", file=sys.stderr)
        print("Install with: cargo install resvg", file=sys.stderr)
        sys.exit(1)
    return path


def _find_tauri_cli() -> list[str] | None:
    """Locate tauri-cli. Returns the invocation prefix (e.g. ['cargo-tauri']) or None.

    `cargo install tauri-cli` installs `cargo-tauri` under ~/.cargo/bin; cargo
    auto-detects it as the `cargo tauri` subcommand. Some users also have the
    standalone `tauri` binary from npm. Try each in order of preference.
    """
    if shutil.which("cargo-tauri"):
        return ["cargo-tauri"]
    if shutil.which("tauri"):
        return ["tauri"]
    return None


def run_tauri_icon(source_svg: Path, out_dir: Path) -> bool:
    """Invoke `tauri icon` to produce icon.icns + icon.ico in out_dir.

    Returns True on success. Tauri also writes a lot of Android/iOS/Windows-Store
    variants we do not need; those are ignored by the caller.
    """
    cli = _find_tauri_cli()
    if cli is None:
        print("Error: tauri-cli not found on PATH.", file=sys.stderr)
        print("Install with: cargo install tauri-cli", file=sys.stderr)
        return False
    out_dir.mkdir(parents=True, exist_ok=True)
    try:
        subprocess.run(
            [*cli, "icon", str(source_svg), "-o", str(out_dir)],
            check=True, capture_output=True, text=True,
        )
    except subprocess.CalledProcessError as e:
        print(f"tauri icon failed: {e.stderr or e}", file=sys.stderr)
        return False
    return True


RESVG = _find_resvg()


# Size tables — keep in sync with references/platform-icons.md.

APPICONSET: list[tuple[str, int, str, str]] = [
    # (filename, pixel size, nominal size, scale)
    ("icon_16x16.png",       16,   "16x16",   "1x"),
    ("icon_16x16@2x.png",    32,   "16x16",   "2x"),
    ("icon_32x32.png",       32,   "32x32",   "1x"),
    ("icon_32x32@2x.png",    64,   "32x32",   "2x"),
    ("icon_128x128.png",     128,  "128x128", "1x"),
    ("icon_128x128@2x.png",  256,  "128x128", "2x"),
    ("icon_256x256.png",     256,  "256x256", "1x"),
    ("icon_256x256@2x.png",  512,  "256x256", "2x"),
    ("icon_512x512.png",     512,  "512x512", "1x"),
    ("icon_512x512@2x.png",  1024, "512x512", "2x"),
]

MENU_BAR_SIZES: list[tuple[str, int]] = [
    ("icon_template.png",    16),
    ("icon_template@2x.png", 32),
]

LINUX_HICOLOR_SIZES: list[int] = [16, 32, 48, 64, 128, 256, 512]

WINDOWS_ICO_SIZES: list[int] = [16, 32, 48, 64, 128, 256]

FAVICON_ICO_SIZES: list[int] = [16, 32]


# ---------------------------------------------------------------------------
# SVG helpers
# ---------------------------------------------------------------------------

ACCENT_HEX_RE = re.compile(r"#D71921", re.IGNORECASE)

def read_svg(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def strip_accent(svg_text: str) -> str:
    """Replace the one permitted #D71921 accent pip with currentColor.

    Menu-bar template icons must be pure ink-on-transparent; macOS inverts them
    for selected state. Any remaining color would break that convention.
    """
    return ACCENT_HEX_RE.sub("currentColor", svg_text)


def render_png(svg_text: str, out_path: Path, pixel_size: int) -> None:
    """Rasterize an SVG string to a PNG at an exact pixel size via resvg.

    Rendering direct at the target size (not downscaled from a large master)
    gives resvg's analytical anti-aliasing the cleanest edges — especially at
    16x16 where Cairo historically blurs thin strokes.

    resvg consumes a path on disk; we write the in-memory SVG (including the
    template-stripped variant) to a tempfile, rasterize, then delete.
    """
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".svg", delete=False, encoding="utf-8"
    ) as tmp:
        tmp.write(svg_text)
        tmp_path = tmp.name
    try:
        subprocess.run(
            [RESVG, "-w", str(pixel_size), "-h", str(pixel_size), tmp_path, str(out_path)],
            check=True, capture_output=True, text=True,
        )
    except subprocess.CalledProcessError as e:
        raise RuntimeError(f"resvg failed on {tmp_path}: {e.stderr or e}") from e
    finally:
        Path(tmp_path).unlink(missing_ok=True)


# ---------------------------------------------------------------------------
# Per-platform emitters
# ---------------------------------------------------------------------------

def write_contents_json(appiconset_dir: Path) -> None:
    images = [
        {
            "size": nominal,
            "idiom": "mac",
            "filename": fname,
            "scale": scale,
        }
        for (fname, _px, nominal, scale) in APPICONSET
    ]
    contents = {
        "images": images,
        "info": {"version": 1, "author": "heimdall logo-generator skill"},
    }
    (appiconset_dir / "Contents.json").write_text(
        json.dumps(contents, indent=2) + "\n", encoding="utf-8"
    )


def emit_macos(
    out_root: Path,
    master_svg: str,
    template_svg: str,
    tauri_out: Path | None = None,
) -> bool:
    # iconutil consumes .iconset (strict format). Xcode consumes .appiconset
    # (richer Asset Catalog format with Contents.json). The PNG set is identical;
    # only the directory extension and the Contents.json presence differ.
    # Strategy: render PNGs once into .iconset, compile .icns, then rename the
    # directory to .appiconset and add Contents.json for Xcode.
    #
    # tauri_out: if provided (i.e. --use-tauri was set and the tauri pre-render
    # succeeded), copy tauri's `icon.icns` instead of compiling via iconutil.
    # The .appiconset/ and menu-bar template PNGs are still generated via resvg
    # because tauri icon does not produce them in the layout Xcode expects.
    mac_dir = out_root / "macos"
    mac_dir.mkdir(parents=True, exist_ok=True)
    iconset = mac_dir / "AppIcon.iconset"
    appiconset = mac_dir / "AppIcon.appiconset"

    # Clean any prior appiconset so the rename lands cleanly.
    if appiconset.exists():
        shutil.rmtree(appiconset)
    if iconset.exists():
        shutil.rmtree(iconset)
    iconset.mkdir(parents=True)

    for fname, px, _nominal, _scale in APPICONSET:
        render_png(master_svg, iconset / fname, px)
        print(f"  [mac] {fname} {px}x{px}")

    menu_bar = mac_dir / "menu-bar"
    menu_bar.mkdir(parents=True, exist_ok=True)
    for fname, px in MENU_BAR_SIZES:
        render_png(template_svg, menu_bar / fname, px)
        print(f"  [mac] menu-bar/{fname} {px}x{px}")

    icns_path = mac_dir / "heimdall.icns"
    compile_ok = False

    if tauri_out is not None:
        tauri_icns = tauri_out / "icon.icns"
        if tauri_icns.exists():
            shutil.copyfile(tauri_icns, icns_path)
            print(f"  [mac] heimdall.icns copied from tauri output")
            compile_ok = True
        else:
            print(f"  [mac] WARNING: tauri output missing icon.icns at {tauri_icns}")
    else:
        iconutil = shutil.which("iconutil")
        if iconutil:
            try:
                subprocess.run(
                    [iconutil, "-c", "icns", str(iconset), "-o", str(icns_path)],
                    check=True,
                )
                print(f"  [mac] heimdall.icns compiled via iconutil")
                compile_ok = True
            except subprocess.CalledProcessError as e:
                print(f"  [mac] WARNING: iconutil failed: {e}")
        else:
            print("  [mac] iconutil not found (non-macOS host); .icns compile skipped.")

    # Promote .iconset -> .appiconset for Xcode consumption; add Contents.json.
    iconset.rename(appiconset)
    write_contents_json(appiconset)
    print(f"  [mac] Contents.json")

    return compile_ok


def emit_linux(out_root: Path, master_svg: str) -> None:
    for px in LINUX_HICOLOR_SIZES:
        target = out_root / "linux" / "hicolor" / f"{px}x{px}" / "apps" / "heimdall.png"
        render_png(master_svg, target, px)
        print(f"  [lin] hicolor/{px}x{px}/apps/heimdall.png")


def emit_windows(
    out_root: Path,
    master_svg: str,
    tauri_out: Path | None = None,
) -> None:
    """Write a multi-size .ico.

    With `--use-tauri`, copy tauri's `icon.ico` directly. Without it, rasterize
    each required size via resvg and assemble a multi-layer .ico with Pillow.
    """
    win_dir = out_root / "windows"
    win_dir.mkdir(parents=True, exist_ok=True)
    ico_path = win_dir / "heimdall.ico"

    if tauri_out is not None:
        tauri_ico = tauri_out / "icon.ico"
        if tauri_ico.exists():
            shutil.copyfile(tauri_ico, ico_path)
            print(f"  [win] heimdall.ico copied from tauri output")
            return
        print(f"  [win] WARNING: tauri output missing icon.ico at {tauri_ico}; falling back to Pillow")

    layers: list[Image.Image] = []
    tmp_pngs: list[Path] = []
    tmp_dir = win_dir / ".tmp_ico"
    tmp_dir.mkdir(parents=True, exist_ok=True)
    try:
        for px in WINDOWS_ICO_SIZES:
            p = tmp_dir / f"w{px}.png"
            render_png(master_svg, p, px)
            tmp_pngs.append(p)
            layers.append(Image.open(p).convert("RGBA"))

        # Pillow: pass `sizes` on the largest-canvas image; it embeds all layers.
        largest = layers[-1]
        largest.save(
            ico_path,
            format="ICO",
            sizes=[(px, px) for px in WINDOWS_ICO_SIZES],
        )
        print(f"  [win] heimdall.ico ({len(WINDOWS_ICO_SIZES)} sizes)")
    finally:
        for p in tmp_pngs:
            p.unlink(missing_ok=True)
        try:
            tmp_dir.rmdir()
        except OSError:
            pass


def emit_web(out_root: Path, master_svg: str, master_path: Path) -> None:
    web_dir = out_root / "web"
    web_dir.mkdir(parents=True, exist_ok=True)

    # favicon.svg — direct copy of master
    (web_dir / "favicon.svg").write_text(master_svg, encoding="utf-8")
    print("  [web] favicon.svg")

    # favicon.ico — 16 + 32
    tmp_dir = web_dir / ".tmp_favicon"
    tmp_dir.mkdir(parents=True, exist_ok=True)
    try:
        layers: list[Image.Image] = []
        for px in FAVICON_ICO_SIZES:
            p = tmp_dir / f"f{px}.png"
            render_png(master_svg, p, px)
            layers.append(Image.open(p).convert("RGBA"))
        ico_path = web_dir / "favicon.ico"
        layers[-1].save(
            ico_path,
            format="ICO",
            sizes=[(px, px) for px in FAVICON_ICO_SIZES],
        )
        print(f"  [web] favicon.ico ({len(FAVICON_ICO_SIZES)} sizes)")
    finally:
        for f in tmp_dir.iterdir():
            f.unlink(missing_ok=True)
        tmp_dir.rmdir()


# ---------------------------------------------------------------------------
# CLI entry
# ---------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(
        description="Render the complete heimdall platform icon set from a master SVG.",
    )
    parser.add_argument(
        "master_svg",
        help="Path to the master SVG (e.g. assets/icons/master.svg)",
    )
    parser.add_argument(
        "--out-root",
        default=None,
        help="Output root directory (default: alongside master.svg, one level up from its parent)",
    )
    parser.add_argument(
        "--only",
        choices=["macos", "linux", "windows", "web", "all"],
        default="all",
        help="Emit only one platform (default: all)",
    )
    parser.add_argument(
        "--use-tauri",
        action="store_true",
        help=(
            "Delegate .icns and .ico packaging to `tauri icon` (requires "
            "`cargo install tauri-cli`). Other outputs (AppIcon.appiconset, "
            "menu-bar templates, Linux hicolor tree, favicon) still use resvg."
        ),
    )
    args = parser.parse_args()

    master_path = Path(args.master_svg).resolve()
    if not master_path.exists():
        print(f"Error: master SVG not found: {master_path}")
        return 1

    master_svg = read_svg(master_path)
    template_svg = strip_accent(master_svg)

    # Default output root: parent of master.svg (so master.svg at assets/icons/master.svg
    # puts everything else under assets/icons/).
    if args.out_root:
        out_root = Path(args.out_root).resolve()
    else:
        out_root = master_path.parent
    out_root.mkdir(parents=True, exist_ok=True)

    print(f"Rendering icon set")
    print(f"  master : {master_path}")
    print(f"  output : {out_root}")
    if args.use_tauri:
        print("  mode   : --use-tauri (.icns + .ico via tauri icon)")
    print()

    # Pre-render via tauri if requested. Produces icon.icns + icon.ico in a
    # tempdir; we copy only those two files (tauri also writes Android/iOS/
    # Windows-Store variants we do not ship).
    tauri_out: Path | None = None
    if args.use_tauri and args.only in ("macos", "windows", "all"):
        tauri_tmp = Path(tempfile.mkdtemp(prefix="heimdall-tauri-icon-"))
        print(f"Running `tauri icon` (output -> {tauri_tmp}) ...")
        if run_tauri_icon(master_path, tauri_tmp):
            tauri_out = tauri_tmp
        else:
            print("  [tauri] pre-render failed; falling back to default pipeline")
        print()

    try:
        if args.only in ("macos", "all"):
            print("macOS:")
            emit_macos(out_root, master_svg, template_svg, tauri_out=tauri_out)
            print()
        if args.only in ("linux", "all"):
            print("Linux:")
            emit_linux(out_root, master_svg)
            print()
        if args.only in ("windows", "all"):
            print("Windows:")
            emit_windows(out_root, master_svg, tauri_out=tauri_out)
            print()
        if args.only in ("web", "all"):
            print("Web:")
            emit_web(out_root, master_svg, master_path)
            print()
    finally:
        if tauri_out is not None and tauri_out.exists():
            shutil.rmtree(tauri_out, ignore_errors=True)

    print("[OK] Icon set rendering complete.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
