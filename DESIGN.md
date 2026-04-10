# DESIGN.md -- Claude Usage Tracker

A data-dense analytics dashboard for Claude Code usage tracking. Dark-first, utility-focused, zero decoration. Every pixel earns its place by showing data.

---

## 1. Visual Theme & Atmosphere

| Attribute | Value |
|-----------|-------|
| Mood | Quiet, technical, no-nonsense |
| Density | 7/10 -- data dashboard, not marketing page |
| Motion | 2/10 -- hover transitions only (150ms ease), no scroll or entrance animations |
| Variance | 3/10 -- uniform card grid, left-aligned tables, predictable layout |
| Philosophy | Minimal elevation. No gradients on surfaces, no glow effects. Borders and background steps define structure. Brand-tinted shadows permitted for interactive feedback and overlays. Content density over whitespace. |

**Anti-patterns (never use):**
- Emojis anywhere in rendered output
- Neon glows, 3D effects, glassmorphism, neutral-black shadows (tint with brand color instead)
- Gradient backgrounds on cards or sections
- Centered hero sections or marketing-style layouts
- AI copywriting cliches ("Elevate", "Seamless", "Next-Gen")
- `border-radius` > 12px on containers (no pill-shaped cards)
- Pure black `#000000` or pure white `#FFFFFF` as text colors
- Heavy drop shadows (`shadow-lg` or equivalent)

---

## 2. Color Palette & Roles

### Dark Theme (default)

| Token | Hex | Role |
|-------|-----|------|
| `--bg` | `#0c0c0c` | Page canvas |
| `--bg-secondary` | `#111111` | Filter bar, recessed surfaces |
| `--card` | `#141414` | Card surface |
| `--card-hover` | `#1a1a1a` | Card hover state |
| `--border-subtle` | `rgba(255,255,255,0.03)` | Bento grid gaps, recessed dividers |
| `--border` | `rgba(255,255,255,0.06)` | Card edges, structural dividers (default) |
| `--border-strong` | `rgba(255,255,255,0.12)` | Active filter states, emphasized separators |
| `--border-hover` | `rgba(255,255,255,0.12)` | Interactive border highlight on hover |
| `--border-accent` | `rgba(99,102,241,0.30)` | Active sort column header, focused inputs |
| `--text` | `#ededed` | Primary text (off-white, never pure white) |
| `--text-secondary` | `#888888` | Secondary body text |
| `--muted` | `#666666` | Labels, timestamps, tertiary text |
| `--accent` | `#6366f1` | Brand highlight, interactive elements (indigo) |
| `--accent-muted` | `rgba(99,102,241,0.15)` | Accent backgrounds (active filters, tags) |
| `--blue` | `#3b82f6` | Informational, model distribution charts |
| `--purple` | `#a78bfa` | Output tokens in charts |
| `--green` | `#22c55e` | Cost values, success states (exclusive) |
| `--green-muted` | `rgba(34,197,94,0.15)` | Positive trend badge background |
| `--red` | `#ef4444` | Error states, high-usage warnings |
| `--red-muted` | `rgba(239,68,68,0.15)` | Negative trend badge background |
| `--yellow` | `#eab308` | Cache creation tokens, medium-usage warnings |
| `--hover-bg` | `rgba(255,255,255,0.03)` | Table row hover |
| `--chart-grid` | `rgba(255,255,255,0.04)` | Chart gridlines |
| `--chart-text` | `#666666` | Chart axis labels |
| `--toast-error-bg` | `rgba(239,68,68,0.12)` | Error toast background |
| `--toast-error-text` | `#fca5a5` | Error toast text |
| `--toast-success-bg` | `rgba(34,197,94,0.12)` | Success toast background |
| `--toast-success-text` | `#86efac` | Success toast text |

### Light Theme

