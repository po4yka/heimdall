use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use crate::litellm::LiteLlmSnapshot;

pub const PRICING_VERSION: &str = "2026-04-29";
pub const PRICING_VALID_FROM: &str = "2026-04-29T00:00:00Z";
pub const COST_CONFIDENCE_HIGH: &str = "high";
pub const COST_CONFIDENCE_MEDIUM: &str = "medium";
pub const COST_CONFIDENCE_LOW: &str = "low";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelPricing {
    pub input: f64,
    pub output: f64,
    pub cache_write: f64,
    pub cache_read: f64,
    /// If total input+output tokens exceed this threshold, tokens above it
    /// are billed at the `*_above_threshold` rates instead.
    pub threshold_tokens: Option<i64>,
    pub input_above_threshold: Option<f64>,
    pub output_above_threshold: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct CostEstimate {
    pub estimated_cost_nanos: i64,
    pub pricing_version: String,
    pub pricing_model: String,
    pub cost_confidence: String,
}

/// Hot-swappable pricing override map. Reads take a brief read lock; writes
/// (`set_overrides`, `replace_overrides`) take an exclusive lock and swap
/// the entire `HashMap` atomically. Empty by default until populated at
/// startup or by a settings PATCH.
static PRICING_OVERRIDES: RwLock<Option<HashMap<String, ModelPricing>>> = RwLock::new(None);

/// Install custom pricing overrides from config. Call once at startup.
///
/// First call seeds the map; subsequent calls replace it wholesale (matching
/// the legacy `OnceLock::set` semantics for the startup path while still
/// allowing later hot-reloads via `replace_overrides`).
pub fn set_overrides(overrides: HashMap<String, ModelPricing>) {
    replace_overrides(overrides);
}

/// Atomically swap the entire pricing override map. Called by the settings
/// PATCH handler after a successful write so newly-saved overrides take effect
/// on the next cost calculation without restarting the process.
///
/// The 5-tier pricing fallback (exact hardcoded → prefix hardcoded → keyword
/// hardcoded → LiteLLM cache → unknown) is preserved unchanged — overrides
/// remain the topmost tier; this only replaces the storage backing them.
pub fn replace_overrides(overrides: HashMap<String, ModelPricing>) {
    match PRICING_OVERRIDES.write() {
        Ok(mut guard) => {
            *guard = Some(overrides);
        }
        Err(poisoned) => {
            // RwLock poisoning is recoverable for our use case: a previously
            // panicking writer left no partially-initialized state since we
            // always assign a fresh map.
            tracing::warn!("pricing override RwLock was poisoned; recovering");
            let mut guard = poisoned.into_inner();
            *guard = Some(overrides);
        }
    }
}

/// Look up a single override entry. Used by the 5-tier fallback in
/// `lookup_pricing` — copies the `ModelPricing` (a `Copy` value type) so the
/// read lock can be released before returning.
fn get_override(model: &str) -> Option<ModelPricing> {
    let guard = PRICING_OVERRIDES.read().ok()?;
    guard.as_ref()?.get(model).copied()
}

/// Snapshot the current override map. Test-only helper used to verify that
/// `replace_overrides` swapped successfully without coupling tests to the
/// `RwLock` internals.
#[cfg(test)]
pub fn current_overrides() -> HashMap<String, ModelPricing> {
    PRICING_OVERRIDES
        .read()
        .ok()
        .and_then(|g| g.as_ref().cloned())
        .unwrap_or_default()
}

// ── LiteLLM pricing source ────────────────────────────────────────────────────

/// Runtime LiteLLM pricing map (populated once at startup when source = LiteLlm).
/// Guarded by OnceLock so it is safe to set from main and read from all threads.
static LITELLM_MAP: OnceLock<HashMap<String, ModelPricing>> = OnceLock::new();

/// Install a LiteLLM-sourced pricing map. Called at startup when config says
/// `source = "litellm"`. Safe to call multiple times — subsequent calls are
/// no-ops (OnceLock semantics).
pub fn set_litellm_map(map: HashMap<String, ModelPricing>) {
    let _ = LITELLM_MAP.set(map);
}

/// Convert an already-loaded `LiteLlmSnapshot` into a `ModelPricing` map.
/// Entries missing either rate are silently skipped.
pub fn load_litellm_cache_from_snapshot(
    snapshot: LiteLlmSnapshot,
) -> HashMap<String, ModelPricing> {
    snapshot
        .entries
        .into_iter()
        .filter_map(|(key, entry)| {
            // Both rates must be present to form a usable ModelPricing.
            let input = entry.input_cost_per_token?;
            let output = entry.output_cost_per_token?;
            Some((
                key,
                ModelPricing {
                    input,
                    output,
                    // LiteLLM doesn't always expose cache rates; use standard
                    // multipliers as safe defaults (same as config override logic).
                    cache_write: input * 1.25,
                    cache_read: input * 0.1,
                    threshold_tokens: None,
                    input_above_threshold: None,
                    output_above_threshold: None,
                },
            ))
        })
        .collect()
}

/// Look up a model in the LiteLLM map. Returns None if the map is unset or
/// the model is absent.
fn get_litellm(model: &str) -> Option<&'static ModelPricing> {
    LITELLM_MAP.get()?.get(model)
}

