---
name: heimdall-rust-api-discipline
description: Rust API design discipline for heimdall — borrowed args, Fn callback lifetimes, HRTB workarounds, Drop design, and value-passing perf for large structs.
---

# Heimdall Rust API Discipline

## Purpose

Encode API-design discipline from production Rust pitfalls. Apply every rule to every changed public or `pub(crate)` function signature in a diff. Heimdall-specific file references are included where concrete risk exists.

## Borrowed args over owned references

**Severity: WARNING**

Flag `&String`, `&Vec<T>`, and `&PathBuf` in public or crate-public function signatures. The owned-reference shapes force callers to hold an allocation even when they have a slice.

```rust
// BAD
fn scan(path: &PathBuf) {}
fn log(msg: &String) {}

// GOOD
fn scan(path: &std::path::Path) {}  // or: impl AsRef<Path>
fn log(msg: &str) {}
```

Risk areas in heimdall (check these paths):
- `src/scanner/` — file-path arguments throughout
- `src/server/` — request/response handler signatures
- `src/oauth/` — token and URL string args

Grep: `rg 'fn .+\(&String|fn .+\(&Vec<|fn .+\(&PathBuf' src/ --type rust -n`

Reference: `crabbook/borrowed_args.md`

## `Fn` callbacks with reference args in async contexts

**Severity: CRITICAL for tokio::spawn closures**

Heimdall uses `tokio::spawn` extensively in `src/server/sse.rs` and `src/scanner/watcher.rs`. Closures passed to `tokio::spawn` must be `'static + Send`. Capturing `&mut State` inside such a closure is a compile error — the reference cannot be `'static`.

Correct patterns:
```rust
// BAD: captures &mut state -- does not compile
tokio::spawn(async move { state.handle(event) });

// GOOD option 1: Arc<Mutex<State>> for shared mutable state
let state = Arc::new(Mutex::new(State::new()));
let state_clone = Arc::clone(&state);
tokio::spawn(async move { state_clone.lock().await.handle(event) });

// GOOD option 2: mpsc channel -- pass ownership of events to an owner task
let (tx, mut rx) = tokio::sync::mpsc::channel(32);
tokio::spawn(async move {
    while let Some(event) = rx.recv().await { state.handle(event); }
});
```

Decision: `Arc<Mutex<T>>` is correct for low-contention config/status state. For high-frequency scanner events, prefer message-passing (the channel model above) to avoid lock contention.

Reference: `crabbook/borrowing_in_generic_functions.md`

## HRTB closure-inference workaround

**Severity: WARNING**

When a closure argument fails to type-check because its return type is a reference into the argument (e.g., `|s: &str| -> &str`), the compiler may infer a fixed lifetime instead of `for<'a>`. Workaround: name the function or use a helper shim:

```rust
fn force_hrtb<F: for<'a> Fn(&'a str) -> &'a str>(f: F) -> F { f }
// Then use: let cb = force_hrtb(|s| s);
```

In heimdall, this is most likely to arise in `src/server/` route handlers and `src/scanner/providers/` callback registration patterns.

Reference: `crabbook/borrowing_in_generic_functions.md`

## `impl Drop` blocks partial moves

**Severity: WARNING**

Before adding `impl Drop` to any new struct, check whether any field in that struct needs to be moved out (consumed) — by `Drop::drop` itself or by external callers. `impl Drop` makes this impossible (only `mem::take` or `unsafe ptr::read` can extract values).

Checklist:
1. List every field. Ask: "does any path want to move this out?"
2. If yes: extract the field into a dedicated guard type using `ManuallyDrop`.
3. If no: `impl Drop` is fine — document field drop order (struct fields drop in declaration order).

Pattern:
```rust
#[repr(transparent)]
struct Guard(std::mem::ManuallyDrop<Resource>);
impl Drop for Guard {
    fn drop(&mut self) {
        // SAFETY: only called once from Drop
        let r = unsafe { std::mem::ManuallyDrop::take(&mut self.0) };
        r.close();
    }
}
```

Reference: `crabbook/you_dont_want_drop.md`

## `fn(T) -> T` vs `fn(&mut T)` for large structs

