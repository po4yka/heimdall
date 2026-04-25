# DESIGN.md – Heimdall (Coding Agent Usage Tracker)

A data-dense analytics dashboard for tracking coding agent usage (currently Claude Code and Codex). Apple-Swiss refined design: refined-dark + warm-off-white-light canvases, type-driven hierarchy via size/weight/space, semantic color with blue-gray interactive accent, red reserved for error/destructive. Calm enough for multi-hour sessions. The canonical source of truth is `.agents/skills/industrial-design/SKILL.md`; the legacy `.claude/skills/industrial-design/SKILL.md` copy remains for Claude compatibility.

---

## 1. Visual Theme & Atmosphere

| Attribute | Value |
|-----------|-------|
| Mood | Calm precision. Refined tool, not instrument panel. Dark = designed charcoal surface, not OLED-off. Light = printed technical manual on warm paper. |
| Density | 7/10 – data dashboard, not marketing page |
| Motion | 2/10 – hover transitions only (150-250ms `cubic-bezier(0.25,0.1,0.25,1)`), no scroll or entrance animations, no spring/bounce |
| Variance | 3/10 – left-aligned tables, concentric-radii card grid, predictable layout; break the pattern once per screen for emphasis |
| Philosophy | Structure through space, not decoration. Type-weight and whitespace create hierarchy. Monochrome surface; semantic color for meaning (blue-gray = interactive, red = error, green = healthy, amber = caution). Both dark and light modes first-class. |

**Anti-patterns (never use):**
- Emojis anywhere in rendered output
- Gradients on surfaces or UI chrome
- Shadows on content surfaces (cards, tables, panels). Liquid Glass translucency acceptable only on the sticky top header (navigation-layer chrome).
- Toast popups (use inline `[SAVED]` / `[ERROR: ...]` bracket status text)
- Skeleton loading screens (use `[LOADING...]` bracket text or minimal thin progress bar)
- Zebra striping in tables
- Filled icons, multi-color icons, or emoji as UI
- Centered hero sections or marketing-style layouts
- AI copywriting cliches ("Elevate", "Seamless", "Next-Gen")
- `border-radius` > 16px on cards; respect concentric radii (`inner_radius = outer_radius - padding`)
- Pure-black text on light backgrounds when the card is `#FFFFFF` (use `--text-primary` `#1A1A1A`)
- Parallax, scroll-jacking, spring/bounce easing
- More than one "break the pattern" moment per screen
- **Dot-matrix display type (Doto or similar). Space Grotesk / Space Mono outside legacy references. LED-meter segmented progress bars. Dot-grid backgrounds. Pure `#000000` canvas (use `#0A0A0A`). ALL-CAPS monospace outside `<th>` column headers. Red as a primary interactive/"important" color — red is reserved for error/destructive semantics.**

---

## 2. Color Palette & Roles

All colors must be declared as CSS variables. Never hardcode hex. The full table lives in `.agents/skills/industrial-design/references/tokens.md`; the rows below are the binding set for this project.

### Dark Theme (default)

| Token | Hex | Role |
|-------|-----|------|
| `--black` | `#0A0A0A` | Page canvas (refined dark; not OLED black) |
| `--surface` | `#111111` | Elevated surfaces, cards |
| `--surface-raised` | `#1A1A1A` | Secondary elevation, hover, active-row highlight |
| `--border` | `#222222` | Subtle dividers (decorative only) |
| `--border-visible` | `#333333` | Intentional borders, wireframe lines, filter outlines |
| `--text-disabled` | `#666666` | Disabled text, timestamps, decorative meta |
| `--text-secondary` | `#999999` | Labels, captions, metadata, chart axis labels |
| `--text-primary` | `#E8E8E8` | Body text, table cells |
| `--text-display` | `#FFFFFF` | Hero numbers, headlines, primary chart series |
| `--accent-interactive` | `#4A7FA5` | Primary interactive affordance: links, selected states, primary buttons, active filter chips. Same value light + dark. |
| `--accent` | `#D71921` | Error / destructive / over-limit only. Not a decorative accent. |
| `--accent-subtle` | `rgba(215,25,33,0.15)` | Tint for error/destructive states only |
| `--success` | `#4A9E5C` | Confirmed, completed, cost/usage within healthy range |
| `--warning` | `#D4A843` | Caution, moderate-usage, cache-creation tokens |
| `--error` | `#D71921` | Alias of `--accent` – errors ARE the red signal |

### Light Theme