const PRICING_TABLE: &[(&str, ModelPricing)] = &[
    (
        "gpt-5.5",
        ModelPricing {
            input: 5.00,
            output: 30.0,
            cache_write: 5.00,
            cache_read: 0.50,
            threshold_tokens: None,
            input_above_threshold: None,
            output_above_threshold: None,
        },
    ),
    (
        "gpt-5.4",
        ModelPricing {
            input: 2.50,
            output: 15.0,
            cache_write: 2.50,
            cache_read: 0.25,
            threshold_tokens: None,
            input_above_threshold: None,
            output_above_threshold: None,
        },
    ),
    (
        "gpt-5.4-mini",
        ModelPricing {
            input: 0.75,
            output: 4.50,
            cache_write: 0.75,
            cache_read: 0.075,
            threshold_tokens: None,
            input_above_threshold: None,
            output_above_threshold: None,
        },
    ),
    (
        "gpt-5.4-nano",
        ModelPricing {
            input: 0.20,
            output: 1.25,
            cache_write: 0.20,
            cache_read: 0.02,
            threshold_tokens: None,
            input_above_threshold: None,
            output_above_threshold: None,
        },
    ),
    (
        "gpt-5.3-codex",
        ModelPricing {
            input: 1.75,
            output: 14.0,
            cache_write: 1.75,
            cache_read: 0.175,
            threshold_tokens: None,
            input_above_threshold: None,
            output_above_threshold: None,
        },
    ),
    (
        "claude-opus-4-6",
        ModelPricing {
            input: 15.0,
            output: 75.0,
            cache_write: 18.75,
            cache_read: 1.50,
            threshold_tokens: None,
            input_above_threshold: None,
            output_above_threshold: None,
        },
    ),
    (
        "claude-opus-4-5",
        ModelPricing {
            input: 15.0,
            output: 75.0,
            cache_write: 18.75,
            cache_read: 1.50,
            threshold_tokens: None,
            input_above_threshold: None,
            output_above_threshold: None,
        },
    ),
    (
        "claude-sonnet-4-6",
        ModelPricing {
            input: 3.0,
            output: 15.0,
            cache_write: 3.75,
            cache_read: 0.30,
            threshold_tokens: None,
            input_above_threshold: None,
            output_above_threshold: None,
        },
    ),
    (
        "claude-sonnet-4-5",
        ModelPricing {
            input: 3.0,
            output: 15.0,
            cache_write: 3.75,
            cache_read: 0.30,
            threshold_tokens: Some(200_000),
            input_above_threshold: Some(6.0),
            output_above_threshold: Some(22.5),
        },
    ),
    (
        "claude-haiku-4-5",
        ModelPricing {
            input: 1.0,
            output: 5.0,
            cache_write: 1.25,
            cache_read: 0.10,
            threshold_tokens: None,
            input_above_threshold: None,
            output_above_threshold: None,
        },
    ),
    (
        "claude-haiku-4-6",
        ModelPricing {
            input: 1.0,
            output: 5.0,
            cache_write: 1.25,
            cache_read: 0.10,
            threshold_tokens: None,
            input_above_threshold: None,
            output_above_threshold: None,
        },
    ),
];

pub fn builtin_catalog() -> HashMap<String, ModelPricing> {
    PRICING_TABLE
        .iter()
        .map(|(name, pricing)| ((*name).to_string(), *pricing))
        .collect()
}

/// One entry in the response of `GET /api/pricing-models`. Surfaces the
/// hardcoded default rates so the Settings UI's model-picker can render
/// "Claude Sonnet 4.6 — $3 / $15" hints next to override fields.
///
/// Rates come from the `PRICING_TABLE` exact-match tier only — overrides
/// (whether config-loaded or PATCH'd) are intentionally excluded so the
/// response always reflects the *built-in default*, not the live override.
#[derive(Debug, Clone, serde::Serialize)]
pub struct KnownModel {
    pub model: String,
    pub family: String,
    pub default_input: f64,
    pub default_output: f64,
    pub default_cache_write: Option<f64>,
    pub default_cache_read: Option<f64>,
}

/// Derive a coarse family label from a model name. Used for grouping in the
/// UI's model-picker. Matches the same prefix logic as `lookup_pricing`'s
/// keyword tier so the family agrees with where pricing falls back to.
fn model_family(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.starts_with("claude") {
        "claude".to_string()
    } else if lower.starts_with("gpt") || lower.contains("codex") {
        "openai".to_string()
    } else if lower.starts_with("gemini") {
        "google".to_string()
    } else if lower.starts_with("grok") {
        "xai".to_string()
    } else if lower.starts_with("llama") {
        "meta".to_string()
    } else {
        "other".to_string()
    }
}

/// Return all hardcoded models heimdall knows about, sorted alphabetically by
/// model name. Used by `GET /api/pricing-models` to populate the Settings UI's
/// model autocomplete.
///
/// Source: `PRICING_TABLE` exact-match tier. LiteLLM and override entries are
/// not included — the UI requests *defaults* so it can show users the rates
/// they would override.
pub fn known_models() -> Vec<KnownModel> {
    let mut models: Vec<KnownModel> = PRICING_TABLE
        .iter()
        .map(|(name, p)| KnownModel {
            model: (*name).to_string(),
            family: model_family(name),
            default_input: p.input,
            default_output: p.output,
            default_cache_write: Some(p.cache_write),
            default_cache_read: Some(p.cache_read),
        })
        .collect();
    models.sort_by(|a, b| a.model.cmp(&b.model));
    models
}

/// Look up pricing for a model across overrides, built-ins, heuristic
/// fallbacks, and the LiteLLM cache.
///
/// Test-only helper: production code goes through `calc_cost_nanos` /
/// `estimate_cost`, which call `lookup_pricing` directly.
#[cfg(test)]
pub fn get_pricing(model: &str) -> Option<ModelPricing> {
    lookup_pricing(model).map(|l| l.pricing)
}

/// Result of `lookup_pricing`: pricing rates plus the resolved canonical
/// model name and confidence tier. Held by value (`ModelPricing` is `Copy`)
/// so the override lookup can release its read lock immediately and so all
/// five tiers — overrides, hardcoded, prefix, keyword, LiteLLM — share one
/// shape regardless of underlying storage.
struct PricingLookup {
    pricing: ModelPricing,
    pricing_model: String,
    cost_confidence: &'static str,
}

