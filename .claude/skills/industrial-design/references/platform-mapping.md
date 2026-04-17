# Industrial Design System — Platform Mapping

This system targets web surfaces only. Two implementations are documented: vanilla HTML/CSS (reference) and this repo's actual stack — Preact + Tailwind v4 + Preact signals (practical).

---

## 1. HTML / CSS (reference)

Load fonts via Google Fonts `<link>` or `@import`. Use CSS custom properties, `rem` for type, `px` for spacing/borders. Dark/light via `prefers-color-scheme` or a class/attribute toggle.

```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link
  href="https://fonts.googleapis.com/css2?family=Doto:wght@400..700&family=Space+Grotesk:wght@300;400;500;700&family=Space+Mono:wght@400;700&display=swap"
  rel="stylesheet">
```

```css
:root {
  --black: #000000;
  --surface: #111111;
  --surface-raised: #1A1A1A;
  --border: #222222;
  --border-visible: #333333;
  --text-disabled: #666666;
  --text-secondary: #999999;
  --text-primary: #E8E8E8;
  --text-display: #FFFFFF;
  --accent: #D71921;
  --accent-subtle: rgba(215,25,33,0.15);
  --success: #4A9E5C;
  --warning: #D4A843;
  --interactive: #5B9BF6;
  --space-xs: 4px;
  --space-sm: 8px;
  --space-md: 16px;
  --space-lg: 24px;
  --space-xl: 32px;
  --space-2xl: 48px;
  --space-3xl: 64px;
  --space-4xl: 96px;
}

[data-theme="light"] {
  --black: #F5F5F5;
  --surface: #FFFFFF;
  --surface-raised: #F0F0F0;
  --border: #E8E8E8;
  --border-visible: #CCCCCC;
  --text-disabled: #999999;
  --text-secondary: #666666;
  --text-primary: #1A1A1A;
  --text-display: #000000;
  --interactive: #007AFF;
}

body {
  background: var(--black);
  color: var(--text-primary);
  font-family: "Space Grotesk", "DM Sans", system-ui, sans-serif;
  font-size: 16px;
  line-height: 1.5;
}

.num, .data, .label { font-family: "Space Mono", "JetBrains Mono", monospace; }
.display         { font-family: "Doto", "Space Mono", monospace; }
```

Tabular numerals for any live-updating numeric display:
```css
.num, .stat-value, .chart-label { font-feature-settings: "tnum"; }
```

---

## 2. Preact + Tailwind v4 (this repo)

Stack in `claude-usage-tracker`:
- Preact 10 + `@preact/signals` for reactive state
- Tailwind v4 (CLI) compiled from `src/ui/input.css` → `src/ui/style.css`
- esbuild bundles `src/ui/app.tsx` → `src/ui/app.js` (IIFE, ES2020, not minified)
- Both compiled outputs are committed so `cargo build` works without Node
- Rust embeds the HTML/CSS/JS via `include_str!` in `src/server/assets.rs`

### 2.1 Declare tokens in `src/ui/input.css`

Tailwind v4 uses the `@theme` directive to map CSS custom properties to utilities. Declare the industrial palette there so classes like `bg-[--surface]` and utilities generated from `@theme` both resolve to the same values.

```css
/* src/ui/input.css */
@import "tailwindcss";

@theme {
  --color-black: #000000;
  --color-surface: #111111;
  --color-surface-raised: #1A1A1A;
  --color-border: #222222;
  --color-border-visible: #333333;
  --color-text-disabled: #666666;
  --color-text-secondary: #999999;
  --color-text-primary: #E8E8E8;
  --color-text-display: #FFFFFF;
  --color-accent: #D71921;
  --color-success: #4A9E5C;
  --color-warning: #D4A843;
  --color-interactive: #5B9BF6;

  --font-sans: "Space Grotesk", "DM Sans", system-ui, sans-serif;
  --font-mono: "Space Mono", "JetBrains Mono", monospace;
  --font-display: "Doto", "Space Mono", monospace;
}

/* Light-mode overrides toggled via data-theme on <html> */
[data-theme="light"] {
  --color-black: #F5F5F5;
  --color-surface: #FFFFFF;
  --color-surface-raised: #F0F0F0;
  --color-border: #E8E8E8;
  --color-border-visible: #CCCCCC;
  --color-text-disabled: #999999;
  --color-text-secondary: #666666;
  --color-text-primary: #1A1A1A;
  --color-text-display: #000000;
  --color-interactive: #007AFF;
}

/* Tabular numerals for all numeric displays (auto-refresh friendly) */
.num, [data-numeric] { font-feature-settings: "tnum"; }
```

Rebuild after edits: `npm run build:ui`.

### 2.2 Load fonts in `src/ui/index.html`

Add the three Google Font families to `<head>` (with `preconnect` for speed):

```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link
  href="https://fonts.googleapis.com/css2?family=Doto:wght@400..700&family=Space+Grotesk:wght@300;400;500;700&family=Space+Mono:wght@400;700&display=swap"
  rel="stylesheet">
```

### 2.3 Theme toggle

The existing hook in `src/ui/lib/theme.ts` sets `document.documentElement.dataset.theme`. Keep the `data-theme="light"` attribute convention — the CSS above reacts to it automatically. No JS change needed when migrating palettes; only update tokens in `input.css`.

### 2.4 State (signals)

Use `@preact/signals` for any interactive state — filters, sort columns, active segments, rescan status. Example pattern already in use in `src/ui/state/store.ts`. Avoid React-style class state; prefer signals for their automatic fine-grained reactivity.

### 2.5 XSS protection

Any string rendered from server data MUST pass through `esc()` in `src/ui/lib/format.ts` before being inserted into the DOM. This is a hard rule — JSX's own escaping does not cover raw `dangerouslySetInnerHTML` or template-string HTML.

### 2.6 ApexCharts theming

Charts in this repo use ApexCharts. Thread the industrial palette through via CSS variables (there's already precedent in `src/ui/lib/charts.ts`):

```ts
import ApexCharts from "apexcharts";

const cssVar = (name: string) =>
  getComputedStyle(document.documentElement).getPropertyValue(name).trim();

const industrialTheme: ApexCharts.ApexOptions = {
  chart: {
    background: "transparent",
    toolbar: { show: false },
    fontFamily: 'var(--font-mono), "Space Mono", monospace',
    animations: { enabled: false },
  },
  colors: [cssVar("--color-text-display"), cssVar("--color-accent")],
  grid: { borderColor: cssVar("--color-border"), strokeDashArray: 0 },
  xaxis: { labels: { style: { colors: cssVar("--color-text-secondary") } } },
  yaxis: { labels: { style: { colors: cssVar("--color-text-secondary") } } },
  tooltip: { theme: "dark" },
  legend: { show: false },
  stroke: { width: 1.5, curve: "straight" },
};
```

Rules:
- Monochrome palette: `--text-display` for primary series, `--accent` for the single emphasized series (over-limit, active hover, destructive). Never more than one non-monochrome color per chart.
- Differentiate multiple series with **opacity** (100 / 60 / 30) or **dash pattern** before reaching for additional hues.
- Grid: horizontal lines only, `--border` color.
- No toolbar, no area fill, no shadow, no legend box — label lines directly.
- Destroy and recreate (or call `updateOptions`) on theme toggle so `getComputedStyle` re-reads variables.

### 2.7 Rebuild commands

```bash
npm run build:ts   # src/ui/app.tsx → src/ui/app.js
npm run build:css  # src/ui/input.css → src/ui/style.css
npm run build:ui   # both
```

Commit both `app.js` and `style.css`. The Rust binary inlines them at compile time via `include_str!`.
