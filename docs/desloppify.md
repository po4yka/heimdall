# Desloppify for Heimdall

This document captures the project-specific `desloppify` workflow for Heimdall so contributors can rerun it consistently.

## Scope

Heimdall is a mixed Rust + TypeScript repository:

- Backend: Rust code under `src/`, plus `src/main.rs`, `src/lib.rs`, and module subtrees such as `src/server/`, `src/scanner/`, `src/oauth/`, `src/optimizer/`, `src/scheduler/`, `src/hook/`, and `src/agent_status/`
- Frontend: dashboard source under `src/ui/`

You can run `desloppify` in two ways:

- Combined run: one queue for the whole repo. Use this when continuing an existing root scan.
- Split runs: separate backend and frontend passes. This is the cleaner setup for new work because generated UI artifacts and Rust/TypeScript concerns stay isolated.

## One-time setup

`desloppify` needs Python 3.11+.

```bash
pip install --upgrade "desloppify[full]"
desloppify update-skill codex
```

Local `desloppify` state lives in `.desloppify/`, which is already ignored by git in this repository.

## Known excludes

Exclude obvious non-source or generated paths before scanning:

- `target/`
- `node_modules/`
- `src/ui/app.js`
- `src/ui/style.css`

Questionable exclude candidate:

- `.omc/`

Only exclude `.omc/` if you know it is disposable local state for your current run. Do not exclude it by default just because it is already ignored by git.

Suggested commands:

```bash
desloppify exclude target
desloppify exclude node_modules
desloppify exclude src/ui/app.js
desloppify exclude src/ui/style.css
```

## Validation commands

Backend validation:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Frontend validation:

```bash
npm install
npm run build:ui
./node_modules/.bin/tsc --noEmit
```

Repo-specific frontend rules:

- `src/ui/app.tsx` and `src/ui/components/*.tsx` are the source of truth.
- `src/ui/app.js` and `src/ui/style.css` are generated and committed artifacts. Do not edit them directly.
- If frontend source changes, rebuild `app.js` and `style.css` before finishing.

Repo-specific backend rules:

- Keep SQL in `src/scanner/db.rs`.
- Use additive migrations only.
- Avoid `.unwrap()` in library code.
- Prefer `thiserror` in library error types and `anyhow` in CLI/main paths.

## Recommended loop

The working loop is always the same:

```bash
desloppify scan --path <scope>
desloppify next
```

Then repeat:

1. Run `desloppify next`
2. Fix the current item properly
3. Run the resolve command that `desloppify` gave you
4. Run `desloppify next` again
5. Rescan periodically after a cluster of fixes

Use `desloppify backlog` only when you need broader context that is not currently driving execution.

## Combined run prompt

Use this when continuing one existing root-level `desloppify` run for the entire repository.

```text
I want you to improve the quality of the Heimdall codebase. This repository is a mixed Rust + TypeScript project with one existing combined `desloppify` run, so continue working from a single repo-wide queue unless I explicitly tell you to split it.

Run ALL of the following (requires Python 3.11+):

pip install --upgrade "desloppify[full]"
desloppify update-skill codex

Add `.desloppify/` to `.gitignore` if it is not already there — it contains local state that should not be committed.

Before scanning, exclude the obvious non-source/generated paths:
- `target/`
- `node_modules/`
- `src/ui/app.js`
- `src/ui/style.css`

If you find questionable exclude candidates such as `.omc/`, ask before excluding them.

This repo has two code areas:
- Rust backend under `src/`
- TypeScript frontend under `src/ui/`

The frontend source of truth is the TS/TSX/CSS/HTML in `src/ui/`. `src/ui/app.js` and `src/ui/style.css` are generated and committed artifacts, so do not edit them directly. Rebuild them after frontend source changes:

npm install
npm run build:ui
./node_modules/.bin/tsc --noEmit

For backend changes, validate with:

cargo fmt --check
cargo clippy -- -D warnings
cargo test

Backend constraints:
- Keep SQL in `src/scanner/db.rs`
- Use additive migrations only
- Avoid `.unwrap()` in library code
- Prefer `thiserror` in library code and `anyhow` in CLI/main paths

Now scan the whole repo:

desloppify scan --path .
desloppify next

Your goal is to get the strict score as high as possible. The scoring resists gaming — the only way to improve it is to actually make the code better.

THE LOOP: run `desloppify next`. It is the execution queue from the living plan, not the whole backlog. It tells you what to fix now, which file, and the resolve command to run when done. Fix it, resolve it, run `next` again. Over and over. This is your main job.

Use `desloppify backlog` only when you need to inspect broader open work that is not currently driving execution.

Do not be lazy. Large refactors and small detailed fixes both count. Fix things properly, not minimally.

Use `desloppify plan` / `desloppify plan queue` to reorder priorities or cluster related issues. Rescan periodically.
```