fn lookup_pricing(model: &str) -> Option<PricingLookup> {
    if model.is_empty() {
        return None;
    }

    if let Some(p) = get_override(model) {
        return Some(PricingLookup {
            pricing: p,
            pricing_model: model.to_string(),
            cost_confidence: COST_CONFIDENCE_HIGH,
        });
    }

    for (name, pricing) in PRICING_TABLE {
        if *name == model {
            return Some(PricingLookup {
                pricing: *pricing,
                pricing_model: (*name).to_string(),
                cost_confidence: COST_CONFIDENCE_HIGH,
            });
        }
    }

    // Prefix tier: longest matching key wins so that e.g. "gpt-5.4-mini-2026-01"
    // resolves to "gpt-5.4-mini" and not the shorter "gpt-5.4" entry.
    let prefix_match = PRICING_TABLE
        .iter()
        .filter(|(name, _)| model.starts_with(name))
        .max_by_key(|(name, _)| name.len());
    if let Some((name, pricing)) = prefix_match {
        return Some(PricingLookup {
            pricing: *pricing,
            pricing_model: (*name).to_string(),
            cost_confidence: COST_CONFIDENCE_HIGH,
        });
    }

    let lower = model.to_lowercase();
    if lower.contains("opus") {
        return get_builtin("claude-opus-4-6").map(|pricing| PricingLookup {
            pricing: *pricing,
            pricing_model: "claude-opus-4-6".to_string(),
            cost_confidence: COST_CONFIDENCE_MEDIUM,
        });
    }
    if lower.contains("sonnet") {
        return get_builtin("claude-sonnet-4-6").map(|pricing| PricingLookup {
            pricing: *pricing,
            pricing_model: "claude-sonnet-4-6".to_string(),
            cost_confidence: COST_CONFIDENCE_MEDIUM,
        });
    }
    if lower.contains("haiku") {
        return get_builtin("claude-haiku-4-5").map(|pricing| PricingLookup {
            pricing: *pricing,
            pricing_model: "claude-haiku-4-5".to_string(),
            cost_confidence: COST_CONFIDENCE_MEDIUM,
        });
    }
    if lower.contains("gpt-5.5") {
        return get_builtin("gpt-5.5").map(|pricing| PricingLookup {
            pricing: *pricing,
            pricing_model: "gpt-5.5".to_string(),
            cost_confidence: COST_CONFIDENCE_MEDIUM,
        });
    }
    if lower.contains("gpt-5.4-mini") {
        return get_builtin("gpt-5.4-mini").map(|pricing| PricingLookup {
            pricing: *pricing,
            pricing_model: "gpt-5.4-mini".to_string(),
            cost_confidence: COST_CONFIDENCE_MEDIUM,
        });
    }
    if lower.contains("gpt-5.4-nano") {
        return get_builtin("gpt-5.4-nano").map(|pricing| PricingLookup {
            pricing: *pricing,
            pricing_model: "gpt-5.4-nano".to_string(),
            cost_confidence: COST_CONFIDENCE_MEDIUM,
        });
    }
    if lower.contains("gpt-5.4") {
        return get_builtin("gpt-5.4").map(|pricing| PricingLookup {
            pricing: *pricing,
            pricing_model: "gpt-5.4".to_string(),
            cost_confidence: COST_CONFIDENCE_MEDIUM,
        });
    }
    if lower.contains("codex") {
        return get_builtin("gpt-5.3-codex").map(|pricing| PricingLookup {
            pricing: *pricing,
            pricing_model: "gpt-5.3-codex".to_string(),
            cost_confidence: COST_CONFIDENCE_MEDIUM,
        });
    }

    // Tier 5: LiteLLM cache — only reached for models NOT matched by any hardcoded
    // tier above.  This guarantees Claude/GPT-5 families always use hardcoded prices.
    if let Some(pricing) = get_litellm(model) {
        return Some(PricingLookup {
            pricing: *pricing,
            pricing_model: model.to_string(),
            cost_confidence: COST_CONFIDENCE_LOW,
        });
    }

    None
}

fn lookup_catalog_pricing(
    model: &str,
    catalog: &HashMap<String, ModelPricing>,
) -> Option<PricingLookup> {
    if model.is_empty() {
        return None;
    }

    if let Some(pricing) = catalog.get(model) {
        return Some(PricingLookup {
            pricing: *pricing,
            pricing_model: model.to_string(),
            cost_confidence: COST_CONFIDENCE_HIGH,
        });
    }

    for (name, pricing) in catalog {
        if model.starts_with(name) {
            return Some(PricingLookup {
                pricing: *pricing,
                pricing_model: name.clone(),
                cost_confidence: COST_CONFIDENCE_HIGH,
            });
        }
    }

    let lower = model.to_lowercase();
    let fallback = if lower.contains("opus") {
        [
            "claude-opus-4-7",
            "claude-opus-4-6",
            "claude-opus-4-5",
            "claude-opus-4-1",
        ]
        .iter()
        .find(|key| catalog.contains_key(**key))
        .map(|key| (*key).to_string())
    } else if lower.contains("sonnet") {
        ["claude-sonnet-4-6", "claude-sonnet-4-5", "claude-sonnet-4"]
            .iter()
            .find(|key| catalog.contains_key(**key))
            .map(|key| (*key).to_string())
    } else if lower.contains("haiku") {
        [
            "claude-haiku-4-6",
            "claude-haiku-4-5",
            "claude-haiku-3-5",
            "claude-haiku-3",
        ]
        .iter()
        .find(|key| catalog.contains_key(**key))
        .map(|key| (*key).to_string())
    } else if lower.contains("gpt-5.5") {
        catalog
            .contains_key("gpt-5.5")
            .then(|| "gpt-5.5".to_string())
    } else if lower.contains("gpt-5.4-mini") {
        catalog
            .contains_key("gpt-5.4-mini")
            .then(|| "gpt-5.4-mini".to_string())
    } else if lower.contains("gpt-5.4-nano") {
        catalog
            .contains_key("gpt-5.4-nano")
            .then(|| "gpt-5.4-nano".to_string())
    } else if lower.contains("gpt-5.4") {
        catalog
            .contains_key("gpt-5.4")
            .then(|| "gpt-5.4".to_string())
    } else if lower.contains("codex") {
        ["gpt-5.3-codex"]
            .iter()
            .find(|key| catalog.contains_key(**key))
            .map(|key| (*key).to_string())
    } else {
        None
    }?;

    let pricing = catalog.get(&fallback)?;
    Some(PricingLookup {
        pricing: *pricing,
        pricing_model: fallback,
        cost_confidence: COST_CONFIDENCE_MEDIUM,
    })
}

/// Look up only from built-in table (avoids infinite recursion in substring fallback).
fn get_builtin(model: &str) -> Option<&'static ModelPricing> {
    PRICING_TABLE
        .iter()
        .find(|(name, _)| *name == model)
        .map(|(_, p)| p)
}

/// Calculate cost in nanos (1 dollar = 1_000_000_000 nanos) for the given token counts.
///
/// This avoids floating-point drift when summing many small costs.
/// Rate is $/MTok, so: cost_nanos = tokens * (rate / 1e6) * 1e9 = tokens * rate * 1000.
pub fn calc_cost_nanos(
    model: &str,
    input: i64,
    output: i64,
    cache_read: i64,
    cache_creation: i64,
) -> i64 {
    let Some(lookup) = lookup_pricing(model) else {
        return 0;
    };
    calc_cost_nanos_with_pricing(&lookup.pricing, input, output, cache_read, cache_creation)
}

