# logo-generator (heimdall adaptation)

Adapted from [op7418/logo-generator-skill](https://github.com/op7418/logo-generator-skill) via `blog/.claude/skills/logo-generator/`. This copy is scoped to heimdall and adds a **platform-icon export layer** that blog does not need.

## What this skill does here

- Generate 6+ variants of a monochrome SVG mark for heimdall, HeimdallBar, or the Preact dashboard.
- Use `currentColor` throughout so the mark inherits `--color-text-primary`.
- Export a complete platform icon set from the chosen master SVG: macOS `.icns` + `AppIcon.appiconset`, macOS menu-bar template PNGs, Linux freedesktop hicolor tree, Windows `.ico`, and web favicon.
- Optionally render showcase presentations via Gemini's Nano Banana (`gemini-3.1-flash-image-preview`) against on-brand neutral backgrounds.

Invoke by describing what you need — e.g. "design a heimdall sentinel-eye mark and export the platform icon set" — or refer to it as `logo-generator`.

## Intentional deviations from the blog source

| Upstream (blog) | Here (heimdall) | Why |
| --- | --- | --- |
| Swiss / Geist typography | Space Grotesk + Space Mono + Doto | Matches DESIGN.md and `.claude/skills/industrial-design/`. |
| No chromatic accent | One optional `#D71921` pip | Heimdall's "one red accent per screen" rule permits a single signal pip; everything else stays monochrome. |
| 2px radii on all presentation containers | Same 2px default | Kept — aligns with industrial-design surfaces. |
| Outputs: 1024×1024 PNG + favicon | **Full platform icon set** via `render_icon_set.py` | Heimdall ships a macOS menu-bar app (`HeimdallBar`) and Linux/Windows binaries; needs `.icns`, `.ico`, hicolor tree, and menu-bar template. |
| `references/design_patterns.md` | Same content, new `heimdall-brand.md` added | Generic SVG patterns are domain-agnostic; brand-specific motifs (sentinel, dot-matrix H) live in a separate file. |
| 6 showcase backgrounds tuned to graphite/paper | Same 6, defaults re-ordered to `void` + `swiss_flat` + `clinical` | Matches heimdall's OLED dark default. |

## Setup

```bash
# Rasterizer — required for Phase 4 icon-set export (~2 min to compile)
cargo install resvg

# Python packaging + showcase deps
cd .claude/skills/logo-generator
pip install -r requirements.txt   # Pillow, python-dotenv, google-genai

# Optional — opt-in tauri icon pipeline for .icns + .ico (~3 min to compile)
cargo install tauri-cli           # enables --use-tauri flag on render_icon_set.py

# Optional — only needed for Phase 5 Gemini showcase renders
cp .env.example .env              # add GEMINI_API_KEY
```

No native library prerequisites. resvg is a self-contained Rust binary (tiny-skia-backed, zero runtime deps). Pillow handles `.ico` writing by default; passing `--use-tauri` delegates `.icns` and `.ico` packaging to `tauri icon` (uses the same Rust crates tauri uses internally). Phase 5 (Gemini showcase) is optional.

## Files

```
logo-generator/
├── SKILL.md                                 # workflow (Phase 0–6, Opus 4.7-tuned)
├── README.md                                # this file
├── requirements.txt                         # Pillow, python-dotenv, google-genai
├── .env.example                             # env template (gitignored locally)
├── .gitignore                               # excludes .env and output/
├── scripts/
│   ├── svg_to_png.py                        # resvg wrapper (single size)
│   ├── render_icon_set.py                   # master SVG → full platform icon set (--use-tauri opt-in)
│   ├── generate_showcase.py                 # Nano Banana showcase renderer (6 on-brand styles)
│   └── validate_svg.py                      # grammar-contract validator (stdlib-only)
├── references/
│   ├── svg-contract.md                      # hard grammar rules (enforced by validator)
│   ├── exemplar-library.md                  # 5 annotated canonical exemplars for few-shot
│   ├── heimdall-logo-brief.md               # creative brief: Norse anchor + collision list + variant table
│   ├── heimdall-brand.md                    # concept motifs (sentinel / horn / monogram)
│   ├── platform-icons.md                    # size + format matrix per platform
│   ├── logo-usage-rules.md                  # clear space, minimum sizes, file organization
│   ├── design_patterns.md                   # SVG pattern library (port from blog)
│   └── background_styles.md                 # 6 on-brand backgrounds, heimdall-tuned
├── assets/
│   ├── showcase_template.html               # interactive HTML gallery (heimdall-skinned)
│   └── appiconset-contents.json.tmpl        # Contents.json template for macOS xcassets
└── ci/
    ├── package.json                         # svglint + odiff-bin devDeps
    ├── package-lock.json                    # pinned install
    ├── .svglintrc.js                        # declarative grammar rules (mirror of svg-contract.md)
    └── visual_regression.sh                 # 3-gate runner: validator + svglint + odiff
```

The GitHub Actions workflow at `.github/workflows/logo-assets.yml` runs the same three gates on any PR touching `assets/icons/` or `.claude/skills/logo-generator/`.

## Validator (mechanical grammar gate)

`scripts/validate_svg.py` is a stdlib-only Python tool that parses each SVG and enforces the contract in `references/svg-contract.md`. It catches the documented Opus 4.7 / LLM SVG failure modes (LLM4SVG, Chat2SVG, SVGenius):

- Wrong `viewBox` (must be `0 0 100 100`)
- Forbidden elements (`<text>`, `<use>`, `<linearGradient>`, `<filter>`, …)
- Disallowed colors (chromatic drift outside the heimdall palette)
- Unclosed filled `<path>` (missing `Z`)
- Decimal arc flags and out-of-range coordinates (tokenization drift)
- Multiple `#D71921` accent pips (brief allows exactly one)
- Transforms on individual primitives (`<g>`-only rule)

```bash
python .claude/skills/logo-generator/scripts/validate_svg.py <file.svg>          # exits 0 on pass
python .claude/skills/logo-generator/scripts/validate_svg.py --strict <file.svg> # warnings become errors
```

The skill workflow runs the validator automatically in Phase 2c after each generated variant. Variants that fail the contract are rejected before the user-facing gallery.

## Output location

Generated assets land under `assets/icons/` at the repo root (not inside the skill). Suggested git policy:

- **Commit:** `assets/icons/master.svg`, `assets/icons/macos/heimdall.icns`, `assets/icons/windows/heimdall.ico`, `assets/icons/web/favicon.svg`, `assets/icons/web/favicon.ico`, `assets/icons/macos/AppIcon.appiconset/Contents.json`.
- **Optional to commit:** the per-size PNGs. They are fully reproducible from master.svg, so either track them for zero-dep builds or add `assets/icons/**/*.png` to `.gitignore` and regenerate on demand.

## Upstream

- Original: <https://github.com/op7418/logo-generator-skill>
- Intermediate adaptation (Swiss/Geist): `blog/.claude/skills/logo-generator/`