| Token | Hex | Role |
|-------|-----|------|
| `--black` | `#F5F5F5` | Page canvas (warm off-white, paper-like) |
| `--surface` | `#FFFFFF` | Cards |
| `--surface-raised` | `#F0F0F0` | Secondary elevation |
| `--border` | `#E8E8E8` | Subtle dividers |
| `--border-visible` | `#CCCCCC` | Intentional borders |
| `--text-disabled` | `#707070` | Disabled / meta |
| `--text-secondary` | `#4F4F4F` | Labels, captions |
| `--text-primary` | `#1A1A1A` | Body text |
| `--text-display` | `#000000` | Headlines, hero numbers (pure black ink on paper — correct) |

**Identical across modes:** `--accent-interactive` (`#4A7FA5`), `--accent` (`#D71921`), status colors (`--success`, `--warning`), fonts, type scale, spacing, component shapes.

### Color Rules
- Never hardcode hex – always `var(--token-name)`.
- `--accent-interactive` (blue-gray) is the primary interactive signal — use for all tappable text, selected states, primary buttons, active filter chips. Users learn it fast because it's consistent.
- `--accent` (red) is strictly semantic: error / destructive / over-limit. Never a decorative or "most important" signal.
- `--success` / `--warning` / `--accent` as status colors apply to data values. Color the **value itself**, not labels or row backgrounds. Labels stay `--text-secondary`.
- Trend arrows inherit value color.
- Progress bars use semantic thresholds: `--success` (<70%), `--warning` (70-90%), `--accent` (≥90%). Smooth pill bars, color-encoded — no segmented LED-meter geometry.
- Data-viz differentiation priority: opacity (100/60/30) → pattern (solid/striped/dotted) → line style → color (last resort).

### Border Hierarchy
Use `--border` for decorative dividers and card edges (default). Use `--border-visible` for intentional borders, wireframe outlines, filter outlines, and focus rings. Depth comes from background steps + borders, not shadows.

### Shadow Policy
- **No shadows, period.** Flat surfaces. Depth comes from `--surface` → `--surface-raised` background steps and from border contrast (`--border` → `--border-visible`).
- No neutral shadows, no brand-tinted shadows, no inset shadows. Overlays rely on a dark backdrop (`rgba(0,0,0,0.8)` for modals) plus a 1px `--border-visible` outline.

### Chart Series Colors

Primary palette for multi-series charts, in application order:
1. `--text-display` (white / black by mode) — the primary series
2. `--text-display` at 60% opacity — secondary series
3. `--text-display` at 30% opacity — tertiary series
4. `--accent-interactive` — a single emphasized / selected series
5. `--accent` — only if one series is semantically "over limit" or "error"

For model-distribution donuts or categorical series that genuinely need distinct colors, fall back to the status palette (`--success`, `--warning`, `--accent-interactive`, `--accent`) in that order — never more than four colors on a single chart.

---

## 3. Typography Rules

| Element | Font | Size | Weight | Color | Extras |
|---------|------|------|--------|-------|--------|
| Page title | Inter | 24-28px (`--display-md`) | 500 | `--text-display` | letter-spacing -0.01em, sentence-case |
| Section title | Inter | 13-14px | 500 | `--text-primary` | sentence-case; 16px bottom margin |
| Hero / stat card value | Inter | 36-48px (`--display-md` / `--display-lg`) | 600 | `--text-display`; status color when encoding data | letter-spacing -0.02em, line-height 1.05, `font-feature-settings: "tnum"` |
| Stat card label | Inter | 12px (`--caption`) | 400 | `--text-secondary` | sentence-case |
| Table column header (`<th>`) | Geist Mono | 11px (`--label-th`) | 400 | `--text-secondary` | **ALL CAPS (sole exception)**, letter-spacing 0.08em |
| Table cell (text) | Inter | 14px (`--body-sm`) | 400 | `--text-primary` | left-aligned |
| Table cell (numbers) | Geist Mono | 14px (`--body-sm`) | 400 | `--text-primary`; `--success` / `--warning` / `--accent` when status-encoded | right-aligned, `font-feature-settings: "tnum"` |
| Body text | Inter | 15px (`--body`) | 400 | `--text-primary` | line-height 1.5 |
| Caption | Inter | 12px (`--caption`) | 400 | `--text-secondary` | timestamps, footnotes, sentence-case |
| Filter labels | Inter | 12px (`--caption`) | 500 | `--text-secondary` | sentence-case |
| Footer | Inter | 12px (`--caption`) | 400 | `--text-secondary` | sentence-case |
| Chart title | Inter | 13px | 500 | `--text-primary` | sentence-case |
| Inline status | Inter | 12px (`--caption`) | 400 | status color | `[Saved]`, `[Error: ...]`, `[Loading...]` sentence-case |

