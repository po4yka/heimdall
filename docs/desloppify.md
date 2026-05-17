# Desloppify for Heimdall

This document captures the project-specific `desloppify` workflow for Heimdall so contributors can rerun it consistently.

## Scope

Heimdall is a mixed Rust + TypeScript repository:

- Backend: Rust code under `src/`, plus `src/main.rs`, `src/lib.rs`, and module subtrees such as `src/server/`, `src/scanner/`, `src/oauth/`, `src/optimizer/`, `src/scheduler/`, `src/hook/`, and `src/agent_status/`
- Frontend: dashboard source under `src/ui/`

You can run `desloppify` in two ways:

- Combined run: one queue for the whole repo. Use this when continuing an existing root scan.
- Split runs: separate backend and frontend passes. This is the cleaner setup for new work because generated UI artifacts and Rust/TypeScript concerns stay isolated.

## Prompt-design notes

The three embedded prompts below are LLM input, not narrative documentation. They follow three rules borrowed from [talk-normal's regression iteration arc](https://github.com/hexiecs/talk-normal/blob/main/regressions/rule-17-negation-frame.md) — preserve them on edit:

1. **Repo-specific hard constraints come first.** Attention decays sharply down a prompt, so rules a contributor cannot derive from a generic Rust or TypeScript style guide live at position 2 (right after the goal sentence), not at the bottom under "additional notes". When in doubt, promote.
2. **Structural prohibitions beat word lists.** `no Doto / no Space Grotesk / no Space Mono` misses the next unlisted typeface; `do not introduce any UI face outside the design-system whitelist` covers it. Same shape for `.unwrap()`: the structural rule is "no panic-on-error in library code", not a list of method names that the next std-lib release will extend.
3. **High-prior-to-violate rules ship with a BAD/GOOD pair.** Bare prohibitions on idioms the model has been trained to use (`.unwrap()` is canonical-Rust-tutorial material) get copied verbatim or worked around. A one-line GOOD example anchors the alternative direction.

If you add a rule here that does not satisfy all three, the next contributor will leak it. Iterate it the way talk-normal iterated rule 17: log the leak under [`regressions/`](../regressions/) and rewrite.

## Repo-specific hard constraints

These are the rules that cannot be derived from a generic style guide. They appear at position 2 in each embedded prompt below, in the same wording — keep them in sync.

### Backend (Rust under `src/`, excluding `src/ui/`)

- **No panic-on-error in library code.** Anything that panics on a recoverable error is banned in `src/scanner/`, `src/server/`, `src/pricing/`, `src/oauth/`, `src/optimizer/`, `src/agent_status/`, `src/scheduler/`, and `src/hook/`. That includes `.unwrap()`, `.expect("...")` on `Result` / `Option`, `panic!`, `unreachable!`, `unimplemented!`, and `todo!`. Acceptable in `tests/`, `#[cfg(test)]` modules, and `main.rs` / CLI glue. - **BAD:** `let conn = open_db(path).unwrap();` - **GOOD:** `let conn = open_db(path)?;` (with a `thiserror`-derived error at the function boundary)
- **Library errors via `thiserror`; CLI/main via `anyhow`.** Library functions return `Result<T, ThisErrorEnum>` so callers can match; `main.rs` and CLI glue use `anyhow::Result<T>` and `?` to bubble. - **BAD:** `pub fn parse(...) -> anyhow::Result<Vec<Turn>>` in a library module. - **GOOD:** `pub fn parse(...) -> Result<Vec<Turn>, ParseError>` with `#[derive(thiserror::Error)]`.
- **All SQL lives in `src/scanner/db.rs`.** No inline SQL in handlers, providers, or detectors. New queries become typed functions in `db.rs` that callers invoke.
- **Schema migrations are additive only.** `ALTER TABLE ... ADD COLUMN` with a `has_column` guard. No `DROP COLUMN`, no `DROP TABLE`, no value-type narrowing. Backfill defaults via idempotent `UPDATE ... WHERE column IS NULL` after the ADD.
- **Provider-specific parsing stays per-provider.** Cross-provider helpers in `parser.rs`; per-provider quirks (Pi `responseId` last-wins, Codex cumulative-token cross-check, Claude / Xcode `message.id` dedup) stay in their own provider modules.

### Frontend (TypeScript / Preact under `src/ui/`)

- **TSX is source of truth; `app.js` and `style.css` are committed build output.** Edit `src/ui/*.tsx` and `src/ui/input.css`; never edit the compiled artifacts directly. Rebuild with `npm run build:ui` after source changes and commit the regenerated artifacts alongside source.
- **Single design system: Inter + Geist Mono.** Inter for UI / headings / body; Geist Mono for numbers, code, tabular columns. Do not introduce any UI face outside this whitelist without a design-system entry — `Doto`, `Space Grotesk`, `Space Mono`, and any other display, body, or monospace face are out by structural rule, not by name. - **BAD:** `font-family: 'Roboto Mono', monospace;` in a new component. - **GOOD:** consume the `--font-mono` token, which resolves to Geist Mono.
- **All dynamic text passes through `esc()` in `src/ui/lib/format.ts`.** No raw template literals into the DOM, no `innerHTML` with user data, no `dangerouslySetInnerHTML`. The `esc()` call is the structural gate; reviewers grep for it. - **BAD:** `<span>{turn.model}</span>` (where `model` arrived from the API). - **GOOD:** `<span>{esc(turn.model)}</span>`.
- **One chromatic accent per screen.** `--accent-interactive` (`#4A7FA5`) for primary interactive (links, selected, primary buttons). `--accent` (`#D71921`) is reserved for semantic error / destructive / over-limit only. Status colours (`--success`, `--warning`) unchanged.
- **Sentence-case for UI copy.** ALL-CAPS monospace is reserved for `<th>` table column headers only — stat-card labels, section titles, filter labels, and chart titles use 11–12 px sentence-case in `--text-secondary`.

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

HARD CONSTRAINTS — apply these as a filter on every fix you propose, before any other consideration. These are rules that cannot be derived from a generic style guide.

Backend (Rust under src/, excluding src/ui/):
- No panic-on-error in library code. Anything that panics on a recoverable error is banned in src/scanner/, src/server/, src/pricing/, src/oauth/, src/optimizer/, src/agent_status/, src/scheduler/, src/hook/. That includes .unwrap(), .expect("...") on Result/Option, panic!, unreachable!, unimplemented!, and todo!. Acceptable in tests/, #[cfg(test)] modules, and main.rs / CLI glue.
  BAD:  let conn = open_db(path).unwrap();
  GOOD: let conn = open_db(path)?;
- Library errors via thiserror; CLI/main via anyhow. Library functions return Result<T, ThisErrorEnum>; main.rs / CLI glue uses anyhow::Result<T>.
  BAD:  pub fn parse(...) -> anyhow::Result<Vec<Turn>>     (in a library module)
  GOOD: pub fn parse(...) -> Result<Vec<Turn>, ParseError> (with #[derive(thiserror::Error)])
- All SQL lives in src/scanner/db.rs. No inline SQL in handlers, providers, or detectors.
- Schema migrations are additive only. ALTER TABLE ... ADD COLUMN with a has_column guard. No DROP, no value-type narrowing.
- Provider-specific parsing stays per-provider. Pi responseId last-wins, Codex cumulative-token cross-check, Claude/Xcode message.id dedup — each lives in its own provider module.

Frontend (TypeScript / Preact under src/ui/):
- TSX is source of truth; app.js and style.css are committed build output. Never edit compiled artifacts directly.
- Single design system: Inter for UI / headings / body, Geist Mono for numbers / code. Do not introduce any UI face outside this whitelist without a design-system entry.
  BAD:  font-family: 'Roboto Mono', monospace;
  GOOD: consume --font-mono token (Geist Mono)
- All dynamic text passes through esc() in src/ui/lib/format.ts. No raw template literals into the DOM, no innerHTML with user data, no dangerouslySetInnerHTML.
  BAD:  <span>{turn.model}</span>
  GOOD: <span>{esc(turn.model)}</span>
- One chromatic accent per screen: --accent-interactive (#4A7FA5) for interactive, --accent (#D71921) reserved for semantic error only.
- Sentence-case for UI copy; ALL-CAPS reserved for <th> table column headers only.

SETUP (requires Python 3.11+):

pip install --upgrade "desloppify[full]"
desloppify update-skill codex

Add `.desloppify/` to `.gitignore` if it is not already there — it contains local state that should not be committed.

EXCLUDES — exclude these before scanning:
- target/
- node_modules/
- src/ui/app.js
- src/ui/style.css

If you find questionable exclude candidates such as `.omc/`, ask before excluding them.

VALIDATION:
- Backend: cargo fmt --check && cargo clippy -- -D warnings && cargo test
- Frontend: npm install && npm run build:ui && ./node_modules/.bin/tsc --noEmit

Now scan the whole repo:

desloppify scan --path .
desloppify next

THE LOOP: run `desloppify next`. It is the execution queue from the living plan, not the whole backlog. It tells you what to fix now, which file, and the resolve command to run when done. Fix it, resolve it, run `next` again. Over and over. This is your main job.

Use `desloppify backlog` only when you need to inspect broader open work that is not currently driving execution. Use `desloppify plan` / `desloppify plan queue` to reorder priorities or cluster related issues. Rescan periodically.

Your goal is to get the strict score as high as possible. The scoring resists gaming — the only way to improve it is to actually make the code better. Fix things properly, not minimally.
```

## Frontend-only prompt

Use this for a clean dashboard-focused pass.

```text
I want you to improve the quality of the Heimdall frontend codebase. This repo is mixed Rust + TypeScript, so do NOT scan the whole repository as one project. Treat the frontend as the TypeScript UI under `src/ui/` only.

HARD CONSTRAINTS — apply these as a filter on every fix you propose, before any other consideration:

- TSX is source of truth; app.js and style.css are committed build output. Edit src/ui/*.tsx and src/ui/input.css; never edit compiled artifacts directly. Rebuild with `npm run build:ui` after source changes and commit the regenerated artifacts alongside source.
- Single design system: Inter for UI / headings / body, Geist Mono for numbers / code / tabular columns. Do not introduce any UI face outside this whitelist without a design-system entry.
  BAD:  font-family: 'Roboto Mono', monospace;
  GOOD: consume --font-mono token (Geist Mono)
- All dynamic text passes through esc() in src/ui/lib/format.ts. No raw template literals into the DOM, no innerHTML with user data, no dangerouslySetInnerHTML.
  BAD:  <span>{turn.model}</span>     (where model came from the API)
  GOOD: <span>{esc(turn.model)}</span>
- One chromatic accent per screen: --accent-interactive (#4A7FA5) for primary interactive (links, selected, primary buttons); --accent (#D71921) reserved for semantic error / destructive / over-limit only.
- Sentence-case for UI copy. ALL-CAPS monospace reserved for <th> table column headers only.
- No gradients, no shadows on content surfaces, no toast popups — use inline [SAVED] / [ERROR: ...] bracket status text near the trigger.

SETUP (requires Python 3.11+):

pip install --upgrade "desloppify[full]"
desloppify update-skill codex

Add `.desloppify/` to `.gitignore` if it is not already there — it contains local state that should not be committed.

EXCLUDES — check for directories/files that should be excluded and exclude the obvious ones immediately:
- node_modules/
- target/
- src/ui/app.js
- src/ui/style.css

If you find questionable exclude candidates such as `.omc/`, ask before excluding them.

Frontend source-of-truth files:
- src/ui/*.tsx
- src/ui/components/*.tsx
- src/ui/lib/*.ts
- src/ui/state/*.ts
- src/ui/input.css
- src/ui/index.html

VALIDATION (after every cluster of fixes):
npm install
npm run build:ui
./node_modules/.bin/tsc --noEmit

Scan only the frontend project:

desloppify --lang typescript scan --path ./src/ui
desloppify next

THE LOOP: run `desloppify next`. It is the execution queue from the living plan, not the whole backlog. Fix the current item, resolve it, run `next` again. This is your main job.

Use `desloppify backlog` only when you need to inspect broader open work that is not currently driving execution. Use `desloppify plan` / `desloppify plan queue` to reorder priorities or cluster related issues. Rescan periodically.

Your goal is to get the strict score as high as possible. The scoring resists gaming — the only way to improve it is to actually make the code better. Fix things properly, not minimally.
```

## Backend-only prompt

Use this for a Rust-focused pass.

```text
I want you to improve the quality of the Heimdall backend codebase. This repo is mixed Rust + TypeScript, so do NOT scan the whole repository as one project. Treat the backend as the Rust code under `src/`, excluding the frontend subtree `src/ui/`.

HARD CONSTRAINTS — apply these as a filter on every fix you propose, before any other consideration:

- No panic-on-error in library code. Anything that panics on a recoverable error is banned in src/scanner/, src/server/, src/pricing/, src/oauth/, src/optimizer/, src/agent_status/, src/scheduler/, src/hook/. That includes .unwrap(), .expect("...") on Result/Option, panic!, unreachable!, unimplemented!, and todo!. Acceptable in tests/, #[cfg(test)] modules, and main.rs / CLI glue.
  BAD:  let conn = open_db(path).unwrap();
  GOOD: let conn = open_db(path)?;
- Library errors via thiserror; CLI/main via anyhow. Library functions return Result<T, ThisErrorEnum> so callers can match; main.rs / CLI glue uses anyhow::Result<T>.
  BAD:  pub fn parse(...) -> anyhow::Result<Vec<Turn>>     (in a library module)
  GOOD: pub fn parse(...) -> Result<Vec<Turn>, ParseError> (with #[derive(thiserror::Error)])
- All SQL lives in src/scanner/db.rs. No inline SQL in handlers, providers, or detectors. New queries get a typed function in db.rs that callers invoke.
- Schema migrations are additive only. ALTER TABLE ... ADD COLUMN with a has_column guard. No DROP, no value-type narrowing. Backfill via idempotent UPDATE ... WHERE column IS NULL after the ADD.
- Provider-specific parsing stays per-provider. Cross-provider helpers in parser.rs; per-provider quirks (Pi responseId last-wins, Codex cumulative-token cross-check, Claude/Xcode message.id dedup) stay in their own provider modules.

SETUP (requires Python 3.11+):

pip install --upgrade "desloppify[full]"
desloppify update-skill codex

Add `.desloppify/` to `.gitignore` if it is not already there — it contains local state that should not be committed.

EXCLUDES — check for directories/files that should be excluded and exclude the obvious ones immediately:
- target/
- node_modules/
- src/ui/

If you find questionable exclude candidates such as `.omc/`, ask before excluding them.

The backend project covers:
- src/main.rs
- src/lib.rs
- src/server/
- src/scanner/
- src/oauth/
- src/optimizer/
- src/scheduler/
- src/hook/
- src/agent_status/

Do not spend effort on frontend assets in this backend pass.

VALIDATION (after every cluster of fixes):
cargo fmt --check
cargo clippy -- -D warnings
cargo test

Scan only the backend project:

desloppify --lang rust scan --path ./src
desloppify next

THE LOOP: run `desloppify next`. It is the execution queue from the living plan, not the whole backlog. Fix the current item, resolve it, run `next` again. This is your main job.

Use `desloppify backlog` only when you need to inspect broader open work that is not currently driving execution. Use `desloppify plan` / `desloppify plan queue` to reorder priorities or cluster related issues. Rescan periodically.

Your goal is to get the strict score as high as possible. The scoring resists gaming — the only way to improve it is to actually make the code better. Fix things properly, not minimally.
```
