---
name: logo-generator
description: Generate SVG logos and a complete cross-platform application-icon set for heimdall (macOS .icns + menu-bar template, Linux freedesktop hicolor PNGs, Windows .ico, web favicon). Use when the user wants to create a logo or mark for heimdall, HeimdallBar, the Preact dashboard, or any related surface; generate multiple concept variants; or export the full icon set from a master SVG. Enforces heimdall's industrial-design system (DESIGN.md + .claude/skills/industrial-design/): strict monochrome, one optional red accent (#D71921), Space Grotesk/Space Mono/Doto typography, hairline rules, no gradients, no shadows, no glow. Optional Gemini 3.1 Flash Image Preview (Nano Banana) showcase renders require a local GEMINI_API_KEY — phase 5 is skipped if none is configured.
when_to_use: Invoke when the user asks for a heimdall logo, mark, app icon, favicon, HeimdallBar menu-bar icon, or platform icon set. Also invoke when iterating on existing variants in `assets/icons/variants/`.
argument-hint: "[concept-direction] [--template] [--variants N]"
effort: xhigh
allowed-tools: [Read, Write, Edit, Glob, Grep, Bash]
---

# Logo Generator — heimdall adaptation

Adapted from [op7418/logo-generator-skill](https://github.com/op7418/logo-generator-skill) by way of `blog/.claude/skills/logo-generator/`. Workflow is preserved; design constraints are rewritten to heimdall's industrial monochrome system, and a **platform-icon export layer** is added so a single master SVG produces every asset heimdall ships (app bundle, menu-bar template, Linux hicolor tree, Windows .ico, favicon).

## Non-negotiable heimdall constraints

Before generating anything, read `DESIGN.md` and `.claude/skills/industrial-design/references/tokens.md`. The mark and any surrounding presentation MUST satisfy:

- **Monochrome only.** The mark renders in `currentColor`. In deliverables that bake in a fill, allow only `#000000`, `#FFFFFF`, `#E8E8E8` (`--text-primary` dark), or `#1A1A1A` (`--text-primary` light). No other fills.
- **One optional red pip.** `#D71921` (`--accent`) is the only non-neutral token and is reserved for **one** tiny signal element (a dot, a single stroked line) — the "one red accent per screen" rule from the industrial-design system. Omit it by default; include only when the user asks for the active-sentinel variant.
- **No gradients.** `<linearGradient>` and `<radialGradient>` are forbidden in the mark.
- **No shadows / no glow.** No `filter=`, no `<feGaussianBlur>`, no `<feDropShadow>`, no `text-shadow`, no phosphor, no halo. Flat geometric forms only.
- **Hairline rules.** Stroke widths sit between 2 and 4 units inside `viewBox="0 0 100 100"`. Thinner reads as fragile at 16px.
- **Typography reference, not embedded.** If the mark includes a letterform, reference Space Grotesk (heading), Space Mono (labels/numbers), or Doto (hero display) — but convert to outlined `<path>` data at export. Do not ship `<text>` that depends on a font being installed.
- **No pill radii.** Container radius in presentation HTML is exactly `2px` (matches heimdall card radius allowances up to 16px but defaults to 2px for technical surfaces).

Heimdall has no existing logo. DESIGN.md defines the brand surface but not the mark itself — the relevant brand signals are: industrial warmth (Braun, Teenage Engineering), the dot-matrix motif (`references/tokens.md` §6), and the sentinel/watcher idea implied by the Norse-mythology name.

## SVG grammar contract

Every SVG you produce **MUST** obey `references/svg-contract.md`. The contract is mechanically enforced by `scripts/validate_svg.py`; a variant that fails validation does not advance past Phase 2. Load the full contract before your first generation.

The critical rules (abbreviated — full rules in the contract):

- `viewBox="0 0 100 100"` exactly. No other.
- Element vocabulary: `svg`, `g`, `title`, `desc`, `defs`, `clipPath`, `path`, `rect`, `circle`, `ellipse`, `line`, `polygon`, `polyline`. Nothing else. No `<text>`, `<use>`, `<image>`, `<linearGradient>`, `<filter>`.
- Fills/strokes from a fixed palette only: `none`, `currentColor`, `#000`, `#fff`, `#E8E8E8`, `#1A1A1A`, `#D71921`. The `#D71921` pip appears at most **once** in the mark.
- Every `<path>` with a non-`none` fill **MUST** end `d=` with `Z`.
- All coordinates in `[-10, 110]`. Out-of-range = tokenization drift = reject.
- Arc commands: `large-arc-flag` and `sweep-flag` must be literal `0` or `1`. Self-critique every arc before submitting (Section "Arc-flag discipline" in Phase 2).
- Transforms on `<g>` groups only, never on individual primitives.

Before generating the first variant, read these three files in order:

1. `references/svg-contract.md` — the hard rules the validator enforces
2. `references/exemplar-library.md` — 5 annotated canonical exemplars; use these as coordinate anchors rather than re-deriving geometry from scratch
3. `references/heimdall-logo-brief.md` — the creative brief (concepts, collision list, direction ranking)

## Workflow

### Phase 1: Information gathering

**Before asking anything, read `references/heimdall-logo-brief.md`.** It contains the creative brief (Norse mythology anchor, collision-avoidance list, three ranked concept directions, the 6-variant generation plan, the 16×16 gate rules) and settles most questions the upstream skill would ask. The brief was produced from web research and should be treated as load-bearing — update it, do not re-derive it.

With the brief loaded, confirm one thing with the user and proceed:

> "Generating 6 variants across Horn / Horizon / Arc / Monogram directions per `heimdall-logo-brief.md`, optional `#D71921` pip, 16×16 gate enforced. Proceed, or do you want to steer the direction?"

If the user wants to override the brief (different concept, different target surface, pure monochrome with no pip, etc.), take the override and log the deviation. Do not re-run a full interview.

### Phase 2a — Spatial plan (mandatory before any SVG)

Research (LLM4SVG / Chat2SVG, CVPR 2025) shows that LLMs fail SVG generation primarily at the coordinate-tokenization layer: numbers are treated as character sequences, so coordinates drift and arcs misfire. The fix is a structured planning pass before the model commits to XML. **Do not write any SVG until this plan is complete.**

For each variant you intend to generate, output as a plain-text block:

```
## Variant NN — <concept name>

Scene (one sentence): ...
Elements in render order (back to front):
  - <tag> anchor=(x,y) bounds=((x1,y1),(x2,y2)) — note
  - <tag> anchor=(x,y) bounds=((x1,y1),(x2,y2)) — note
Key anchor coordinates:
  - <name>: (x, y)
  - ...
Arc commands (if any):
  - arc from (x1,y1) to (x2,y2), radius r
      * subtends ≤180°? → large-arc-flag = 0
      * clockwise sweep as drawn? → sweep-flag = 1
Accent pip (if any): at (x, y), r=N
```

Only after the plan is written do you proceed to Phase 2b. The plan is cheap (~200 tokens) and catches coordinate drift before rasterization can expose it.

### Phase 2b — SVG generation

Generate **6 categorically different variants**, each obeying the grammar contract. Bias toward variants that read at 16×16.

**Coordinate discipline:**
- Integer coordinates only. No decimals unless you have a geometric reason (e.g. centroid of a triangle).
- Every element's smallest dimension ≥ 8 units (survives 16×16 raster).
- Negative space ≥ 40% of the canvas.
- One or two core forms max.

**Arc-flag discipline** (the #1 LLM failure mode):

Before writing any `A` command, output a short reasoning line in a comment immediately above the `<path>`:

```svg
<!-- Arc from (22,78) to (78,22), r=56.
     Subtends 90° (quarter circle) → large-arc-flag=0.
     Drawn clockwise as viewed → sweep-flag=1. -->
<path d="M 22 78 A 56 56 0 0 1 78 22" ... />
```

Match the style of `exemplar-library.md` E4. Without this reasoning block, the model reliably swaps the two flags.

**Group discipline:**
- Wrap the mark in a single `<g>` with a meaningful `<title>` child.
- Transforms (if any) only on `<g>`, never on individual primitives.

**Multi-turn refinement:** If iterating on a prior variant, **return a complete replacement SVG, never a diff**. Diff refinement causes cumulative coordinate drift.

### Phase 2c — Automated contract gate

After each variant is written, run:

```bash
python "${CLAUDE_SKILL_DIR}/scripts/validate_svg.py" assets/icons/variants/<file>.svg --strict
```

Any FAIL output blocks the variant from advancing to the user-facing gallery. Fix the grammar violation (or reject the variant outright) before continuing. The validator catches: wrong viewBox, forbidden elements, chromatic drift, unclosed filled paths, decimal arc flags, out-of-range coordinates, rogue transforms, and multi-pip accent use.

### Phase 2d — Present the gallery

Only variants that passed 2c reach the gallery. Build it from `assets/showcase_template.html`. For each variant include:

- the SVG rendered at 128×128
- the SVG rendered at 16×16 (actual size — this is the legibility gate)
- the pattern-family label (Horn, Horizon, Arc, Monogram, ...)
- a one-sentence design rationale

### Phase 3: Iteration

Let the user narrow from 6+ to a single master:

- adjust proportions, stroke weight, rotation
- combine elements across variants
- generate additional variants inside a direction

**Critical multi-turn discipline:** When the user asks for a modification to an existing variant, **return a complete replacement SVG, not a diff**. Diff-style "change line 7" refinement causes cumulative coordinate drift in LLM-generated SVG (documented in LLM4SVG 2025). Always regenerate the full file, re-run the Phase 2a spatial plan for the revised variant, re-validate via Phase 2c, and only then present.

Do not regenerate the full set of 6 unless asked — iterate within a direction. When the user picks a final, write it to `assets/icons/master.svg` and re-run the validator one more time before advancing to Phase 4.

### Phase 4: Platform icon set export

Run:

```bash
python .claude/skills/logo-generator/scripts/render_icon_set.py assets/icons/master.svg
```

This produces, under `assets/icons/`:

```
macos/
  AppIcon.appiconset/           # 10 PNGs (16@1x..1024@1x) + Contents.json
    icon_16x16.png, icon_16x16@2x.png
    icon_32x32.png, icon_32x32@2x.png
    icon_128x128.png, icon_128x128@2x.png
    icon_256x256.png, icon_256x256@2x.png
    icon_512x512.png, icon_512x512@2x.png
    Contents.json
  heimdall.icns                 # compiled via `iconutil -c icns`
  menu-bar/
    icon_template.png           # 16×16 monochrome (NSImage template)
    icon_template@2x.png        # 32×32
linux/
  hicolor/
    16x16/apps/heimdall.png
    32x32/apps/heimdall.png
    48x48/apps/heimdall.png
    64x64/apps/heimdall.png
    128x128/apps/heimdall.png
    256x256/apps/heimdall.png
    512x512/apps/heimdall.png
windows/
  heimdall.ico                  # multi-size: 16, 32, 48, 64, 128, 256
web/
  favicon.ico                   # 16 + 32
  favicon.svg                   # copy of master
```

Exact sizes, color-space rules, and the menu-bar template convention live in `references/platform-icons.md`.

The script has a `--template` mode that strips the accent pip and writes the result to `macos/menu-bar/icon_template*.png`. Menu-bar icons must be ink-on-transparent only — macOS will invert them for the selected state.

Fallbacks when a tool is unavailable:
- `iconutil` is macOS-only. On Linux/Windows, the script still emits the `.appiconset` directory and PNGs; compile `.icns` on a Mac.
- Pillow writes `.ico`; no external dependency.

### Phase 5: Showcase generation (optional — requires GEMINI_API_KEY)

Only proceed if the user wants rendered presentation images AND `.env` has `GEMINI_API_KEY`.

1. Export a reference PNG via `scripts/svg_to_png.py assets/icons/master.svg --size 1024`.
2. Pick 3–4 on-brand backgrounds from `references/background_styles.md` (`void` and `swiss_flat` are heimdall defaults; `clinical` pairs well with the dashboard surface).
3. Run `python scripts/generate_showcase.py heimdall path/to/reference.png --style <key>` or `--all-styles`.
4. Compose the final presentation page using `assets/showcase_template.html`.

### Phase 6: Delivery + wiring

Hand the user:

- `assets/icons/master.svg` (canonical source)
- the full platform icon tree under `assets/icons/`
- optional showcase PNGs
- a one-line wiring checklist (do not auto-wire):
  - HeimdallBar: symlink or copy `assets/icons/macos/AppIcon.appiconset` into `macos/HeimdallBar/App/Sources/Assets.xcassets/` and set `CFBundleIconFile=AppIcon` in `macos/HeimdallBar/App/Sources/Info.plist`.
  - HeimdallBar menu-bar button: reference `icon_template.png` via `NSImage(named:)` with `isTemplate = true`.
  - Dashboard favicon: drop `web/favicon.ico` and `web/favicon.svg` next to `src/ui/index.html` and add `<link rel="icon" href="/favicon.svg">` + `<link rel="alternate icon" href="/favicon.ico">`.
  - Linux packaging: install `linux/hicolor/` into `$prefix/share/icons/hicolor/`.
  - Windows packaging: bundle `windows/heimdall.ico`.

## Technical notes

### Environment setup

```bash
# Rasterizer (one-time install, ~2 min to compile)
cargo install resvg                       # Rust binary, zero runtime deps

# Python packaging + showcase deps
cd .claude/skills/logo-generator
pip install -r requirements.txt           # Pillow, python-dotenv, google-genai
cp .env.example .env                      # optional; only needed for phase 5
```

No native library prerequisites. `resvg` is a self-contained Rust binary (tiny-skia-backed). Pillow handles `.ico` writing. Python-dotenv + google-genai are only needed for the optional Gemini showcase phase.

### Output sizes

- Master SVG: `viewBox="0 0 100 100"`, scale-free.
- PNG master: 1024×1024 by default. Use `--size 2048` for high-DPI showcase work.
- Favicon: verify readability at 16×16 before finalizing — if the mark fragments, go back to phase 3.
- Showcase renders: 16:9 at 2K (hardcoded in `generate_showcase.py`).

## Common patterns (quick reference)

All examples use `currentColor` — the mark inherits `var(--color-text-primary)` from `src/ui/style.css`.

### Sentinel eye (watcher motif)
```svg
<svg viewBox="0 0 100 100">
  <circle cx="50" cy="50" r="32" fill="none" stroke="currentColor" stroke-width="3"/>
  <circle cx="50" cy="50" r="8" fill="currentColor"/>
  <!-- optional accent pip -->
  <circle cx="50" cy="50" r="2" fill="#D71921"/>
</svg>
```

### Dot-matrix H monogram
```svg
<svg viewBox="0 0 100 100">
  <g fill="currentColor">
    <!-- left column -->
    <circle cx="30" cy="30" r="3"/><circle cx="30" cy="42" r="3"/>
    <circle cx="30" cy="54" r="3"/><circle cx="30" cy="66" r="3"/>
    <circle cx="30" cy="78" r="3"/><circle cx="30" cy="22" r="3"/>
    <!-- crossbar -->
    <circle cx="42" cy="50" r="3"/><circle cx="54" cy="50" r="3"/>
    <circle cx="66" cy="50" r="3"/>
    <!-- right column -->
    <circle cx="78" cy="30" r="3"/><circle cx="78" cy="42" r="3"/>
    <circle cx="78" cy="54" r="3"/><circle cx="78" cy="66" r="3"/>
    <circle cx="78" cy="78" r="3"/><circle cx="78" cy="22" r="3"/>
  </g>
</svg>
```

### Radial signal sweep
```svg
<svg viewBox="0 0 100 100">
  <g fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
    <circle cx="50" cy="50" r="32"/>
    <path d="M 50 50 L 78 30"/>
    <circle cx="50" cy="50" r="3" fill="currentColor"/>
  </g>
</svg>
```

For more pattern families see `references/design_patterns.md`.

## Troubleshooting

- **SVG not displaying correctly:** check `viewBox` and that every `<path>` is closed.
- **`resvg binary not found on PATH`:** install with `cargo install resvg` (takes ~2 min to compile). If you do not have cargo, install via `rustup` first: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`.
- **PNG export looks faint at 16×16:** check stroke-widths in the source SVG. resvg's analytical AA is accurate — if the mark fades, the stroke is genuinely sub-pixel. Raise stroke-width to ≥ 8 units or switch to filled shapes (see `references/svg-contract.md` §6).
- **`iconutil` not found on Linux/Windows:** the script skips `.icns` compilation and emits a warning. Run the `.icns` step later on a Mac or in CI.
- **Menu-bar icon looks wrong in the bar:** it must be template-only — no accent, no fill except `currentColor`/`#000`. Re-export with `--template`.
- **Dashboard favicon looks fuzzy at 16px:** the mark is too complex. Return to phase 3 and simplify.
- **Showcase drift to colour:** re-run — Nano Banana is non-deterministic. If it drifts consistently, use `swiss_flat` (tightest prompt).

## Pipeline

**Default (2026):** `resvg` (Rust) for rasterization → Pillow for `.ico` writing → macOS `iconutil` for `.icns` compilation. Zero native-library dependencies for the rasterizer; Pillow is pip-installable; `iconutil` is macOS-built-in (skipped on other platforms, `.icns` compiles later on a Mac).

**Opt-in alternative — `--use-tauri`:** pass `--use-tauri` to `render_icon_set.py` to delegate the `.icns` and `.ico` packaging to the `tauri icon` subcommand (requires `cargo install tauri-cli`). The rest of the pipeline (`AppIcon.appiconset`, menu-bar template, Linux hicolor tree, favicon) still runs on resvg+Pillow — those surfaces are not covered by `tauri icon`'s output layout. Reasons to opt in:

- You want `.icns` compilation to work cross-platform (not limited to macOS `iconutil`).
- You prefer the `icns` / `ico` Rust crates (same ones tauri uses internally) over Pillow for the ICO container.

Reasons to stay on the default:

- No extra Rust prereq (`cargo install tauri-cli` is ~3 min compile).
- `iconutil`-compiled `.icns` uses the `ic12` type (Apple's higher-resolution 16×16@2x entry); tauri's `icns` crate uses `ic13`. Both are valid; `ic12` is the slightly more traditional macOS layout.

## CI gates

Three layered gates run locally and in GitHub Actions (`.github/workflows/logo-assets.yml`). The workflow triggers on any change under `assets/icons/**` or `.claude/skills/logo-generator/**`.

1. **Grammar contract** — `scripts/validate_svg.py --strict` (Python stdlib only). Authoritative. Catches forbidden elements, chromatic drift, unclosed filled paths, decimal arc flags, coordinate drift, transform discipline, accent-pip singleton.
2. **svglint** — `ci/.svglintrc.js` mirrors the subset of the grammar contract that maps to svglint's declarative rule syntax. Runs from `ci/` via `npm ci && npx svglint --ci`. Overlaps deliberately with gate 1 — a JS-first reviewer gets the same enforcement via a tool they know.
3. **Visual regression** — `ci/visual_regression.sh` runs the Python validator, svglint, then rasterizes every `v*.svg` at 128px and 16px via resvg and uses `odiff` to compare against the committed PNGs. Any pixel delta above the per-size threshold (16px ≤ 4 diff, 128px ≤ 80 diff) is a fail. Thresholds are overridable via `MAX_DIFF_PIXELS_SMALL` / `MAX_DIFF_PIXELS_LARGE` env vars.

Running locally:

```bash
# One-time setup
cd .claude/skills/logo-generator/ci && npm install

# Run all three gates
bash .claude/skills/logo-generator/ci/visual_regression.sh
```

If an SVG change is intentional, regenerate the PNG previews and commit them alongside the SVG — the gate then passes.

## Deferred

Nothing remaining — the upgrade path from the original research (`resvg`, `tauri icon`, `svglint`, `odiff`) is fully integrated.

## Opus 4.7 optimization notes

This skill is tuned for Claude Opus 4.7 (April 2026). Key settings:

- **`effort: xhigh`** in the frontmatter activates adaptive-thinking at xhigh effort — appropriate for spatial geometry work (Phase 2a spatial plan, Phase 2b SVG generation). Drop to `high` for Phase 3 refinement if response speed matters.
- **1M-context advantage:** the three reference files (`svg-contract.md`, `exemplar-library.md`, `heimdall-logo-brief.md`) are ~18 KB combined — fits in context trivially. Keep them loaded throughout the session.
- **Thinking display:** Opus 4.7 defaults to `thinking.display: "omitted"`. To inspect the model's geometry reasoning during Phase 2a, set `display: "summarized"` in your API call.
- **Visual feedback loop:** for borderline variants, rasterize the SVG to PNG at 128×128 and pass it back as a multimodal follow-up — Opus 4.7's 2576px-long-edge vision accuracy (98.5% on visual-acuity benchmarks, up from 54.5% on 4.6) means the model can identify its own rendering errors when given the rasterized output.

## What this skill is NOT for

- Full brand identity systems (typography scale, tone of voice) — that lives in `DESIGN.md` and `.claude/skills/industrial-design/`.
- Dashboard illustrations or decorative widget icons beyond a single mark — use `/industrial-design` for those.
- Re-introducing gradients, shadows, or chromatic accents. DESIGN.md is the gate — change it first.
