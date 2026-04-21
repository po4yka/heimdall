use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::litellm::{LiteLlmSnapshot, read_cache as litellm_read_cache};

pub const PRICING_VERSION: &str = "2026-04-10";
pub const PRICING_VALID_FROM: &str = "2026-04-10T00:00:00Z";
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

static PRICING_OVERRIDES: OnceLock<HashMap<String, ModelPricing>> = OnceLock::new();

/// Install custom pricing overrides from config. Call once at startup.
pub fn set_overrides(overrides: HashMap<String, ModelPricing>) {
    let _ = PRICING_OVERRIDES.set(overrides);
}

fn get_override(model: &str) -> Option<&ModelPricing> {
    PRICING_OVERRIDES.get()?.get(model)
}

// ── LiteLLM pricing source ────────────────────────────────────────────────────

/// Where model pricing data originates.
#[allow(dead_code)]
pub enum PricingSource {
    /// Built-in hardcoded table only (current default).
    Static,
    /// Supplement the hardcoded table with a LiteLLM cache file.
    LiteLlm { cache_path: PathBuf },
}

/// Runtime LiteLLM pricing map (populated once at startup when source = LiteLlm).
/// Guarded by OnceLock so it is safe to set from main and read from all threads.
static LITELLM_MAP: OnceLock<HashMap<String, ModelPricing>> = OnceLock::new();

/// Install a LiteLLM-sourced pricing map. Called at startup when config says
/// `source = "litellm"`. Safe to call multiple times — subsequent calls are
/// no-ops (OnceLock semantics).
pub fn set_litellm_map(map: HashMap<String, ModelPricing>) {
    let _ = LITELLM_MAP.set(map);
}

/// Load the LiteLLM cache file at `path` and convert its per-MTok rates into
/// `ModelPricing` values. Returns `None` if the file is absent or unparseable.
///
/// This is the test seam: pass any `Path` to avoid touching `~/.cache`.
#[allow(dead_code)]
pub fn load_litellm_cache(path: &Path) -> Option<HashMap<String, ModelPricing>> {
    let snapshot: LiteLlmSnapshot = litellm_read_cache(path)?;
    Some(load_litellm_cache_from_snapshot(snapshot))
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

/// Look up pricing for a model across overrides, built-ins, heuristic
/// fallbacks, and the LiteLLM cache.
#[allow(dead_code)]
pub fn get_pricing(model: &str) -> Option<&ModelPricing> {
    match lookup_pricing(model)? {
        PricingLookup::Borrowed { pricing, .. } => Some(pricing),
    }
}

enum PricingLookup<'a> {
    Borrowed {
        pricing: &'a ModelPricing,
        pricing_model: String,
        cost_confidence: &'static str,
    },
}

fn lookup_pricing(model: &str) -> Option<PricingLookup<'_>> {
    if model.is_empty() {
        return None;
    }

    if let Some(p) = get_override(model) {
        return Some(PricingLookup::Borrowed {
            pricing: p,
            pricing_model: model.to_string(),
            cost_confidence: COST_CONFIDENCE_HIGH,
        });
    }

    for (name, pricing) in PRICING_TABLE {
        if *name == model {
            return Some(PricingLookup::Borrowed {
                pricing,
                pricing_model: (*name).to_string(),
                cost_confidence: COST_CONFIDENCE_HIGH,
            });
        }
    }

    for (name, pricing) in PRICING_TABLE {
        if model.starts_with(name) {
            return Some(PricingLookup::Borrowed {
                pricing,
                pricing_model: (*name).to_string(),
                cost_confidence: COST_CONFIDENCE_HIGH,
            });
        }
    }

    let lower = model.to_lowercase();
    if lower.contains("opus") {
        return get_builtin("claude-opus-4-6").map(|pricing| PricingLookup::Borrowed {
            pricing,
            pricing_model: "claude-opus-4-6".to_string(),
            cost_confidence: COST_CONFIDENCE_MEDIUM,
        });
    }
    if lower.contains("sonnet") {
        return get_builtin("claude-sonnet-4-6").map(|pricing| PricingLookup::Borrowed {
            pricing,
            pricing_model: "claude-sonnet-4-6".to_string(),
            cost_confidence: COST_CONFIDENCE_MEDIUM,
        });
    }
    if lower.contains("haiku") {
        return get_builtin("claude-haiku-4-5").map(|pricing| PricingLookup::Borrowed {
            pricing,
            pricing_model: "claude-haiku-4-5".to_string(),
            cost_confidence: COST_CONFIDENCE_MEDIUM,
        });
    }
    if lower.contains("gpt-5.4-mini") {
        return get_builtin("gpt-5.4-mini").map(|pricing| PricingLookup::Borrowed {
            pricing,
            pricing_model: "gpt-5.4-mini".to_string(),
            cost_confidence: COST_CONFIDENCE_MEDIUM,
        });
    }
    if lower.contains("gpt-5.4-nano") {
        return get_builtin("gpt-5.4-nano").map(|pricing| PricingLookup::Borrowed {
            pricing,
            pricing_model: "gpt-5.4-nano".to_string(),
            cost_confidence: COST_CONFIDENCE_MEDIUM,
        });
    }
    if lower.contains("gpt-5.4") {
        return get_builtin("gpt-5.4").map(|pricing| PricingLookup::Borrowed {
            pricing,
            pricing_model: "gpt-5.4".to_string(),
            cost_confidence: COST_CONFIDENCE_MEDIUM,
        });
    }
    if lower.contains("codex") {
        return get_builtin("gpt-5.3-codex").map(|pricing| PricingLookup::Borrowed {
            pricing,
            pricing_model: "gpt-5.3-codex".to_string(),
            cost_confidence: COST_CONFIDENCE_MEDIUM,
        });
    }

    // Tier 5: LiteLLM cache — only reached for models NOT matched by any hardcoded
    // tier above.  This guarantees Claude/GPT-5 families always use hardcoded prices.
    if let Some(pricing) = get_litellm(model) {
        return Some(PricingLookup::Borrowed {
            pricing,
            pricing_model: model.to_string(),
            cost_confidence: COST_CONFIDENCE_LOW,
        });
    }

    None
}