**Severity: WARNING on hot paths**

Heimdall has several large structs that appear in hot-ish paths:

| File | Struct(s) | Approx size |
|---|---|---|
| `src/models.rs` | session, turn, pricing models | 43 KB file |
| `src/pricing.rs` | pricing calculation state | 39 KB file |
| `src/pricing_sync.rs` | sync pricing state | 96 KB file |
| `src/config.rs` | `Config` | 55 KB file |

For structs larger than 4 pointer-sized fields, `fn(T) -> T` forces a `memcpy` in and out — the compiler cannot optimize back to in-place mutation because panic semantics require the original to remain valid during the function. On a path that processes many turns or sessions, this silently multiplies allocation cost.

Rule:
- Use `fn(&mut T)` for any processing of the large structs above on paths that run per-session, per-turn, or per-pricing-event.
- Use `fn(T) -> T` only for explicit ownership-transfer semantics (builder, state-machine transition on small structs).
- Profile with `cargo flamegraph` before adopting value-passing chains for large structs.

Reference: `crabbook/consume_and_borrowing.md`

## `#[derive(Clone)]` on resource-backed types

**Severity: WARNING**

Deriving `Clone` on a struct containing `Arc<T>` does not duplicate the resource — it clones the handle, giving two owners pointing to the same resource. This is often the correct behavior, but must be documented. The hazard is when callers expect an isolated copy (for testing, per-connection isolation) but get aliasing instead.

In heimdall: `Config`, `AppState`, and database pool handles are passed with `.clone()` throughout `src/server/` and `src/scanner/`. Verify that all such clones intentionally share the underlying resource and add a doc comment: `/// Cloning shares the underlying pool/config.`

## `Deref` on non-pointer types causes method collision

**Severity: WARNING**

`Deref<Target = T>` on a non-smart-pointer newtype exposes all of `T`'s methods via auto-deref. If either `Wrapper` or `T` later adds a method with the same name, resolution changes silently — no compile error, different behavior. This also forecloses adding bounds in future versions without breaking callers.

Use `AsRef<T>`, `From<T>`/`Into<T>`, or explicit accessor methods instead of `Deref` on application newtypes. Check `src/models.rs` and `src/scanner/` for any non-pointer `Deref` implementations.

## `async fn` in traits not object-safe, no `Send` bound

**Severity: WARNING**

`async fn` in traits (stable Rust 1.75) is not `dyn`-safe and does not automatically add `Send` to the returned future. Teams replacing `#[async_trait]` with native syntax hit:
1. `Box<dyn MyTrait>` compile errors (not object-safe).
2. `tokio::spawn(obj.method())` failures (future not `Send`).

Fix: use `#[trait_variant::make(MyTraitSend: Send)]` from the `trait-variant` crate, or keep `#[async_trait]` for traits that need `dyn` dispatch. Do not mass-replace `#[async_trait]` in heimdall without auditing every `tokio::spawn` and `Box<dyn>` use site in `src/server/` and `src/scanner/`.

## Quick review checklist

Apply to every changed public or `pub(crate)` signature in a diff:

1. Any `&String`, `&Vec<T>`, or `&PathBuf` parameter? → prefer `&str`, `&[T]`, `&Path`, or `impl AsRef<...>`.
2. Any `&mut State` captured in a `tokio::spawn` closure? → use `Arc<Mutex<T>>` or message-passing.
3. Any closure callback failing HRTB inference? → name the function or use `force_hrtb` shim.
4. Any new `impl Drop` on a struct with a field that callers need to consume? → use `ManuallyDrop` guard.
5. Any `fn(T) -> T` consuming a large struct (> 4 pointer fields) in per-session/per-turn code? → use `fn(&mut T)`.
6. Any `#[derive(Clone)]` on a struct with `Arc<T>` fields where callers might expect an isolated copy? → add doc comment clarifying sharing semantics.
7. Any `Deref` on a non-smart-pointer newtype? → use `AsRef`/`From`/accessors instead.
8. Any `async fn` in a trait now used with `dyn Trait` or `tokio::spawn`? → add `trait_variant` or keep `#[async_trait]`.