## Frontend-only prompt

Use this for a clean dashboard-focused pass.

```text
I want you to improve the quality of the Heimdall frontend codebase. This repo is mixed Rust + TypeScript, so do NOT scan the whole repository as one project. Treat the frontend as the TypeScript UI under `src/ui/` only.

Run ALL of the following (requires Python 3.11+):

pip install --upgrade "desloppify[full]"
desloppify update-skill codex

Add `.desloppify/` to `.gitignore` if it is not already there — it contains local state that should not be committed.

Before scanning, check for directories/files that should be excluded and exclude the obvious ones immediately:
- `node_modules/`
- `target/`
- `src/ui/app.js`
- `src/ui/style.css`

If you find questionable exclude candidates such as `.omc/`, ask before excluding them.

This repo’s frontend source of truth is:
- `src/ui/*.tsx`
- `src/ui/components/*.tsx`
- `src/ui/lib/*.ts`
- `src/ui/state/*.ts`
- `src/ui/input.css`
- `src/ui/index.html`

Do not edit generated files directly unless the workflow requires rebuilt outputs. If you change frontend source files, rebuild the committed artifacts afterward:

npm install
npm run build:ui
./node_modules/.bin/tsc --noEmit

Now scan only the frontend project:

desloppify --lang typescript scan --path ./src/ui
desloppify next

Your goal is to get the strict score as high as possible. The scoring resists gaming — the only way to improve it is to actually make the code better.

THE LOOP: run `desloppify next`. It is the execution queue from the living plan, not the whole backlog. It tells you what to fix now, which file, and the resolve command to run when done. Fix it, resolve it, run `next` again. Over and over. This is your main job.

Use `desloppify backlog` only when you need to inspect broader open work that is not currently driving execution.

Do not be lazy. Large refactors and small detailed fixes both count. Fix things properly, not minimally.

Use `desloppify plan` / `desloppify plan queue` to reorder priorities or cluster related issues. Rescan periodically.

This frontend has repo-specific constraints:
- Preserve the industrial monochrome UI language
- `src/ui/app.tsx` and TSX files are the source of truth
- `src/ui/app.js` and `src/ui/style.css` are generated and committed; rebuild them after source changes
- Avoid XSS regressions; dynamic text should remain safely escaped
- Validate changes with `npm run build:ui` and `./node_modules/.bin/tsc --noEmit`
```

## Backend-only prompt

Use this for a Rust-focused pass.

```text
I want you to improve the quality of the Heimdall backend codebase. This repo is mixed Rust + TypeScript, so do NOT scan the whole repository as one project. Treat the backend as the Rust code under `src/`, excluding the frontend subtree `src/ui/`.

Run ALL of the following (requires Python 3.11+):

pip install --upgrade "desloppify[full]"
desloppify update-skill codex

Add `.desloppify/` to `.gitignore` if it is not already there — it contains local state that should not be committed.

Before scanning, check for directories/files that should be excluded and exclude the obvious ones immediately:
- `target/`
- `node_modules/`
- `src/ui/`

If you find questionable exclude candidates such as `.omc/`, ask before excluding them.

The backend project is the Rust service and binaries in `src/`, including modules such as:
- `src/main.rs`
- `src/lib.rs`
- `src/server/`
- `src/scanner/`
- `src/oauth/`
- `src/optimizer/`
- `src/scheduler/`
- `src/hook/`
- `src/agent_status/`

Do not spend effort on frontend assets in this backend pass.

Now scan only the backend project:

desloppify --lang rust scan --path ./src
desloppify next

Your goal is to get the strict score as high as possible. The scoring resists gaming — the only way to improve it is to actually make the code better.

THE LOOP: run `desloppify next`. It is the execution queue from the living plan, not the whole backlog. It tells you what to fix now, which file, and the resolve command to run when done. Fix it, resolve it, run `next` again. Over and over. This is your main job.

Use `desloppify backlog` only when you need to inspect broader open work that is not currently driving execution.

Do not be lazy. Large refactors and small detailed fixes both count. Fix things properly, not minimally.

Use `desloppify plan` / `desloppify plan queue` to reorder priorities or cluster related issues. Rescan periodically.

This backend has repo-specific constraints:
- Keep all SQL in `src/scanner/db.rs`
- Use additive migrations only; never destructive schema changes
- No `.unwrap()` in library code
- Use `thiserror` for library errors and `anyhow` in CLI/main paths
- Preserve provider-specific parsing and tagging behavior
- Validate Rust changes with `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`
```
