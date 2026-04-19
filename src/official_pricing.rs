use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use regex::Regex;
use rusqlite::Connection;

use crate::pricing::{self, ModelPricing};
use crate::scanner::db;

const FETCH_TIMEOUT_SECS: u64 = 10;

pub const STATUS_SUCCESS: &str = "success";
pub const STATUS_FETCH_ERROR: &str = "fetch_error";
pub const STATUS_PARSE_ERROR: &str = "parse_error";

#[derive(Debug, Clone, Copy)]
pub struct PricingSourceDef {
    pub slug: &'static str,
    pub provider: &'static str,
    pub url: &'static str,
    pub priority: i64,
}

pub const OPENAI_DEVELOPER_PRICING: PricingSourceDef = PricingSourceDef {
    slug: "openai_api_docs",
    provider: "openai",
    url: "https://developers.openai.com/api/docs/pricing",
    priority: 100,
};

pub const ANTHROPIC_DOCS_PRICING: PricingSourceDef = PricingSourceDef {
    slug: "anthropic_api_docs",
    provider: "anthropic",
    url: "https://platform.claude.com/docs/en/about-claude/pricing",
    priority: 100,
};

pub const CLAUDE_MARKETING_PRICING: PricingSourceDef = PricingSourceDef {
    slug: "claude_marketing_pricing",
    provider: "anthropic",
    url: "https://claude.com/pricing",
    priority: 90,
};

pub const SOURCES: &[PricingSourceDef] = &[
    OPENAI_DEVELOPER_PRICING,
    ANTHROPIC_DOCS_PRICING,
    CLAUDE_MARKETING_PRICING,
];