fn lookup_catalog_pricing<'a>(
    model: &str,
    catalog: &'a HashMap<String, ModelPricing>,
) -> Option<PricingLookup<'a>> {
    if model.is_empty() {
        return None;
    }

    if let Some(pricing) = catalog.get(model) {
        return Some(PricingLookup::Borrowed {
            pricing,
            pricing_model: model.to_string(),
            cost_confidence: COST_CONFIDENCE_HIGH,
        });
    }

    for (name, pricing) in catalog {
        if model.starts_with(name) {
            return Some(PricingLookup::Borrowed {
                pricing,
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
    Some(PricingLookup::Borrowed {
        pricing,
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

/// Returns true if this model resolves to any available pricing source.
#[allow(dead_code)]
pub fn is_billable(model: &str) -> bool {
    lookup_pricing(model).is_some()
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
    let p = match lookup {
        PricingLookup::Borrowed { pricing, .. } => pricing,
    };

    calc_cost_nanos_with_pricing(p, input, output, cache_read, cache_creation)
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

    let (pricing, pricing_model, cost_confidence) = match lookup {
        PricingLookup::Borrowed {
            pricing,
            pricing_model,
            cost_confidence,
        } => (*pricing, pricing_model, cost_confidence),
    };

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

    let (pricing, pricing_model, cost_confidence) = match lookup {
        PricingLookup::Borrowed {
            pricing,
            pricing_model,
            cost_confidence,
        } => (*pricing, pricing_model, cost_confidence),
    };

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

    let (pricing, pricing_model, cost_confidence) = match lookup {
        PricingLookup::Borrowed {
            pricing,
            pricing_model,
            cost_confidence,
        } => (*pricing, pricing_model, cost_confidence),
    };

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
    let PricingLookup::Borrowed { pricing: p, .. } = lookup;
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

    #[test]
    fn test_is_billable() {
        assert!(is_billable("claude-sonnet-4-6"));
        assert!(!is_billable("gpt-4o"));
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

    // ── LiteLLM cache loading ─────────────────────────────────────────────────

    #[test]
    fn test_load_litellm_cache_round_trip() {
        use crate::litellm::{LiteLlmModelEntry, LiteLlmSnapshot, write_cache};
        use std::collections::HashMap as HM;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("litellm_pricing.json");

        let mut entries = HM::new();
        entries.insert(
            "gemini-2.5-flash".to_string(),
            LiteLlmModelEntry {
                input_cost_per_token: Some(0.075),
                output_cost_per_token: Some(0.30),
            },
        );
        let snap = LiteLlmSnapshot {
            fetched_at: chrono::Utc::now().to_rfc3339(),
            entries,
        };
        write_cache(&path, &snap).unwrap();

        let map = load_litellm_cache(&path).unwrap();
        assert!(map.contains_key("gemini-2.5-flash"));
        let p = &map["gemini-2.5-flash"];
        assert!((p.input - 0.075).abs() < 1e-9);
        assert!((p.output - 0.30).abs() < 1e-9);
    }

    #[test]
    fn test_load_litellm_cache_missing_returns_none() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("missing.json");
        assert!(load_litellm_cache(&path).is_none());
    }

    #[test]
    fn test_load_litellm_cache_skips_entries_without_both_rates() {
        use crate::litellm::{LiteLlmModelEntry, LiteLlmSnapshot, write_cache};
        use std::collections::HashMap as HM;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("litellm_pricing.json");

        let mut entries = HM::new();
        // Only has input rate — should be skipped
        entries.insert(
            "partial-model".to_string(),
            LiteLlmModelEntry {
                input_cost_per_token: Some(1.0),
                output_cost_per_token: None,
            },
        );
        // Both rates present — should be included
        entries.insert(
            "full-model".to_string(),
            LiteLlmModelEntry {
                input_cost_per_token: Some(1.0),
                output_cost_per_token: Some(3.0),
            },
        );
        let snap = LiteLlmSnapshot {
            fetched_at: chrono::Utc::now().to_rfc3339(),
            entries,
        };
        write_cache(&path, &snap).unwrap();

        let map = load_litellm_cache(&path).unwrap();
        assert!(!map.contains_key("partial-model"));
        assert!(map.contains_key("full-model"));
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
}
