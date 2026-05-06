# Rust API Discipline

Apply these rules to every changed public or `pub(crate)` function signature in a diff.

## Borrowed args over owned references

Flag `&String`, `&Vec<T>`, `&PathBuf` in function signatures — prefer `&str`, `&[T]`, `&Path`, or `impl AsRef<...>`.

Risk areas: `src/scanner/`, `src/server/`, `src/oauth/`.

```bash
rg 'fn .+\(&String|fn .+\(&Vec<|fn .+\(&PathBuf' src/ --type rust -n
```

## `Fn` callbacks with reference args in async contexts

Closures in `tokio::spawn` must be `'static + Send`. Never capture `&mut State`.

Options:
- `Arc<Mutex<State>>` for low-contention shared state.
- `mpsc::channel` for high-frequency events (scanner, SSE).

Risk areas: `src/server/sse.rs`, `src/scanner/watcher.rs`.

## HRTB closure-inference workaround

When a closure fails HRTB inference, name the function or use:
```rust
fn force_hrtb<F: for<'a> Fn(&'a str) -> &'a str>(f: F) -> F { f }
```

## `impl Drop` blocks partial moves

Before adding `impl Drop`: check if any field needs to be consumed. If yes, use a `ManuallyDrop` guard type.

## Large struct value-passing

`fn(T) -> T` for large structs forces `memcpy` per call. Use `fn(&mut T)` for per-session/per-turn processing of:
- `src/models.rs` structs (43 KB file)
- `src/pricing.rs` structs (39 KB file)
- `src/pricing_sync.rs` structs (96 KB file)
- `src/config.rs` `Config` (55 KB file)

## Quick checklist

1. `&String`/`&Vec<T>`/`&PathBuf` params? → borrowed form.
2. `&mut State` in `tokio::spawn`? → `Arc<Mutex<T>>` or channel.
3. Closure HRTB failure? → name function or `force_hrtb`.
4. New `impl Drop` with consume-able field? → `ManuallyDrop` guard.
5. `fn(T)->T` on large struct in hot path? → `fn(&mut T)`.

## `#[derive(Clone)]` on resource-backed types

Cloning an `Arc<T>` shares the resource, not copies it. Document with: `/// Cloning shares the underlying pool/config.`

## `Deref` on non-pointer types

Causes silent method collision when either the wrapper or the target adds a same-named method. Use `AsRef`/`From`/accessors instead.

## `async fn` in traits — not object-safe, no `Send`

Native `async fn in trait` (Rust 1.75) is not `dyn`-safe and has no `Send` bound. For `tokio::spawn` contexts use `#[trait_variant::make(TraitSend: Send)]`. Do not mass-replace `#[async_trait]` without auditing every `Box<dyn>` and spawn site.