#[derive(Debug, Clone)]
pub struct PricingSyncRun {
    pub fetched_at: String,
    pub source_slug: String,
    pub source_url: String,
    pub provider: String,
    pub status: String,
    pub raw_body: String,
    pub error_text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OfficialModelPricing {
    pub source_slug: String,
    pub provider: String,
    pub model_id: String,
    pub model_label: String,
    pub input_usd_per_mtok: f64,
    pub cache_write_usd_per_mtok: f64,
    pub cache_read_usd_per_mtok: f64,
    pub output_usd_per_mtok: f64,
    pub threshold_tokens: Option<i64>,
    pub input_above_threshold: Option<f64>,
    pub output_above_threshold: Option<f64>,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct StoredPricingModel {
    pub run_id: i64,
    pub source_slug: String,
    pub provider: String,
    pub model_id: String,
    pub model_label: String,
    pub input_usd_per_mtok: f64,
    pub cache_write_usd_per_mtok: f64,
    pub cache_read_usd_per_mtok: f64,
    pub output_usd_per_mtok: f64,
    pub threshold_tokens: Option<i64>,
    pub input_above_threshold: Option<f64>,
    pub output_above_threshold: Option<f64>,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct PricingSyncSummary {
    pub total_sources: usize,
    pub successful_sources: usize,
    pub changed_models: Vec<String>,
    pub repriced_turns: usize,
    pub repriced_sessions: usize,
    pub pricing_version: Option<String>,
}

pub fn sync_pricing(conn: &Connection) -> Result<PricingSyncSummary> {
    let old_latest = db::load_latest_pricing_models(conn)?;
    let old_catalog = build_effective_catalog(&old_latest);

    let mut successful_sources = 0;

    for source in SOURCES {
        let fetched_at = Utc::now().to_rfc3339();
        match fetch_source_body(source.url) {
            Ok(raw_body) => {
                let parsed = parse_source(source, &raw_body);
                let (status, error_text) = if parsed.is_empty() {
                    (
                        STATUS_PARSE_ERROR.to_string(),
                        "no recognizable pricing rows found".to_string(),
                    )
                } else {
                    successful_sources += 1;
                    (STATUS_SUCCESS.to_string(), String::new())
                };

                let run_id = db::insert_pricing_sync_run(
                    conn,
                    &PricingSyncRun {
                        fetched_at,
                        source_slug: source.slug.to_string(),
                        source_url: source.url.to_string(),
                        provider: source.provider.to_string(),
                        status: status.clone(),
                        raw_body,
                        error_text,
                    },
                )?;
                if status == STATUS_SUCCESS {
                    db::insert_pricing_sync_models(conn, run_id, &parsed)?;
                }
            }
            Err(err) => {
                db::insert_pricing_sync_run(
                    conn,
                    &PricingSyncRun {
                        fetched_at,
                        source_slug: source.slug.to_string(),
                        source_url: source.url.to_string(),
                        provider: source.provider.to_string(),
                        status: STATUS_FETCH_ERROR.to_string(),
                        raw_body: String::new(),
                        error_text: err,
                    },
                )?;
            }
        }
    }

    let new_latest = db::load_latest_pricing_models(conn)?;
    let new_catalog = build_effective_catalog(&new_latest);
    let changed_models = diff_catalogs(&old_catalog, &new_catalog);

    let mut repriced_turns = 0;
    let mut repriced_sessions = 0;
    let mut pricing_version = None;
    if !changed_models.is_empty() {
        let version = effective_catalog_version(&new_latest);
        repriced_turns = db::reprice_turns_with_catalog(conn, &new_catalog, &version)?;
        repriced_sessions = db::count_sessions(conn)?;
        pricing_version = Some(version);
    }

    Ok(PricingSyncSummary {
        total_sources: SOURCES.len(),
        successful_sources,
        changed_models,
        repriced_turns,
        repriced_sessions,
        pricing_version,
    })
}

pub fn build_effective_catalog(latest: &[StoredPricingModel]) -> HashMap<String, ModelPricing> {
    let mut merged = pricing::builtin_catalog();
    let mut rows = latest.to_vec();
    rows.sort_by(|a, b| {
        source_priority(&b.source_slug)
            .cmp(&source_priority(&a.source_slug))
            .then_with(|| a.model_id.cmp(&b.model_id))
    });

    for row in rows {
        let incoming = ModelPricing {
            input: row.input_usd_per_mtok,
            output: row.output_usd_per_mtok,
            cache_write: row.cache_write_usd_per_mtok,
            cache_read: row.cache_read_usd_per_mtok,
            threshold_tokens: row.threshold_tokens,
            input_above_threshold: row.input_above_threshold,
            output_above_threshold: row.output_above_threshold,
        };

        let merged_row = if let Some(existing) = merged.get(&row.model_id) {
            ModelPricing {
                input: incoming.input,
                output: incoming.output,
                cache_write: incoming.cache_write,
                cache_read: incoming.cache_read,
                threshold_tokens: incoming.threshold_tokens.or(existing.threshold_tokens),
                input_above_threshold: incoming
                    .input_above_threshold
                    .or(existing.input_above_threshold),
                output_above_threshold: incoming
                    .output_above_threshold
                    .or(existing.output_above_threshold),
            }
        } else {
            incoming
        };

        merged.insert(row.model_id, merged_row);
    }

    merged
}

pub fn effective_catalog_version(latest: &[StoredPricingModel]) -> String {
    let mut source_runs: BTreeMap<&str, i64> = BTreeMap::new();
    for row in latest {
        source_runs
            .entry(row.source_slug.as_str())
            .and_modify(|existing| *existing = (*existing).max(row.run_id))
            .or_insert(row.run_id);
    }

    let parts: Vec<String> = source_runs
        .into_iter()
        .map(|(slug, run_id)| format!("{slug}#{run_id}"))
        .collect();
    if parts.is_empty() {
        "official:none".to_string()
    } else {
        format!("official:{}", parts.join(";"))
    }
}

fn diff_catalogs(
    old_catalog: &HashMap<String, ModelPricing>,
    new_catalog: &HashMap<String, ModelPricing>,
) -> Vec<String> {
    let mut keys: BTreeSet<String> = BTreeSet::new();
    keys.extend(old_catalog.keys().cloned());
    keys.extend(new_catalog.keys().cloned());

    keys.into_iter()
        .filter(|key| old_catalog.get(key) != new_catalog.get(key))
        .collect()
}

fn source_priority(slug: &str) -> i64 {
    SOURCES
        .iter()
        .find(|source| source.slug == slug)
        .map(|source| source.priority)
        .unwrap_or_default()
}

fn fetch_source_body(url: &str) -> std::result::Result<String, String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| err.to_string())?;

    rt.block_on(async move {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
            .user_agent("heimdall-pricing-sync/1.0")
            .build()
            .map_err(|err| err.to_string())?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|err| err.to_string())?;
        let response = response.error_for_status().map_err(|err| err.to_string())?;
        response.text().await.map_err(|err| err.to_string())
    })
}