**Font stacks** (declare in `src/ui/input.css` under `@theme`):
- Body / UI / Headings: `'Inter', system-ui, sans-serif`
- Data / Code / Numbers: `'Geist Mono', ui-monospace, 'SF Mono', monospace`

**Rules:**
- Max 2 font families on a screen (Inter + Geist Mono).
- Max 3 font sizes per screen; max 2 weights (typically 400 + 500, or 400 + 600 for a hero number).
- All numbers, token counts, cost values, and progress percentages use Geist Mono with `font-feature-settings: "tnum"` for tabular alignment during the auto-refresh cycle.
- **Sentence-case throughout.** The sole ALL-CAPS exception: `<th>` column headers, which use Geist Mono 11px ALL-CAPS at 0.08em tracking — justified by tabular convention.
- Body text uses `-webkit-font-smoothing: antialiased`.
- Before adding a new font size or weight, first try spacing or color to make the distinction.

---

## 4. Component Stylings

### Stat Cards
- Background: `var(--surface)`, 1px solid `var(--border)` (or none), radius: `12-16px`
- Padding: `16-24px`
- Layout: label above (Inter 12px, sentence-case, `--text-secondary`), hero value below (Inter `--display-md` or `--display-lg`, weight 600, left-aligned), optional sub-text (`--caption`, `--text-disabled`)
- Cost / status values: the value text takes the status color (`--success`, `--warning`, `--accent`) — label and background stay neutral
- Optional sparkline slot: right-aligned, 80×32px, `--text-secondary` stroke
- All numeric values: Geist Mono with `font-feature-settings: "tnum"`
- Hover: border brightens to `--border-visible`. No translate, no shadow, no wash.

### Cards (generic)
- Background: `var(--surface)`, 1px solid `var(--border)`, radius: `12-16px`
- Padding: `16-24px`
- No shadows at rest, no shadows on hover. Depth = border + background step only.
- Respect concentric radii: a button inside a 16px-radius card with 16px padding carries `border-radius: 0` (calc result); with 8px padding → 8px; with 4px padding → 12px.
- Never box the most important element on a screen – let it float on `--black`.

### Data Tables
- Full-width, collapsed borders
- Column header (`<th>`): Geist Mono 11px **ALL-CAPS** (the sole ALL-CAPS instance in the system), letter-spacing 0.08em, `--text-secondary`, bottom border `--border-visible`, sticky on scroll, `--surface` background
- Rows: `1px solid --border` bottom divider, 12-16px vertical padding
- Cell text: Inter. Cell numbers: Geist Mono with `tnum`.
- No zebra striping, no cell backgrounds.
- Hover: row background → `--surface-raised`. No accent tint at rest.
- Active sort column: header gets `border-bottom: 2px solid --text-display`. Sort indicator inline, Geist Mono arrow.
- Active row: `--surface-raised` background + left `2px solid --accent-interactive` bar (blue-gray selection indicator; use sparingly for current selection only)
- Cell padding: `12px 16px`. Numbers right, text left.

### Filter Bar
- Background: `var(--surface)`, `1px solid --border-visible` bottom
- Horizontal flex with `16px` gap, wraps on small screens
- Model checkboxes: pill chips, `1px solid --border-visible`, Inter 12px sentence-case, `4px 12px` padding
- Active chip: `--accent-interactive` border + text (blue-gray). Optional subtle bg fill at 10% opacity for stronger selection signal.
- Range buttons: segmented control (§2.8 in components.md). Container `1px solid --border-visible`, active segment `--accent-interactive` background + `--text-display` text, transition 200ms ease-out.
- Separators: 1px wide, 20px tall, `--border` color.
- No inset shadows.

### Buttons
Per components.md §2. Four variants: **Primary** (`--accent-interactive` fill, white text, 8px radius), **Secondary** (transparent, `1px solid --border-visible`, `--text-primary`, 8px radius), **Ghost** (transparent, no border, `--text-secondary`), **Destructive** (transparent, red border, red text, 8px radius). All Inter 13-14px, weight 500, sentence-case, `0` letter-spacing, `10px 16px` padding, min-height 40px.

