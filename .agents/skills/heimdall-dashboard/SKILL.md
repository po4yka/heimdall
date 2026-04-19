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
3. If the change is type-sensitive, run `npm run typecheck`.
4. Commit the generated `src/ui/app.js` and `src/ui/style.css` alongside the source change.

## UI guardrails

- Keep pricing and business logic in Rust, not in the dashboard.
- Reuse existing chart and table abstractions before adding new ones.
- Prefer inline status treatments over toast-like feedback.
- Keep provider-aware filtering intact when introducing new panels or metrics.