| Token | Hex | Role |
|-------|-----|------|
| `--bg` | `#fafafa` | Page canvas |
| `--bg-secondary` | `#f5f5f5` | Filter bar, recessed surfaces |
| `--card` | `#ffffff` | Card surface |
| `--card-hover` | `#f9f9f9` | Card hover state |
| `--border-subtle` | `rgba(0,0,0,0.04)` | Bento grid gaps, recessed dividers |
| `--border` | `rgba(0,0,0,0.07)` | Card edges, structural dividers (default) |
| `--border-strong` | `rgba(0,0,0,0.14)` | Active filter states, emphasized separators |
| `--border-hover` | `rgba(0,0,0,0.14)` | Interactive border highlight on hover |
| `--border-accent` | `rgba(79,70,229,0.25)` | Active sort column header, focused inputs |
| `--text` | `#171717` | Primary text (off-black, never pure black) |
| `--text-secondary` | `#737373` | Secondary body text |
| `--muted` | `#a3a3a3` | Labels, timestamps, tertiary text |
| `--accent` | `#4f46e5` | Brand highlight (deeper indigo) |
| `--accent-muted` | `rgba(79,70,229,0.08)` | Accent backgrounds |
| `--green` | `#16a34a` | Costs, success |
| `--red` | `#dc2626` | Errors, high-usage |
| `--yellow` | `#ca8a04` | Warnings, medium-usage |

### Color Rules
- Never hardcode hex -- always use `var(--token-name)`.
- `--accent` is for brand title, active filters, and interactive highlights only.
- `--green` is exclusively for cost values and success states.
- `--blue`/`--purple` are for chart series and model tags.
- `--muted` is for labels, timestamps, and secondary text.
- Progress bars use semantic thresholds: green (<70%), yellow (70-90%), red (>90%).

### Border Hierarchy (4 tiers)
Use `--border-subtle` for background-level dividers (grid gaps), `--border` for card edges (default), `--border-strong` for active/emphasized states, `--border-accent` for the currently active sort column or focused input. This communicates depth without shadows in dark mode.

### Shadow Policy
- **No neutral shadows.** Never use `rgba(0,0,0,x)` for box-shadow.
- **Brand-tinted shadows only**, for elevation on interactive overlays:
  - Dark: `0 4px 16px rgba(99,102,241,0.08)` (indigo-tinted)
  - Light: `0 4px 16px rgba(79,70,229,0.06)` (indigo-tinted)
- Apply to: toast notifications, future modals/popovers, dropdown menus.
- Cards at rest: no shadow. Depth from border + background color steps only.

### Chart Series Colors

| Series | Value | Use |
|--------|-------|-----|
| Input tokens | `rgba(59,130,246,0.8)` | Blue |
| Output tokens | `rgba(167,139,250,0.8)` | Purple |
| Cache read | `rgba(34,197,94,0.5)` | Green (50% opacity) |
| Cache creation | `rgba(234,179,8,0.5)` | Yellow (50% opacity) |

Model distribution palette: `#6366f1`, `#3b82f6`, `#22c55e`, `#a78bfa`, `#eab308`, `#f472b6`, `#14b8a6`, `#60a5fa`

---

## 3. Typography Rules

| Element | Font | Size | Weight | Color | Extras |
|---------|------|------|--------|-------|--------|
| Page title | Inter, system sans | 14px | 600 | `--text` (with `--accent` on brand word) | `letter-spacing: -0.01em` |
| Section title | Inter, system sans | 11px | 600 | `--muted` | Uppercase, `letter-spacing: 0.08em` |
| Stat card value | JetBrains Mono | 28px | 600 | `--text` or `--green` for costs | `letter-spacing: -0.02em`, `line-height: 1` |
| Stat card label | JetBrains Mono | 10px | 500 | `--muted` | Uppercase, `letter-spacing: 0.08em` (console readout style) |
| Table header | JetBrains Mono | 10px | 500 | `--muted` | Uppercase, `letter-spacing: 0.08em` (console readout style) |
| Table cell | Inter, system sans | 13px | 400 | `--text` | |
| Numbers / costs | JetBrains Mono | 13px | 500 | `--green` or `--text` | `.num` / `.cost` class |
| Model tags | Inter, system sans | 11px | 500 | `--accent` | On `rgba(99,102,241,0.1)` bg, 4px radius |
| Filter labels | JetBrains Mono | 10px | 600 | `--muted` | Uppercase, `letter-spacing: 0.08em` (console readout style) |
| Footer | Inter, system sans | 11px | 400 | `--muted` | Links in `--accent` |
| Chart title | JetBrains Mono | 10px | 600 | `--muted` | Uppercase, `letter-spacing: 0.08em` (console readout style) |
| Toast text | Inter, system sans | 12px | 500 | Semantic toast color | |

**Font stacks:**
- Body: `'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif`
- Mono: `'JetBrains Mono', 'SF Mono', monospace`