### Progress Bars (Rate Windows)
- **Smooth pill bars.** Single-fill geometry with 2px border-radius at each end. No segments, no gaps.
- Label + value above in Inter (labels) / Geist Mono (numbers). Bar below.
- Track: `--border` (dark) / `#E0E0E0` (light). Fill: `--success` (<70%), `--warning` (70-90%), `--accent` (≥90% or overflow). Generic progress: `--accent-interactive`.
- Overflow: fill continues past 100% in `--accent` red.
- Heights: hero 16-20px, standard 8-12px, compact 4-6px.
- Always pair with numeric readout – bar = proportion, number = precision.

### Inline Status (replaces toasts)
- No toast popups. Status appears inline near the trigger.
- Format: `[Saved]`, `[Error: description]`, `[Loading...]` in Inter `--caption` sentence-case (bracket notation preserved as semantic marker).
- Error: `--accent` text color. Success: `--success`. Loading: `--text-secondary`.
- Persist until next action or 4s, whichever is earlier. No animations beyond a 150ms opacity fade in/out.

### Pagination
- Flex between page info (left) and prev/next buttons (right)
- `--caption` size, `--text-secondary` color, Inter sentence-case
- Top border: `1px solid --border`
- Disabled buttons: opacity 0.4, `cursor: not-allowed`

---

## 5. Layout Principles

### Spacing Scale (tokens in `--space-*`)

| Token | Value | Use |
|-------|-------|-----|
| `--space-xs` | 4px | Icon-to-label gaps, tight padding ("these belong together") |
| `--space-sm` | 8px | Component internal spacing |
| `--space-md` | 16px | Standard padding, element gaps ("same group, different items") |
| `--space-lg` | 24px | Group separation, card padding |
| `--space-xl` | 32px | Section margins |
| `--space-2xl` | 48px | Major section breaks ("new group starts here") |
| `--space-3xl` | 64px | Page-level vertical rhythm |
| `--space-4xl` | 96px | Hero breathing room ("new context") |

Spacing communicates relationship. If reaching for a divider, try more spacing first.

### Grid System
- Bento grid: `max-width: 1400px`, centered, 4 columns
- Column spans: `.bento-2`, `.bento-3`, `.bento-full`
- Gap: `16px` uniform (was 12px – loosened to align with `--space-md` rhythm)
- Stats row: `auto-fit` with `minmax(200px, 1fr)` – fluid column count

### Page Structure
```
Header (sticky, 48px, flat -- no frosted glass, no shadow)
  Filter Bar (sticky below header, 1px bottom border)
    Bento Grid (main content)
      Rate Windows (conditional, full width, segmented progress bars)
      Stats Cards (fluid row, 7 cards)
      Charts Row (2-col daily + 1-col model + 1-col projects)
      Data Sections (full width, conditional)
      Tables (full width)
    Footer
```

### Whitespace Philosophy
- Confidence through emptiness. Resist filling space.
- Cards are the primary spatial unit – no free-floating text outside cards.
- Header is compact (48px) because it's always visible (sticky).
- Asymmetry > symmetry. Prefer large-left/small-right; top-heavy; edge-anchored.

---

## 6. Depth & Elevation

| Surface | Treatment |
|---------|-----------|
| Page canvas | `var(--black)` – deepest layer |
| Filter bar | `var(--surface)` – one step up, no blur |
| Header | `var(--surface)` – flat, 1px `--border-visible` bottom. No frosted glass. |
| Cards | `var(--surface)` + `1px solid var(--border)`. No elevation shadow. |
| Card hover | Border brightens to `--border-visible`. No translate, no shadow. |
| Tables | Inside cards, no additional elevation |
| Inline status | Same layer as content, no elevation |
| Modals | Backdrop `rgba(0,0,0,0.8)`, dialog = `--surface` + `1px solid --border-visible`, 16px radius, max 480px |
| Sticky thead | `z-index: 1`, `--surface` background |

No shadow-as-border tricks. A card is a card: `background: var(--surface); border: 1px solid var(--border);`. Layout shift on hover is prevented by using `--border-visible` at the same 1px width, not by swapping from shadow to shadow.

No frosted glass. No backdrop-filter. The palette already does the work; blur would add visual weight that contradicts "subtract, don't add."

---

## 7. Do's and Don'ts

