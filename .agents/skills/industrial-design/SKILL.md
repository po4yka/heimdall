---
name: industrial-design
description: This skill should be used when the user explicitly says "industrial style", "industrial design", "/industrial-design", or directly asks to use/apply the heimdall design system. NEVER trigger automatically for generic UI or design tasks. The directory name is retained for historical reasons; the current design system is Apple-Swiss refined, not Nothing-industrial.
version: 2.0.0
allowed-tools: [Read, Write, Edit, Glob, Grep]
---

# heimdall design system — Apple-Swiss refined

A senior product designer's toolkit trained in Swiss International Typographic rigor (Müller-Brockmann grid discipline, Akzidenz-Grotesk → Helvetica → Inter lineage, hierarchy through size and space) blended with Apple's 2026 system craft (SF Pro optical sizes, concentric corner radii, Liquid Glass navigation-layer chrome, semantic color). Data-dense without cosplaying as a cockpit.

Both dark and light mode are first-class. The product is a long-session analytics dashboard users live in for hours — the aesthetic must be calm enough to sustain that.

**Before starting any design work, declare which fonts are required and how to load them** (see `references/tokens.md` §1). The web stack uses Inter + Geist Mono; the SwiftUI stack uses the SF Pro system cascade. Never assume fonts are already available.

---

## 1. DESIGN PHILOSOPHY

- **Calm precision.** Every element earns its pixel; removal is the default. Density is permitted, aggression is not.
- **Structure through space, not decoration.** Grid discipline, type-weight differentiation, and whitespace communicate hierarchy. Borders, background tints, and color are the last resorts, not the first.
- **Monochrome canvas, semantic color.** The surface is grayscale. Color appears when it encodes meaning — blue-gray for interactive affordances, green for healthy, amber for caution, red for error/destructive. Never decorative.
- **Type does the hierarchy.** Size, weight, and space create three layers of importance. Color, borders, and icons are not primary hierarchy tools.
- **Sentence case.** ALL-CAPS monospaced labels are reserved for `<th>` table column headers only, where tabular convention justifies them. Everywhere else the label is sentence-case at 11–12px in `--text-secondary`.
- **Concentric radii.** Nested shapes share a conceptual center. `inner_radius = outer_radius - padding`. Independently chosen radii produce "pinched" or "flared" perceptual failures.
- **Flat content, navigable chrome.** Content surfaces are flat with 1px border separation. Liquid Glass translucency is acceptable only on the sticky top header (the one navigation-layer element). Data panels, tables, cards, charts do not use blur, glass, or shadow.

---

## 2. CRAFT RULES — HOW TO COMPOSE

### 2.1 Visual Hierarchy: The Three-Layer Rule

Every screen has exactly **three layers of importance.** Not two, not five. Three.

| Layer | What | How |
|-------|------|-----|
| **Primary** | The ONE thing the user sees first. A number, a headline, a state. | Inter Display at 40–48px; 600 weight for hero numbers, 500 for headlines. `--text-display`. 48–96px breathing room. |
| **Secondary** | Supporting context. Labels, descriptions, related data. | Inter Text at body/subheading (14–16px). `--text-primary`. Grouped tight (8–16px) to the primary. |
| **Tertiary** | Metadata, navigation, system info. Visible but never competing. | Inter Text at 11–12px sentence-case. `--text-secondary` or `--text-disabled`. Pushed to edges or bottom. Exception: `<th>` column headers use Geist Mono ALL-CAPS 11px at 0.08em tracking. |

**The test:** Squint at the screen. Can you still tell what's most important? If two things compete, one needs to shrink, fade, or move.

**Common mistake:** Making everything "secondary." Evenly-sized elements with even spacing produce visual flatness. Be brave — the primary is absurdly large; the tertiary is absurdly small. The contrast IS the hierarchy.

### 2.2 Font Discipline

Per screen, use maximum:
- **2 font families** (Inter + Geist Mono on web; SF Pro Text + SF Pro Display + SF Mono via the system stack on SwiftUI).
- **3 font sizes** (one large, one medium, one small).
- **2 font weights** (Regular + one other — usually Medium or Semibold, rarely Bold).

Think of it as a budget. Every additional size/weight costs visual coherence. Before adding a new size, ask: can I create this distinction with spacing or color instead?

| Decision | Size | Weight | Color |
|----------|:---:|:---:|:---:|
| Heading vs. body | Yes | No | No |
| Label vs. value | No | No | Yes |
| Active vs. inactive nav | No | No | Yes |
| Hero number vs. unit | Yes | No | No |
| Section title vs. content | Yes | Optional | No |

**Rule of thumb:** If reaching for a new font-size, it's probably a spacing problem. Add distance instead.

### 2.3 Spacing as Meaning

Spacing is the primary tool for communicating relationships. 4px base unit — all gaps/paddings/margins are multiples.

```
Tight (4–8px)   = "These belong together" (icon + label, number + unit)
Medium (16px)    = "Same group, different items" (list items, form fields)
Wide (24–32px)   = "New group starts here" (section breaks)
Vast (48–64px)   = "This is a new context" (hero to content, major divisions)
```

**If a divider line is needed, the spacing is probably wrong.** Dividers are a symptom of insufficient spacing contrast. Use them only in data-dense lists where items are structurally identical.

### 2.4 Container Strategy (prefer top)

1. **Spacing alone** (proximity groups items)
2. A single divider line
3. A subtle border outline
4. A surface card with background change

Each step down adds visual weight. Use the lightest tool that works. Never box the most important element — let it float on the background.

### 2.5 Color as Hierarchy

