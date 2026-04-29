use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use regex::Regex;
use rusqlite::Connection;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::agent_status;
use crate::agent_status::models::{InjectedResponses, ProviderStatus};
use crate::currency::RatesSnapshot;
use crate::openai;
use crate::pricing::{self, ModelPricing};
use crate::pricing_defs::*;
use crate::scanner::db;

const FETCH_TIMEOUT_SECS: u64 = 10;
const PARSER_VERSION: &str = "official_pricing/v3";
const OPENAI_USAGE_URL: &str = "https://api.openai.com/v1/organization/usage/completions";

#[derive(Debug, Clone)]
struct FetchedSourceBody {
    http_status: u16,
    content_type: String,
    etag: String,
    last_modified: String,
    raw_body: String,
    normalized_body: String,
}

pub fn sync_pricing(
    conn: &Connection,
    options: &OfficialSyncOptions,
) -> Result<PricingSyncSummary> {
    let old_latest = db::load_latest_pricing_models(conn)?;
    let old_catalog = build_effective_catalog(&old_latest);

    let mut successful_sources = 0;
    let mut metadata_runs = 0;
    let mut metadata_records = 0;

    for source in SOURCES {
        let fetched_at = Utc::now().to_rfc3339();
        match fetch_source(source.url) {
            Ok(fetched) => {
                let parsed = parse_source(source, &fetched.raw_body);
                let records = build_records_for_pricing_source(
                    source,
                    &fetched.normalized_body,
                    &parsed,
                    &fetched_at,
                );
                let (status, error_text) = if parsed.is_empty() {
                    (
                        STATUS_PARSE_ERROR.to_string(),
                        "no recognizable pricing rows found".to_string(),
                    )
                } else {
                    successful_sources += 1;
                    (STATUS_SUCCESS.to_string(), String::new())
                };

                let metadata_run_id = db::insert_official_sync_run(
                    conn,
                    &OfficialSyncRunRecord {
                        fetched_at: fetched_at.clone(),
                        source_slug: source.slug.to_string(),
                        source_kind: OfficialSourceKind::Pricing.as_str().to_string(),
                        source_url: source.url.to_string(),
                        provider: source.provider.to_string(),
                        authority: OfficialSourceAuthority::ProviderDocs.as_str().to_string(),
                        format: OfficialSourceFormat::Html.as_str().to_string(),
                        cadence: OfficialSourceCadence::Daily.as_str().to_string(),
                        status: status.clone(),
                        http_status: Some(i64::from(fetched.http_status)),
                        content_type: fetched.content_type.clone(),
                        etag: fetched.etag.clone(),
                        last_modified: fetched.last_modified.clone(),
                        raw_body: fetched.raw_body.clone(),
                        normalized_body: fetched.normalized_body.clone(),
                        error_text: error_text.clone(),
                        parser_version: PARSER_VERSION.to_string(),
                        raw_body_sha256: sha256_hex(&fetched.raw_body),
                        normalized_body_sha256: sha256_hex(&fetched.normalized_body),
                        extracted_sha256: hash_records(&records),
                    },
                )?;
                db::insert_official_extracted_records(conn, metadata_run_id, &records)?;
                metadata_runs += 1;
                metadata_records += records.len();

                let run_id = db::insert_pricing_sync_run(
                    conn,
                    &PricingSyncRun {
                        fetched_at,
                        source_slug: source.slug.to_string(),
                        source_url: source.url.to_string(),
                        provider: source.provider.to_string(),
                        status: status.clone(),
                        raw_body: fetched.raw_body,
                        error_text,
                    },
                )?;
                if status == STATUS_SUCCESS {
                    db::insert_pricing_sync_models(conn, run_id, &parsed)?;
                }
            }
            Err(err) => {
                db::insert_official_sync_run(
                    conn,
                    &OfficialSyncRunRecord {
                        fetched_at: fetched_at.clone(),
                        source_slug: source.slug.to_string(),
                        source_kind: OfficialSourceKind::Pricing.as_str().to_string(),
                        source_url: source.url.to_string(),
                        provider: source.provider.to_string(),
                        authority: OfficialSourceAuthority::ProviderDocs.as_str().to_string(),
                        format: OfficialSourceFormat::Html.as_str().to_string(),
                        cadence: OfficialSourceCadence::Daily.as_str().to_string(),
                        status: STATUS_FETCH_ERROR.to_string(),
                        http_status: None,
                        content_type: String::new(),
                        etag: String::new(),
                        last_modified: String::new(),
                        raw_body: String::new(),
                        normalized_body: String::new(),
                        error_text: err.clone(),
                        parser_version: PARSER_VERSION.to_string(),
                        raw_body_sha256: String::new(),
                        normalized_body_sha256: String::new(),
                        extracted_sha256: String::new(),
                    },
                )?;
                metadata_runs += 1;
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

    for source in CONTENT_SOURCES {
        let fetched_at = Utc::now().to_rfc3339();
        match fetch_source(source.url) {
            Ok(fetched) => {
                let records =
                    build_records_for_content_source(source, &fetched.normalized_body, &fetched_at);
                let status = if records.is_empty() {
                    STATUS_PARSE_ERROR
                } else {
                    successful_sources += 1;
                    STATUS_SUCCESS
                };

                let run_id = db::insert_official_sync_run(
                    conn,
                    &OfficialSyncRunRecord {
                        fetched_at,
                        source_slug: source.slug.to_string(),
                        source_kind: source.kind.as_str().to_string(),
                        source_url: source.url.to_string(),
                        provider: source.provider.to_string(),
                        authority: source.authority.as_str().to_string(),
                        format: source.format.as_str().to_string(),
                        cadence: source.cadence.as_str().to_string(),
                        status: status.to_string(),
                        http_status: Some(i64::from(fetched.http_status)),
                        content_type: fetched.content_type,
                        etag: fetched.etag,
                        last_modified: fetched.last_modified,
                        raw_body_sha256: sha256_hex(&fetched.raw_body),
                        normalized_body_sha256: sha256_hex(&fetched.normalized_body),
                        extracted_sha256: hash_records(&records),
                        raw_body: fetched.raw_body,
                        normalized_body: fetched.normalized_body,
                        error_text: if status == STATUS_PARSE_ERROR {
                            "no recognizable metadata rows found".to_string()
                        } else {
                            String::new()
                        },
                        parser_version: PARSER_VERSION.to_string(),
                    },
                )?;
                db::insert_official_extracted_records(conn, run_id, &records)?;
                metadata_runs += 1;
                metadata_records += records.len();
            }
            Err(err) => {
                db::insert_official_sync_run(
                    conn,
                    &OfficialSyncRunRecord {
                        fetched_at,
                        source_slug: source.slug.to_string(),
                        source_kind: source.kind.as_str().to_string(),
                        source_url: source.url.to_string(),
                        provider: source.provider.to_string(),
                        authority: source.authority.as_str().to_string(),
                        format: source.format.as_str().to_string(),
                        cadence: source.cadence.as_str().to_string(),
                        status: STATUS_FETCH_ERROR.to_string(),
                        http_status: None,
                        content_type: String::new(),
                        etag: String::new(),
                        last_modified: String::new(),
                        raw_body: String::new(),
                        normalized_body: String::new(),
                        error_text: err,
                        parser_version: PARSER_VERSION.to_string(),
                        raw_body_sha256: String::new(),
                        normalized_body_sha256: String::new(),
                        extracted_sha256: String::new(),
                    },
                )?;
                metadata_runs += 1;
            }
        }
    }

    for source in STATUS_SOURCES {
        let (run_count, record_count, success_count) = sync_status_source(conn, source)?;
        metadata_runs += run_count;
        metadata_records += record_count;
        successful_sources += success_count;
    }

    for source in EXCHANGE_RATE_SOURCES {
        let fetched_at = Utc::now().to_rfc3339();
        match fetch_source(source.url) {
            Ok(fetched) => {
                let records = parse_exchange_rates(source, &fetched.raw_body, &fetched_at);
                let status = if records.is_empty() {
                    STATUS_PARSE_ERROR
                } else {
                    successful_sources += 1;
                    STATUS_SUCCESS
                };
                let run_id = db::insert_official_sync_run(
                    conn,
                    &OfficialSyncRunRecord {
                        fetched_at,
                        source_slug: source.slug.to_string(),
                        source_kind: OfficialSourceKind::ExchangeRates.as_str().to_string(),
                        source_url: source.url.to_string(),
                        provider: source.provider.to_string(),
                        authority: source.authority.as_str().to_string(),
                        format: source.format.as_str().to_string(),
                        cadence: source.cadence.as_str().to_string(),
                        status: status.to_string(),
                        http_status: Some(i64::from(fetched.http_status)),
                        content_type: fetched.content_type,
                        etag: fetched.etag,
                        last_modified: fetched.last_modified,
                        raw_body_sha256: sha256_hex(&fetched.raw_body),
                        normalized_body_sha256: sha256_hex(&fetched.normalized_body),
                        extracted_sha256: hash_records(&records),
                        raw_body: fetched.raw_body,
                        normalized_body: fetched.normalized_body,
                        error_text: if status == STATUS_PARSE_ERROR {
                            "no exchange rates found".to_string()
                        } else {
                            String::new()
                        },
                        parser_version: PARSER_VERSION.to_string(),
                    },
                )?;
                db::insert_official_extracted_records(conn, run_id, &records)?;
                metadata_runs += 1;
                metadata_records += records.len();
            }
            Err(err) => {
                db::insert_official_sync_run(
                    conn,
                    &OfficialSyncRunRecord {
                        fetched_at,
                        source_slug: source.slug.to_string(),
                        source_kind: OfficialSourceKind::ExchangeRates.as_str().to_string(),
                        source_url: source.url.to_string(),
                        provider: source.provider.to_string(),
                        authority: source.authority.as_str().to_string(),
                        format: source.format.as_str().to_string(),
                        cadence: source.cadence.as_str().to_string(),
                        status: STATUS_FETCH_ERROR.to_string(),
                        http_status: None,
                        content_type: String::new(),
                        etag: String::new(),
                        last_modified: String::new(),
                        raw_body: String::new(),
                        normalized_body: String::new(),
                        error_text: err,
                        parser_version: PARSER_VERSION.to_string(),
                        raw_body_sha256: String::new(),
                        normalized_body_sha256: String::new(),
                        extracted_sha256: String::new(),
                    },
                )?;
                metadata_runs += 1;
            }
        }
    }

    let (usage_runs, usage_records, usage_successes) =
        sync_openai_usage_reconciliation(conn, options)?;
    metadata_runs += usage_runs;
    metadata_records += usage_records;
    successful_sources += usage_successes;

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
        total_sources: SOURCES.len()
            + CONTENT_SOURCES.len()
            + STATUS_SOURCES.len()
            + EXCHANGE_RATE_SOURCES.len()
            + 1,
        successful_sources,
        metadata_runs,
        metadata_records,
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
        source_priority(&a.source_slug)
            .cmp(&source_priority(&b.source_slug))
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

fn fetch_source(url: &str) -> std::result::Result<FetchedSourceBody, String> {
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
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().await.map_err(|err| err.to_string())?;
        if !status.is_success() {
            return Err(format!("HTTP {} for {}", status.as_u16(), url));
        }
        let normalized_body = strip_markup(&body);
        Ok(FetchedSourceBody {
            http_status: status.as_u16(),
            content_type: headers
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_string(),
            etag: headers
                .get(reqwest::header::ETAG)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_string(),
            last_modified: headers
                .get(reqwest::header::LAST_MODIFIED)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_string(),
            raw_body: body,
            normalized_body,
        })
    })
}

fn hash_records(records: &[OfficialExtractedRecord]) -> String {
    if records.is_empty() {
        return String::new();
    }
    let joined = records
        .iter()
        .map(|record| record.payload_json.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    sha256_hex(&joined)
}

fn sha256_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn to_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
fn fetch_source_body(url: &str) -> std::result::Result<String, String> {
    fetch_source(url).map(|fetched| fetched.raw_body)
}

#[cfg(test)]
fn sync_pricing_with_fetch<F>(
    conn: &Connection,
    sources: &[PricingSourceDef],
    mut fetch: F,
) -> Result<PricingSyncSummary>
where
    F: FnMut(&PricingSourceDef) -> std::result::Result<String, String>,
{
    let old_latest = db::load_latest_pricing_models(conn)?;
    let old_catalog = build_effective_catalog(&old_latest);
    let mut successful_sources = 0;
    let mut metadata_runs = 0;
    let mut metadata_records = 0;

    for source in sources {
        let fetched_at = Utc::now().to_rfc3339();
        match fetch(source) {
            Ok(raw_body) => {
                let normalized_body = strip_markup(&raw_body);
                let parsed = parse_source(source, &raw_body);
                let records = build_records_for_pricing_source(
                    source,
                    &normalized_body,
                    &parsed,
                    &fetched_at,
                );
                let (status, error_text) = if parsed.is_empty() {
                    (
                        STATUS_PARSE_ERROR.to_string(),
                        "no recognizable pricing rows found".to_string(),
                    )
                } else {
                    successful_sources += 1;
                    (STATUS_SUCCESS.to_string(), String::new())
                };

                let metadata_run_id = db::insert_official_sync_run(
                    conn,
                    &OfficialSyncRunRecord {
                        fetched_at: fetched_at.clone(),
                        source_slug: source.slug.to_string(),
                        source_kind: OfficialSourceKind::Pricing.as_str().to_string(),
                        source_url: source.url.to_string(),
                        provider: source.provider.to_string(),
                        authority: OfficialSourceAuthority::ProviderDocs.as_str().to_string(),
                        format: OfficialSourceFormat::Html.as_str().to_string(),
                        cadence: OfficialSourceCadence::Daily.as_str().to_string(),
                        status: status.clone(),
                        http_status: Some(200),
                        content_type: "text/html".to_string(),
                        etag: String::new(),
                        last_modified: String::new(),
                        raw_body: raw_body.clone(),
                        normalized_body: normalized_body.clone(),
                        error_text: error_text.clone(),
                        parser_version: PARSER_VERSION.to_string(),
                        raw_body_sha256: sha256_hex(&raw_body),
                        normalized_body_sha256: sha256_hex(&normalized_body),
                        extracted_sha256: hash_records(&records),
                    },
                )?;
                db::insert_official_extracted_records(conn, metadata_run_id, &records)?;
                metadata_runs += 1;
                metadata_records += records.len();

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
                        fetched_at: fetched_at.clone(),
                        source_slug: source.slug.to_string(),
                        source_url: source.url.to_string(),
                        provider: source.provider.to_string(),
                        status: STATUS_FETCH_ERROR.to_string(),
                        raw_body: String::new(),
                        error_text: err.clone(),
                    },
                )?;
                db::insert_official_sync_run(
                    conn,
                    &OfficialSyncRunRecord {
                        fetched_at,
                        source_slug: source.slug.to_string(),
                        source_kind: OfficialSourceKind::Pricing.as_str().to_string(),
                        source_url: source.url.to_string(),
                        provider: source.provider.to_string(),
                        authority: OfficialSourceAuthority::ProviderDocs.as_str().to_string(),
                        format: OfficialSourceFormat::Html.as_str().to_string(),
                        cadence: OfficialSourceCadence::Daily.as_str().to_string(),
                        status: STATUS_FETCH_ERROR.to_string(),
                        http_status: None,
                        content_type: String::new(),
                        etag: String::new(),
                        last_modified: String::new(),
                        raw_body: String::new(),
                        normalized_body: String::new(),
                        error_text: err,
                        parser_version: PARSER_VERSION.to_string(),
                        raw_body_sha256: String::new(),
                        normalized_body_sha256: String::new(),
                        extracted_sha256: String::new(),
                    },
                )?;
                metadata_runs += 1;
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
        total_sources: sources.len(),
        successful_sources,
        metadata_runs,
        metadata_records,
        changed_models,
        repriced_turns,
        repriced_sessions,
        pricing_version,
    })
}

fn build_records_for_pricing_source(
    source: &PricingSourceDef,
    text: &str,
    pricing_rows: &[OfficialModelPricing],
    fetched_at: &str,
) -> Vec<OfficialExtractedRecord> {
    let mut records = Vec::new();

    for pricing in pricing_rows {
        records.push(OfficialExtractedRecord {
            source_slug: pricing.source_slug.clone(),
            provider: pricing.provider.clone(),
            record_type: "pricing_model".to_string(),
            record_key: pricing.model_id.clone(),
            model_id: pricing.model_id.clone(),
            effective_at: fetched_at.to_string(),
            payload_json: to_json(pricing),
        });

        let metadata = build_model_metadata_record(source, text, pricing);
        records.push(OfficialExtractedRecord {
            source_slug: source.slug.to_string(),
            provider: source.provider.to_string(),
            record_type: "model_metadata".to_string(),
            record_key: metadata.model_id.clone(),
            model_id: metadata.model_id.clone(),
            effective_at: fetched_at.to_string(),
            payload_json: to_json(&metadata),
        });
    }

    for tool in parse_tool_pricing(source, text) {
        let tool_key = format!(
            "{}:{}:{}",
            tool.provider,
            tool.tool_slug,
            tool.model_id.clone().unwrap_or_default()
        );
        records.push(OfficialExtractedRecord {
            source_slug: tool.source_slug.clone(),
            provider: tool.provider.clone(),
            record_type: "tool_pricing".to_string(),
            record_key: tool_key,
            model_id: tool.model_id.clone().unwrap_or_default(),
            effective_at: fetched_at.to_string(),
            payload_json: to_json(&tool),
        });
    }

    records
}

fn build_model_metadata_record(
    source: &PricingSourceDef,
    text: &str,
    pricing: &OfficialModelPricing,
) -> OfficialModelMetadataRecord {
    let lower = text.to_ascii_lowercase();
    let lifecycle = if source.provider == "anthropic"
        && lower.contains(&format!(
            "{} (deprecated)",
            pricing.model_label.to_ascii_lowercase()
        )) {
        Some(ModelLifecycleMetadata {
            stage: ModelLifecycleStage::Deprecated,
            announced_at: None,
            generally_available_at: None,
            deprecation_announced_at: None,
            sunset_at: None,
            replacement_model_id: None,
            notes: vec!["deprecated marker present on official pricing page".to_string()],
        })
    } else {
        Some(ModelLifecycleMetadata {
            stage: ModelLifecycleStage::GenerallyAvailable,
            announced_at: None,
            generally_available_at: None,
            deprecation_announced_at: None,
            sunset_at: None,
            replacement_model_id: None,
            notes: vec!["listed on official pricing page".to_string()],
        })
    };

    let prompt_caching = Some(PromptCachingPolicyMetadata {
        supported: pricing.cache_read_usd_per_mtok > 0.0 || pricing.cache_write_usd_per_mtok > 0.0,
        default_ttl_seconds: if source.provider == "anthropic" {
            Some(300)
        } else {
            None
        },
        max_ttl_seconds: if source.provider == "anthropic" {
            Some(3600)
        } else {
            None
        },
        refresh_resets_ttl: None,
        write_priced_as_input: Some(
            (pricing.cache_write_usd_per_mtok - pricing.input_usd_per_mtok).abs() < f64::EPSILON,
        ),
        cache_read_discount_pct: if pricing.input_usd_per_mtok > 0.0 {
            Some(100.0 - ((pricing.cache_read_usd_per_mtok / pricing.input_usd_per_mtok) * 100.0))
        } else {
            None
        },
        cache_write_multiplier: if pricing.input_usd_per_mtok > 0.0 {
            Some(pricing.cache_write_usd_per_mtok / pricing.input_usd_per_mtok)
        } else {
            None
        },
        notes: if source.provider == "anthropic" {
            vec!["Anthropic pricing page lists 5m default and 1h extended cache tiers".to_string()]
        } else {
            vec!["Derived from cached-input pricing on official OpenAI pricing page".to_string()]
        },
    });

    let context_window = if source.provider == "anthropic"
        && pricing.model_label.contains("Opus 4.7")
        && lower.contains("opus 4.7 uses a new tokenizer")
    {
        Some(ContextWindowMetadata {
            max_input_tokens: None,
            max_output_tokens: None,
            max_context_tokens: None,
            tokenizer_family: Some(TokenizerFamily::ProviderSpecific),
            tokenizer_name: None,
            tokenizer_notes: vec![
                "Official pricing page notes a new tokenizer with potentially higher token counts"
                    .to_string(),
            ],
            truncation_behavior: None,
        })
    } else {
        None
    };

    let mut processing_modes = Vec::new();
    if source.provider == "anthropic" && lower.contains("batch api discount") {
        processing_modes.push(ProcessingModePricingMetadata {
            mode: ProcessingMode::Batch,
            region_scope: None,
            input_usd_per_mtok: None,
            cache_write_usd_per_mtok: None,
            cache_read_usd_per_mtok: None,
            output_usd_per_mtok: None,
            relative_uplift_pct: Some(-50.0),
            notes: vec!["Official pricing page references Batch API discount".to_string()],
        });
    }
    if source.provider == "anthropic"
        && lower
            .contains("us-only inference via the inference_geo parameter incurs a 1.1x multiplier")
    {
        processing_modes.push(ProcessingModePricingMetadata {
            mode: ProcessingMode::Regional,
            region_scope: Some("us_only".to_string()),
            input_usd_per_mtok: Some(pricing.input_usd_per_mtok * 1.1),
            cache_write_usd_per_mtok: Some(pricing.cache_write_usd_per_mtok * 1.1),
            cache_read_usd_per_mtok: Some(pricing.cache_read_usd_per_mtok * 1.1),
            output_usd_per_mtok: Some(pricing.output_usd_per_mtok * 1.1),
            relative_uplift_pct: Some(10.0),
            notes: vec![
                "Official pricing page documents a 1.1x US-only inference multiplier".to_string(),
            ],
        });
    }

    if source.provider == "openai" && pricing.model_id == "gpt-5.3-codex" {
        if let Some(batch) =
            parse_openai_processing_mode(text, "gpt-5.3-codex", ProcessingMode::Batch, -50.0)
        {
            processing_modes.push(batch);
        }
        if let Some(priority) =
            parse_openai_processing_mode(text, "gpt-5.3-codex", ProcessingMode::Priority, 100.0)
        {
            processing_modes.push(priority);
        }
    }

    let notes = if pricing.notes.is_empty() {
        Vec::new()
    } else {
        vec![pricing.notes.clone()]
    };

    OfficialModelMetadataRecord {
        provider: source.provider.to_string(),
        model_id: pricing.model_id.clone(),
        model_label: pricing.model_label.clone(),
        lifecycle,
        context_window,
        prompt_caching,
        processing_modes,
        notes,
    }
}

fn parse_openai_processing_mode(
    text: &str,
    model_id: &str,
    mode: ProcessingMode,
    relative_uplift_pct: f64,
) -> Option<ProcessingModePricingMetadata> {
    let label = match mode {
        ProcessingMode::Batch => "Batch",
        ProcessingMode::Priority => "Priority",
        _ => return None,
    };
    let pattern = format!(
        r"{label}\s+Category Model Input Cached input Output .*?{model}\$(?P<input>\d+(?:\.\d+)?)\$(?P<cached>\d+(?:\.\d+)?)\$(?P<output>\d+(?:\.\d+)?)",
        label = regex::escape(label),
        model = regex::escape(model_id)
    );
    let re = Regex::new(&pattern).ok()?;
    let caps = re.captures(text)?;
    Some(ProcessingModePricingMetadata {
        mode,
        region_scope: None,
        input_usd_per_mtok: parse_decimal(&caps, "input"),
        cache_write_usd_per_mtok: parse_decimal(&caps, "input"),
        cache_read_usd_per_mtok: parse_decimal(&caps, "cached"),
        output_usd_per_mtok: parse_decimal(&caps, "output"),
        relative_uplift_pct: Some(relative_uplift_pct),
        notes: vec!["Parsed from official OpenAI pricing mode table".to_string()],
    })
}

fn parse_tool_pricing(source: &PricingSourceDef, text: &str) -> Vec<OfficialToolPricing> {
    if source.provider == "openai" {
        return parse_openai_tool_pricing(source, text);
    }
    if source.provider == "anthropic" {
        return parse_anthropic_tool_pricing(source, text);
    }
    Vec::new()
}

fn parse_openai_tool_pricing(source: &PricingSourceDef, text: &str) -> Vec<OfficialToolPricing> {
    let mut rows = Vec::new();
    let specs = [
        (
            "web-search",
            "Web Search",
            r"Web search Web search \(all models\)\$(?P<price>\d+(?:\.\d+)?) / 1k calls",
            ToolBillingUnit::Per1KCalls,
            None,
        ),
        (
            "web-search-preview-reasoning",
            "Web Search Preview (Reasoning)",
            r"Web search preview \(reasoning models, including .*?\)\$(?P<price>\d+(?:\.\d+)?) / 1k calls",
            ToolBillingUnit::Per1KCalls,
            None,
        ),
        (
            "web-search-preview-non-reasoning",
            "Web Search Preview (Non-Reasoning)",
            r"Web search preview \(non-reasoning models\)\$(?P<price>\d+(?:\.\d+)?) / 1k calls",
            ToolBillingUnit::Per1KCalls,
            None,
        ),
        (
            "file-search-storage",
            "File Search Storage",
            r"File search Storage\$(?P<price>\d+(?:\.\d+)?) / GB per day \(1 GB free\)",
            ToolBillingUnit::PerSession,
            Some(1.0),
        ),
        (
            "file-search-call",
            "File Search Tool Call",
            r"Tool call\$(?P<price>\d+(?:\.\d+)?) / 1k calls",
            ToolBillingUnit::Per1KCalls,
            None,
        ),
    ];

    for (tool_slug, tool_label, pattern, billing_unit, included_units) in specs {
        let Ok(re) = Regex::new(pattern) else {
            continue;
        };
        let Some(caps) = re.captures(text) else {
            continue;
        };
        let Some(unit_price_usd) = parse_decimal(&caps, "price") else {
            continue;
        };
        rows.push(OfficialToolPricing {
            source_slug: source.slug.to_string(),
            provider: source.provider.to_string(),
            tool_slug: tool_slug.to_string(),
            tool_label: tool_label.to_string(),
            model_id: None,
            billing_unit,
            unit_price_usd,
            included_units,
            notes: vec!["Parsed from official OpenAI pricing page".to_string()],
        });
    }

    if let Ok(re) = Regex::new(
        r"Containers Hosted Shell and Code Interpreter 1 GB \$(?P<gb1>\d+(?:\.\d+)?), 4 GB \$(?P<gb4>\d+(?:\.\d+)?), 16 GB \$(?P<gb16>\d+(?:\.\d+)?), 64 GB \$(?P<gb64>\d+(?:\.\d+)?) per 20-minute session per container",
    ) && let Some(caps) = re.captures(text)
    {
        for (size, key) in [
            ("1gb", "gb1"),
            ("4gb", "gb4"),
            ("16gb", "gb16"),
            ("64gb", "gb64"),
        ] {
            if let Some(unit_price_usd) = parse_decimal(&caps, key) {
                rows.push(OfficialToolPricing {
                    source_slug: source.slug.to_string(),
                    provider: source.provider.to_string(),
                    tool_slug: format!("container-{size}"),
                    tool_label: format!("Container Session {size}"),
                    model_id: None,
                    billing_unit: ToolBillingUnit::PerSession,
                    unit_price_usd,
                    included_units: None,
                    notes: vec![
                        "Hosted Shell and Code Interpreter session pricing (20-minute session)"
                            .to_string(),
                    ],
                });
            }
        }
    }

    rows
}

fn parse_anthropic_tool_pricing(source: &PricingSourceDef, text: &str) -> Vec<OfficialToolPricing> {
    let mut rows = Vec::new();
    if let Ok(re) = Regex::new(r"Bash tool The bash tool adds (?P<input>\d+) input tokens")
        && let Some(caps) = re.captures(text)
        && let Some(input_tokens) = caps
            .name("input")
            .and_then(|m| m.as_str().parse::<f64>().ok())
    {
        rows.push(OfficialToolPricing {
            source_slug: source.slug.to_string(),
            provider: source.provider.to_string(),
            tool_slug: "bash-tool-overhead".to_string(),
            tool_label: "Bash Tool Overhead".to_string(),
            model_id: None,
            billing_unit: ToolBillingUnit::PerCall,
            unit_price_usd: 0.0,
            included_units: None,
            notes: vec![format!(
                "Billed at model token rates; official docs state each call adds {input_tokens} input tokens"
            )],
        });
    }
    rows
}

fn build_records_for_content_source(
    source: &OfficialContentSourceDef,
    text: &str,
    fetched_at: &str,
) -> Vec<OfficialExtractedRecord> {
    match source.kind {
        OfficialSourceKind::ModelCatalog => parse_model_catalog(source, text, fetched_at),
        OfficialSourceKind::ReleaseNotes => parse_release_note_records(source, text),
        _ => Vec::new(),
    }
}

fn parse_model_catalog(
    source: &OfficialContentSourceDef,
    text: &str,
    fetched_at: &str,
) -> Vec<OfficialExtractedRecord> {
    let mut out = Vec::new();
    if source.provider == "openai" {
        let Ok(re) = Regex::new(
            r"\b(gpt-[a-z0-9.\-]+|o[1345](?:-[a-z0-9.\-]+)?|computer-use-preview|whisper-1|tts-1(?:-hd)?|text-embedding-[a-z0-9.\-]+)\b",
        ) else {
            return out;
        };
        let mut seen = BTreeSet::new();
        for cap in re.captures_iter(text) {
            let Some(model_id) = cap.get(1).map(|m| m.as_str().to_string()) else {
                continue;
            };
            if !seen.insert(model_id.clone()) {
                continue;
            }
            let payload = OfficialModelMetadataRecord {
                provider: source.provider.to_string(),
                model_id: model_id.clone(),
                model_label: model_id.clone(),
                lifecycle: Some(ModelLifecycleMetadata {
                    stage: ModelLifecycleStage::GenerallyAvailable,
                    announced_at: None,
                    generally_available_at: None,
                    deprecation_announced_at: None,
                    sunset_at: None,
                    replacement_model_id: None,
                    notes: vec!["Listed on official models catalog page".to_string()],
                }),
                context_window: None,
                prompt_caching: None,
                processing_modes: Vec::new(),
                notes: vec!["Catalog presence snapshot".to_string()],
            };
            out.push(OfficialExtractedRecord {
                source_slug: source.slug.to_string(),
                provider: source.provider.to_string(),
                record_type: "catalog_model".to_string(),
                record_key: model_id.clone(),
                model_id,
                effective_at: fetched_at.to_string(),
                payload_json: to_json(&payload),
            });
        }
    } else if source.provider == "anthropic" {
        let Ok(re) = Regex::new(r"Claude (?P<family>Opus|Sonnet|Haiku) (?P<version>[0-9.]+)")
        else {
            return out;
        };
        let mut seen = BTreeSet::new();
        for caps in re.captures_iter(text) {
            let Some(family) = caps.name("family").map(|m| m.as_str()) else {
                continue;
            };
            let Some(version) = caps.name("version").map(|m| m.as_str()) else {
                continue;
            };
            let model_id = normalize_anthropic_model(family, version);
            if !seen.insert(model_id.clone()) {
                continue;
            }
            let payload = OfficialModelMetadataRecord {
                provider: source.provider.to_string(),
                model_id: model_id.clone(),
                model_label: format!("Claude {family} {version}"),
                lifecycle: Some(ModelLifecycleMetadata {
                    stage: ModelLifecycleStage::GenerallyAvailable,
                    announced_at: None,
                    generally_available_at: None,
                    deprecation_announced_at: None,
                    sunset_at: None,
                    replacement_model_id: None,
                    notes: vec!["Listed on official models overview page".to_string()],
                }),
                context_window: None,
                prompt_caching: None,
                processing_modes: Vec::new(),
                notes: vec!["Catalog presence snapshot".to_string()],
            };
            out.push(OfficialExtractedRecord {
                source_slug: source.slug.to_string(),
                provider: source.provider.to_string(),
                record_type: "catalog_model".to_string(),
                record_key: model_id.clone(),
                model_id,
                effective_at: fetched_at.to_string(),
                payload_json: to_json(&payload),
            });
        }
    }
    out
}

fn parse_release_note_records(
    source: &OfficialContentSourceDef,
    text: &str,
) -> Vec<OfficialExtractedRecord> {
    parse_release_notes(source, text)
        .into_iter()
        .map(|note| OfficialExtractedRecord {
            source_slug: note.source_slug.clone(),
            provider: note.provider.clone(),
            record_type: "release_note".to_string(),
            record_key: note.snapshot_id.clone(),
            model_id: note.affected_models.first().cloned().unwrap_or_default(),
            effective_at: note.published_at.clone().unwrap_or_default(),
            payload_json: to_json(&note),
        })
        .collect()
}

fn parse_release_notes(source: &OfficialContentSourceDef, text: &str) -> Vec<ReleaseNoteSnapshot> {
    if source.provider == "anthropic" {
        return parse_anthropic_release_notes(source, text);
    }
    parse_openai_release_notes(source, text)
}

fn parse_anthropic_release_notes(
    source: &OfficialContentSourceDef,
    text: &str,
) -> Vec<ReleaseNoteSnapshot> {
    let Ok(date_re) = Regex::new(r"[A-Za-z]+ \d{1,2}(?:st|nd|rd|th), \d{4}") else {
        return Vec::new();
    };
    split_release_note_sections(text, &date_re)
        .into_iter()
        .take(20)
        .filter_map(|(date, body)| {
            if body.is_empty() {
                return None;
            }
            let title = first_sentence(&body);
            Some(ReleaseNoteSnapshot {
                source_slug: source.slug.to_string(),
                provider: source.provider.to_string(),
                snapshot_id: sha256_hex(&format!("{}:{title}", date)),
                title: title.clone(),
                url: source.url.to_string(),
                published_at: Some(date),
                kind: classify_release_note_kind(&body),
                summary: truncate_for_summary(&body),
                affected_models: extract_affected_models(&body),
                notes: vec!["Parsed from official Anthropic release notes page".to_string()],
            })
        })
        .collect()
}

fn parse_openai_release_notes(
    source: &OfficialContentSourceDef,
    text: &str,
) -> Vec<ReleaseNoteSnapshot> {
    let Ok(date_re) = Regex::new(r"(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) \d{1,2}")
    else {
        return Vec::new();
    };
    split_release_note_sections(text, &date_re)
        .into_iter()
        .take(20)
        .filter_map(|(date, body)| {
            if body.is_empty() {
                return None;
            }
            let title = first_sentence(&body);
            Some(ReleaseNoteSnapshot {
                source_slug: source.slug.to_string(),
                provider: source.provider.to_string(),
                snapshot_id: sha256_hex(&format!("{date}:{title}")),
                title: title.clone(),
                url: source.url.to_string(),
                published_at: Some(date),
                kind: classify_release_note_kind(&body),
                summary: truncate_for_summary(&body),
                affected_models: extract_affected_models(&body),
                notes: vec!["Parsed from official OpenAI changelog".to_string()],
            })
        })
        .collect()
}

fn split_release_note_sections(text: &str, date_re: &Regex) -> Vec<(String, String)> {
    let matches: Vec<_> = date_re.find_iter(text).collect();
    let mut sections = Vec::with_capacity(matches.len());
    for (idx, found) in matches.iter().enumerate() {
        let body_start = found.end();
        let body_end = matches.get(idx + 1).map_or(text.len(), |next| next.start());
        let body = text[body_start..body_end].trim().to_string();
        if body.is_empty() {
            continue;
        }
        sections.push((found.as_str().to_string(), body));
    }
    sections
}

fn sync_status_source(
    conn: &Connection,
    source: &StatusSourceDef,
) -> Result<(usize, usize, usize)> {
    let fetched_at = Utc::now().to_rfc3339();
    if let Some(incidents_url) = source.incidents_url {
        let summary = fetch_source(source.summary_url);
        let incidents = fetch_source(incidents_url);
        return match (summary, incidents) {
            (Ok(summary), Ok(incidents)) => {
                let snapshot = agent_status::poll_with_injection(InjectedResponses {
                    claude_summary: None,
                    openai_status: Some(summary.raw_body.clone()),
                    openai_incidents: Some(incidents.raw_body.clone()),
                });
                let provider = snapshot.openai.unwrap_or(ProviderStatus {
                    indicator: Default::default(),
                    description: String::new(),
                    components: Vec::new(),
                    active_incidents: Vec::new(),
                    page_url: source.page_url.to_string(),
                });
                let status_record = OfficialExtractedRecord {
                    source_slug: source.slug.to_string(),
                    provider: source.provider.to_string(),
                    record_type: "status_snapshot".to_string(),
                    record_key: source.provider.to_string(),
                    model_id: String::new(),
                    effective_at: fetched_at.clone(),
                    payload_json: to_json(&StatusSnapshotRecord {
                        provider: source.provider.to_string(),
                        source_slug: source.slug.to_string(),
                        page_url: source.page_url.to_string(),
                        snapshot: provider.clone(),
                    }),
                };
                let incident_records = provider
                    .active_incidents
                    .iter()
                    .map(|incident| OfficialExtractedRecord {
                        source_slug: source.slug.to_string(),
                        provider: source.provider.to_string(),
                        record_type: "status_incident".to_string(),
                        record_key: incident
                            .shortlink
                            .clone()
                            .unwrap_or_else(|| sha256_hex(&incident.name)),
                        model_id: String::new(),
                        effective_at: incident.started_at.clone(),
                        payload_json: to_json(incident),
                    })
                    .collect::<Vec<_>>();

                let summary_run_id = db::insert_official_sync_run(
                    conn,
                    &OfficialSyncRunRecord {
                        fetched_at: fetched_at.clone(),
                        source_slug: source.slug.to_string(),
                        source_kind: OfficialSourceKind::StatusSummary.as_str().to_string(),
                        source_url: source.summary_url.to_string(),
                        provider: source.provider.to_string(),
                        authority: source.authority.as_str().to_string(),
                        format: source.format.as_str().to_string(),
                        cadence: source.cadence.as_str().to_string(),
                        status: STATUS_SUCCESS.to_string(),
                        http_status: Some(i64::from(summary.http_status)),
                        content_type: summary.content_type,
                        etag: summary.etag,
                        last_modified: summary.last_modified,
                        raw_body_sha256: sha256_hex(&summary.raw_body),
                        normalized_body_sha256: sha256_hex(&summary.normalized_body),
                        extracted_sha256: sha256_hex(&status_record.payload_json),
                        raw_body: summary.raw_body,
                        normalized_body: summary.normalized_body,
                        error_text: String::new(),
                        parser_version: PARSER_VERSION.to_string(),
                    },
                )?;
                db::insert_official_extracted_records(conn, summary_run_id, &[status_record])?;

                let incident_run_id = db::insert_official_sync_run(
                    conn,
                    &OfficialSyncRunRecord {
                        fetched_at,
                        source_slug: format!("{}_incidents", source.slug),
                        source_kind: OfficialSourceKind::StatusIncidents.as_str().to_string(),
                        source_url: incidents_url.to_string(),
                        provider: source.provider.to_string(),
                        authority: source.authority.as_str().to_string(),
                        format: source.format.as_str().to_string(),
                        cadence: source.cadence.as_str().to_string(),
                        status: STATUS_SUCCESS.to_string(),
                        http_status: Some(i64::from(incidents.http_status)),
                        content_type: incidents.content_type,
                        etag: incidents.etag,
                        last_modified: incidents.last_modified,
                        raw_body_sha256: sha256_hex(&incidents.raw_body),
                        normalized_body_sha256: sha256_hex(&incidents.normalized_body),
                        extracted_sha256: hash_records(&incident_records),
                        raw_body: incidents.raw_body,
                        normalized_body: incidents.normalized_body,
                        error_text: String::new(),
                        parser_version: PARSER_VERSION.to_string(),
                    },
                )?;
                db::insert_official_extracted_records(conn, incident_run_id, &incident_records)?;
                Ok((2, 1 + incident_records.len(), 2))
            }
            (Err(err), _) | (_, Err(err)) => {
                db::insert_official_sync_run(
                    conn,
                    &OfficialSyncRunRecord {
                        fetched_at,
                        source_slug: source.slug.to_string(),
                        source_kind: OfficialSourceKind::StatusSummary.as_str().to_string(),
                        source_url: source.summary_url.to_string(),
                        provider: source.provider.to_string(),
                        authority: source.authority.as_str().to_string(),
                        format: source.format.as_str().to_string(),
                        cadence: source.cadence.as_str().to_string(),
                        status: STATUS_FETCH_ERROR.to_string(),
                        http_status: None,
                        content_type: String::new(),
                        etag: String::new(),
                        last_modified: String::new(),
                        raw_body: String::new(),
                        normalized_body: String::new(),
                        error_text: err,
                        parser_version: PARSER_VERSION.to_string(),
                        raw_body_sha256: String::new(),
                        normalized_body_sha256: String::new(),
                        extracted_sha256: String::new(),
                    },
                )?;
                Ok((1, 0, 0))
            }
        };
    }

    match fetch_source(source.summary_url) {
        Ok(summary) => {
            let snapshot = agent_status::poll_with_injection(InjectedResponses {
                claude_summary: Some(summary.raw_body.clone()),
                openai_status: None,
                openai_incidents: None,
            });
            let provider = snapshot.claude.unwrap_or(ProviderStatus {
                indicator: Default::default(),
                description: String::new(),
                components: Vec::new(),
                active_incidents: Vec::new(),
                page_url: source.page_url.to_string(),
            });
            let mut records = vec![OfficialExtractedRecord {
                source_slug: source.slug.to_string(),
                provider: source.provider.to_string(),
                record_type: "status_snapshot".to_string(),
                record_key: source.provider.to_string(),
                model_id: String::new(),
                effective_at: fetched_at.clone(),
                payload_json: to_json(&StatusSnapshotRecord {
                    provider: source.provider.to_string(),
                    source_slug: source.slug.to_string(),
                    page_url: source.page_url.to_string(),
                    snapshot: provider.clone(),
                }),
            }];
            records.extend(provider.active_incidents.iter().map(|incident| {
                OfficialExtractedRecord {
                    source_slug: source.slug.to_string(),
                    provider: source.provider.to_string(),
                    record_type: "status_incident".to_string(),
                    record_key: incident
                        .shortlink
                        .clone()
                        .unwrap_or_else(|| sha256_hex(&incident.name)),
                    model_id: String::new(),
                    effective_at: incident.started_at.clone(),
                    payload_json: to_json(incident),
                }
            }));
            let run_id = db::insert_official_sync_run(
                conn,
                &OfficialSyncRunRecord {
                    fetched_at,
                    source_slug: source.slug.to_string(),
                    source_kind: OfficialSourceKind::StatusSummary.as_str().to_string(),
                    source_url: source.summary_url.to_string(),
                    provider: source.provider.to_string(),
                    authority: source.authority.as_str().to_string(),
                    format: source.format.as_str().to_string(),
                    cadence: source.cadence.as_str().to_string(),
                    status: STATUS_SUCCESS.to_string(),
                    http_status: Some(i64::from(summary.http_status)),
                    content_type: summary.content_type,
                    etag: summary.etag,
                    last_modified: summary.last_modified,
                    raw_body_sha256: sha256_hex(&summary.raw_body),
                    normalized_body_sha256: sha256_hex(&summary.normalized_body),
                    extracted_sha256: hash_records(&records),
                    raw_body: summary.raw_body,
                    normalized_body: summary.normalized_body,
                    error_text: String::new(),
                    parser_version: PARSER_VERSION.to_string(),
                },
            )?;
            db::insert_official_extracted_records(conn, run_id, &records)?;
            Ok((1, records.len(), 1))
        }
        Err(err) => {
            db::insert_official_sync_run(
                conn,
                &OfficialSyncRunRecord {
                    fetched_at,
                    source_slug: source.slug.to_string(),
                    source_kind: OfficialSourceKind::StatusSummary.as_str().to_string(),
                    source_url: source.summary_url.to_string(),
                    provider: source.provider.to_string(),
                    authority: source.authority.as_str().to_string(),
                    format: source.format.as_str().to_string(),
                    cadence: source.cadence.as_str().to_string(),
                    status: STATUS_FETCH_ERROR.to_string(),
                    http_status: None,
                    content_type: String::new(),
                    etag: String::new(),
                    last_modified: String::new(),
                    raw_body: String::new(),
                    normalized_body: String::new(),
                    error_text: err,
                    parser_version: PARSER_VERSION.to_string(),
                    raw_body_sha256: String::new(),
                    normalized_body_sha256: String::new(),
                    extracted_sha256: String::new(),
                },
            )?;
            Ok((1, 0, 0))
        }
    }
}

fn parse_exchange_rates(
    source: &ExchangeRateSourceDef,
    raw_body: &str,
    fetched_at: &str,
) -> Vec<OfficialExtractedRecord> {
    let Ok(snapshot) = serde_json::from_str::<RatesSnapshot>(raw_body) else {
        return Vec::new();
    };
    snapshot
        .rates
        .iter()
        .map(|(quote_currency, rate)| {
            let payload = ExchangeRateRecord {
                provider: source.provider.to_string(),
                source_slug: source.slug.to_string(),
                base_currency: snapshot.base.clone(),
                quote_currency: quote_currency.clone(),
                rate: *rate,
                upstream_provider: source.upstream_provider.map(str::to_string),
                observed_at: fetched_at.to_string(),
            };
            OfficialExtractedRecord {
                source_slug: source.slug.to_string(),
                provider: source.provider.to_string(),
                record_type: "exchange_rate".to_string(),
                record_key: format!("{}-{}", snapshot.base, quote_currency),
                model_id: String::new(),
                effective_at: fetched_at.to_string(),
                payload_json: to_json(&payload),
            }
        })
        .collect()
}

fn sync_openai_usage_reconciliation(
    conn: &Connection,
    options: &OfficialSyncOptions,
) -> Result<(usize, usize, usize)> {
    let fetched_at = Utc::now().to_rfc3339();
    let Some(admin_key) = options.openai_admin_key.as_deref() else {
        db::insert_official_sync_run(
            conn,
            &OfficialSyncRunRecord {
                fetched_at,
                source_slug: "openai_usage_api".to_string(),
                source_kind: OfficialSourceKind::UsageReconciliation.as_str().to_string(),
                source_url: OPENAI_USAGE_URL.to_string(),
                provider: "openai".to_string(),
                authority: OfficialSourceAuthority::UpstreamReference
                    .as_str()
                    .to_string(),
                format: OfficialSourceFormat::Json.as_str().to_string(),
                cadence: OfficialSourceCadence::Daily.as_str().to_string(),
                status: STATUS_SKIPPED.to_string(),
                http_status: None,
                content_type: String::new(),
                etag: String::new(),
                last_modified: String::new(),
                raw_body: String::new(),
                normalized_body: String::new(),
                error_text: "missing OpenAI admin key".to_string(),
                parser_version: PARSER_VERSION.to_string(),
                raw_body_sha256: String::new(),
                normalized_body_sha256: String::new(),
                extracted_sha256: String::new(),
            },
        )?;
        return Ok((1, 0, 0));
    };

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let reconciliation = rt.block_on(openai::fetch_org_usage_reconciliation(
        admin_key,
        options.openai_lookback_days,
        0.0,
    ));
    let status = if reconciliation.available {
        STATUS_SUCCESS
    } else {
        STATUS_FETCH_ERROR
    };
    let normalized_body = to_json(&reconciliation);
    let record = OfficialExtractedRecord {
        source_slug: "openai_usage_api".to_string(),
        provider: "openai".to_string(),
        record_type: "usage_reconciliation".to_string(),
        record_key: format!("{}:{}", reconciliation.start_date, reconciliation.end_date),
        model_id: String::new(),
        effective_at: fetched_at.clone(),
        payload_json: normalized_body.clone(),
    };
    let run_id = db::insert_official_sync_run(
        conn,
        &OfficialSyncRunRecord {
            fetched_at,
            source_slug: "openai_usage_api".to_string(),
            source_kind: OfficialSourceKind::UsageReconciliation.as_str().to_string(),
            source_url: OPENAI_USAGE_URL.to_string(),
            provider: "openai".to_string(),
            authority: OfficialSourceAuthority::UpstreamReference
                .as_str()
                .to_string(),
            format: OfficialSourceFormat::Json.as_str().to_string(),
            cadence: OfficialSourceCadence::Daily.as_str().to_string(),
            status: status.to_string(),
            http_status: None,
            content_type: "application/json".to_string(),
            etag: String::new(),
            last_modified: String::new(),
            raw_body: String::new(),
            normalized_body: normalized_body.clone(),
            error_text: reconciliation.error.clone().unwrap_or_default(),
            parser_version: PARSER_VERSION.to_string(),
            raw_body_sha256: String::new(),
            normalized_body_sha256: sha256_hex(&normalized_body),
            extracted_sha256: sha256_hex(&record.payload_json),
        },
    )?;
    db::insert_official_extracted_records(conn, run_id, &[record])?;
    Ok((1, 1, usize::from(reconciliation.available)))
}

fn classify_release_note_kind(text: &str) -> ReleaseNoteKind {
    let lower = text.to_ascii_lowercase();
    if lower.contains("deprecat") || lower.contains("retir") || lower.contains("sunset") {
        ReleaseNoteKind::Deprecation
    } else if lower.contains("pricing") || lower.contains("discount") || lower.contains("cost") {
        ReleaseNoteKind::Pricing
    } else if lower.contains("context window") || lower.contains("1m token") {
        ReleaseNoteKind::ContextWindow
    } else if lower.contains("tool")
        || lower.contains("file search")
        || lower.contains("code interpreter")
    {
        ReleaseNoteKind::Tooling
    } else if lower.contains("reliability") || lower.contains("uptime") || lower.contains("latency")
    {
        ReleaseNoteKind::Reliability
    } else if lower.contains("launch") || lower.contains("released") || lower.contains("introduced")
    {
        ReleaseNoteKind::Launch
    } else if lower.contains("support") || lower.contains("added") || lower.contains("updated") {
        ReleaseNoteKind::Capability
    } else {
        ReleaseNoteKind::Update
    }
}

fn extract_affected_models(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    if let Ok(re) =
        Regex::new(r"\b(gpt-[a-z0-9.\-]+|o[1345](?:-[a-z0-9.\-]+)?|computer-use-preview)\b")
    {
        for caps in re.captures_iter(text) {
            if let Some(model_id) = caps.get(1).map(|m| m.as_str().to_string())
                && seen.insert(model_id.clone())
            {
                out.push(model_id);
            }
        }
    }
    if let Ok(re) = Regex::new(r"Claude (?P<family>Opus|Sonnet|Haiku) (?P<version>[0-9.]+)") {
        for caps in re.captures_iter(text) {
            let Some(family) = caps.name("family").map(|m| m.as_str()) else {
                continue;
            };
            let Some(version) = caps.name("version").map(|m| m.as_str()) else {
                continue;
            };
            let model_id = normalize_anthropic_model(family, version);
            if seen.insert(model_id.clone()) {
                out.push(model_id);
            }
        }
    }
    out
}

fn first_sentence(text: &str) -> String {
    text.split(". ")
        .next()
        .unwrap_or(text)
        .trim()
        .trim_end_matches('.')
        .to_string()
}

fn truncate_for_summary(text: &str) -> String {
    const MAX: usize = 240;
    if text.len() <= MAX {
        text.to_string()
    } else {
        format!("{}...", &text[..MAX])
    }
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

    if let Some(row) = capture_openai_short_context(text, "gpt-5.5", source) {
        rows.push(row);
    }
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
    use std::collections::HashMap;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    use crate::models::{Session, Turn};
    use crate::scanner::db::{count_sessions, init_db, insert_turns, upsert_sessions};

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        conn
    }

    fn model_row(
        source_slug: &str,
        model_id: &str,
        input: f64,
        cache_write: f64,
        cache_read: f64,
        output: f64,
    ) -> StoredPricingModel {
        StoredPricingModel {
            run_id: 1,
            source_slug: source_slug.to_string(),
            provider: "anthropic".to_string(),
            model_id: model_id.to_string(),
            model_label: model_id.to_string(),
            input_usd_per_mtok: input,
            cache_write_usd_per_mtok: cache_write,
            cache_read_usd_per_mtok: cache_read,
            output_usd_per_mtok: output,
            threshold_tokens: None,
            input_above_threshold: None,
            output_above_threshold: None,
            notes: String::new(),
        }
    }

    fn spawn_http_server(response: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0_u8; 2048];
            let _ = stream.read(&mut buf);
            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        });
        format!("http://{}", addr)
    }

    #[test]
    fn parse_openai_docs_extracts_standard_rows() {
        let text = "Flagship models Standard Short context Long context Model Input Cached input Output Input Cached input Output gpt-5.5 $5.00 $0.50 $30.00 - - - gpt-5.4 $2.50 $0.25 $15.00 $5.00 $0.50 $22.50 gpt-5.4-mini $0.75 $0.075 $4.50 - - - gpt-5.4-nano $0.20 $0.02 $1.25 - - - Specialized models Prices per 1M tokens. Standard Batch Priority Standard Category Model Input Cached input Output ChatGPT gpt-5.3-chat-latest $1.75 $0.175 $14.00 Codex gpt-5.3-codex $1.75 $0.175 $14.00";
        let rows = parse_openai_docs(text, &OPENAI_DEVELOPER_PRICING);
        assert!(rows.iter().any(|row| row.model_id == "gpt-5.5"));
        assert!(rows.iter().any(|row| row.model_id == "gpt-5.4"));
        assert!(rows.iter().any(|row| row.model_id == "gpt-5.4-mini"));
        assert!(rows.iter().any(|row| row.model_id == "gpt-5.3-codex"));
        let gpt55 = rows.iter().find(|row| row.model_id == "gpt-5.5").unwrap();
        assert_eq!(gpt55.input_usd_per_mtok, 5.0);
        assert_eq!(gpt55.cache_read_usd_per_mtok, 0.50);
        assert_eq!(gpt55.output_usd_per_mtok, 30.0);
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

    #[test]
    fn build_effective_catalog_prefers_higher_priority_source() {
        let rows = vec![
            model_row(
                CLAUDE_MARKETING_PRICING.slug,
                "claude-sonnet-4-5",
                9.0,
                11.25,
                0.9,
                45.0,
            ),
            model_row(
                ANTHROPIC_DOCS_PRICING.slug,
                "claude-sonnet-4-5",
                4.0,
                5.0,
                0.4,
                20.0,
            ),
        ];

        let catalog = build_effective_catalog(&rows);
        let sonnet = catalog.get("claude-sonnet-4-5").unwrap();
        assert_eq!(sonnet.input, 4.0);
        assert_eq!(sonnet.cache_write, 5.0);
        assert_eq!(sonnet.cache_read, 0.4);
        assert_eq!(sonnet.output, 20.0);
        assert_eq!(sonnet.threshold_tokens, Some(200_000));
        assert_eq!(sonnet.input_above_threshold, Some(6.0));
        assert_eq!(sonnet.output_above_threshold, Some(22.5));
    }

    #[test]
    fn effective_catalog_version_uses_max_run_per_source_in_sorted_order() {
        let rows = vec![
            StoredPricingModel {
                run_id: 12,
                source_slug: OPENAI_DEVELOPER_PRICING.slug.to_string(),
                provider: "openai".to_string(),
                ..model_row(
                    OPENAI_DEVELOPER_PRICING.slug,
                    "gpt-5.4",
                    2.5,
                    2.5,
                    0.25,
                    15.0,
                )
            },
            StoredPricingModel {
                run_id: 7,
                source_slug: ANTHROPIC_DOCS_PRICING.slug.to_string(),
                ..model_row(
                    ANTHROPIC_DOCS_PRICING.slug,
                    "claude-sonnet-4-6",
                    3.0,
                    3.75,
                    0.3,
                    15.0,
                )
            },
            StoredPricingModel {
                run_id: 9,
                source_slug: ANTHROPIC_DOCS_PRICING.slug.to_string(),
                ..model_row(
                    ANTHROPIC_DOCS_PRICING.slug,
                    "claude-haiku-4-5",
                    1.0,
                    1.25,
                    0.1,
                    5.0,
                )
            },
        ];

        assert_eq!(
            effective_catalog_version(&rows),
            "official:anthropic_api_docs#9;openai_api_docs#12"
        );
        assert_eq!(effective_catalog_version(&[]), "official:none");
    }

    #[test]
    fn diff_catalogs_reports_added_removed_and_changed_models() {
        let old_catalog = HashMap::from([
            (
                "gpt-5.4".to_string(),
                ModelPricing {
                    input: 2.5,
                    output: 15.0,
                    cache_write: 2.5,
                    cache_read: 0.25,
                    threshold_tokens: None,
                    input_above_threshold: None,
                    output_above_threshold: None,
                },
            ),
            (
                "removed-model".to_string(),
                ModelPricing {
                    input: 1.0,
                    output: 2.0,
                    cache_write: 1.0,
                    cache_read: 0.1,
                    threshold_tokens: None,
                    input_above_threshold: None,
                    output_above_threshold: None,
                },
            ),
        ]);
        let new_catalog = HashMap::from([
            (
                "gpt-5.4".to_string(),
                ModelPricing {
                    input: 3.0,
                    output: 15.0,
                    cache_write: 3.0,
                    cache_read: 0.25,
                    threshold_tokens: None,
                    input_above_threshold: None,
                    output_above_threshold: None,
                },
            ),
            (
                "new-model".to_string(),
                ModelPricing {
                    input: 4.0,
                    output: 5.0,
                    cache_write: 4.0,
                    cache_read: 0.4,
                    threshold_tokens: None,
                    input_above_threshold: None,
                    output_above_threshold: None,
                },
            ),
        ]);

        assert_eq!(
            diff_catalogs(&old_catalog, &new_catalog),
            vec![
                "gpt-5.4".to_string(),
                "new-model".to_string(),
                "removed-model".to_string(),
            ]
        );
    }

    #[test]
    fn fetch_source_body_returns_error_for_http_failures() {
        let url = spawn_http_server(
            "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 12\r\nConnection: close\r\n\r\ntry later...",
        );

        let error = fetch_source_body(&url).unwrap_err();
        assert!(error.contains("503"));
    }

    #[test]
    fn sync_pricing_records_fetch_and_parse_failures_without_repricing() {
        let conn = test_conn();

        let summary = sync_pricing_with_fetch(&conn, SOURCES, |source| match source.slug {
            "openai_api_docs" => Ok("<html>no pricing table here</html>".to_string()),
            "anthropic_api_docs" => Err("upstream unavailable".to_string()),
            "claude_marketing_pricing" => Ok(
                "Sonnet 4.5 Input $3 / MTok Output $15 / MTok Prompt caching Write $3.75 / MTok Read $0.30 / MTok"
                    .to_string(),
            ),
            other => panic!("test fixture missing handler for source slug: {other}"),
        })
        .unwrap();

        assert_eq!(summary.total_sources, 3);
        assert_eq!(summary.successful_sources, 1);
        assert!(summary.changed_models.is_empty());
        assert_eq!(summary.repriced_turns, 0);
        assert_eq!(summary.repriced_sessions, 0);
        assert_eq!(summary.pricing_version, None);
        assert_eq!(count_sessions(&conn).unwrap(), 0);

        let runs: Vec<(String, String, String)> = conn
            .prepare(
                "SELECT source_slug, status, error_text
                 FROM pricing_sync_runs
                 ORDER BY id",
            )
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .filter_map(|row| row.ok())
            .collect();
        assert_eq!(
            runs,
            vec![
                (
                    "openai_api_docs".to_string(),
                    STATUS_PARSE_ERROR.to_string(),
                    "no recognizable pricing rows found".to_string(),
                ),
                (
                    "anthropic_api_docs".to_string(),
                    STATUS_FETCH_ERROR.to_string(),
                    "upstream unavailable".to_string(),
                ),
                (
                    "claude_marketing_pricing".to_string(),
                    STATUS_SUCCESS.to_string(),
                    String::new(),
                ),
            ]
        );

        let latest = db::load_latest_pricing_models(&conn).unwrap();
        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].source_slug, "claude_marketing_pricing");
        assert_eq!(latest[0].model_id, "claude-sonnet-4-5");
    }

    #[test]
    fn sync_pricing_reprices_turns_and_sets_effective_catalog_version() {
        let conn = test_conn();
        insert_turns(
            &conn,
            &[Turn {
                session_id: "openai:s1".into(),
                provider: "openai".into(),
                timestamp: "2026-04-19T12:00:00Z".into(),
                model: "gpt-5.4".into(),
                input_tokens: 1_000_000,
                output_tokens: 0,
                message_id: "msg-1".into(),
                pricing_version: "static@old".into(),
                pricing_model: "gpt-5.4".into(),
                billing_mode: "estimated_local".into(),
                cost_confidence: "high".into(),
                estimated_cost_nanos: 2_500_000_000,
                ..Default::default()
            }],
        )
        .unwrap();
        upsert_sessions(
            &conn,
            &[Session {
                session_id: "openai:s1".into(),
                provider: "openai".into(),
                ..Default::default()
            }],
        )
        .unwrap();

        let summary = sync_pricing_with_fetch(&conn, &[OPENAI_DEVELOPER_PRICING], |_| {
            Ok(
                "Flagship models Standard Short context Long context Model Input Cached input Output Input Cached input Output gpt-5.4 $2.50 $0.25 $15.00 $5.00 $0.50 $22.50"
                    .to_string(),
            )
        })
        .unwrap();

        assert_eq!(summary.total_sources, 1);
        assert_eq!(summary.successful_sources, 1);
        assert_eq!(summary.changed_models, vec!["gpt-5.4".to_string()]);
        assert_eq!(summary.repriced_turns, 1);
        assert_eq!(summary.repriced_sessions, 1);
        assert_eq!(
            summary.pricing_version,
            Some("official:openai_api_docs#1".to_string())
        );

        let turn: (i64, String) = conn
            .query_row(
                "SELECT estimated_cost_nanos, pricing_version
                 FROM turns
                 WHERE message_id = 'msg-1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(turn.0, 4_325_000_000);
        assert_eq!(turn.1, "official:openai_api_docs#1");
    }

    #[test]
    fn official_source_catalogs_cover_status_and_exchange_rates() {
        assert_eq!(
            OfficialSourceKind::StatusIncidents.as_str(),
            "status_incidents"
        );
        assert_eq!(
            OfficialSourceAuthority::ProviderStatus.as_str(),
            "provider_status"
        );
        assert_eq!(OfficialSourceCadence::Daily.as_str(), "daily");

        assert_eq!(STATUS_SOURCES.len(), 2);
        assert_eq!(
            OPENAI_STATUS_SOURCE.incidents_url,
            Some("https://status.openai.com/api/v2/incidents.json")
        );
        assert_eq!(ANTHROPIC_STATUS_SOURCE.vendor, StatusVendor::Statuspage);

        assert_eq!(EXCHANGE_RATE_SOURCES.len(), 1);
        assert_eq!(
            FRANKFURTER_EXCHANGE_RATE_SOURCE.url,
            "https://api.frankfurter.dev/v1/latest?from=USD"
        );
        assert_eq!(
            FRANKFURTER_EXCHANGE_RATE_SOURCE.upstream_provider,
            Some("ecb")
        );
    }

    #[test]
    fn official_model_snapshot_holds_extended_metadata_categories() {
        let snapshot = OfficialModelSnapshot {
            pricing: OfficialModelPricing {
                source_slug: OPENAI_DEVELOPER_PRICING.slug.to_string(),
                provider: "openai".to_string(),
                model_id: "gpt-5.4".to_string(),
                model_label: "GPT-5.4".to_string(),
                input_usd_per_mtok: 2.5,
                cache_write_usd_per_mtok: 2.5,
                cache_read_usd_per_mtok: 0.25,
                output_usd_per_mtok: 15.0,
                threshold_tokens: Some(270_000),
                input_above_threshold: Some(5.0),
                output_above_threshold: Some(22.5),
                notes: "official docs".to_string(),
            },
            lifecycle: Some(ModelLifecycleMetadata {
                stage: ModelLifecycleStage::GenerallyAvailable,
                announced_at: Some("2026-03-01T00:00:00Z".to_string()),
                generally_available_at: Some("2026-03-15T00:00:00Z".to_string()),
                deprecation_announced_at: None,
                sunset_at: None,
                replacement_model_id: None,
                notes: vec!["default flagship".to_string()],
            }),
            context_window: Some(ContextWindowMetadata {
                max_input_tokens: Some(270_000),
                max_output_tokens: Some(128_000),
                max_context_tokens: Some(398_000),
                tokenizer_family: Some(TokenizerFamily::Cl100kO200k),
                tokenizer_name: Some("o200k_base".to_string()),
                tokenizer_notes: vec!["tool-call JSON counts toward context".to_string()],
                truncation_behavior: Some("reject_over_limit".to_string()),
            }),
            prompt_caching: Some(PromptCachingPolicyMetadata {
                supported: true,
                default_ttl_seconds: Some(300),
                max_ttl_seconds: Some(3_600),
                refresh_resets_ttl: Some(true),
                write_priced_as_input: Some(true),
                cache_read_discount_pct: Some(90.0),
                cache_write_multiplier: Some(1.0),
                notes: vec!["long-context tiers can differ".to_string()],
            }),
            processing_modes: vec![
                ProcessingModePricingMetadata {
                    mode: ProcessingMode::Batch,
                    region_scope: None,
                    input_usd_per_mtok: Some(1.25),
                    cache_write_usd_per_mtok: Some(1.25),
                    cache_read_usd_per_mtok: Some(0.125),
                    output_usd_per_mtok: Some(7.5),
                    relative_uplift_pct: Some(-50.0),
                    notes: vec!["batch discount".to_string()],
                },
                ProcessingModePricingMetadata {
                    mode: ProcessingMode::Regional,
                    region_scope: Some("eu".to_string()),
                    input_usd_per_mtok: Some(2.75),
                    cache_write_usd_per_mtok: None,
                    cache_read_usd_per_mtok: None,
                    output_usd_per_mtok: Some(16.5),
                    relative_uplift_pct: Some(10.0),
                    notes: vec!["regional data residency uplift".to_string()],
                },
            ],
            tool_pricing: vec![OfficialToolPricing {
                source_slug: OPENAI_DEVELOPER_PRICING.slug.to_string(),
                provider: "openai".to_string(),
                tool_slug: "web-search".to_string(),
                tool_label: "Web Search".to_string(),
                model_id: Some("gpt-5.4".to_string()),
                billing_unit: ToolBillingUnit::Per1KCalls,
                unit_price_usd: 12.0,
                included_units: Some(100.0),
                notes: vec!["search content tokens billed separately".to_string()],
            }],
            release_notes: vec![ReleaseNoteSnapshot {
                source_slug: "openai_release_notes".to_string(),
                provider: "openai".to_string(),
                snapshot_id: "2026-03-15-gpt-5-4".to_string(),
                title: "GPT-5.4 launch".to_string(),
                url: "https://developers.openai.com/release-notes".to_string(),
                published_at: Some("2026-03-15T00:00:00Z".to_string()),
                kind: ReleaseNoteKind::Launch,
                summary: "Introduced GPT-5.4 with expanded long-context pricing.".to_string(),
                affected_models: vec!["gpt-5.4".to_string()],
                notes: vec!["pricing changed same day".to_string()],
            }],
            source_integrity: Some(SourceIntegrityMetadata {
                captured_at: Some("2026-03-15T00:05:00Z".to_string()),
                response_status_code: Some(200),
                content_type: Some("text/html; charset=utf-8".to_string()),
                etag: Some("\"pricing-v2\"".to_string()),
                last_modified: Some("Sat, 15 Mar 2026 00:00:00 GMT".to_string()),
                raw_body_bytes: Some(18_240),
                normalized_body_bytes: Some(4_096),
                digests: vec![
                    SourceHashDigest {
                        algorithm: IntegrityHashAlgorithm::Sha256,
                        scope: IntegrityPayloadScope::RawBody,
                        value: "abc123".to_string(),
                    },
                    SourceHashDigest {
                        algorithm: IntegrityHashAlgorithm::Sha256,
                        scope: IntegrityPayloadScope::ExtractedPricingRows,
                        value: "def456".to_string(),
                    },
                ],
                parser_version: Some("official_pricing/v2".to_string()),
                parser_warnings: vec!["marketing page omitted batch tier".to_string()],
                signature: Some(SourceSignatureMetadata {
                    algorithm: Some("ed25519".to_string()),
                    key_id: Some("pricing-key-1".to_string()),
                    verified: true,
                    notes: vec!["signature verified after normalization".to_string()],
                }),
            }),
        };

        assert_eq!(
            snapshot.lifecycle.as_ref().map(|lifecycle| lifecycle.stage),
            Some(ModelLifecycleStage::GenerallyAvailable)
        );
        assert_eq!(
            snapshot
                .context_window
                .as_ref()
                .and_then(|window| window.tokenizer_family),
            Some(TokenizerFamily::Cl100kO200k)
        );
        assert_eq!(snapshot.processing_modes.len(), 2);
        assert_eq!(
            snapshot.tool_pricing[0].billing_unit,
            ToolBillingUnit::Per1KCalls
        );
        assert_eq!(snapshot.release_notes[0].kind, ReleaseNoteKind::Launch);
        assert_eq!(
            snapshot
                .source_integrity
                .as_ref()
                .map(|integrity| integrity.digests.len()),
            Some(2)
        );
    }
}