### Do
- Use CSS variables for every color value
- Put numbers, costs, labels, and table cells in Geist Mono with `font-feature-settings: "tnum"`
- Use sentence-case throughout (stat labels, section titles, filter labels, chart titles). Reserve ALL-CAPS monospace for `<th>` table column headers only.
- Use a plain `border: 1px solid var(--border)` on cards – clean and stable
- Make tables sortable with `--text-display` border-bottom on the active sort column
- Support both light and dark themes (toggle via `data-theme` attribute)
- Keep chart heights consistent: 240px standard, 300px for the daily chart
- Run all dynamic text through `esc()` before rendering
- Use `auto-fit` / `minmax` for fluid grid layouts
- Format large numbers with abbreviations (1.5M, 2.3K)
- Use `--accent-interactive` for primary tappable/selected affordances; `--accent` (red) for error/destructive only
- Apply opacity (100/60/30) or pattern before reaching for a new chart color
- Respect concentric corner radii when nesting shapes (`inner = outer - padding`)

### Don't
- Add decorative elements that don't display data
- Use more than 2 font families (Inter + Geist Mono)
- Add any shadow on content surfaces (cards, tables, panels). Liquid Glass translucency is permitted only on the sticky top header.
- Add `backdrop-filter: blur(...)` on content surfaces (sticky header only)
- Use toast popups (use inline `[STATUS]` text)
- Use skeleton loaders (use `[LOADING...]` bracket text)
- Zebra-stripe tables
- Create custom scrollbars or override native scroll behavior
- Add entrance animations or scroll-triggered effects
- Use filled or multi-color icons
- Make the header taller than 48px
- Add tooltips unless showing precise values on chart hover
- Use `z-index` values above 999
- Introduce a second "break the pattern" moment on a single screen
- Use spring/bounce easing

---

## 8. Responsive Behavior

| Breakpoint | Grid | Adjustments |
|------------|------|-------------|
| > 1024px | 4 columns | Full layout, all features visible |
| 768-1024px | 2 columns | `.bento-3` collapses to 2-col span |
| < 768px | 1 column | Single column stack, padding shrinks to 12px, gap to 8px |
| < 480px | 1 column | Stat values shrink to 28px, sparklines hidden |

### Responsive Rules
- Filter bar wraps naturally via `flex-wrap: wrap`
- Tables scroll horizontally via `overflow-x: auto` on the card wrapper
- Charts resize within their card containers (ApexCharts handles responsively)
- Header padding shrinks from 24px to 12px below 768px
- Grid gap tightens from 16px to 8px below 768px
- No content is hidden on mobile except sparklines at 480px

---

## 9. Agent Prompt Guide

### Quick Reference
```
Background:    #0A0A0A (dark) / #F5F5F5 (light)
Cards:         #111111 (dark) / #FFFFFF (light)
Borders:       #222222 (dark) / #E8E8E8 (light) -- default
               #333333 (dark) / #CCCCCC (light) -- visible
Primary text:  #E8E8E8 (dark) / #1A1A1A (light)
Display text:  #FFFFFF (dark) / #000000 (light)
Interactive:   #4A7FA5 -- blue-gray, primary interactive affordance (both modes)
Error:         #D71921 -- red, error/destructive/over-limit ONLY
Success:       #4A9E5C  |  Warning: #D4A843
Fonts:         Inter (body/UI/headings), Geist Mono (numbers/code/<th>)
Case:          sentence-case everywhere except <th> (ALL-CAPS Geist Mono 11px 0.08em)
Card radius:   12-16px (respect concentric: inner = outer - padding)
Grid gap:      16px (4px modular base)
Max width:     1400px
```

### Ready-to-Use Prompts

**"Add a new stat card"**
> Create a stat card: `background: var(--surface)` + `1px solid var(--border)` + 16px radius + 20px padding. Label above (Inter 12px sentence-case `--text-secondary`), hero value below (Inter 36-48px weight 600 `--text-display`, or status color when encoding data), optional sub-text (`--caption` `--text-disabled`). Apply `font-family: var(--font-mono)` + `font-feature-settings: "tnum"` to any numeric value. Hover: border brightens to `--border-visible`. No shadow.

**"Add a new data table"**
> Create a full-width table inside a card. Column header (`<th>`): sticky, Geist Mono 11px **ALL-CAPS** letter-spacing 0.08em `--text-secondary` (this is the sole ALL-CAPS exception), `1px solid --border-visible` bottom border. Rows: `1px solid --border` divider, 12-16px vertical padding. No zebra striping, no cell backgrounds. Hover: row bg → `--surface-raised`. Sortable columns: `cursor: pointer`, inline Geist Mono arrow, active sort column gets `border-bottom: 2px solid --text-display`. Numbers right-aligned Geist Mono with `tnum`; text left-aligned Inter. Run all dynamic text through `esc()`.