fn calc_cost_nanos_with_pricing(
    p: &ModelPricing,
    input: i64,
    output: i64,
    cache_read: i64,
    cache_creation: i64,
) -> i64 {
    let total_tokens = input + output;

    let (input_cost, output_cost) =
        if let (Some(threshold), Some(input_above), Some(output_above)) = (
            p.threshold_tokens,
            p.input_above_threshold,
            p.output_above_threshold,
        ) {
            if total_tokens > threshold {
                // Split each of input and output proportionally around the threshold.
                // The proportion of tokens that fall below the threshold:
                let below_ratio = threshold as f64 / total_tokens as f64;

                let input_below = (input as f64 * below_ratio) as i64;
                let input_above_count = input - input_below;
                let input_c = (input_below as f64 * p.input * 1000.0) as i64
                    + (input_above_count as f64 * input_above * 1000.0) as i64;

                let output_below = (output as f64 * below_ratio) as i64;
                let output_above_count = output - output_below;
                let output_c = (output_below as f64 * p.output * 1000.0) as i64
                    + (output_above_count as f64 * output_above * 1000.0) as i64;

                (input_c, output_c)
            } else {
                (
                    (input as f64 * p.input * 1000.0) as i64,
                    (output as f64 * p.output * 1000.0) as i64,
                )
            }
        } else {
            (
                (input as f64 * p.input * 1000.0) as i64,
                (output as f64 * p.output * 1000.0) as i64,
            )
        };

    let cache_read_cost = (cache_read as f64 * p.cache_read * 1000.0) as i64;
    let cache_write_cost = (cache_creation as f64 * p.cache_write * 1000.0) as i64;

    input_cost + output_cost + cache_read_cost + cache_write_cost
}

/// Per-type cost breakdown for a single pricing calculation.
///
/// All four components use the same floor-rounding (truncating cast to i64) as
/// `calc_cost_nanos_with_pricing`, so their sum equals the value returned by
/// `estimate_cost` for identical inputs — guaranteed by unit tests.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct CostBreakdown {
    pub input_cost_nanos: i64,
    pub output_cost_nanos: i64,
    pub cache_read_cost_nanos: i64,
    pub cache_write_cost_nanos: i64,
}

impl CostBreakdown {
    /// Sum of all four components.  Equal to `estimate_cost`'s
    /// `estimated_cost_nanos` for identical inputs.
    pub fn total_nanos(&self) -> i64 {
        self.input_cost_nanos
            + self.output_cost_nanos
            + self.cache_read_cost_nanos
            + self.cache_write_cost_nanos
    }
}

/// Compute a 4-way cost breakdown in integer nanos.
///
/// Returns `(CostBreakdown, pricing_version, pricing_model, cost_confidence)`.
/// Mirrors `estimate_cost` exactly: same pricing lookup, same per-token rate
/// formula (`tokens * rate * 1000.0) as i64`), so that
/// `breakdown.total_nanos() == estimate_cost(…).estimated_cost_nanos` always holds.
///
/// When the model is unknown, all four components are zero and `cost_confidence`
/// is `"low"`.
pub fn estimate_cost_breakdown(
    model: &str,
    input_tokens: i64,
    output_tokens: i64,
    cache_read_tokens: i64,
    cache_creation_tokens: i64,
) -> (CostBreakdown, String, String, String) {
    let Some(lookup) = lookup_pricing(model) else {
        return (
            CostBreakdown::default(),
            PRICING_VERSION.to_string(),
            String::new(),
            COST_CONFIDENCE_LOW.to_string(),
        );
    };

    let PricingLookup {
        pricing,
        pricing_model,
        cost_confidence,
    } = lookup;

    let total_tokens = input_tokens + output_tokens;

    let (input_cost_nanos, output_cost_nanos) =
        if let (Some(threshold), Some(input_above), Some(output_above)) = (
            pricing.threshold_tokens,
            pricing.input_above_threshold,
            pricing.output_above_threshold,
        ) {
            if total_tokens > threshold {
                let below_ratio = threshold as f64 / total_tokens as f64;

                let input_below = (input_tokens as f64 * below_ratio) as i64;
                let input_above_count = input_tokens - input_below;
                let ic = (input_below as f64 * pricing.input * 1000.0) as i64
                    + (input_above_count as f64 * input_above * 1000.0) as i64;

                let output_below = (output_tokens as f64 * below_ratio) as i64;
                let output_above_count = output_tokens - output_below;
                let oc = (output_below as f64 * pricing.output * 1000.0) as i64
                    + (output_above_count as f64 * output_above * 1000.0) as i64;

                (ic, oc)
            } else {
                (
                    (input_tokens as f64 * pricing.input * 1000.0) as i64,
                    (output_tokens as f64 * pricing.output * 1000.0) as i64,
                )
            }
        } else {
            (
                (input_tokens as f64 * pricing.input * 1000.0) as i64,
                (output_tokens as f64 * pricing.output * 1000.0) as i64,
            )
        };

    let cache_read_cost_nanos = (cache_read_tokens as f64 * pricing.cache_read * 1000.0) as i64;
    let cache_write_cost_nanos =
        (cache_creation_tokens as f64 * pricing.cache_write * 1000.0) as i64;

    let breakdown = CostBreakdown {
        input_cost_nanos,
        output_cost_nanos,
        cache_read_cost_nanos,
        cache_write_cost_nanos,
    };

    (
        breakdown,
        format!("{PRICING_VERSION}@{PRICING_VALID_FROM}"),
        pricing_model,
        cost_confidence.to_string(),
    )
}

pub fn estimate_cost(
    model: &str,
    input: i64,
    output: i64,
    cache_read: i64,
    cache_creation: i64,
) -> CostEstimate {
    let Some(lookup) = lookup_pricing(model) else {
        return CostEstimate {
            estimated_cost_nanos: 0,
            pricing_version: PRICING_VERSION.to_string(),
            pricing_model: String::new(),
            cost_confidence: COST_CONFIDENCE_LOW.to_string(),
        };
    };

    let PricingLookup {
        pricing,
        pricing_model,
        cost_confidence,
    } = lookup;

    CostEstimate {
        estimated_cost_nanos: calc_cost_nanos_with_pricing(
            &pricing,
            input,
            output,
            cache_read,
            cache_creation,
        ),
        pricing_version: format!("{PRICING_VERSION}@{PRICING_VALID_FROM}"),
        pricing_model,
        cost_confidence: cost_confidence.to_string(),
    }
}