**Rules:**
- All numbers, token counts, and cost values use monospace.
- Body text uses `-webkit-font-smoothing: antialiased`.
- No font sizes larger than 28px anywhere in the dashboard.
- **Tabular numerals:** Apply `font-feature-settings: "tnum"` to all numeric displays (stat values, table cells, chart labels, progress percentages). This forces equal-width digits so columns align and live-updating values don't cause layout jitter. Essential for the 30-second auto-refresh cycle.
- **Console readout pattern:** All system labels (stat card labels, table headers, filter labels, chart titles, section titles) use JetBrains Mono at 10px, uppercase, with 0.08em letter-spacing. This creates a unified "terminal readout" aesthetic that reinforces the developer-tool identity. Only body text, page title, model tags, and footer use Inter.

---

## 4. Component Stylings

### Stat Cards
- Background: `var(--card)`, shadow-as-border: `box-shadow: 0 0 0 1px var(--border)`, radius: `12px`
- Padding: `20px`, layout: flexbox with space-between
- Label above value (monospace, console readout style), sub-text below in `--muted` at 11px
- **Interaction-only accent:** At rest, card is neutral. On hover, a faint accent wash appears -- `var(--accent)` at 2-3% opacity as background tint, plus border lightens and subtle `translateY(-1px)`
- Optional sparkline slot: 80x32px, right-aligned, 70% opacity
- Cost values: `.cost-value` class turns text `--green`
- Trend badges: inline pill with `--green-muted`/`--red-muted` bg, 4px radius
- All numeric values: `font-feature-settings: "tnum"` for stable column alignment during auto-refresh

### Cards (generic)
- **Shadow-as-border:** Use `box-shadow: 0 0 0 1px var(--border)` instead of CSS `border`. This avoids box-model side effects (no layout shift on hover, smoother rounded-corner rendering at fractional zoom).
- Radius: `12px`, padding: `20px`
- Background: `var(--card)`, no elevation shadow at rest
- Hover: `box-shadow: 0 0 0 1px var(--border-hover)`, `transform: translateY(-1px)`, 150ms transition
- Flat variant (`.card-flat`): no hover effect, no transform

### Data Tables
- Full width, collapsed borders
- Header: sticky, bottom border only, uppercase monospace labels in `--muted` (console readout style)
- Rows: bottom border `var(--border)`, hover `var(--hover-bg)`
- **Interaction-only accent:** Row hover reveals a warm highlight -- `var(--green)` at 3-5% opacity for cost rows, `var(--accent)` at 3% for others. Color appears only on engagement, never at rest. Adds a subtle "this is interactive" signal.
- Last row: no bottom border
- Sortable columns: `cursor: pointer`, sort arrow indicator (9px, 60% opacity)
- **Active sort column:** Header cell gets `border-bottom: 2px solid var(--border-accent)` to indicate current sort
- Cell padding: `10px 16px`
- Table cards use shadow-as-border (`box-shadow: 0 0 0 1px var(--border)`), zero padding (table fills edge-to-edge), section header gets `20px` padding

### Filter Bar
- Background: `var(--bg-secondary)`, bottom border
- Horizontal flex with `10px` gap, wraps on small screens
- Model checkboxes: pill labels, `4px` radius, `1px` border, 11px text
- Active state: `--accent-muted` bg, `--accent` border, `box-shadow: inset 0 1px 2px rgba(0,0,0,0.15)` (tactile pressed feel)
- Range buttons: connected group with shared `6px` outer radius, `11px` text
- Active range: `--accent-muted` bg, `--accent` text, `600` weight, `box-shadow: inset 0 1px 2px rgba(0,0,0,0.15)`
- Separators: `1px` wide, `20px` tall, `--border` color
- **Inset shadow pattern:** Active/checked controls use a subtle inset shadow to feel physically "pressed into" the surface, making it immediately scannable which filters are engaged vs. idle

### Buttons
- Ghost style: transparent bg, `1px solid var(--border)`, `--text-secondary` color
- Hover: `--border-hover` border, `--text` color
- Disabled: `opacity: 0.4`, `cursor: not-allowed`
- Sizes: filter buttons `2px 8px` padding, action buttons `4px 10px`

### Progress Bars (Rate Windows)
- Track: `var(--border)`, `6px` height, `4px` radius
- Fill: color by threshold -- green (<70%), yellow (70-90%), red (>90%)
- Width transition: `300ms ease`