**"Add a new chart"**
> Wrap in a card with Inter 13px sentence-case `--text-primary` title. Chart container: 240px (standard) or 300px (hero/daily). ApexCharts options: `background: 'transparent'`, `toolbar: { show: false }`, `fontFamily: 'var(--font-mono), "Geist Mono", ui-monospace, monospace'`, `animations: { enabled: false }`, `legend: { show: false }`. Colors: `--text-display` primary, `--text-display` at 60/30% opacity for secondary series. `--accent-interactive` for one selected/emphasized series. `--accent` (red) for a single error/over-limit series only. Grid: horizontal lines in `--border`. Destroy and recreate (or `updateOptions`) on theme toggle.

**"Add a new filter control"**
> Place inside the filter bar. Group label: Inter 12px sentence-case `--text-secondary`. Chips: `1px solid --border-visible`, transparent bg, Inter 12px sentence-case, `4px 12px` padding, pill radius (999px). Active chip: `--accent-interactive` border + text (optionally subtle 10% bg fill). Range buttons: segmented control with `1px solid --border-visible` container; active segment = `--accent-interactive` background + `--text-display` text; transition 200ms ease-out. No inset shadows.

**"Add an inline status message"**
> Replace any toast with inline status text near the trigger: Inter `--caption` sentence-case, format `[Saved]` (`--success`), `[Error: message]` (`--accent`), `[Loading...]` (`--text-secondary`). 150ms opacity fade in/out. Auto-dismiss after 4 seconds or on next action.

---

## 10. Source of truth

- Skill: `.agents/skills/industrial-design/SKILL.md`
- Tokens: `.agents/skills/industrial-design/references/tokens.md`
- Components: `.agents/skills/industrial-design/references/components.md`
- Platform mapping (Preact + Tailwind v4): `.agents/skills/industrial-design/references/platform-mapping.md`

---

## 11. Active-Period Averaging

**Formula:** `avg_cost_per_active_day = total_cost_nanos / active_days`

**`active_days`** = `COUNT(DISTINCT date)` where `date` is the calendar day (in the client's local timezone) of turns with `estimated_cost_nanos > 0`.

**Example:** If a user worked on 12 of 30 calendar days in a month with a total spend of $24.00, the active-period average is $2.00/day — not $0.80/day (which would result from dividing by 30).

**Display rule:** Show `--` (two dashes) rather than a computed value when `active_days = 0` (no spend in the selected period) to avoid confusing a divide-by-zero edge case with a meaningful zero result.

**Timezone note:** The day boundary respects `tz_offset_min` sent by the client (Phase 14). When absent, days are bucketed in UTC. The `GET /api/heatmap` endpoint computes both `active_days` and `total_cost_nanos` and returns them alongside the 7×24 cell matrix so the StatsCards "Avg / Active Day" card can read them without a separate request.

---

## 12. Cache Hit Rate

**Formula:** `cache_hit_rate = cache_read_tokens / (cache_read_tokens + input_tokens)`

**Denominator rationale:** `cache_read_tokens` is the tokens we avoided re-billing (served from cache). `input_tokens` is the tokens we still paid for at the full input rate. Their sum is the "addressable" token stream — the universe of tokens that could have been served from cache. This denominator makes the rate directly actionable: optimizing it means shifting tokens from the `input` bucket into the `cache_read` bucket.

**Interpretation:**
- **0%** — No cache reuse. Every addressable token was billed at full input price.
- **50%** — Half the addressable tokens came from cache. Cache is providing meaningful savings.
- **100%** — Theoretical maximum. Cache served everything; no new input tokens were billed.

**Display rule:** The card displays `--` (two dashes) rather than a percentage when both `cache_read_tokens` and `input_tokens` are zero (denominator is zero). This distinguishes "no data" from a genuine 0% hit-rate (which can only occur when `cache_read = 0` but `input > 0`).

**Lagging metric note:** This is a behavioral/lagging metric computed from stored turn data, not a real-time gauge. It reflects the aggregate cache behaviour over the selected data range, not the current moment. Users should interpret changes over days or weeks, not seconds.

**Cost savings estimate (tooltip):** Approximated as `cache_read_tokens × (input_rate − cache_read_rate)` per MTok, using the Rust-computed pricing for the dominant model. This is an estimate because the actual savings depend on which model served each turn.