fn parse_source(source: &PricingSourceDef, raw_body: &str) -> Vec<OfficialModelPricing> {
    let stripped = strip_markup(raw_body);
    match source.slug {
        "openai_api_docs" => parse_openai_docs(&stripped, source),
        "anthropic_api_docs" => parse_anthropic_docs(&stripped, source),
        "claude_marketing_pricing" => parse_claude_marketing(&stripped, source),
        _ => Vec::new(),
    }
}

fn parse_openai_docs(text: &str, source: &PricingSourceDef) -> Vec<OfficialModelPricing> {
    let mut rows = Vec::new();

    if let Some(row) = capture_openai_long_context(
        text,
        "gpt-5.4",
        source,
        Some(270_000),
        "long_context_cached_input=$0.50/MTok",
    ) {
        rows.push(row);
    }
    if let Some(row) = capture_openai_short_context(text, "gpt-5.4-mini", source) {
        rows.push(row);
    }
    if let Some(row) = capture_openai_short_context(text, "gpt-5.4-nano", source) {
        rows.push(row);
    }
    if let Some(row) = capture_openai_short_context(text, "gpt-5.3-codex", source) {
        rows.push(row);
    }

    rows
}

fn parse_anthropic_docs(text: &str, source: &PricingSourceDef) -> Vec<OfficialModelPricing> {
    let Ok(re) = Regex::new(
        r"Claude (?P<family>Opus|Sonnet|Haiku) (?P<version>[0-9.]+)(?: \([^)]+\))?\s+\$(?P<input>\d+(?:\.\d+)?) / MTok\s+\$(?P<cache5>\d+(?:\.\d+)?) / MTok\s+\$(?P<cache1h>\d+(?:\.\d+)?) / MTok\s+\$(?P<read>\d+(?:\.\d+)?) / MTok\s+\$(?P<output>\d+(?:\.\d+)?) / MTok",
    ) else {
        return Vec::new();
    };

    re.captures_iter(text)
        .filter_map(|caps| {
            let family = caps.name("family")?.as_str();
            let version = caps.name("version")?.as_str();
            Some(OfficialModelPricing {
                source_slug: source.slug.to_string(),
                provider: source.provider.to_string(),
                model_id: normalize_anthropic_model(family, version),
                model_label: format!("Claude {family} {version}"),
                input_usd_per_mtok: parse_decimal(&caps, "input")?,
                cache_write_usd_per_mtok: parse_decimal(&caps, "cache5")?,
                cache_read_usd_per_mtok: parse_decimal(&caps, "read")?,
                output_usd_per_mtok: parse_decimal(&caps, "output")?,
                threshold_tokens: None,
                input_above_threshold: None,
                output_above_threshold: None,
                notes: format!(
                    "extended_cache_write_1h=${:.2}/MTok",
                    parse_decimal(&caps, "cache1h")?
                ),
            })
        })
        .collect()
}

fn parse_claude_marketing(text: &str, source: &PricingSourceDef) -> Vec<OfficialModelPricing> {
    let Ok(re) = Regex::new(
        r"(?P<label>(?:Haiku|Sonnet|Opus) [0-9.]+)\s+Input\s+\$(?P<input>\d+(?:\.\d+)?) / MTok\s+Output\s+\$(?P<output>\d+(?:\.\d+)?) / MTok\s+Prompt caching\s+Write\s+\$(?P<write>\d+(?:\.\d+)?) / MTok\s+Read\s+\$(?P<read>\d+(?:\.\d+)?) / MTok",
    ) else {
        return Vec::new();
    };

    re.captures_iter(text)
        .filter_map(|caps| {
            let label = caps.name("label")?.as_str();
            let (family, version) = label.split_once(' ')?;
            Some(OfficialModelPricing {
                source_slug: source.slug.to_string(),
                provider: source.provider.to_string(),
                model_id: normalize_anthropic_model(family, version),
                model_label: format!("Claude {label}"),
                input_usd_per_mtok: parse_decimal(&caps, "input")?,
                cache_write_usd_per_mtok: parse_decimal(&caps, "write")?,
                cache_read_usd_per_mtok: parse_decimal(&caps, "read")?,
                output_usd_per_mtok: parse_decimal(&caps, "output")?,
                threshold_tokens: None,
                input_above_threshold: None,
                output_above_threshold: None,
                notes: "prompt_cache_ttl=5m".to_string(),
            })
        })
        .collect()
}

