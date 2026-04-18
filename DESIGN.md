# DESIGN.md -- Claude Usage Tracker

A data-dense analytics dashboard for Claude Code usage tracking. Monochromatic industrial design: OLED dark + warm-off-white light, type-driven hierarchy, one surgical red accent reserved for urgent/destructive states. Every pixel earns its place by showing data. The canonical source of truth is `.claude/skills/industrial-design/SKILL.md`; this file mirrors the product-specific decisions that layer on top of the skill.

---

## 1. Visual Theme & Atmosphere

| Attribute | Value |
|-----------|-------|
| Mood | Instrument panel in a dark room / printed technical manual in light mode |
| Density | 7/10 -- data dashboard, not marketing page |
| Motion | 2/10 -- hover transitions only (150-250ms `cubic-bezier(0.25,0.1,0.25,1)`), no scroll or entrance animations, no spring/bounce |
| Variance | 3/10 -- uniform card grid, left-aligned tables, predictable layout; break pattern in exactly one place per screen |
| Philosophy | Subtract, don't add. Structure is ornament. Monochrome is the canvas; color is an event. Type does the heavy lifting. Both dark and light modes are first-class. |

**Anti-patterns (never use):**
- Emojis anywhere in rendered output
- Gradients on surfaces or UI chrome
- Shadows for elevation (flat surfaces, border separation only)
- Toast popups (use inline `[SAVED]` / `[ERROR: ...]` status text)
- Skeleton loading screens (use `[LOADING...]` bracket text or segmented spinner)
- Zebra striping in tables
- Filled icons, multi-color icons, or emoji as UI
- Centered hero sections or marketing-style layouts
- AI copywriting cliches ("Elevate", "Seamless", "Next-Gen")
- `border-radius` > 16px on cards; buttons are pill (999px) or technical (4-8px)
- Pure-black text on light backgrounds when the card is `#FFFFFF` (use `--text-primary` `#1A1A1A`)
- Parallax, scroll-jacking, spring/bounce easing
- More than one "break the pattern" moment per screen

---

## 2. Color Palette & Roles

All colors must be declared as CSS variables. Never hardcode hex. The full table lives in `.claude/skills/industrial-design/references/tokens.md`; the rows below are the binding set for this project.

### Dark Theme (default)

| Token | Hex | Role |
|-------|-----|------|
| `--black` | `#000000` | Page canvas (OLED background) |
| `--surface` | `#111111` | Elevated surfaces, cards |
| `--surface-raised` | `#1A1A1A` | Secondary elevation, hover, active-row highlight |
| `--border` | `#222222` | Subtle dividers (decorative only) |
| `--border-visible` | `#333333` | Intentional borders, wireframe lines, filter outlines |
| `--text-disabled` | `#666666` | Disabled text, timestamps, decorative meta |
| `--text-secondary` | `#999999` | Labels, captions, metadata, chart axis labels |
| `--text-primary` | `#E8E8E8` | Body text, table cells |
| `--text-display` | `#FFFFFF` | Hero numbers, headlines, primary chart series |
| `--accent` | `#D71921` | Signal red: active destructive, over-limit, urgent state. One per screen. |
| `--accent-subtle` | `rgba(215,25,33,0.15)` | Accent tint backgrounds (sparingly) |
| `--success` | `#4A9E5C` | Confirmed, completed, cost/usage within healthy range |
| `--warning` | `#D4A843` | Caution, moderate-usage, cache-creation tokens |
| `--error` | `#D71921` | Shares accent red -- errors ARE the accent moment |
| `--interactive` | `#5B9BF6` | Tappable text: links, picker values. Not for buttons. |

### Light Theme

| Token | Hex | Role |
|-------|-----|------|
| `--black` | `#F5F5F5` | Page canvas (warm off-white) |
| `--surface` | `#FFFFFF` | Cards |
| `--surface-raised` | `#F0F0F0` | Secondary elevation |
| `--border` | `#E8E8E8` | Subtle dividers |
| `--border-visible` | `#CCCCCC` | Intentional borders |
| `--text-disabled` | `#999999` | Disabled / meta |
| `--text-secondary` | `#666666` | Labels, captions |
| `--text-primary` | `#1A1A1A` | Body text |
| `--text-display` | `#000000` | Headlines, hero numbers |
| `--interactive` | `#007AFF` | Links |

**Identical across modes:** accent red, status colors (`--success`, `--warning`, `--accent`/`--error`), ALL-CAPS labels, fonts, type scale, spacing, component shapes.

