---
name: dashboard-rebuild
description: Rebuild the committed dashboard artifacts (src/ui/app.js, src/ui/style.css) after editing src/ui/*.tsx or src/ui/input.css, and stage them for commit. Use whenever a dashboard source file has been edited or whenever the working tree shows .tsx/.input.css changes without matching app.js/style.css updates.
allowed-tools: [Bash, Read]
---

# Dashboard rebuild

Cargo builds embed `src/ui/app.js` and `src/ui/style.css` via `include_str!`. They are committed to git so `cargo build` works without Node.js. **Source edits to `*.tsx` or `input.css` that aren't matched by rebuilt artifacts ship a working source tree but a broken dashboard.**

## When to invoke

- The working tree contains modified files under `src/ui/` matching `*.tsx` or `input.css` and the matching artifact (`app.js` or `style.css`) is unchanged.
- Right after editing dashboard source as part of a larger task.
- Before committing dashboard changes.

## Run

Always check first whether `node_modules` is installed:

```bash
test -d node_modules || npm install
```

Then rebuild and verify:

```bash
./node_modules/.bin/tsc --noEmit
npm run build:ui
git status --porcelain src/ui/app.js src/ui/style.css
```

If `git status` shows `app.js` or `style.css` modified, stage them alongside the source edits:

```bash
git add src/ui/app.js src/ui/style.css
```

If neither file changed after the rebuild, the source edit may have been a no-op for the bundle (e.g., type-only change). Note this so the user knows.

## Reporting

After running, report:
1. Whether `tsc --noEmit` passed.
2. Which artifacts changed (`app.js`, `style.css`, both, or neither).
3. The exact files now staged.

Do **not** commit. Staging only — the user owns the commit boundary, and unrelated edits in the working tree should not be swept into the same commit.
