# Serde Pitfalls

Review serde deserialization code in heimdall. Apply to every `#[derive(Serialize, Deserialize)]` and serde attribute in a diff.

## `#[serde(untagged)]` swallows all variant errors

On failure, reports only "data did not match any variant" — all specific errors lost. Hazard for production webhook handlers.

Mitigations: use `#[serde(tag = "...")]` if a discriminant field exists; use `serde-untagged` crate; add `#[serde(other)]` fallback.

Grep: `rg 'serde.*untagged' src/ --type rust -n`

## `#[serde(flatten)]` + `#[serde(deny_unknown_fields)]` silently broken

The combination is unsupported — unknown fields are NOT rejected despite the attribute. Also: `flatten` on internally-tagged enum variants fails deserialization while serialization succeeds (asymmetry roundtrip tests miss).

Grep: `rg 'serde\(flatten\)' src/ --type rust -l` then check parent for `deny_unknown_fields`.

## Internally tagged enum rejects tuple variants

`#[serde(tag)]` / `#[serde(tag, content)]` only supports unit, newtype-of-struct, and struct variants. Tuple variants cause a compile error at the point the variant is added — not at the enum definition.

## Roundtrip testing gap

Every API/config type needs both a serialize test AND a deserialize test (including an invalid-payload negative test).

## Quick checklist

1. `#[serde(untagged)]` on external type? → ensure actionable errors or use tagged alternative.
2. `flatten` + `deny_unknown_fields` combined? → remove or use custom Deserialize.
3. New variant in tagged enum? → confirm struct/newtype shape, not tuple.
4. New Serialize+Deserialize type? → add roundtrip test + negative deserialization test.
