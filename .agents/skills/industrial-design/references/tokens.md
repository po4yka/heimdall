# heimdall design system — Tokens

Canonical token definitions for the Apple-Swiss refined design system. Directory name `industrial-design/` is legacy; contents are current.

## 1. TYPOGRAPHY

### Font Stack (web — Preact dashboard)

| Role | Font | Fallback | Weight |
|------|------|----------|--------|
| **Body / UI / Headings** | `"Inter"` | `"Inter Variable", system-ui, sans-serif` | 400–700 |
| **Data / Code / Tabular** | `"Geist Mono"` | `"Geist Mono Variable", ui-monospace, "SF Mono", monospace` | 400, 500, 700 |

**Why these fonts:** Inter (Rasmus Andersson, 2017–2024) is the dominant screen-native grotesque of 2024–2026 — Univers/Helvetica DNA adapted for UI metrics, handles every weight from Light through Black, and ships with tabular + proportional number variants. Geist Mono (Vercel, 2024) is explicitly designed to pair with Inter — same x-height, same proportion, same stroke weight — and inherits the Swiss-grotesque lineage via Suisse Int'l and SF Mono. Both are free, MIT, and self-hostable.

**Do not use:** Space Grotesk, Space Mono, Doto (these were the Nothing/industrial era — removed). Helvetica Now (poor screen rendering below 16px). Geometric sans (Futura, Nunito, Poppins — reduce legibility at small sizes).

### Font Stack (Swift — Heimdall)

Use the SF Pro system cascade via SwiftUI's font modifiers:
- `.font(.largeTitle)` / `.font(.title)` / `.font(.headline)` for headings (auto-selects SF Pro Display above 20pt, Text below)
- `.font(.body)` / `.font(.callout)` for body
- `.font(.caption)` / `.font(.caption2)` for metadata
- `.monospacedDigit()` for numeric columns (tabular figures)
- `.font(.system(.body, design: .monospaced))` for code / paths

No custom font loading on SwiftUI. Apple's system stack is correct out of the box.

### Type Scale (web)

| Token | Size | Line Height | Letter Spacing | Use |
|-------|------|-------------|----------------|-----|
| `--display-xl` | 48px | 1.05 | -0.02em | Hero numbers |
| `--display-lg` | 36px | 1.1 | -0.02em | Section heroes, percentages |
| `--display-md` | 28px | 1.15 | -0.01em | Page titles |
| `--heading` | 20px | 1.2 | -0.005em | Section headings |
| `--subheading` | 17px | 1.3 | 0 | Subsections |
| `--body` | 15px | 1.5 | 0 | Body text (base) |
| `--body-sm` | 14px | 1.5 | 0 | Secondary body, table cells |
| `--caption` | 12px | 1.4 | 0 | Timestamps, footnotes |
| `--label` | 11px | 1.2 | 0.02em | Small metadata (sentence-case) |
| `--label-th` | 11px | 1.2 | 0.08em | `<th>` column headers (ALL-CAPS, the one exception) |

### Typographic Rules

- **Inter at all sizes** for body, headings, labels, metadata. Sentence-case throughout.
- **Hero numbers:** Inter at 36–48px, weight 500 (Medium) or 600 (Semibold). Tight letter-spacing (-0.02em).
- **Table column headers (`<th>`):** Geist Mono 11px, ALL-CAPS, letter-spacing 0.08em — the sole ALL-CAPS instance permitted. Justified by tabular convention.
- **Tabular numerals:** always on for numeric columns, hero numbers, percentages, currency. `font-feature-settings: "tnum"`.
- **Hierarchy:** 4 levels max (display > heading > body > label). Use weight (400/500/600) and size for differentiation, not new families.

---

## 2. COLOR SYSTEM

### Primary Palette (Dark Mode)

| Token | Hex | Contrast on `--black` | Role |
|-------|-----|------------------------|------|
| `--black` | `#0A0A0A` | — | Page canvas (refined dark, not OLED black) |
| `--surface` | `#111111` | 1.1:1 | Elevated surfaces, cards |
| `--surface-raised` | `#1A1A1A` | 1.5:1 | Secondary elevation, hover state |
| `--border` | `#222222` | — | Subtle dividers (decorative) |
| `--border-visible` | `#333333` | — | Intentional borders, wireframe lines |
| `--text-disabled` | `#666666` | 4.0:1 | Disabled text, decorative meta |
| `--text-secondary` | `#999999` | 6.3:1 | Labels, captions, metadata |
| `--text-primary` | `#E8E8E8` | 14.6:1 | Body text, table cells |
| `--text-display` | `#FFFFFF` | 20.5:1 | Hero numbers, headlines |

### Primary Palette (Light Mode)

