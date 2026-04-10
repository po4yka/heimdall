---
name: dashboard-design
description: Premium data dashboard design rules for Codex-usage-tracker. Enforces editorial typography, warm monochrome palette, proper data density, Chart.js theming, and anti-AI-slop patterns. Apply when editing HTML, CSS, or TypeScript UI files.
---

# Protocol: Premium Analytics Dashboard Design

## 1. Scope

Apply these rules when editing:
- `src/ui/index.html` -- dashboard HTML structure
- `src/ui/input.css` -- Tailwind v4 entry (source)
- `src/ui/style.css` -- generated CSS (committed)
- `src/ui/app.tsx` -- entry point, data loading, filter logic
- `src/ui/components/*.tsx` -- Preact JSX components
- `src/ui/state/store.ts` -- Preact signals state
- `src/ui/lib/*.ts` -- shared utilities

## 2. Design Parameters

### VISUAL_DENSITY: 7/10 (Data Dashboard)
- Stats cards: compact but readable, 16px padding
- Tables: 10-12px cell padding, 13px font
- Charts: 240-300px height, clear labels
- Sections: 16-24px gaps between cards/sections

### MOTION_INTENSITY: 2/10 (Utility-First)
- Transitions on hover states only: 150ms ease
- No scroll animations, no entrance animations
- Progress bars: 300ms width transition for smooth updates
- Auto-refresh data: no visual flash on update

### DESIGN_VARIANCE: 3/10 (Clean & Predictable)
- Consistent card grid layout
- Left-aligned tables, no creative layouts
- Standard header/filter-bar/content/footer structure

## 3. Absolute Bans (Anti-AI-Slop)

### Never Use
- Emojis anywhere in HTML, CSS, JS, or rendered text
- Inter, Roboto, or Open Sans fonts
- Pure black `#000000` or pure white `#FFFFFF` as text colors
- Neon glows, glassmorphism, 3D effects
- `shadow-lg` or heavy drop shadows -- max `0 1px 3px rgba(0,0,0,0.08)`
- Gradient backgrounds on cards or sections
- `border-radius` > 12px on cards (no pill-shaped containers)
- Placeholder names like "John Doe" or fake data like "99.99%"
- AI copywriting cliches: "Elevate", "Seamless", "Next-Gen"
- Centered hero sections or marketing-style layouts
- `h-screen` -- use `min-height: 100dvh` if needed