### Color Rules
- Never hardcode hex -- always `var(--token-name)`.
- `--accent` is a signal, not decoration. Max **one** `--accent` element per screen as UI (the "break the pattern" moment), unless encoding data status on a value.
- `--success` / `--warning` / `--accent` as status colors are exempt from the one-accent rule when encoding data values. Apply the color to the **value itself**, not labels or row backgrounds. Labels stay `--text-secondary`.
- Trend arrows inherit value color.
- Progress bars use semantic thresholds: `--success` (<70%), `--warning` (70-90%), `--accent` (>=90%).
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
4. `--accent` — only if one series is semantically "over limit", "urgent", or the one highlighted moment

For model-distribution donuts or categorical series that genuinely need distinct colors, fall back to the status palette (`--success`, `--warning`, `--interactive`, `--accent`) in that order — never more than four colors on a single chart.

---

## 3. Typography Rules

| Element | Font | Size | Weight | Color | Extras |
|---------|------|------|--------|-------|--------|
| Page title | Space Grotesk | 24-36px (`--heading` / `--display-md`) | 500 | `--text-display` | letter-spacing -0.01em |
| Section title | Space Mono | 11px (`--label`) | 400 | `--text-secondary` | ALL CAPS, letter-spacing 0.08em (instrument label) |
| Hero / stat card value | Space Mono (Doto for single hero) | 36-48px (`--display-md` / `--display-lg`) | 400 | `--text-display`; status color when encoding data | letter-spacing -0.02em, line-height 1.05 |
| Stat card label | Space Mono | 11px (`--label`) | 400 | `--text-secondary` | ALL CAPS, letter-spacing 0.08em |
| Table header | Space Mono | 11px (`--label`) | 400 | `--text-secondary` | ALL CAPS, letter-spacing 0.08em, `--border-visible` bottom border |
| Table cell (text) | Space Grotesk | 14px (`--body-sm`) | 400 | `--text-primary` | left-aligned |
| Table cell (numbers) | Space Mono | 14px (`--body-sm`) | 400 | `--text-primary`; `--success` / `--warning` / `--accent` when status-encoded | right-aligned, `font-feature-settings: "tnum"` |
| Body text | Space Grotesk | 16px (`--body`) | 400 | `--text-primary` | line-height 1.5 |
| Caption | Space Mono | 12px (`--caption`) | 400 | `--text-secondary` | timestamps, footnotes |
| Filter labels | Space Mono | 11px (`--label`) | 400 | `--text-secondary` | ALL CAPS, letter-spacing 0.08em |
| Footer | Space Grotesk | 12px (`--caption`) | 400 | `--text-secondary` | |
| Chart title | Space Mono | 11px (`--label`) | 400 | `--text-secondary` | ALL CAPS |
| Inline status | Space Mono | 12px (`--caption`) | 400 | status color | `[SAVED]`, `[ERROR: ...]`, `[LOADING...]` |

**Font stacks** (declare in `src/ui/input.css` under `@theme`):
- Body / UI: `'Space Grotesk', 'DM Sans', system-ui, sans-serif`
- Data / Labels: `'Space Mono', 'JetBrains Mono', 'SF Mono', monospace`
- Display (hero only): `'Doto', 'Space Mono', monospace`

**Rules:**
- Max 2 font families on a screen (Space Grotesk + Space Mono). Doto reserved for hero moments only, at 36px+.
- Max 3 font sizes per screen; max 2 weights.
- All numbers, token counts, cost values, and progress percentages use Space Mono with `font-feature-settings: "tnum"` for tabular alignment during the auto-refresh cycle.
- Labels (stat cards, table headers, filter labels, chart titles, section titles) are always Space Mono, ALL CAPS, 11px, `--text-secondary`, letter-spacing 0.08em. This creates the unified instrument-readout aesthetic.
- Body text uses `-webkit-font-smoothing: antialiased`.
- Before adding a new font size or weight, first try spacing or color to make the distinction.

---

## 4. Component Stylings

### Stat Cards
- Background: `var(--surface)`, 1px solid `var(--border)` (or none), radius: `12-16px`
- Padding: `16-24px`
- Layout: label above (Space Mono, ALL CAPS, `--label` size, `--text-secondary`), hero value below (Space Mono, `--display-md` or `--display-lg`, left-aligned), optional sub-text (`--caption`, `--text-disabled`)
- Cost / status values: the value text takes the status color (`--success`, `--warning`, `--accent`) — label and background stay neutral
- Optional sparkline slot: right-aligned, 80x32px, `--text-secondary` stroke
- All numeric values: `font-feature-settings: "tnum"`
- Hover: border brightens to `--border-visible`. No translate, no shadow, no accent wash.