pub fn estimate_cost_with_catalog(
    model: &str,
    input: i64,
    output: i64,
    cache_read: i64,
    cache_creation: i64,
    catalog: &HashMap<String, ModelPricing>,
    pricing_version: &str,
) -> CostEstimate {
    let Some(lookup) = lookup_catalog_pricing(model, catalog) else {
        return CostEstimate {
            estimated_cost_nanos: 0,
            pricing_version: pricing_version.to_string(),
            pricing_model: String::new(),
            cost_confidence: COST_CONFIDENCE_LOW.to_string(),
        };
    };

    let PricingLookup {
        pricing,
        pricing_model,
        cost_confidence,
    } = lookup;

    CostEstimate {
        estimated_cost_nanos: calc_cost_nanos_with_pricing(
            &pricing,
            input,
            output,
            cache_read,
            cache_creation,
        ),
        pricing_version: pricing_version.to_string(),
        pricing_model,
        cost_confidence: cost_confidence.to_string(),
    }
}

/// Calculate cost in dollars for the given token counts.
pub fn calc_cost(
    model: &str,
    input: i64,
    output: i64,
    cache_read: i64,
    cache_creation: i64,
) -> f64 {
    calc_cost_nanos(model, input, output, cache_read, cache_creation) as f64 / 1_000_000_000.0
}

/// Estimate the dollar savings (in nanos) from cache reads for a given model.
/// savings = cache_read_tokens × (input_price - cache_read_price)
/// Returns 0 for unknown models or when cache-read pricing is missing/worse
/// than input pricing (which would be a negative saving — clamp to zero).
pub fn calc_cache_savings_nanos(model: &str, cache_read_tokens: i64) -> i64 {
    if cache_read_tokens <= 0 {
        return 0;
    }
    let Some(lookup) = lookup_pricing(model) else {
        return 0;
    };
    let p = lookup.pricing;
    let delta = p.input - p.cache_read;
    if delta <= 0.0 {
        return 0;
    }
    // Same units as calc_cost_nanos: tokens × (price-per-million) × 1000
    // -> nanodollars (per-million × 1000 gives nanodollars per token).
    (cache_read_tokens as f64 * delta * 1000.0) as i64
}