fn capture_openai_short_context(
    text: &str,
    model_id: &str,
    source: &PricingSourceDef,
) -> Option<OfficialModelPricing> {
    let pattern = format!(
        r"{model}\s+\$(?P<input>\d+(?:\.\d+)?)\s+\$(?P<cached>\d+(?:\.\d+)?)\s+\$(?P<output>\d+(?:\.\d+)?)",
        model = regex::escape(model_id)
    );
    let re = Regex::new(&pattern).ok()?;
    let caps = re.captures(text)?;
    Some(OfficialModelPricing {
        source_slug: source.slug.to_string(),
        provider: source.provider.to_string(),
        model_id: model_id.to_string(),
        model_label: model_id.to_string(),
        input_usd_per_mtok: parse_decimal(&caps, "input")?,
        cache_write_usd_per_mtok: parse_decimal(&caps, "input")?,
        cache_read_usd_per_mtok: parse_decimal(&caps, "cached")?,
        output_usd_per_mtok: parse_decimal(&caps, "output")?,
        threshold_tokens: None,
        input_above_threshold: None,
        output_above_threshold: None,
        notes: String::new(),
    })
}

fn capture_openai_long_context(
    text: &str,
    model_id: &str,
    source: &PricingSourceDef,
    threshold_tokens: Option<i64>,
    notes: &str,
) -> Option<OfficialModelPricing> {
    let pattern = format!(
        r"{model}\s+\$(?P<input>\d+(?:\.\d+)?)\s+\$(?P<cached>\d+(?:\.\d+)?)\s+\$(?P<output>\d+(?:\.\d+)?)\s+\$(?P<input_above>\d+(?:\.\d+)?)\s+\$(?P<cached_above>\d+(?:\.\d+)?)\s+\$(?P<output_above>\d+(?:\.\d+)?)",
        model = regex::escape(model_id)
    );
    let re = Regex::new(&pattern).ok()?;
    let caps = re.captures(text)?;
    Some(OfficialModelPricing {
        source_slug: source.slug.to_string(),
        provider: source.provider.to_string(),
        model_id: model_id.to_string(),
        model_label: model_id.to_string(),
        input_usd_per_mtok: parse_decimal(&caps, "input")?,
        cache_write_usd_per_mtok: parse_decimal(&caps, "input")?,
        cache_read_usd_per_mtok: parse_decimal(&caps, "cached")?,
        output_usd_per_mtok: parse_decimal(&caps, "output")?,
        threshold_tokens,
        input_above_threshold: parse_decimal(&caps, "input_above"),
        output_above_threshold: parse_decimal(&caps, "output_above"),
        notes: format!(
            "{notes};long_context_cached_input=${:.2}/MTok",
            parse_decimal(&caps, "cached_above")?
        ),
    })
}

fn parse_decimal(caps: &regex::Captures<'_>, key: &str) -> Option<f64> {
    caps.name(key)?.as_str().parse::<f64>().ok()
}

fn normalize_anthropic_model(family: &str, version: &str) -> String {
    let family = family.to_ascii_lowercase();
    let version = version.replace('.', "-");
    format!("claude-{family}-{version}")
}

