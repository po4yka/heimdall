# heimdall design system — Components

All components follow the Apple-Swiss refined register from `SKILL.md` and the tokens in `tokens.md`. Sentence-case labels throughout; ALL-CAPS only on `<th>` column headers.

## 1. CARDS / SURFACES

- Background: `--surface` or `--surface-raised`
- Border: `1px solid --border` or none. Radius: 12–16px cards, 8px compact, 4px technical
- Padding: 16–24px. No shadows. Flat surfaces, border separation.
- **Concentric radii:** inner shapes (buttons, tags, inputs) respect `inner_radius = outer_radius - padding`.

---

## 2. BUTTONS

| Variant | Background | Border | Text | Radius |
|---------|-----------|--------|------|--------|
| Primary | `--accent-interactive` (#4A7FA5) | none | `--text-display` (white) | 8px |
| Secondary | transparent | `1px solid --border-visible` | `--text-primary` | 8px |
| Ghost | transparent | none | `--text-secondary` | 0 |
| Destructive | transparent | `1px solid --accent` (red) | `--accent` (red) | 8px |

All buttons: Inter, 13–14px, weight 500, sentence-case, `0` letter-spacing, padding 10px 16px, min-height 40px.

**Hover:** border/text brightens by one step (secondary → `--text-primary`; primary → subtle opacity 0.9 or background-brighten). No scale, no shadow.

---

## 3. INPUTS

- Underline (`1px solid --border-visible` bottom) or full border (8px radius)
- Label above: `--label` style (Inter 11px sentence-case, `--text-secondary`, 6px below)
- Focus: border → `--accent-interactive`. Error: border → `--accent`, message below in `--accent`
- Data-entry fields for numeric input: Geist Mono, 14px, `font-feature-settings: "tnum"`

---

## 4. LISTS / DATA ROWS

- Dividers: `1px solid --border`, full-width. Row padding: 12–16px vertical
- Left: label (Inter 13–14px, sentence-case, `--text-secondary`). Right: value (`--text-primary`)
- Never alternating row backgrounds. Use dividers.

**Stat rows:** Label left (Inter 12–13px, sentence-case, `--text-secondary`), value right (color = status color when encoding data, else `--text-primary`), unit adjacent in `--caption` size. Trend arrow same color as value.

**Hierarchical rows:** Sub-items indented 16–24px, same divider treatment. No tree lines or expand/collapse — indentation IS the hierarchy.

---

## 5. TABLES / DATA GRIDS

- **Header row (`<th>`): Geist Mono 11px, ALL-CAPS, letter-spacing 0.08em, `--text-secondary`.** This is the sole ALL-CAPS instance permitted in the system — justified by tabular convention.
- Cell text: Inter for prose, Geist Mono for numbers. Cell padding: 12px 16px.
- Numbers right, text left. `font-feature-settings: "tnum"` on numeric cells. No zebra striping, no cell backgrounds.
- Active sort column: `<th>` gets `border-bottom: 2px solid --text-display`. Sort arrow inline, Geist Mono.
- Active row: `--surface-raised` background, left `2px solid --accent-interactive` indicator (interactive selection).

---

## 6. NAVIGATION

- Horizontal text bar desktop, bottom bar mobile
- Labels: Inter 14px, sentence-case. Active: `--text-display` + 2px underline in `--accent-interactive`. Inactive: `--text-secondary`.
- No bracket `[ HOME ]` notation (retired). Plain sentence-case.
- **Back button:** Circular 40–44px, `--surface` bg, thin chevron `<`, top-left 16px from edges

---

## 7. TAGS / CHIPS

- Border: `1px solid --border-visible`, no fill. Text: Inter, 12px, sentence-case.
- Radius: 999px (pill) for filter chips, 4px (technical) for status tags. Padding: 4px 10px. Min-height 28px.
- Active: `--accent-interactive` border + text (inverted treatment optional).

---

## 8. SEGMENTED CONTROL

- Container: `1px solid --border-visible`, 8px rounded
- Active: `--accent-interactive` bg, `--text-display` text. Inactive: transparent, `--text-secondary`
- Text: Inter, 12–13px, sentence-case. Height: 36–44px. Transition: 200ms ease-out
- Max 2–4 segments

---

## 9. DATE / PERIOD NAVIGATION

- Layout: `< Label >` — back arrow, label (sentence-case), forward arrow
- Label: Inter 14px. Arrows: thin chevrons, `--text-secondary`, 44px touch target
- No calendar popovers — linear stepping IS the interaction

---

## 10. TOGGLES / SWITCHES

- Pill track, circle thumb. Off: `--border-visible` track, `--text-disabled` thumb
- On: `--accent-interactive` track, `--text-display` thumb. Min touch target: 44px

---

## 11. PROGRESS BARS

Smooth single-fill pill bars. Color-encoded by threshold. No segments, no LED-meter geometry.

**Anatomy:** Label + value above, single full-width pill bar below with 2px rounded ends.

**Fill colors (threshold logic preserved from prior system):**

| State | Fill | When |
|-------|------|------|
| Good | `--success` | <70% of limit |
| Moderate | `--warning` | 70–90% |
| Over limit | `--accent` (red) | ≥90% or overflow |
| Neutral | `--accent-interactive` | Generic interactive progress (e.g., upload, sync) |

**Empty track:** `--border` (dark) / `#E0E0E0` (light).

**Overflow:** fill continues past 100% in `--accent` red (visually extends beyond track width with same radius).

**Sizes:** Hero 16–20px, Standard 8–12px, Compact 4–6px height.

Always pair with numeric readout. Bar = proportion, number = precision.

---

## 12. DATA VISUALIZATION

- **Bar charts:** vertical, `--text-display` fill, `--border` remainder. 2px rounded top ends.
- **Gauges:** thin stroke circles + tick marks, numeric readout centered/adjacent.
- **Heatmaps:** opacity on `--text-primary`, never a color ramp.
- **Category differentiation:** opacity (100/60/30) → pattern → line style → color (last resort).
- Always show numeric value alongside any visual.

**Charts:** Line 1.5–2px `--text-display`; average dashed 1px `--text-secondary`. Axis labels: Inter or Geist Mono at `--caption` size, sentence-case (or ALL-CAPS only where tabular column naming applies). Grid: `--border`, horizontal only. No area fill, no legend boxes — label lines directly.

---

## 13. WIDGETS (DASHBOARD CARDS)

- `--surface` bg, 12–16px radius. Hero metric: large Inter Semibold, left-aligned
- Unit: `--caption` size, adjacent, sentence-case. Category label: sentence-case at `--caption` top-left
- **No instrument-gauge decorations** (compass, thermometer, dial). Data is the visual.

---

## 14. OVERLAYS & LAYERING

- **Header (sticky top):** optional Liquid Glass treatment — `backdrop-filter: blur(20px)` + `--surface` at 72% opacity. Only the top nav-chrome layer gets translucency; everything else stays flat.
- **Modals:** Backdrop `rgba(0,0,0,0.6)`, dialog `--surface` + `1px solid --border-visible` + 12px radius, centered max 480px. Close button top-right ghost.
- **Bottom sheets:** `--surface`, 2px handle bar centered, 12px top radius, drag-to-dismiss.
- **Dropdowns:** `--surface-raised`, `1px solid --border-visible`, 8px radius, 40px items. Selected: left 2px `--accent-interactive` bar. No shadow.
- **Toasts:** NONE. Use inline bracket status text near the trigger: `[SAVED]`, `[ERROR: ...]`. Inter `--caption`, sentence-case, colored per status.

---

## 15. STATE PATTERNS

- **Error:** Input border → `--accent` + message below. Form-level: summary box `1px solid --accent`. Inline: `[Error]` prefix.
- **Empty:** Centered, 64–96px padding. Headline `--text-secondary` sentence-case, 1 sentence description `--text-disabled`. Optional thin monoline illustration. No mascots. No dot-matrix decorations.
- **Loading:** `[Loading...]` bracket text in `--text-secondary`, or a minimal thin progress bar at the top of the content area. No skeleton screens.
- **Disabled:** Opacity 0.4 or `--text-disabled`. Borders fade to `--border`.