### Cards (generic)
- Background: `var(--surface)`, 1px solid `var(--border)`, radius: `12-16px`
- Padding: `16-24px`
- No shadows at rest, no shadows on hover. Depth = border + background step only.
- Never box the most important element on a screen -- let it float on `--black`.

### Data Tables
- Full-width, collapsed borders
- Header: `--label` style, bottom border `--border-visible`, sticky on scroll, `--surface` background
- Rows: `1px solid --border` bottom divider, 12-16px vertical padding
- No zebra striping, no cell backgrounds.
- Hover: row background → `--surface-raised`. No accent tint at rest.
- Active sort column: header gets `border-bottom: 2px solid --text-display`. Sort indicator inline, Space Mono arrow.
- Active row: `--surface-raised` background + left `2px solid --accent` bar (use sparingly — e.g., current selection only)
- Cell padding: `12px 16px`. Numbers right, text left.

### Filter Bar
- Background: `var(--surface)`, `1px solid --border-visible` bottom
- Horizontal flex with `16px` gap, wraps on small screens
- Model checkboxes: pill chips, `1px solid --border-visible`, Space Mono `--caption` ALL CAPS, `4px 12px` padding
- Active chip: `--text-display` border + text (inverted). No background fill at rest.
- Range buttons: segmented control (§2.8 in components.md). Container `1px solid --border-visible`, active segment `--text-display` background + `--black` text, transition 200ms ease-out.
- Separators: 1px wide, 20px tall, `--border` color.
- No inset shadows. Active state = inverted fill, nothing else.

### Buttons
Per components.md §2. Four variants: Primary (white fill, black text, pill), Secondary (transparent, border, pill), Ghost (transparent, no border), Destructive (transparent, red border, red text, pill). All Space Mono 13px ALL CAPS, `0.06em` letter-spacing, `12px 24px` padding, min-height `44px`.

### Progress Bars (Rate Windows)
- Prefer **segmented progress bars** (§11 in components.md) -- signature visualization. Discrete square-ended blocks with 2px gaps.
- Label + value above in Space Mono. Bar below.
- Empty segments: `--border`. Filled segments: `--text-display` (neutral), `--success` (<70%), `--warning` (70-90%), `--accent` (>=90% or overflow).
- Heights: hero 16-20px, standard 8-12px, compact 4-6px.
- Always pair with numeric readout -- bar = proportion, number = precision.

### Inline Status (replaces toasts)
- No toast popups. Status appears inline near the trigger.
- Format: `[SAVED]`, `[ERROR: description]`, `[LOADING...]` in Space Mono `--caption` ALL CAPS.
- Error: `--accent` text color. Success: `--success`. Loading: `--text-secondary`.
- Persist until next action or 4s, whichever is earlier. No animations beyond a 150ms opacity fade in/out.

### Pagination
- Flex between page info (left) and prev/next buttons (right)
- `--caption` size, `--text-secondary` color, Space Mono
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
- Gap: `16px` uniform (was 12px -- loosened to align with `--space-md` rhythm)
- Stats row: `auto-fit` with `minmax(200px, 1fr)` -- fluid column count

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
- Cards are the primary spatial unit -- no free-floating text outside cards.
- Header is compact (48px) because it's always visible (sticky).
- Asymmetry > symmetry. Prefer large-left/small-right; top-heavy; edge-anchored.

---

## 6. Depth & Elevation

| Surface | Treatment |
|---------|-----------|
| Page canvas | `var(--black)` -- deepest layer |
| Filter bar | `var(--surface)` -- one step up, no blur |
| Header | `var(--surface)` -- flat, 1px `--border-visible` bottom. No frosted glass. |
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
- Put numbers, costs, labels, and table cells in Space Mono with `font-feature-settings: "tnum"`
- Use Space Mono ALL CAPS for all system labels (stat labels, table headers, filter labels, chart titles, section titles)
- Use a plain `border: 1px solid var(--border)` on cards -- clean and stable
- Make tables sortable with `--text-display` border-bottom on the active sort column
- Support both light and dark themes (toggle via `data-theme` attribute)
- Keep chart heights consistent: 240px standard, 300px for the daily chart
- Run all dynamic text through `esc()` before rendering
- Use `auto-fit` / `minmax` for fluid grid layouts
- Format large numbers with abbreviations (1.5M, 2.3K)
- Reserve `--accent` for the one urgent moment per screen
- Apply opacity (100/60/30) or pattern before reaching for a new chart color