fn strip_markup(raw: &str) -> String {
    let scripts = Regex::new(r"(?is)<script\b[^>]*>.*?</script>").expect("valid script regex");
    let styles = Regex::new(r"(?is)<style\b[^>]*>.*?</style>").expect("valid style regex");
    let tags = Regex::new(r"(?is)<[^>]+>").expect("valid tag regex");

    let text = scripts.replace_all(raw, " ");
    let text = styles.replace_all(&text, " ");
    let text = tags.replace_all(&text, " ");
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_openai_docs_extracts_standard_rows() {
        let text = "Flagship models Standard Short context Long context Model Input Cached input Output Input Cached input Output gpt-5.4 $2.50 $0.25 $15.00 $5.00 $0.50 $22.50 gpt-5.4-mini $0.75 $0.075 $4.50 - - - gpt-5.4-nano $0.20 $0.02 $1.25 - - - Specialized models Prices per 1M tokens. Standard Batch Priority Standard Category Model Input Cached input Output ChatGPT gpt-5.3-chat-latest $1.75 $0.175 $14.00 Codex gpt-5.3-codex $1.75 $0.175 $14.00";
        let rows = parse_openai_docs(text, &OPENAI_DEVELOPER_PRICING);
        assert!(rows.iter().any(|row| row.model_id == "gpt-5.4"));
        assert!(rows.iter().any(|row| row.model_id == "gpt-5.4-mini"));
        assert!(rows.iter().any(|row| row.model_id == "gpt-5.3-codex"));
        let gpt54 = rows.iter().find(|row| row.model_id == "gpt-5.4").unwrap();
        assert_eq!(gpt54.threshold_tokens, Some(270_000));
        assert_eq!(gpt54.input_above_threshold, Some(5.0));
        assert_eq!(gpt54.output_above_threshold, Some(22.5));
    }

    #[test]
    fn parse_anthropic_docs_extracts_cache_rows() {
        let text = "Model Base Input Tokens 5m Cache Writes 1h Cache Writes Cache Hits & Refreshes Output Tokens Claude Opus 4.6 $5 / MTok $6.25 / MTok $10 / MTok $0.50 / MTok $25 / MTok Claude Sonnet 4.6 $3 / MTok $3.75 / MTok $6 / MTok $0.30 / MTok $15 / MTok Claude Haiku 4.5 $1 / MTok $1.25 / MTok $2 / MTok $0.10 / MTok $5 / MTok MTok = Millions of tokens";
        let rows = parse_anthropic_docs(text, &ANTHROPIC_DOCS_PRICING);
        assert!(rows.iter().any(|row| row.model_id == "claude-opus-4-6"));
        assert!(rows.iter().any(|row| row.model_id == "claude-sonnet-4-6"));
        assert!(rows.iter().any(|row| row.model_id == "claude-haiku-4-5"));
        let sonnet = rows
            .iter()
            .find(|row| row.model_id == "claude-sonnet-4-6")
            .unwrap();
        assert_eq!(sonnet.cache_write_usd_per_mtok, 3.75);
        assert_eq!(sonnet.cache_read_usd_per_mtok, 0.30);
        assert!(sonnet.notes.contains("extended_cache_write_1h=$6.00/MTok"));
    }

    #[test]
    fn parse_claude_marketing_extracts_prompt_cache_rows() {
        let text = "Haiku 4.5 Input $1 / MTok Output $5 / MTok Prompt caching Write $1.25 / MTok Read $0.10 / MTok Opus 4.6 Input $5 / MTok Output $25 / MTok Prompt caching Write $6.25 / MTok Read $0.50 / MTok Sonnet 4.5 Input $3 / MTok Output $15 / MTok Prompt caching Write $3.75 / MTok Read $0.30 / MTok";
        let rows = parse_claude_marketing(text, &CLAUDE_MARKETING_PRICING);
        assert!(rows.iter().any(|row| row.model_id == "claude-haiku-4-5"));
        assert!(rows.iter().any(|row| row.model_id == "claude-opus-4-6"));
        assert!(rows.iter().any(|row| row.model_id == "claude-sonnet-4-5"));
    }

    #[test]
    fn build_effective_catalog_overlays_builtins() {
        let rows = vec![StoredPricingModel {
            run_id: 9,
            source_slug: OPENAI_DEVELOPER_PRICING.slug.to_string(),
            provider: "openai".to_string(),
            model_id: "gpt-5.4".to_string(),
            model_label: "gpt-5.4".to_string(),
            input_usd_per_mtok: 3.0,
            cache_write_usd_per_mtok: 3.0,
            cache_read_usd_per_mtok: 0.3,
            output_usd_per_mtok: 18.0,
            threshold_tokens: None,
            input_above_threshold: None,
            output_above_threshold: None,
            notes: String::new(),
        }];
        let catalog = build_effective_catalog(&rows);
        let gpt54 = catalog.get("gpt-5.4").unwrap();
        assert_eq!(gpt54.input, 3.0);
        assert_eq!(gpt54.output, 18.0);
    }
}