/// Format a token count for display (e.g., 1.5M, 2.3K, 999).
pub fn fmt_tokens(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Format cost for display.
pub fn fmt_cost(c: f64) -> String {
    format!("${:.4}", c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let p = get_pricing("claude-sonnet-4-6").unwrap();
        assert_eq!(p.input, 3.0);
        assert_eq!(p.output, 15.0);
    }

    #[test]
    fn test_all_known_models() {
        for (name, _) in PRICING_TABLE {
            assert!(get_pricing(name).is_some(), "Missing pricing for {name}");
        }
    }

    #[test]
    fn test_prefix_match() {
        let p = get_pricing("claude-sonnet-4-6-20260401").unwrap();
        assert_eq!(p.input, 3.0);
    }

    #[test]
    fn test_prefix_match_longest_wins_mini() {
        // "gpt-5.4-mini-2026-01" must resolve to gpt-5.4-mini (input=0.75),
        // not the shorter gpt-5.4 entry (input=2.50).
        let p = get_pricing("gpt-5.4-mini-2026-01").unwrap();
        assert_eq!(p.input, 0.75);
    }

    #[test]
    fn test_prefix_match_gpt54_plain() {
        // A versioned gpt-5.4 name (no -mini/-nano suffix) still resolves
        // to gpt-5.4, not to any longer entry that contains it.
        let p = get_pricing("gpt-5.4-2026-01").unwrap();
        assert_eq!(p.input, 2.50);
    }

    #[test]
    fn test_prefix_match_gpt55_plain() {
        let p = get_pricing("gpt-5.5-2026-04-24").unwrap();
        assert_eq!(p.input, 5.00);
        assert_eq!(p.cache_read, 0.50);
        assert_eq!(p.output, 30.0);
    }

    #[test]
    fn test_substring_opus() {
        let p = get_pricing("new-opus-5-model").unwrap();
        assert_eq!(p.input, 15.0);
    }

    #[test]
    fn test_substring_case_insensitive() {
        let p = get_pricing("Claude-Opus-Next").unwrap();
        assert_eq!(p.input, 15.0);
    }

    #[test]
    fn test_unknown_returns_none() {
        assert!(get_pricing("gpt-4o").is_none());
        assert!(get_pricing("").is_none());
    }

    #[test]
    fn test_calc_cost_sonnet_input() {
        let cost = calc_cost("claude-sonnet-4-6", 1_000_000, 0, 0, 0);
        assert!((cost - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_calc_cost_opus_output() {
        let cost = calc_cost("claude-opus-4-6", 0, 1_000_000, 0, 0);
        assert!((cost - 75.0).abs() < 0.001);
    }

    #[test]
    fn test_calc_cost_cache_read() {
        let cost = calc_cost("claude-opus-4-6", 0, 0, 1_000_000, 0);
        assert!((cost - 1.50).abs() < 0.001);
    }

    #[test]
    fn test_calc_cost_cache_write() {
        let cost = calc_cost("claude-opus-4-6", 0, 0, 0, 1_000_000);
        assert!((cost - 18.75).abs() < 0.001);
    }

    #[test]
    fn test_calc_cost_unknown_zero() {
        assert_eq!(calc_cost("gpt-4o", 1_000_000, 500_000, 0, 0), 0.0);
    }

    #[test]
    fn test_fmt_tokens() {
        assert_eq!(fmt_tokens(1_500_000), "1.50M");
        assert_eq!(fmt_tokens(1_500), "1.5K");
        assert_eq!(fmt_tokens(999), "999");
    }

    #[test]
    fn test_fmt_cost() {
        assert_eq!(fmt_cost(3.0), "$3.0000");
    }

    // --- Volume discount tests ---

    #[test]
    fn test_sonnet_45_has_threshold() {
        let p = get_pricing("claude-sonnet-4-5").unwrap();
        assert_eq!(p.threshold_tokens, Some(200_000));
        assert_eq!(p.input_above_threshold, Some(6.0));
        assert_eq!(p.output_above_threshold, Some(22.5));
    }

    #[test]
    fn test_sonnet_46_no_threshold() {
        let p = get_pricing("claude-sonnet-4-6").unwrap();
        assert_eq!(p.threshold_tokens, None);
    }

    #[test]
    fn test_volume_discount_below_threshold() {
        // 100K input + 50K output = 150K total, below 200K threshold
        // Should use base rates: 3.0 input, 15.0 output
        let cost = calc_cost("claude-sonnet-4-5", 100_000, 50_000, 0, 0);
        let expected = 100_000.0 * 3.0 / 1_000_000.0 + 50_000.0 * 15.0 / 1_000_000.0;
        assert!(
            (cost - expected).abs() < 0.0001,
            "Below threshold: got {cost}, expected {expected}"
        );
    }

    #[test]
    fn test_volume_discount_above_threshold() {
        // 200K input + 200K output = 400K total, above 200K threshold
        // below_ratio = 200K/400K = 0.5
        // input: 100K at $3, 100K at $6
        // output: 100K at $15, 100K at $22.5
        let cost = calc_cost("claude-sonnet-4-5", 200_000, 200_000, 0, 0);
        let expected_input = 100_000.0 * 3.0 / 1e6 + 100_000.0 * 6.0 / 1e6;
        let expected_output = 100_000.0 * 15.0 / 1e6 + 100_000.0 * 22.5 / 1e6;
        let expected = expected_input + expected_output;
        assert!(
            (cost - expected).abs() < 0.001,
            "Above threshold: got {cost}, expected {expected}"
        );
    }

    #[test]
    fn test_volume_discount_at_threshold() {
        // Exactly at threshold: 150K input + 50K output = 200K
        // Should use base rates only
        let cost = calc_cost("claude-sonnet-4-5", 150_000, 50_000, 0, 0);
        let expected = 150_000.0 * 3.0 / 1e6 + 50_000.0 * 15.0 / 1e6;
        assert!(
            (cost - expected).abs() < 0.0001,
            "At threshold: got {cost}, expected {expected}"
        );
    }

    #[test]
    fn test_no_threshold_model_unaffected() {
        // Opus has no threshold -- large token counts should use base rates
        let cost = calc_cost("claude-opus-4-6", 1_000_000, 500_000, 0, 0);
        let expected = 1_000_000.0 * 15.0 / 1e6 + 500_000.0 * 75.0 / 1e6;
        assert!(
            (cost - expected).abs() < 0.001,
            "No threshold model: got {cost}, expected {expected}"
        );
    }

    // --- Nanos precision tests ---

    #[test]
    fn test_calc_cost_nanos_basic() {
        // 1M input tokens of sonnet at $3/MTok = $3.0 = 3_000_000_000 nanos
        let nanos = calc_cost_nanos("claude-sonnet-4-6", 1_000_000, 0, 0, 0);
        assert_eq!(nanos, 3_000_000_000);
    }

    #[test]
    fn test_calc_cost_nanos_output() {
        // 1M output tokens of opus at $75/MTok = $75.0
        let nanos = calc_cost_nanos("claude-opus-4-6", 0, 1_000_000, 0, 0);
        assert_eq!(nanos, 75_000_000_000);
    }

    #[test]
    fn test_calc_cost_nanos_unknown_zero() {
        assert_eq!(calc_cost_nanos("gpt-4o", 1_000_000, 500_000, 0, 0), 0);
    }

    #[test]
    fn test_calc_cost_wraps_nanos() {
        // Verify calc_cost is consistent with calc_cost_nanos
        let nanos = calc_cost_nanos("claude-sonnet-4-6", 1_000_000, 0, 0, 0);
        let dollars = calc_cost("claude-sonnet-4-6", 1_000_000, 0, 0, 0);
        assert!((dollars - nanos as f64 / 1e9).abs() < 1e-9);
    }

    #[test]
    fn test_nanos_precision_many_small() {
        // Sum many small costs in nanos -- should be exact
        let mut total_nanos: i64 = 0;
        for _ in 0..1000 {
            total_nanos += calc_cost_nanos("claude-sonnet-4-6", 100, 50, 0, 0);
        }
        let single = calc_cost_nanos("claude-sonnet-4-6", 100_000, 50_000, 0, 0);
        assert_eq!(total_nanos, single);
    }

    #[test]
    fn test_nanos_with_cache() {
        // 1M cache_read of opus at $1.50/MTok = 1_500_000_000 nanos
        let nanos = calc_cost_nanos("claude-opus-4-6", 0, 0, 1_000_000, 0);
        assert_eq!(nanos, 1_500_000_000);
        // 1M cache_write of opus at $18.75/MTok = 18_750_000_000 nanos
        let nanos = calc_cost_nanos("claude-opus-4-6", 0, 0, 0, 1_000_000);
        assert_eq!(nanos, 18_750_000_000);
    }

    #[test]
    fn test_nanos_large_token_count() {
        // 10 billion tokens at Opus input rate ($15/MTok)
        // Cost = 10e9 * 15 / 1e6 = $150,000
        // Nanos = 150_000 * 1e9 = 150_000_000_000_000 -- within i64 range
        let nanos = calc_cost_nanos("claude-opus-4-6", 10_000_000_000, 0, 0, 0);
        assert!(nanos > 0, "Should not overflow to negative");
        let cost = calc_cost("claude-opus-4-6", 10_000_000_000, 0, 0, 0);
        assert!((cost - 150_000.0).abs() < 1.0);
    }

    #[test]
    fn test_volume_discount_output_only() {
        // Above threshold with only output tokens
        let cost_small = calc_cost("claude-sonnet-4-5", 0, 100_000, 0, 0);
        let cost_large = calc_cost("claude-sonnet-4-5", 0, 300_000, 0, 0);
        // Larger should cost more
        assert!(cost_large > cost_small);
        // Above-threshold rate is higher ($22.5 vs $15), so cost_large > 3x cost_small
        // Just verify it's positive and proportional
        assert!(cost_large > 0.0);
    }

    #[test]
    fn test_prefix_match_priority() {
        // Exact prefix match should work for versioned models
        let p = get_pricing("claude-opus-4-6-20260401").unwrap();
        assert_eq!(p.input, 15.0);
        let p2 = get_pricing("claude-sonnet-4-5-20250929").unwrap();
        assert_eq!(p2.input, 3.0);
    }

    // ── estimate_cost_breakdown invariant tests ───────────────────────────────
    // Core invariant: breakdown.total_nanos() == estimate_cost(…).estimated_cost_nanos
    // for identical inputs. Verified across 8 model/token combinations.

    #[test]
    fn test_breakdown_sum_equals_estimate_sonnet_46_input_only() {
        let model = "claude-sonnet-4-6";
        let (bd, _, _, _) = estimate_cost_breakdown(model, 1_000_000, 0, 0, 0);
        let est = estimate_cost(model, 1_000_000, 0, 0, 0);
        assert_eq!(
            bd.total_nanos(),
            est.estimated_cost_nanos,
            "breakdown total must equal estimate_cost for {model} input-only"
        );
        assert_eq!(bd.input_cost_nanos, 3_000_000_000);
        assert_eq!(bd.output_cost_nanos, 0);
        assert_eq!(bd.cache_read_cost_nanos, 0);
        assert_eq!(bd.cache_write_cost_nanos, 0);
    }

    #[test]
    fn test_breakdown_sum_equals_estimate_opus_46_all_types() {
        let model = "claude-opus-4-6";
        let (bd, _, _, _) = estimate_cost_breakdown(model, 100_000, 50_000, 200_000, 80_000);
        let est = estimate_cost(model, 100_000, 50_000, 200_000, 80_000);
        assert_eq!(
            bd.total_nanos(),
            est.estimated_cost_nanos,
            "breakdown total must equal estimate_cost for {model} all-types"
        );
    }

    #[test]
    fn test_breakdown_sum_equals_estimate_haiku_45() {
        let model = "claude-haiku-4-5";
        let (bd, _, _, _) = estimate_cost_breakdown(model, 500_000, 250_000, 100_000, 50_000);
        let est = estimate_cost(model, 500_000, 250_000, 100_000, 50_000);
        assert_eq!(
            bd.total_nanos(),
            est.estimated_cost_nanos,
            "breakdown total must equal estimate_cost for {model}"
        );
    }

    #[test]
    fn test_breakdown_sum_equals_estimate_sonnet_45_above_threshold() {
        // Sonnet 4.5 has a 200K token threshold
        let model = "claude-sonnet-4-5";
        let (bd, _, _, _) = estimate_cost_breakdown(model, 200_000, 200_000, 50_000, 25_000);
        let est = estimate_cost(model, 200_000, 200_000, 50_000, 25_000);
        assert_eq!(
            bd.total_nanos(),
            est.estimated_cost_nanos,
            "breakdown total must equal estimate_cost for {model} above-threshold"
        );
    }

    #[test]
    fn test_breakdown_sum_equals_estimate_gpt54() {
        let model = "gpt-5.4";
        let (bd, _, _, _) = estimate_cost_breakdown(model, 1_000_000, 500_000, 0, 0);
        let est = estimate_cost(model, 1_000_000, 500_000, 0, 0);
        assert_eq!(
            bd.total_nanos(),
            est.estimated_cost_nanos,
            "breakdown total must equal estimate_cost for {model}"
        );
    }

    #[test]
    fn test_breakdown_sum_equals_estimate_gpt54_mini() {
        let model = "gpt-5.4-mini";
        let (bd, _, _, _) = estimate_cost_breakdown(model, 300_000, 150_000, 100_000, 50_000);
        let est = estimate_cost(model, 300_000, 150_000, 100_000, 50_000);
        assert_eq!(
            bd.total_nanos(),
            est.estimated_cost_nanos,
            "breakdown total must equal estimate_cost for {model}"
        );
    }

    #[test]
    fn test_breakdown_sum_equals_estimate_cache_heavy() {
        // Opus with mostly cache activity
        let model = "claude-opus-4-6";
        let (bd, _, _, _) = estimate_cost_breakdown(model, 10_000, 5_000, 1_000_000, 500_000);
        let est = estimate_cost(model, 10_000, 5_000, 1_000_000, 500_000);
        assert_eq!(
            bd.total_nanos(),
            est.estimated_cost_nanos,
            "breakdown total must equal estimate_cost for {model} cache-heavy"
        );
        // Cache-read cost at $1.50/MTok: 1M * 1.50 * 1000 = 1_500_000_000
        assert_eq!(bd.cache_read_cost_nanos, 1_500_000_000);
        // Cache-write cost at $18.75/MTok: 500K * 18.75 * 1000 = 9_375_000_000
        assert_eq!(bd.cache_write_cost_nanos, 9_375_000_000);
    }

    #[test]
    fn test_breakdown_unknown_model_all_zero() {
        let (bd, _, pm, conf) = estimate_cost_breakdown("gpt-4o", 1_000_000, 0, 0, 0);
        assert_eq!(bd.total_nanos(), 0);
        assert_eq!(bd.input_cost_nanos, 0);
        assert_eq!(bd.output_cost_nanos, 0);
        assert_eq!(bd.cache_read_cost_nanos, 0);
        assert_eq!(bd.cache_write_cost_nanos, 0);
        assert!(pm.is_empty());
        assert_eq!(conf, COST_CONFIDENCE_LOW);
    }

    // ── Hardcoded-wins invariant tests ────────────────────────────────────────
    // These tests work with a fresh process (no litellm map installed), verifying
    // that hardcoded tiers 1-4 return the right prices. The LiteLLM tier (5) is
    // tested in integration via set_litellm_map in the litellm_priority tests.

    #[test]
    fn test_claude_sonnet_46_returns_hardcoded_input_price() {
        // claude-sonnet-4-6 must return $3/MTok input — tier 1 exact match.
        let est = estimate_cost("claude-sonnet-4-6", 1_000_000, 0, 0, 0);
        // $3.0 = 3_000_000_000 nanos
        assert_eq!(est.estimated_cost_nanos, 3_000_000_000);
        assert_eq!(est.cost_confidence, COST_CONFIDENCE_HIGH);
    }

    #[test]
    fn test_gpt5_4_returns_hardcoded_input_price() {
        // gpt-5.4 must return $2.50/MTok input — tier 1 exact match.
        let est = estimate_cost("gpt-5.4", 1_000_000, 0, 0, 0);
        // $2.50 = 2_500_000_000 nanos
        assert_eq!(est.estimated_cost_nanos, 2_500_000_000);
        assert_eq!(est.cost_confidence, COST_CONFIDENCE_HIGH);
    }

    #[test]
    fn test_gpt5_5_returns_hardcoded_input_price() {
        let est = estimate_cost("gpt-5.5", 1_000_000, 0, 0, 0);
        assert_eq!(est.estimated_cost_nanos, 5_000_000_000);
        assert_eq!(est.pricing_model, "gpt-5.5");
        assert_eq!(est.cost_confidence, COST_CONFIDENCE_HIGH);
    }

    #[test]
    fn test_gpt55_family_prefix_returns_hardcoded_not_litellm() {
        let est = estimate_cost("gpt-5.5-something-new", 1_000_000, 0, 0, 0);
        assert_eq!(est.estimated_cost_nanos, 5_000_000_000);
        assert_eq!(est.pricing_model, "gpt-5.5");
        assert_eq!(est.cost_confidence, COST_CONFIDENCE_HIGH);
    }

    #[test]
    fn test_gpt54_family_prefix_returns_hardcoded_not_litellm() {
        // "gpt-5.4-something-new" starts_with "gpt-5.4" — hits tier 2 prefix match.
        // Must return $2.50 input (hardcoded gpt-5.4 rate), NOT any LiteLLM entry.
        let est = estimate_cost("gpt-5.4-something-new", 1_000_000, 0, 0, 0);
        assert_eq!(est.estimated_cost_nanos, 2_500_000_000);
        // Prefix match is tier 2 → HIGH confidence
        assert_eq!(est.cost_confidence, COST_CONFIDENCE_HIGH);
    }

    #[test]
    fn test_unknown_model_returns_zero_cost_without_litellm() {
        // "gemini-2.5-flash" has no hardcoded entry; without LiteLLM map loaded,
        // cost should be zero.
        let est = estimate_cost("gemini-2.5-flash", 1_000_000, 0, 0, 0);
        assert_eq!(est.estimated_cost_nanos, 0);
        assert_eq!(est.cost_confidence, COST_CONFIDENCE_LOW);
    }

    // ── known_models()  ──────────────────────────────────────────────────────

    #[test]
    fn test_known_models_sorted_alphabetically() {
        let models = known_models();
        assert!(!models.is_empty());
        let mut sorted = models.clone();
        sorted.sort_by(|a, b| a.model.cmp(&b.model));
        for (a, b) in models.iter().zip(sorted.iter()) {
            assert_eq!(a.model, b.model, "known_models() must be sorted by name");
        }
    }

    #[test]
    fn test_known_models_includes_claude_and_openai() {
        let models = known_models();
        let names: Vec<&str> = models.iter().map(|m| m.model.as_str()).collect();
        assert!(
            names.iter().any(|n| n.starts_with("claude-")),
            "expected at least one claude-* entry; got: {names:?}"
        );
        assert!(
            names.iter().any(|n| n.starts_with("gpt-")),
            "expected at least one gpt-* entry; got: {names:?}"
        );
    }

    #[test]
    fn test_known_models_family_label() {
        let models = known_models();
        for entry in &models {
            if entry.model.starts_with("claude-") {
                assert_eq!(entry.family, "claude", "{}", entry.model);
            } else if entry.model.starts_with("gpt-") {
                assert_eq!(entry.family, "openai", "{}", entry.model);
            }
        }
    }

    // ── replace_overrides() hot-swap  ────────────────────────────────────────
    //
    // Pricing overrides live in process-wide state, so these tests serialize
    // against a local mutex to avoid trampling each other (and `lookup_pricing`
    // tests run elsewhere). Each test snapshots the existing map, mutates,
    // asserts, and restores so the rest of the suite is unaffected.
    static OVERRIDE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn with_override_guard<F: FnOnce()>(f: F) {
        let _guard = OVERRIDE_TEST_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let snapshot = current_overrides();
        f();
        replace_overrides(snapshot);
    }

    fn make_override(input: f64, output: f64) -> ModelPricing {
        ModelPricing {
            input,
            output,
            cache_write: input * 1.25,
            cache_read: input * 0.1,
            threshold_tokens: None,
            input_above_threshold: None,
            output_above_threshold: None,
        }
    }

    #[test]
    fn replace_overrides_swaps_atomically() {
        with_override_guard(|| {
            // Use a synthetic model name that won't collide with the
            // hardcoded table or any keyword fallback.
            let model = "heimdall-override-test-zzz";

            // Initial overrides: lookup returns the seeded value.
            let mut initial = HashMap::new();
            initial.insert(model.to_string(), make_override(11.0, 22.0));
            replace_overrides(initial);
            let p1 = get_pricing(model).expect("initial override");
            assert!((p1.input - 11.0).abs() < f64::EPSILON);
            assert!((p1.output - 22.0).abs() < f64::EPSILON);

            // Replace wholesale: lookup now returns the new value.
            let mut updated = HashMap::new();
            updated.insert(model.to_string(), make_override(99.0, 199.0));
            replace_overrides(updated);
            let p2 = get_pricing(model).expect("replaced override");
            assert!((p2.input - 99.0).abs() < f64::EPSILON);
            assert!((p2.output - 199.0).abs() < f64::EPSILON);

            // Replace with empty map: synthetic key disappears entirely.
            replace_overrides(HashMap::new());
            assert!(
                get_pricing(model).is_none(),
                "after empty replace, synthetic model must not resolve"
            );
        });
    }

    #[test]
    fn replace_overrides_takes_effect_in_estimate_cost() {
        with_override_guard(|| {
            let model = "heimdall-override-cost-test";
            let mut map = HashMap::new();
            map.insert(model.to_string(), make_override(7.0, 14.0));
            replace_overrides(map);

            // 1M input tokens at $7/MTok = $7 = 7_000_000_000 nanos.
            let est = estimate_cost(model, 1_000_000, 0, 0, 0);
            assert_eq!(est.estimated_cost_nanos, 7_000_000_000);
            assert_eq!(est.pricing_model, model);
            assert_eq!(est.cost_confidence, COST_CONFIDENCE_HIGH);
        });
    }
}