In a monochrome system, the gray scale IS the hierarchy. Max 4 levels per screen:

```
--text-display (100%) → Hero numbers. One per screen.
--text-primary (90%)  → Body text, primary content.
--text-secondary (60%) → Labels, captions, metadata.
--text-disabled (40%) → Disabled, timestamps, hints.
```

**Accent system:**

- **`--accent-interactive` (blue-gray `#4A7FA5`)** — the primary interactive affordance. Selected states, links, tappable text, primary buttons. Same value in light + dark mode.
- **`--accent` (red `#D71921`)** — semantic error / destructive / over-limit only. Not "the urgent thing on screen"; only "this is broken."
- **Status colors (`--success` green, `--warning` amber)** are exempt from the "one accent" rule when encoding data values. Apply color to the **value itself**, not labels or row backgrounds.

See `references/tokens.md` for the full color system.

### 2.6 Consistency vs. Variance

**Be consistent in:** Font families, label treatment (sentence-case everywhere except `<th>`), spacing rhythm, color roles, concentric radii, alignment.

**Break the pattern once per screen** for emphasis: an oversized number, a single colored element in a zone of grayscale, a rare use of italic. One break is emphasis; two breaks is chaos.

### 2.7 Compositional Balance

**Asymmetry > symmetry.** Centered layouts feel generic. Favor deliberately unbalanced composition:
- **Large left, small right:** Hero metric + metadata stack.
- **Top-heavy:** Big headline near top, sparse content below.
- **Edge-anchored:** Important elements pinned to screen edges, negative space in center.

Balance heavy elements with more empty space, not with more heavy elements.

### 2.8 The Refined-Tool Register

1. **Confidence through emptiness.** Large uninterrupted background areas. Resist filling space.
2. **Precision in the small things.** Letter-spacing, exact gray values, 4px gaps. Micro-decisions compound into craft.
3. **Data as presence.** A `36GB/s` number at 48px in Inter Semibold IS the visual. No illustrations needed.
4. **Mechanical honesty.** Controls look like controls. A toggle is a toggle; a button is a button. No skeuomorphic knobs, no LED meters, no gauge faces.
5. **One moment of emphasis.** A large number, an unexpected element, a single blue-gray accent. Restraint makes the one expressive moment powerful.
6. **Motion is informational, not decorative.** Opacity fades, concentric-radius transitions, `cubic-bezier(0.25, 0.1, 0.25, 1)` ease-out. No spring, no bounce, no parallax.

### 2.9 Visual Variety in Data-Dense Screens

When 3+ data sections appear on one screen, vary the visual form:

| Form | Best for | Weight |
|------|----------|--------|
| Hero number (large Inter Semibold) | Single key metric | Heavy — use once |
| Smooth progress bar with threshold color | Progress toward goal | Medium |
| Concentric rings / arcs | Multiple related percentages | Medium |
| Inline compact bar | Secondary metrics in rows | Light |
| Number-only with status color | Values without proportion | Lightest |
| Sparkline | Trends over time | Medium |
| Stat row (label + value) | Simple data points | Light |

Lead section → heaviest treatment. Secondary → different form. Tertiary → lightest. The FORM varies, the VOICE stays the same.

---

## 3. ANTI-PATTERNS — WHAT TO NEVER DO

- No gradients in UI chrome.
- No shadows on content surfaces. No blur on content surfaces. Flat; border separation. Liquid Glass acceptable only on the sticky top header (navigation-layer chrome).
- No skeleton loading screens. Use `[LOADING...]` bracket text or a minimal spinner.
- No toast popups. Use inline status text near the trigger: `[SAVED]`, `[ERROR: ...]`.
- No sad-face illustrations, cute mascots, or multi-paragraph empty states.
- No zebra striping in tables.
- No filled multi-color icons. Monoline at 1.5px stroke, single color.
- No parallax, scroll-jacking, or gratuitous animation.
- No spring / bounce easing. Use subtle ease-out only.
- No border-radius > 16px on cards. Buttons are pill (999px) or technical (4–8px). Respect concentric radii when nesting.
- No ALL-CAPS monospace outside `<th>` column headers. Sentence-case throughout.
- No dot-matrix display type (Doto or similar). No LED-meter segmented progress bars. No dot-grid backgrounds.
- No pure `#000000` canvas on dark mode (reads as OLED panel rather than designed surface). Use `#0A0A0A`.
- No red `--accent` as the primary interactive affordance — red is reserved for error/destructive semantics.
- Data visualization: differentiate with **opacity** (100%/60%/30%) or **pattern** (solid/striped/dotted) before introducing color.

---

## 4. WORKFLOW

1. **Declare fonts** — tell the user which fonts to load (see `references/tokens.md` §1).
2. **Ask mode** — dark or light? Neither is default.
3. **Sketch hierarchy** — identify the 3 layers before writing any code.
4. **Compose** — apply craft rules (Sections 2.1–2.9).
5. **Check tokens** — consult `references/tokens.md` for exact values.
6. **Build components** — consult `references/components.md` for patterns.
7. **Adapt to platform** — consult `references/platform-mapping.md` for output conventions.

---

## 5. REFERENCE FILES

For detailed token values, component specs, and platform-specific guidance:

- **`references/tokens.md`** — Fonts, type scale, color system (dark + light), spacing scale, grid, motion, iconography
- **`references/components.md`** — Cards, buttons, inputs, lists, tables, nav, tags, segmented controls, progress bars, charts, widgets, overlays, state patterns
- **`references/platform-mapping.md`** — HTML/CSS and Preact + Tailwind v4 (this repo's stack) output conventions
