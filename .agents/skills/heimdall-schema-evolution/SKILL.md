---
name: heimdall-schema-evolution
description: Use when adding a new field or changing data flow across Heimdall models, parser/provider code, SQLite schema, API output, and dashboard types. Enforces additive migrations and end-to-end propagation.
---

# Heimdall Schema Evolution

Use this skill when a change crosses `models.rs`, parser/provider code, `src/scanner/db.rs`, server responses, or dashboard types.

## Required flow

1. Add the field to the Rust model in `src/models.rs`.
2. Parse or derive it in the relevant scanner/parser/provider code.
3. Persist it in `src/scanner/db.rs`.
4. Update insert and query code that reads or writes the field.
5. Expose it through server/API responses if the dashboard or CLI needs it.
6. Mirror it in `src/ui/state/types.ts` and UI components if it should render.
7. Add or update tests for the migration and the new data flow.

## Migration rules

- Migrations must be additive.
- Use `has_column` guards before `ALTER TABLE ADD COLUMN`.
- If old rows need a default, follow the add with an idempotent `UPDATE ... WHERE column IS NULL OR column = ''`.
- Keep all SQLite schema work in `src/scanner/db.rs`.
- Never drop columns or tables in a migration path.

## Dashboard propagation

- `src/ui/app.tsx` and `src/ui/components/*.tsx` are the source of truth.
- Do not edit `src/ui/app.js` directly.
- If UI source changes, rebuild committed assets with `npm run build:ui`.

## Sanity checks

- Old databases must still open cleanly.
- Fresh databases must get the new column from `init_db`.
- Existing summaries and filters should remain valid when the new field is absent on older rows.
