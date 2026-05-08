---
name: heimdall-dashboard
description: Use when modifying the Heimdall dashboard in src/ui. Applies the repo's industrial monochrome design rules, TSX source-of-truth workflow, XSS safeguards, and required UI build steps.
---

# Heimdall Dashboard

Use this skill for dashboard work in `src/ui/`.

## Design constraints

- Follow the industrial design system already documented for Heimdall.
- Prefer monochrome hierarchy, dense but readable tables, and red only for urgent or destructive states.
- Preserve dark and light mode parity.
- Use `esc()` for dynamic text that reaches HTML-producing helpers.

## Source of truth

- Edit `src/ui/app.tsx`, `src/ui/components/*.tsx`, `src/ui/lib/*.ts`, and `src/ui/state/*.ts`.
- Never hand-edit `src/ui/app.js` or `src/ui/style.css`.

## Required workflow

1. Make the TSX/CSS changes.
2. Rebuild UI artifacts with `npm run build:ui`.
3. If the change is type-sensitive, run `./node_modules/.bin/tsc --noEmit`.
4. Commit the generated `src/ui/app.js` and `src/ui/style.css` alongside the source change.

## UI guardrails

- Keep pricing and business logic in Rust, not in the dashboard.
- Reuse existing chart and table abstractions before adding new ones.
- Prefer inline status treatments over toast-like feedback.
- Keep provider-aware filtering intact when introducing new panels or metrics.

## Preact signals guardrails

- **`useSignal(prop)` goes stale** — `useSignal` only captures the argument on first call; subsequent parent re-renders with a new prop value leave the signal holding the original value. Never pass a component prop directly as the `useSignal` argument. Use `useComputed(() => props.value)` to derive a reactive value from a prop, or pass the signal object itself down from the parent.
- **Pass signal objects into JSX, not `.value`** — `<p>{mySignal}</p>` lets Preact bind directly to the DOM text node and skip component re-render. `<p>{mySignal.value}</p>` forces a full component re-render on every change. Prefer the signal-object form for read-only display bindings.
- **`effect()` write loops** — an `effect()` that both reads and writes the same signal (or reads signal A and writes signal B that transitively triggers the effect again) reruns unboundedly. Use `signal.peek()` to read a signal value without subscribing to it inside an effect.

## Tailwind v4 guardrails

- **`@apply` in CSS modules requires an explicit import** — `@apply` inside a `.module.css` file without `@import 'tailwindcss'` at the top produces empty output silently in Tailwind v4. Add the import or move the styles to `input.css`.
- **Arbitrary values changed syntax** — `w-[200px]` and `bg-[#ff0000]` style classes may be silently purged in v4 because square brackets now reference CSS variables. Define custom values in `input.css` under `@theme` (`--size-custom: 200px`) and reference them as `w-(--size-custom)`.
