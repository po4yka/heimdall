---
name: heimdall-rust-unsafe-ffi
description: Review and author guidance for all unsafe blocks and libc FFI in heimdall — SAFETY comments, env::set_var, flock/getuid patterns, Drop guarantee, dep unsafe propagation, and manual Send/Sync.
---

# Heimdall Rust Unsafe + libc FFI

## Purpose

Guide review and authoring of `unsafe` code in heimdall. Apply every rule to every `unsafe` block in a diff — not only the first one. Heimdall is a single-crate project with edition-2024 Rust.

## Unsafe inventory

Known `unsafe` locations (verify current state before auditing):

| File | Pattern | Notes |
|---|---|---|
| `src/archive/mod.rs` | `libc::flock` | File locking via libc |
| `src/scheduler/daemon.rs` | `libc::getuid` | Process UID check |
| `src/scheduler/launchd.rs` | `extern "C"` FFI | launchd service registration |
| `src/statusline/mod.rs` | 2 unsafe blocks | Terminal control sequences |
| `src/config.rs` | `unsafe { env::set_var }` | Edition-2024 unsafe in test setup |

## SAFETY comment rules

Every `unsafe {}` block MUST be immediately preceded by a `// SAFETY:` comment (or `/// # Safety` for `unsafe fn`) that states:
1. What invariant is being upheld.
2. Where that invariant is established (caller contract, local variable, prior check).

```rust
// SAFETY: `fd` was opened by our own `open(2)` call above and is still valid;
// LOCK_EX | LOCK_NB is a valid flag combination; return value checked below.
let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
if ret != 0 {
    return Err(std::io::Error::last_os_error());
}
```

Missing SAFETY comment on any `unsafe {}` block is a **CRITICAL** finding.

## `env::set_var` in edition 2024

`std::env::set_var` is `unsafe` in Rust edition 2024 because it is not thread-safe: calling it while other threads are reading the environment is UB.

Rules:
- Only call `env::set_var` in single-threaded contexts (e.g., the very beginning of a test before any `tokio::spawn`, `std::thread::spawn`, or `rayon` parallel scope).
- Always wrap with `// SAFETY: single-threaded: called before any thread spawns in this test`.
- In production code: prefer `std::env::vars()` capture at startup and pass config explicitly; never mutate the environment after `main` has started threads.

Current location: `src/config.rs` test setup — audit that no thread is spawned before the `set_var` call.

## `flock` and `getuid` libc patterns

For all libc FFI calls in Rust 2024, use the `unsafe extern "C"` block syntax:

```rust
extern "C" {
    // declaration only -- no unsafe here in Rust 2021
}
// In Rust 2024, extern blocks without `unsafe` generate a warning; prefer:
unsafe extern "C" {
    fn getuid() -> libc::uid_t;
}
```

For `flock` and `getuid` calls:
- Check return value: `flock` returns 0 on success, -1 on error; retrieve errno via `std::io::Error::last_os_error()`.
- Document the fd source in the SAFETY comment (e.g., "fd opened by `File::open` in this function").
- Use `BorrowedFd::borrow_raw` when passing a raw fd to ensure Rust knows about the borrow.

## Drop is not guaranteed

`mem::forget` is a safe function. Any `unsafe` API that relies on a RAII guard running its `Drop` for soundness is unsound.

Concrete rule: if a `File`, `OwnedFd`, or custom guard in heimdall is expected to run cleanup (close fd, release lock), it must NOT be passed to code that might call `mem::forget` or `ManuallyDrop::new` on it — those are safe calls.

Current risk areas:
- `src/archive/mod.rs`: `flock` guard — verify the guard is not `mem::forget`-ed anywhere on the path between acquisition and release.
- `src/scheduler/daemon.rs`: daemon PID file handle — same check.

Reference: `crabbook/raii_and_memory_safety.md`

## One `unsafe` in a dep breaks local reasoning

A single `unsafe` in a dependency can invalidate type invariants codebase-wide. You cannot assume that safe code in heimdall is sound if a dep calls `str::from_utf8_unchecked` on unvalidated data.