| Token | Hex | Role |
|-------|-----|------|
| `--black` | `#F5F5F5` | Page canvas (warm off-white, paper-like) |
| `--surface` | `#FFFFFF` | Elevated surfaces, cards |
| `--surface-raised` | `#F0F0F0` | Secondary elevation |
| `--border` | `#E8E8E8` | Subtle dividers |
| `--border-visible` | `#CCCCCC` | Intentional borders |
| `--text-disabled` | `#707070` | Disabled text |
| `--text-secondary` | `#4F4F4F` | Labels, captions |
| `--text-primary` | `#1A1A1A` | Body text |
| `--text-display` | `#000000` | Hero numbers, headlines (pure black ink on paper — correct) |

### Accent & Status Colors (identical across modes)

| Token | Hex | Usage |
|-------|-----|-------|
| **`--accent-interactive`** | **`#4A7FA5`** | **Primary interactive affordance — links, selected states, primary buttons, active filter chips. The dominant "tappable signal." Same value light + dark.** |
| `--accent` | `#D71921` | **Semantic error / destructive / over-limit only.** Not a decorative accent. Not a "primary importance" signal. |
| `--accent-subtle` | `rgba(215,25,33,0.15)` | Tint backgrounds for error/destructive states only |
| `--success` | `#4A9E5C` | Confirmed, completed, healthy range |
| `--warning` | `#D4A843` | Caution, moderate-usage, attention |
| `--error` | `#D71921` | Alias of `--accent` — errors ARE the red signal |
| `--info` | `#999999` | Uses `--text-secondary` |

**Data status rules:**
- `--success` = within normal range / good / healthy
- `--warning` = 70–90% / caution / moderate
- `--accent` (red) = ≥90% / over limit / error / destructive
- `--text-primary` = neutral (no status encoding)
- **Apply color to the value itself**, not the label or row background. Labels stay `--text-secondary`. Trend arrows inherit value color.

**Interactive rules:**
- `--accent-interactive` is a semantic signal: blue-gray means "this responds to your click/tap." Use for all primary interactive affordances. Users learn it fast because it's consistent.
- A single screen can have many interactive affordances; the one-accent-per-screen discipline is about **emphasis accents**, not interactive colors.

### Mode Feel

- **Dark feel:** A refined charcoal surface. Data glows at `#E8E8E8` (not harsh white); `#FFFFFF` reserved for hero moments only. The `#0A0A0A` canvas reads as designed, not as "screen is off."
- **Light feel:** Printed technical manual. Warm off-white paper (`#F5F5F5`), black ink (`#000000` — pure black is correct on paper). Cards `#FFFFFF` on off-white canvas produce subtle elevation without shadows.

---

## 3. SPACING

### Spacing Scale (4px base)

| Token | Value | Use |
|-------|-------|-----|
| `--space-2xs` | 2px | Optical adjustments only |
| `--space-xs` | 4px | Icon-to-label gaps, tight padding |
| `--space-sm` | 8px | Component internal spacing |
| `--space-md` | 16px | Standard padding, element gaps |
| `--space-lg` | 24px | Group separation |
| `--space-xl` | 32px | Section margins |
| `--space-2xl` | 48px | Major section breaks |
| `--space-3xl` | 64px | Page-level vertical rhythm |
| `--space-4xl` | 96px | Hero breathing room |

### Concentric Radii

When nesting a shape inside another, the inner radius follows from the outer:

```
inner_radius = outer_radius − padding
```

Example: card with `border-radius: 16px` and `padding: 8px` contains a button — that button's `border-radius` is `8px`, not an independently chosen value. This prevents perceptual "pinching" (inner too small) or "flaring" (inner too large).

Default radii:
- Cards: 12–16px
- Compact cards / stat blocks: 8px
- Technical elements (tag, input, button-secondary): 4px
- Pills (button-primary, filter chip active): 999px (only for capsule elements)

---

## 4. MOTION & INTERACTION

- **Duration:** 150–250ms micro, 300–400ms transitions
- **Easing:** `cubic-bezier(0.25, 0.1, 0.25, 1)` — subtle ease-out. No spring, no bounce.
- Prefer opacity over position. Elements fade, don't slide.
- Hover: border/text brightens. No scale, no shadows.
- No parallax, scroll-jacking, gratuitous animation.
- Liquid Glass chrome on the sticky top header only — translucent backdrop-filter with subtle lensing is acceptable there. Content surfaces (cards, tables, panels) stay flat.

---

## 5. ICONOGRAPHY

- Monoline, 1.5px stroke, no fill. 24×24 base, 20×20 live area. Round caps/joins.
- Color inherits text color. Max 5–6 strokes.
- Preferred: Lucide (thin), Phosphor (thin). Never filled or multi-color.
- **No dot-matrix iconography, no crosshair/glyph decorative motifs, no perforated patterns.**