### Don't
- Add decorative elements that don't display data
- Use more than 2 font families (Space Grotesk + Space Mono; Doto hero-only)
- Add any shadow (neutral, tinted, or inset). Period.
- Add `backdrop-filter: blur(...)` on any surface
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
Background:    #000000 (dark) / #F5F5F5 (light)
Cards:         #111111 (dark) / #FFFFFF (light)
Borders:       #222222 (dark) / #E8E8E8 (light) -- default
               #333333 (dark) / #CCCCCC (light) -- visible
Primary text:  #E8E8E8 (dark) / #1A1A1A (light)
Display text:  #FFFFFF (dark) / #000000 (light)
Accent:        #D71921 -- signal red, one per screen
Success:       #4A9E5C  |  Warning: #D4A843
Fonts:         Space Grotesk (body), Space Mono (data/labels), Doto (hero display only)
Card radius:   12-16px
Grid gap:      16px
Max width:     1400px
```

### Ready-to-Use Prompts

**"Add a new stat card"**
> Create a stat card matching the industrial pattern: card container with `background: var(--surface)` + `border: 1px solid var(--border)` + 16px radius + 20px padding. Label above (Space Mono 11px ALL CAPS `--text-secondary` letter-spacing 0.08em), hero value below (Space Mono 36-48px `--text-display`, or status color when encoding data), optional sub-text (`--caption` `--text-disabled`). Apply `font-feature-settings: "tnum"` to the value. Hover: border brightens to `--border-visible`.

**"Add a new data table"**
> Create a full-width table inside a card. Header: sticky, Space Mono 11px ALL CAPS `--text-secondary` labels, `1px solid --border-visible` bottom border. Rows: `1px solid --border` divider, 12-16px vertical padding. No zebra striping, no cell backgrounds. Hover: row bg → `--surface-raised`. Sortable columns: `cursor: pointer`, inline Space Mono arrow indicator, active sort column gets `border-bottom: 2px solid --text-display`. Numbers right-aligned Space Mono with `tnum`; text left-aligned Space Grotesk. Run all dynamic text through `esc()`.

**"Add a new chart"**
> Wrap in a card with Space Mono 11px ALL CAPS `--text-secondary` title. Chart container: 240px height (standard) or 300px (hero/daily). ApexCharts options: `background: 'transparent'`, `toolbar: { show: false }`, `fontFamily: 'var(--font-mono)'`, `animations: { enabled: false }`, `legend: { show: false }`. Colors: `--text-display` primary, `--text-display` at 60/30% opacity for secondary series. Use `--accent` for a single over-limit / urgent series only. Grid: horizontal lines in `--border` color. Destroy and recreate (or `updateOptions`) on theme toggle.

**"Add a new filter control"**
> Place inside the filter bar. Group label: Space Mono 11px ALL CAPS `--text-secondary`. Chips: `1px solid --border-visible`, transparent bg, Space Mono 12px ALL CAPS, `4px 12px` padding, pill radius. Active chip: invert -- `--text-display` border + text (no fill change). Range buttons: segmented control with `1px solid --border-visible` container; active segment = `--text-display` background + `--black` text; transition 200ms ease-out. No inset shadows.

**"Add an inline status message"**
> Replace any toast with inline status text near the trigger: Space Mono `--caption` ALL CAPS, format `[SAVED]` (`--success`), `[ERROR: message]` (`--accent`), `[LOADING...]` (`--text-secondary`). 150ms opacity fade in/out. Auto-dismiss after 4 seconds or on next action.

---

## 10. Source of truth

- Skill: `.claude/skills/industrial-design/SKILL.md`
- Tokens: `.claude/skills/industrial-design/references/tokens.md`
- Components: `.claude/skills/industrial-design/references/components.md`
- Platform mapping (Preact + Tailwind v4): `.claude/skills/industrial-design/references/platform-mapping.md`

---

## 11. Active-Period Averaging

**Formula:** `avg_cost_per_active_day = total_cost_nanos / active_days`

**`active_days`** = `COUNT(DISTINCT date)` where `date` is the calendar day (in the client's local timezone) of turns with `estimated_cost_nanos > 0`.

**Example:** If a user worked on 12 of 30 calendar days in a month with a total spend of $24.00, the active-period average is $2.00/day — not $0.80/day (which would result from dividing by 30).

**Display rule:** Show `--` (two dashes) rather than a computed value when `active_days = 0` (no spend in the selected period) to avoid confusing a divide-by-zero edge case with a meaningful zero result.

**Timezone note:** The day boundary respects `tz_offset_min` sent by the client (Phase 14). When absent, days are bucketed in UTC. The `GET /api/heatmap` endpoint computes both `active_days` and `total_cost_nanos` and returns them alongside the 7×24 cell matrix so the StatsCards "Avg / Active Day" card can read them without a separate request.
