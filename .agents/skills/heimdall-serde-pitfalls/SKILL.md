---
name: heimdall-serde-pitfalls
description: Serde deserialization hazards in heimdall — untagged enum error swallowing, flatten+deny_unknown_fields bug, internally tagged tuple variants, and roundtrip testing gaps.
---

# Heimdall Serde Pitfalls

## Purpose

Guide review and authoring of `serde` serialization/deserialization code in heimdall. Apply every rule to every `#[derive(Serialize, Deserialize)]` and every serde attribute in a diff. Heimdall uses serde heavily in `src/models.rs`, `src/config.rs`, `src/server/` API types, and webhook payloads.

## `#[serde(untagged)]` swallows all variant errors

**Severity: WARNING**

When deserializing an `#[serde(untagged)]` enum, serde tries each variant in declaration order and discards every variant's specific error. On total failure, it reports only: `"data did not match any variant of untagged enum Foo"`. You lose the specific field name, type mismatch, or missing key that caused each failure.

```rust
#[derive(Deserialize)]
#[serde(untagged)]
enum WebhookPayload {
    Anthropic(AnthropicEvent),
    OpenAI(OpenAIEvent),
    Gemini(GeminiEvent),
}
// On failure: "data did not match any variant" -- completely opaque in prod logs
```

Hazard: this makes debugging silent rejections in production webhook handlers require log enhancement or manual reproduction. The error is often discovered only after hours of investigation under production load.

Mitigations (in order of preference):
1. Use `#[serde(tag = "source")]` instead if the payload contains a type discriminant field.
2. Use `#[serde(try_from = "serde_json::Value")]` and implement `TryFrom` with explicit variant matching that returns typed errors.
3. Add a `#[serde(other)]` fallback variant that captures the raw `Value` for logging.
4. Use the `serde-untagged` crate which preserves per-variant errors.

In heimdall: any `#[serde(untagged)]` on an externally-sourced type (Claude API responses, provider webhook payloads) is high risk. Prefer tagged unions with a discriminant field.

## `#[serde(flatten)]` + `#[serde(deny_unknown_fields)]` silently broken

**Severity: WARNING**

Combining `#[serde(flatten)]` on a field with `#[serde(deny_unknown_fields)]` on the outer struct is **explicitly unsupported** in serde, but serde does not emit a compile-time error. At runtime, `deny_unknown_fields` silently does not work — unknown fields pass through without rejection.

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]  // <-- silently broken when flatten is present
struct Config {
    name: String,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
// Unknown fields are NOT rejected despite deny_unknown_fields
```

Additionally: `#[serde(flatten)]` on an internally-tagged enum variant (`#[serde(tag = "...")]`) fails to deserialize, returning "can only flatten structs and maps" — even though serialization works. This asymmetry means `serialize → deserialize` roundtrip tests on the happy path miss the broken behavior entirely.

In heimdall: audit every struct that combines `flatten` with `deny_unknown_fields`. Treat the combination as non-functional for unknown-field rejection. Use a custom `Deserialize` implementation or a dedicated validator struct instead.

Grep: `rg 'serde\(flatten\)' src/ --type rust -l` then check each file for `deny_unknown_fields` on the parent struct.

## Internally tagged enum rejects tuple variants

**Severity: SUGGESTION**

`#[serde(tag = "type")]` internally tagged and `#[serde(tag = "type", content = "data")]` adjacently tagged enum representations only work for:
- Unit variants (`Foo::Bar`)
- Newtype variants wrapping a struct/map (`Foo::Bar(StructType)`)
- Struct variants (`Foo::Bar { field: T }`)

Tuple variants (`Foo::Bar(u32, String)`) cause a **compile-time error** — but only at the point the variant is added. Teams define the enum shape early, then later add a tuple variant and get a confusing error referencing the `tag` attribute far from the new variant.

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Event {
    Session { id: u64 },     // works
    Error(ErrorInfo),        // works (newtype wrapping struct)
    // Raw(u32, String),     // compile error -- tuple variant not supported
}
```

Fix: wrap tuple data in a named struct before using it in a tagged enum variant.

In heimdall: before adding a new variant to any tagged enum in `src/models.rs` or API response types, confirm the variant shape is struct-like, not tuple-like.

## Roundtrip testing gap

**Severity: WARNING**

Serde bugs (including the `flatten`+`deny_unknown_fields` interaction and `#[serde(untagged)]` error swallowing) are only discovered at runtime because serialization and deserialization are independent code paths. A test that only serializes to JSON and checks the output does not catch deserialization failures.

Rule: for every `#[derive(Serialize, Deserialize)]` type in heimdall's API surface or config types, add a roundtrip test:
```rust
#[test]
fn roundtrip_my_type() {
    let original = MyType { /* ... */ };
    let json = serde_json::to_string(&original).unwrap();
    let restored: MyType = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}
```

Also test deserialization of an invalid payload — verify the error message contains actionable information, not just "data did not match any variant".

## Quick review checklist

Apply to every serde-annotated type in a diff:

1. Any `#[serde(untagged)]` on an externally-sourced type? → verify error messages are actionable or use a tagged alternative.
2. Any `#[serde(flatten)]` combined with `#[serde(deny_unknown_fields)]`? → the combination is non-functional; remove `deny_unknown_fields` or use a custom Deserialize.
3. Any new variant added to a `#[serde(tag = "...")]` enum? → confirm it's a struct/newtype variant, not a tuple variant.
4. Any new `#[derive(Serialize, Deserialize)]` type without a roundtrip test? → add one.
5. Any serde enum used in a `serde_json::from_str` path without testing the error case? → add a negative test.
