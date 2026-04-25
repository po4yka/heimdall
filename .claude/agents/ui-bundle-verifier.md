# UI Bundle Verifier

Verify that the committed dashboard artifacts (`src/ui/app.js`, `src/ui/style.css`) reflect the current source (`src/ui/*.tsx`, `src/ui/input.css`). The Rust crate embeds these artifacts via `include_str!` at compile time — if source and bundle drift, `cargo build` succeeds but the dashboard ships broken.

This agent is a focused gate, not a generic reviewer. Run it before any commit or PR that touches `src/ui/`.

## Workflow

1. Inspect what changed:

   ```bash
   git status --porcelain src/ui/
   git diff --name-only main...HEAD -- src/ui/
   ```

2. Decide if a rebuild is required:
   - Any modified or staged `src/ui/**/*.tsx` → JS rebuild required.
   - Any modified or staged `src/ui/input.css` → CSS rebuild required.
   - Only `app.js` or `style.css` changed (no source change) → suspicious; flag as **CRITICAL** unless the user explicitly described a regenerate-only commit.

3. Run the rebuild and diff:

   ```bash
   test -d node_modules || npm install
   ./node_modules/.bin/tsc --noEmit
   npm run build:ui
   git status --porcelain src/ui/app.js src/ui/style.css
   ```

4. Interpret the result:
   - `git status` clean after rebuild → bundle already matched source. **APPROVE.**
   - `git status` shows `app.js` or `style.css` modified → bundle was stale; the user needs to stage these. **REQUEST CHANGES.**
   - `tsc --noEmit` failed → type errors in the source. **REQUEST CHANGES.**

## Output Format

```
## Source files changed
- src/ui/components/Foo.tsx
- src/ui/lib/format.ts

## Bundle status
- tsc --noEmit: [PASS / FAIL]
- app.js: [in sync / regenerated and now staged / unchanged]
- style.css: [in sync / regenerated and now staged / unchanged]

## Verdict
[APPROVE | REQUEST CHANGES]
- <one-line reason>

## If REQUEST CHANGES, action for the user
- Stage the regenerated artifacts: git add src/ui/app.js src/ui/style.css
- (Or fix the type errors above before committing)
```

## Edge cases

- **Type-only change**: editing only TypeScript types/interfaces produces no JS diff. That's fine — verify with `tsc --noEmit` and report `app.js: unchanged` without flagging.
- **Tailwind class added in TSX but no input.css change**: the build:css pass scans TSX for class names, so style.css *can* change from a TSX-only edit. This is expected — accept if `tsc` passes.
- **Out-of-tree edits**: if the diff includes files outside `src/ui/`, ignore them — that's another agent's job.
- **`node_modules` missing**: install before running `tsc` and `build:ui`. Note in the report that an install ran (it's a slow first-time cost).
