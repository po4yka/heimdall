# Rust Unsafe + libc FFI

Review and author guidance for all `unsafe` blocks and libc FFI in heimdall. Apply every rule to every `unsafe` block in a diff — not only the first.

## Unsafe inventory

Known locations:
- `src/archive/mod.rs` — `libc::flock` (file locking)
- `src/scheduler/daemon.rs` — `libc::getuid` (process UID check)
- `src/scheduler/launchd.rs` — `extern "C"` FFI
- `src/statusline/mod.rs` — 2 unsafe blocks (terminal control)
- `src/config.rs` — `unsafe { env::set_var }` in test setup

## SAFETY comment rules

Every `unsafe {}` block MUST have a `// SAFETY:` comment directly above it stating:
1. What invariant is upheld.
2. Where that invariant is established.

Missing SAFETY comment = **CRITICAL** finding.

## `env::set_var` (edition 2024)

`std::env::set_var` is `unsafe` in Rust edition 2024. Only call in single-threaded contexts (before any `tokio::spawn` or `std::thread::spawn`). Always add: `// SAFETY: single-threaded: called before any thread spawns in this test`.

## `flock` and `getuid` patterns

- Check return value; retrieve errno via `std::io::Error::last_os_error()`.
- Use `unsafe extern "C" { }` block syntax (Rust 2024).
- Document the fd source in the SAFETY comment.

## Drop is not guaranteed

`mem::forget` is safe. Any `unsafe` API relying on a RAII guard's `Drop` for soundness is unsound. RAII guards in `src/archive/mod.rs` and `src/scheduler/daemon.rs` must not be `mem::forget`-able on any sound execution path.

## One `unsafe` in a dep breaks local reasoning

Run `cargo deny check`. Flag deps with soundness advisories.
Every `str::from_utf8_unchecked` / `from_raw_parts` in `src/` needs a SAFETY comment.

## Manual `unsafe impl Sync/Send`

List every field type in the SAFETY comment. Add `static_assertions` regression test.

## Audit checklist

- [ ] SAFETY comment present and explains invariant + origin?
- [ ] `env::set_var`: confirmed single-threaded context?
- [ ] `flock`/`getuid`: return value checked, errno retrieved?
- [ ] Drop-guarantee: RAII guard cannot be `mem::forget`-ed?
- [ ] No `from_utf8_unchecked`/`from_raw_parts` without SAFETY comment?
- [ ] `unsafe impl Sync/Send`: every field type listed?