### Toast Notifications
- Position: fixed, top `56px`, right `16px`, z-index `999`
- **Frosted glass:** `backdrop-filter: blur(12px) saturate(150%)` with semi-transparent semantic background. Underlying dashboard content stays visible-but-blurred, maintaining spatial context.
- Background: semantic `--toast-{type}-bg` (keep existing rgba values -- they're already semi-transparent)
- Text: `--toast-{type}-text`
- Elevation: `box-shadow: 0 4px 16px rgba(99,102,241,0.08)` (brand-tinted)
- Padding: `10px 16px`, radius: `8px`, `1px solid var(--border)`
- Max width: `360px`, auto-dismiss after 6 seconds
- Entry animation: `slideIn 0.2s ease-out` (translateX 20px to 0)

### Pagination
- Flex between page info and prev/next buttons
- `12px` text, `--muted` color
- Top border separator
- Disabled buttons: `opacity: 0.3`

---

## 5. Layout Principles

### Spacing Scale
- `4px` -- inline gaps (icon to text, checkbox to label)
- `8px` -- tight groups (filter pills, toast stack, small-screen grid gap)
- `10px` -- filter bar item gap, table cell padding
- `12px` -- card grid gap, section internal spacing
- `16px` -- section title margin-bottom, toast padding
- `20px` -- card padding, bento grid outer padding
- `24px` -- header/footer horizontal padding, footer top padding

### Grid System
- Bento grid: `max-width: 1400px`, centered, 4 columns
- Column spans: `.bento-2` (2 cols), `.bento-3` (3 cols), `.bento-full` (all)
- Gap: `12px` uniform
- Stats row: `auto-fit` with `minmax(180px, 1fr)` -- fluid column count

### Page Structure
```
Header (sticky, 48px, frosted glass blur)
  Filter Bar (sticky below header)
    Bento Grid (main content)
      Rate Windows (conditional, full width)
      Stats Cards (fluid row, 7 cards)
      Charts Row (2-col daily + 1-col model + 1-col projects)
      Data Sections (full width, conditional)
      Tables (full width)
    Footer
```

### Whitespace Philosophy
- Let data breathe but don't waste space. 12px grid gap is intentionally tight.
- Cards are the primary spatial unit -- no free-floating text outside cards.
- Header is compact (48px) because it's always visible (sticky).

---

## 6. Depth & Elevation

| Surface | Treatment |
|---------|-----------|
| Page canvas | `var(--bg)` -- deepest layer |
| Filter bar | `var(--bg-secondary)` -- slightly elevated. Apply `backdrop-filter: blur(12px) saturate(150%)` if sticky. |
| Header | `var(--bg)` at 80% opacity + `backdrop-filter: blur(12px)` -- floats above content |
| Cards | `var(--card)` + `box-shadow: 0 0 0 1px var(--border)` -- shadow-as-border, no elevation shadow |
| Card hover | Shadow-border lightens to `--border-hover`, 1px upward translate |
| Tables | Inside cards, no additional elevation |
| Toasts | Frosted glass (`backdrop-filter: blur(12px) saturate(150%)`), brand-tinted shadow `0 4px 16px rgba(99,102,241,0.08)` |
| Active controls | `box-shadow: inset 0 1px 2px rgba(0,0,0,0.15)` -- tactile pressed depth |
| Sticky thead | `z-index: 1`, card background |

### Shadow-as-Border Pattern
Use `box-shadow: 0 0 0 1px var(--border)` instead of CSS `border: 1px solid`. Benefits:
- No layout shift when animating border color on hover
- Smoother rounded-corner rendering at fractional zoom levels
- Border lives in the shadow layer, independent of the box model
- Apply to: all cards, table wrappers, and interactive containers

### Brand-Tinted Elevation
When elements need to float above the surface (toasts, modals, popovers), use indigo-tinted shadows:
- Dark: `0 4px 16px rgba(99,102,241,0.08)`
- Light: `0 4px 16px rgba(79,70,229,0.06)`
Never use neutral `rgba(0,0,0,x)` for elevation shadows -- always tint with the brand color.

### Frosted Glass
Apply `backdrop-filter: blur(12px) saturate(150%)` with semi-transparent backgrounds to overlays that sit above dashboard content. This keeps underlying data visible-but-blurred for spatial context. Use on:
- Header (already implemented)
- Toast notifications
- Filter bar (if sticky-scrolling)
- Future: modals, dropdown menus, tooltips

---

## 7. Do's and Don'ts

### Do
- Use CSS variables for every color value
- Put numbers and costs in monospace font with `font-feature-settings: "tnum"` for tabular alignment
- Use monospace (JetBrains Mono) for all system labels -- stat labels, table headers, filter labels, chart titles
- Use `box-shadow: 0 0 0 1px` instead of CSS `border` on cards and containers
- Use `box-shadow: inset 0 1px 2px` on active/checked filter controls for tactile depth
- Tint all elevation shadows with brand indigo -- never use neutral black shadows
- Apply `backdrop-filter: blur(12px) saturate(150%)` on overlays (toasts, sticky bars)
- Make tables sortable with clear sort indicators and `--border-accent` on active sort column
- Support both light and dark themes (toggle via `data-theme` attribute)
- Keep chart heights consistent: 240px standard, 300px for daily chart
- Run all dynamic text through XSS escaping before rendering
- Use `auto-fit` / `minmax` for fluid grid layouts
- Format large numbers with abbreviations (1.5M, 2.3K)
- Reserve color for interaction -- let resting UI stay neutral, reveal accent/green on hover

### Don't
- Add decorative elements that don't display data
- Use more than 2 font families (Inter + JetBrains Mono)
- Use neutral `rgba(0,0,0,x)` for box-shadow elevation (tint with brand color)
- Use CSS `border` on cards when `box-shadow: 0 0 0 1px` achieves the same visual
- Create custom scrollbars or override native scroll behavior
- Add entrance animations or scroll-triggered effects
- Use icon libraries -- inline SVG only where needed (theme toggle)
- Make the header taller than 48px
- Add tooltips unless showing precise values on chart hover
- Use `z-index` values above 999
- Apply the same border token to all hierarchy levels -- use the 4-tier border scale

---

## 8. Responsive Behavior

| Breakpoint | Grid | Adjustments |
|------------|------|-------------|
| > 1024px | 4 columns | Full layout, all features visible |
| 768-1024px | 2 columns | `.bento-3` collapses to 2-col span |
| < 768px | 1 column | Single column stack, padding shrinks to 12px, gap to 8px |
| < 480px | 1 column | Stat values shrink to 22px, sparklines hidden |

### Responsive Rules
- Filter bar wraps naturally via `flex-wrap: wrap`
- Tables scroll horizontally via `overflow-x: auto` on the card wrapper
- Charts resize within their card containers (ApexCharts handles responsively)
- Header padding shrinks from 24px to 12px below 768px
- Grid gap tightens from 12px to 8px below 768px
- No content is hidden on mobile except sparklines at 480px

---

## 9. Agent Prompt Guide

### Quick Reference
```
Background:    #0c0c0c (dark) / #fafafa (light)
Cards:         #141414 (dark) / #ffffff (light)
Borders:       rgba(255,255,255,0.06) (dark) / rgba(0,0,0,0.07) (light)
Primary text:  #ededed (dark) / #171717 (light)
Accent:        #6366f1 (dark) / #4f46e5 (light) -- indigo
Costs:         #22c55e (dark) / #16a34a (light) -- green, monospace only
Errors:        #ef4444 (dark) / #dc2626 (light)
Fonts:         Inter (body), JetBrains Mono (numbers)
Card radius:   12px
Grid gap:      12px
Max width:     1400px
```

### Ready-to-Use Prompts

**"Add a new stat card"**
> Create a stat card matching the existing pattern: `.card.stat-card` container, `.stat-label` (11px uppercase muted), `.stat-value` (28px JetBrains Mono), `.stat-sub` (11px muted). Use `--green` color and `.cost-value` class if showing a cost. Place it inside the `#stats-row` grid.

**"Add a new data table"**
> Create a full-width table section: `.card.card-flat.bento-full.table-card` wrapper, `.section-header` with `.section-title` (11px uppercase muted) and optional `.export-btn`. Table uses sticky headers, `--muted` uppercase th labels, `10px 16px` cell padding, row hover `var(--hover-bg)`. Numbers get `.num` class, costs get `.cost` class.

**"Add a new chart"**
> Wrap in `.card.chart-card` with `h2` title (11px uppercase muted). Chart container: `.chart-wrap` (240px height) or `.chart-wrap.tall` (300px). Use ApexCharts with `background: 'transparent'`, `toolbar: false`, `fontFamily: 'inherit'`, grid color `cssVar('--chart-grid')`. Destroy and recreate on theme toggle.

**"Add a new filter control"**
> Place inside `#filter-bar`. Use `.filter-label` for the group label. Buttons use `.filter-btn` (10px, ghost style) or `.range-btn` inside `.range-group` for connected toggles. Active state: `--accent-muted` bg, `--accent` color.