### Always Use
- System font stack: `-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif`
- Monospace for numbers, costs, token counts: `'SF Mono', 'Geist Mono', 'JetBrains Mono', monospace`
- Off-black text in light mode: `#1a1d27` (not #000)
- Off-white text in dark mode: `#e2e8f0` (not #fff)
- `box-sizing: border-box` universally
- CSS variables for all colors (support light + dark themes)

## 4. Color System (CSS Variables)

### Light Theme (default)
```css
--bg: #f5f6f8;           /* Canvas */
--card: #ffffff;          /* Card surface */
--border: #e2e5ea;        /* Structural dividers */
--text: #1a1d27;          /* Primary text (off-black) */
--muted: #64748b;         /* Secondary text */
--accent: #c4623e;        /* Brand accent (warm terracotta) */
--blue: #3b7ae0;          /* Info/model tags */
--green: #16a34a;         /* Cost/success */
--hover-bg: rgba(0,0,0,0.03);
--chart-grid: #e2e5ea;
--chart-text: #64748b;
--chart-border: #ffffff;
--toast-error-bg: #fef2f2;
--toast-error-text: #dc2626;
--toast-success-bg: #f0fdf4;
--toast-success-text: #16a34a;
```

### Dark Theme
```css
--bg: #0f1117;
--card: #1a1d27;
--border: #2a2d3a;
--text: #e2e8f0;
--muted: #8892a4;
--accent: #d97757;
--blue: #4f8ef7;
--green: #4ade80;
--hover-bg: rgba(255,255,255,0.02);
--chart-grid: #2a2d3a;
--chart-text: #8892a4;
--chart-border: #1a1d27;
--toast-error-bg: #7f1d1d;
--toast-error-text: #fca5a5;
--toast-success-bg: #14532d;
--toast-success-text: #86efac;
```

### Rules
- Never hardcode hex colors -- always use `var(--name)`
- Accent color is for the brand/title and interactive highlights only
- Green is exclusively for cost values and success states
- Blue is for model tags and informational badges
- Muted is for labels, timestamps, secondary text

## 5. Typography Hierarchy

| Element | Font | Size | Weight | Color |
|---------|------|------|--------|-------|
| Page title | System sans | 18px | 600 | `--accent` |
| Section titles | System sans | 13px | 600 | `--muted` (uppercase, tracking 0.05em) |
| Stat card value | System sans | 22px | 700 | `--text` |
| Stat card label | System sans | 11px | 400 | `--muted` (uppercase) |
| Table header | System sans | 11px | 400 | `--muted` (uppercase, tracking 0.05em) |
| Table cell | System sans | 13px | 400 | `--text` |
| Numbers/costs | Monospace | 13px | 400 | `--green` or `--text` |
| Model tags | System sans | 11px | 400 | `--blue` on `rgba(blue, 0.15)` |
| Footer text | System sans | 12px | 400 | `--muted` |
| Filter labels | System sans | 11px | 600 | `--muted` (uppercase, tracking 0.05em) |

## 6. Component Patterns

### Stat Cards
- Border: `1px solid var(--border)`
- Radius: `8px`
- Padding: `16px`
- No shadow (flat design)
- Label above value, sub-text below

### Data Tables
- Full-width, collapsed borders
- Header: bottom border only, uppercase labels
- Rows: bottom border, hover highlight `var(--hover-bg)`
- Last row: no bottom border
- Sortable columns: cursor pointer, sort arrow indicator

### Progress Bars (Rate Windows)
- Track: `var(--border)`, 6px height, 4px radius
- Fill: color by threshold (green <70%, yellow 70-90%, red >90%)
- Width transition: `300ms ease`

### Filter Controls
- Pill-shaped checkboxes: `border-radius: 20px`, 1px border
- Active state: accent background at 12% opacity
- Grouped range buttons: connected with shared border-radius

### Toast Notifications
- Position: fixed top-right, 16px from edges
- Error: `var(--toast-error-bg)` + `var(--toast-error-text)`
- Success: `var(--toast-success-bg)` + `var(--toast-success-text)`
- Radius: 8px, auto-dismiss after 6s
- No shadow in dark mode, subtle shadow in light mode

### Pagination
- Muted page info text (12px)
- Filter-btn style for prev/next buttons
- Disabled state: opacity 0.5, cursor not-allowed

### CSV Export Buttons
- Small, muted style matching filter-btn
- Hover: accent border highlight
- No icon -- text only with download arrow character

## 7. ApexCharts Theming

### All Charts
- Library: ApexCharts 4 (loaded via CDN, renders into `<div>` elements)
- `theme: { mode: apexThemeMode() }` -- reads light/dark from `data-theme` attribute
- `chart.background: 'transparent'` -- let CSS card handle background
- `chart.toolbar: { show: false }` -- clean dashboard look
- `chart.fontFamily: 'inherit'` -- use system font stack
- `dataLabels: { enabled: false }` -- no labels on chart elements
- `grid.borderColor: cssVar('--chart-grid')` -- theme-aware grid lines
- Destroy and recreate charts on theme toggle (ApexCharts doesn't hot-swap themes)

### Stacked Bar Charts (Daily Usage)
- `chart.stacked: true` with 4 series (Input, Output, Cache Read, Cache Creation)
- Colors: `TOKEN_COLORS` (input=blue, output=purple, cache_read=green, cache_creation=yellow)
- `plotOptions.bar.columnWidth: '70%'`
- Y-axis: formatted with `fmt()` (1.5M, 2.3K)
- Legend: top position

### Donut Charts (Model Distribution)
- `chart.type: 'donut'`
- Stroke between segments: `cssVar('--card')` color, 2px width
- `plotOptions.pie.donut.size: '60%'`
- Legend at bottom
- Tooltip: show token count formatted

### Horizontal Bar Charts (Projects)
- `plotOptions.bar.horizontal: true, barHeight: '60%'`
- Truncate long labels with ellipsis prefix (> 22 chars) via `xaxis.categories`
- Two series: Input + Output (not stacked)

### Sparklines
- `chart.sparkline: { enabled: true }` -- native ApexCharts sparkline
- `stroke.curve: 'smooth'`, width 1.5
- Color: `cssVar('--accent')`
- Tooltip disabled, 120x30px container

## 8. Execution Protocol

When modifying the dashboard UI:
1. Check that all colors use CSS variables (never raw hex in HTML or JS template literals)
2. Ensure both light and dark themes work (toggle and verify)
3. Tables must have hover states and sort indicators
4. Numbers and token counts must use monospace font
5. New sections need proper section-title styling (uppercase, muted, letter-spacing)
6. After CSS changes: verify no hardcoded colors leaked into app.ts
7. After TS changes: run `npm run build:ui`, rebuild Rust binary
8. New UI elements must be responsive (collapse to single column below 768px)
9. XSS protection: all dynamic text must pass through `esc()` function
10. Progress bars must use semantic colors: green (<70%), yellow (70-90%), red (>90%)