Action: run `cargo deny check` to find deps with known soundness advisories. Also:
```bash
cargo tree --depth 3 | head -40
rg 'from_utf8_unchecked\|from_raw_parts' src/ --type rust -n
```
Any hit without a SAFETY comment is HIGH risk.

Reference: `crabbook/unsafe_is_unsafe.md`

## Manual `unsafe impl Sync/Send`

Before writing `unsafe impl Sync for T` or `unsafe impl Send for T`:
1. List every field type. For each, verify it is `Sync`/`Send` or document why the wrapper maintains safety despite the field not being so.
2. Check every blanket trait impl (`Debug`, `Clone`, `Display`) — they must not expose inner non-`Sync`/non-`Send` state.
3. Add `static_assertions::assert_impl_all!` or `assert_not_impl_all!` to catch regressions.
4. Tag the `unsafe impl` with a `// SAFETY:` comment.

Grep: `rg 'unsafe impl Sync|unsafe impl Send' src/ --type rust -n`

Reference: `crabbook/send_and_sync.md`

## Pin and self-referential types

Heimdall has no self-referential types today. If future tokio patterns (long-lived servers, custom `Future` combinators, async generators) introduce self-referential state:
- Never author a self-referential struct without `Pin<Box<T>>` + `PhantomPinned`.
- `Box::pin(val)` is the standard way to heap-pin a value.
- `Pin<&mut T>` is required for types that must not move after construction (e.g., C++ types behind FFI).

Reference: `crabbook/pin.md`

## `#[no_mangle]` and `#[link_section]` symbol collision (edition 2024)

**Severity: CRITICAL**

In Rust 2024, `#[no_mangle]`, `#[export_name = "..."]`, and `#[link_section = "..."]` must use the `#[unsafe(...)]` form. The hazard that motivates this: two compilation units exporting the same unmangled symbol causes the linker to silently pick one, calling the wrong function — a soundness bug with no compile-time diagnostic.

Heimdall currently uses `edition = "2024"`. Audit:
```bash
rg '#\[no_mangle\]|#\[export_name' src/ --type rust -n
```
Any hit must use `#[unsafe(no_mangle)]` or `#[unsafe(export_name = "...")]` and have a SAFETY comment asserting the symbol name is unique across the linked binary.

## Panicking inside `Drop::drop` during unwinding aborts the process

**Severity: CRITICAL**

If a panic is already in progress (stack unwinding), and a `Drop` impl panics, Rust immediately aborts the process — this is a "double panic" and cannot be caught by `catch_unwind`. Any `.unwrap()` or `.expect()` inside `drop()` is a double-panic bomb that fires exactly when another error is already in flight.

```rust
// DANGEROUS: double-panic if called during unwind
impl Drop for LockFile {
    fn drop(&mut self) {
        std::fs::remove_file(&self.path).unwrap(); // aborts on unwind
    }
}

// CORRECT: log-and-discard in drop(); expose explicit close() -> Result
impl Drop for LockFile {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.path) {
            tracing::error!("failed to remove lock file: {e}");
        }
    }
}
```

Check heimdall's `src/scheduler/daemon.rs` PID-file guard and `src/archive/mod.rs` flock guard — both perform cleanup in `Drop`. Ensure neither has `.unwrap()` on the cleanup path.

## Audit checklist

Apply to every `unsafe` block in a diff:

- [ ] SAFETY comment present and explains the invariant and its origin?
- [ ] For `env::set_var`: confirmed single-threaded context before any thread spawn?
- [ ] For `flock`/`getuid`/libc: return value checked, `errno` retrieved on error?
- [ ] For `unsafe extern "C"` blocks: Rust 2024 `unsafe extern` syntax used?
- [ ] Drop-guarantee: no RAII guard in the safety chain can be `mem::forget`-ed by callers?
- [ ] No `from_utf8_unchecked` or `from_raw_parts` without a SAFETY comment tracing the invariant?
- [ ] `unsafe impl Sync/Send`: every field type listed in the SAFETY comment?
- [ ] Any `Drop::drop` containing `.unwrap()` or `.expect()` on the cleanup path? → move to explicit `close()`/`flush()` returning `Result`.
- [ ] Any `#[no_mangle]` without `#[unsafe(no_mangle)]` form (edition 2024 requirement)?
