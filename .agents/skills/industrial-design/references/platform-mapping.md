# heimdall design system — Platform Mapping

This system targets web surfaces and SwiftUI (Heimdall). Three implementations are documented: vanilla HTML/CSS (reference), this repo's Preact + Tailwind v4 stack (practical), and the SwiftUI system-font approach (Heimdall).

---

## 1. HTML / CSS (reference)

Load fonts via Google Fonts `<link>` (or self-host). Use CSS custom properties, `rem` for type, `px` for spacing/borders. Dark/light via `data-theme` attribute or `prefers-color-scheme`.

```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link
  href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=Geist+Mono:wght@400;500;700&display=swap"
  rel="stylesheet">
```

```css
:root {
  /* Dark canvas (default) */
  --black: #0A0A0A;
  --surface: #111111;
  --surface-raised: #1A1A1A;
  --border: #222222;
  --border-visible: #333333;
  --text-disabled: #666666;
  --text-secondary: #999999;
  --text-primary: #E8E8E8;
  --text-display: #FFFFFF;

  /* Accents */
  --accent-interactive: #4A7FA5;
  --accent: #D71921;
  --accent-subtle: rgba(215,25,33,0.15);
  --success: #4A9E5C;
  --warning: #D4A843;

  /* Spacing */
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
  --text-disabled: #707070;
  --text-secondary: #4F4F4F;
  --text-primary: #1A1A1A;
  --text-display: #000000;
  /* accent-interactive stays #4A7FA5 in both modes */
}

body {
  background: var(--black);
  color: var(--text-primary);
  font-family: "Inter", system-ui, sans-serif;
  font-size: 15px;
  line-height: 1.5;
}

.num, .data {
  font-family: "Geist Mono", ui-monospace, "SF Mono", monospace;
}

/* Tabular numerals for live-updating numeric displays */
.num, .stat-value, .chart-label { font-feature-settings: "tnum"; }

/* Column headers — the sole ALL-CAPS instance */
th {
  font-family: "Geist Mono", ui-monospace, monospace;
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--text-secondary);
}
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

Tailwind v4 uses the `@theme` directive to map CSS custom properties to utilities. Declare the palette there so classes like `bg-[--surface]` and utilities generated from `@theme` both resolve to the same values.

```css
/* src/ui/input.css */
@import "tailwindcss";

@theme {
  --color-black: #0A0A0A;
  --color-surface: #111111;
  --color-surface-raised: #1A1A1A;
  --color-border: #222222;
  --color-border-visible: #333333;
  --color-text-disabled: #666666;
  --color-text-secondary: #999999;
  --color-text-primary: #E8E8E8;
  --color-text-display: #FFFFFF;
  --color-accent-interactive: #4A7FA5;
  --color-accent: #D71921;
  --color-success: #4A9E5C;
  --color-warning: #D4A843;

  --font-sans: "Inter", system-ui, sans-serif;
  --font-mono: "Geist Mono", ui-monospace, "SF Mono", monospace;
}

/* Light-mode overrides toggled via data-theme on <html> */
[data-theme="light"] {
  --color-black: #F5F5F5;
  --color-surface: #FFFFFF;
  --color-surface-raised: #F0F0F0;
  --color-border: #E8E8E8;
  --color-border-visible: #CCCCCC;
  --color-text-disabled: #707070;
  --color-text-secondary: #4F4F4F;
  --color-text-primary: #1A1A1A;
  --color-text-display: #000000;
  /* accent-interactive same in both modes */
}

/* Tabular numerals for all numeric displays */
.num, [data-numeric] { font-feature-settings: "tnum"; }

/* Column headers — the sole ALL-CAPS instance */
th {
  font-family: var(--font-mono);
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}
```

Rebuild after edits: `npm run build:ui`.

### 2.2 Load fonts in `src/ui/index.html`

Add Inter and Geist Mono to `<head>`:

```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link
  href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=Geist+Mono:wght@400;500;700&display=swap"
  rel="stylesheet">
```

### 2.3 Theme toggle

The existing hook in `src/ui/lib/theme.ts` sets `document.documentElement.dataset.theme`. Keep the `data-theme="light"` attribute convention — the CSS above reacts to it automatically. No JS change needed when migrating palettes; only update tokens in `input.css`.

### 2.4 State (signals)

Use `@preact/signals` for any interactive state — filters, sort columns, active segments, rescan status. Example pattern already in use in `src/ui/state/store.ts`. Avoid React-style class state; prefer signals for their automatic fine-grained reactivity.

### 2.5 XSS protection

Any string rendered from server data MUST pass through `esc()` in `src/ui/lib/format.ts` before being inserted into the DOM. This is a hard rule — JSX's own escaping does not cover raw `dangerouslySetInnerHTML` or template-string HTML.

### 2.6 ApexCharts theming

Charts in this repo use ApexCharts. The factory in `src/ui/lib/charts.ts` is `dashboardChartOptions` (renamed from the prior `industrialChartOptions`). It threads the palette through via CSS variables:

```ts
import ApexCharts from "apexcharts";

const cssVar = (name: string) =>
  getComputedStyle(document.documentElement).getPropertyValue(name).trim();

const dashboardTheme: ApexCharts.ApexOptions = {
  chart: {
    background: "transparent",
    toolbar: { show: false },
    fontFamily: 'var(--font-mono), "Geist Mono", ui-monospace, monospace',
    animations: { enabled: false },
  },
  colors: [cssVar("--color-text-display"), cssVar("--color-accent-interactive")],
  grid: { borderColor: cssVar("--color-border"), strokeDashArray: 0 },
  xaxis: { labels: { style: { colors: cssVar("--color-text-secondary") } } },
  yaxis: { labels: { style: { colors: cssVar("--color-text-secondary") } } },
  tooltip: { theme: "dark" },
  legend: { show: false },
  stroke: { width: 1.5, curve: "straight" },
};
```

Rules:
- Monochrome-first: `--text-display` for primary series. `--accent-interactive` for the secondary highlighted series (selected item, active filter, primary emphasis). `--accent` (red) only for semantic error/over-limit series. Never more than two non-monochrome colors per chart.
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

---

## 3. SwiftUI (Heimdall)

Heimdall is a SwiftUI menu-bar app; it uses Apple's system font stack natively. No font loading required.

### 3.1 Typography modifiers

```swift
// Hero numbers (auto-selects SF Pro Display above 20pt)
Text(usageValue).font(.largeTitle).fontWeight(.semibold)

// Section heading
Text("Usage").font(.title3).fontWeight(.medium)

// Body
Text(label).font(.body)

// Numeric columns (tabular figures)
Text(cost, format: .currency(code: "USD"))
  .monospacedDigit()

// Code / paths
Text(path).font(.system(.body, design: .monospaced))
```

### 3.2 Colors

Use SwiftUI semantic colors — they adapt automatically to the system appearance (light/dark) and respect user accessibility settings:

```swift
.foregroundStyle(.primary)        // label text
.foregroundStyle(.secondary)      // subtitle
.foregroundStyle(Color.accentColor)  // interactive
.foregroundStyle(.red)            // destructive only
```

Do **not** hardcode hex colors in Swift views. The web tokens and the SwiftUI semantic colors intentionally diverge — let each platform use its native vocabulary.

### 3.3 Liquid Glass

On iOS/macOS 26+, the `.glassEffect()` modifier is available for navigation-chrome elements. Do not apply to content surfaces.
