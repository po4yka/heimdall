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
use crate::config::AgentStatusConfig;
use crate::currency::RatesSnapshot;
use crate::models::OpenAiReconciliation;
use crate::openai;
use crate::pricing::{self, ModelPricing};
use crate::scanner::db;

const FETCH_TIMEOUT_SECS: u64 = 10;
const PARSER_VERSION: &str = "official_pricing/v3";
const OPENAI_USAGE_URL: &str = "https://api.openai.com/v1/organization/usage/completions";

pub const STATUS_SUCCESS: &str = "success";
pub const STATUS_FETCH_ERROR: &str = "fetch_error";
pub const STATUS_PARSE_ERROR: &str = "parse_error";
pub const STATUS_SKIPPED: &str = "skipped";

#[derive(Debug, Clone, Copy)]
pub struct PricingSourceDef {
    pub slug: &'static str,
    pub provider: &'static str,
    pub url: &'static str,
    pub priority: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficialSourceKind {
    Pricing,
    ModelCatalog,
    ReleaseNotes,
    StatusSummary,
    StatusIncidents,
    ExchangeRates,
    UsageReconciliation,
}

impl OfficialSourceKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pricing => "pricing",
            Self::ModelCatalog => "model_catalog",
            Self::ReleaseNotes => "release_notes",
            Self::StatusSummary => "status_summary",
            Self::StatusIncidents => "status_incidents",
            Self::ExchangeRates => "exchange_rates",
            Self::UsageReconciliation => "usage_reconciliation",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficialSourceFormat {
    Html,
    Json,
    Markdown,
    Xml,
}

impl OfficialSourceFormat {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Json => "json",
            Self::Markdown => "markdown",
            Self::Xml => "xml",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficialSourceAuthority {
    ProviderDocs,
    ProviderMarketing,
    ProviderStatus,
    ProviderReleaseNotes,
    AggregatorApi,
    UpstreamReference,
}

impl OfficialSourceAuthority {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ProviderDocs => "provider_docs",
            Self::ProviderMarketing => "provider_marketing",
            Self::ProviderStatus => "provider_status",
            Self::ProviderReleaseNotes => "provider_release_notes",
            Self::AggregatorApi => "aggregator_api",
            Self::UpstreamReference => "upstream_reference",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfficialSourceCadence {
    Realtime,
    Hourly,
    Daily,
    Weekly,
    AdHoc,
}

impl OfficialSourceCadence {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Realtime => "realtime",
            Self::Hourly => "hourly",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::AdHoc => "ad_hoc",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusVendor {
    Statuspage,
    IncidentIo,
    Custom,
}

impl StatusVendor {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Statuspage => "statuspage",
            Self::IncidentIo => "incident_io",
            Self::Custom => "custom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelLifecycleStage {
    Preview,
    GenerallyAvailable,
    Legacy,
    Deprecated,
    Sunset,
    Retired,
}

impl ModelLifecycleStage {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Preview => "preview",
            Self::GenerallyAvailable => "generally_available",
            Self::Legacy => "legacy",
            Self::Deprecated => "deprecated",
            Self::Sunset => "sunset",
            Self::Retired => "retired",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenizerFamily {
    Cl100kO200k,
    SentencePiece,
    Bpe,
    ProviderSpecific,
}

impl TokenizerFamily {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Cl100kO200k => "cl100k_o200k",
            Self::SentencePiece => "sentencepiece",
            Self::Bpe => "bpe",
            Self::ProviderSpecific => "provider_specific",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingMode {
    Standard,
    Batch,
    Priority,
    Flex,
    Regional,
}

impl ProcessingMode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Batch => "batch",
            Self::Priority => "priority",
            Self::Flex => "flex",
            Self::Regional => "regional",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolBillingUnit {
    PerCall,
    PerMinute,
    PerImage,
    PerSession,
    Per1KCalls,
    Per1MTokens,
}

impl ToolBillingUnit {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::PerCall => "per_call",
            Self::PerMinute => "per_minute",
            Self::PerImage => "per_image",
            Self::PerSession => "per_session",
            Self::Per1KCalls => "per_1k_calls",
            Self::Per1MTokens => "per_1m_tokens",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseNoteKind {
    Launch,
    Update,
    Deprecation,
    Pricing,
    Capability,
    ContextWindow,
    Tooling,
    Reliability,
}

impl ReleaseNoteKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Launch => "launch",
            Self::Update => "update",
            Self::Deprecation => "deprecation",
            Self::Pricing => "pricing",
            Self::Capability => "capability",
            Self::ContextWindow => "context_window",
            Self::Tooling => "tooling",
            Self::Reliability => "reliability",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrityHashAlgorithm {
    Sha256,
    Sha512,
    Blake3,
}

impl IntegrityHashAlgorithm {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Sha512 => "sha512",
            Self::Blake3 => "blake3",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrityPayloadScope {
    RawBody,
    NormalizedBody,
    ExtractedMetadata,
    ExtractedPricingRows,
}

impl IntegrityPayloadScope {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::RawBody => "raw_body",
            Self::NormalizedBody => "normalized_body",
            Self::ExtractedMetadata => "extracted_metadata",
            Self::ExtractedPricingRows => "extracted_pricing_rows",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfficialContentSourceDef {
    pub slug: &'static str,
    pub provider: &'static str,
    pub url: &'static str,
    pub kind: OfficialSourceKind,
    pub format: OfficialSourceFormat,
    pub authority: OfficialSourceAuthority,
    pub cadence: OfficialSourceCadence,
    pub priority: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusSourceDef {
    pub slug: &'static str,
    pub provider: &'static str,
    pub page_url: &'static str,
    pub summary_url: &'static str,
    pub incidents_url: Option<&'static str>,
    pub vendor: StatusVendor,
    pub format: OfficialSourceFormat,
    pub authority: OfficialSourceAuthority,
    pub cadence: OfficialSourceCadence,
    pub priority: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExchangeRateSourceDef {
    pub slug: &'static str,
    pub provider: &'static str,
    pub url: &'static str,
    pub base_currency: &'static str,
    pub quote_currency: Option<&'static str>,
    pub format: OfficialSourceFormat,
    pub authority: OfficialSourceAuthority,
    pub cadence: OfficialSourceCadence,
    pub upstream_provider: Option<&'static str>,
    pub priority: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModelLifecycleMetadata {
    pub stage: ModelLifecycleStage,
    pub announced_at: Option<String>,
    pub generally_available_at: Option<String>,
    pub deprecation_announced_at: Option<String>,
    pub sunset_at: Option<String>,
    pub replacement_model_id: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextWindowMetadata {
    pub max_input_tokens: Option<i64>,
    pub max_output_tokens: Option<i64>,
    pub max_context_tokens: Option<i64>,
    pub tokenizer_family: Option<TokenizerFamily>,
    pub tokenizer_name: Option<String>,
    pub tokenizer_notes: Vec<String>,
    pub truncation_behavior: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PromptCachingPolicyMetadata {
    pub supported: bool,
    pub default_ttl_seconds: Option<u64>,
    pub max_ttl_seconds: Option<u64>,
    pub refresh_resets_ttl: Option<bool>,
    pub write_priced_as_input: Option<bool>,
    pub cache_read_discount_pct: Option<f64>,
    pub cache_write_multiplier: Option<f64>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProcessingModePricingMetadata {
    pub mode: ProcessingMode,
    pub region_scope: Option<String>,
    pub input_usd_per_mtok: Option<f64>,
    pub cache_write_usd_per_mtok: Option<f64>,
    pub cache_read_usd_per_mtok: Option<f64>,
    pub output_usd_per_mtok: Option<f64>,
    pub relative_uplift_pct: Option<f64>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OfficialToolPricing {
    pub source_slug: String,
    pub provider: String,
    pub tool_slug: String,
    pub tool_label: String,
    pub model_id: Option<String>,
    pub billing_unit: ToolBillingUnit,
    pub unit_price_usd: f64,
    pub included_units: Option<f64>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReleaseNoteSnapshot {
    pub source_slug: String,
    pub provider: String,
    pub snapshot_id: String,
    pub title: String,
    pub url: String,
    pub published_at: Option<String>,
    pub kind: ReleaseNoteKind,
    pub summary: String,
    pub affected_models: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceHashDigest {
    pub algorithm: IntegrityHashAlgorithm,
    pub scope: IntegrityPayloadScope,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceSignatureMetadata {
    pub algorithm: Option<String>,
    pub key_id: Option<String>,
    pub verified: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceIntegrityMetadata {
    pub captured_at: Option<String>,
    pub response_status_code: Option<u16>,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub raw_body_bytes: Option<usize>,
    pub normalized_body_bytes: Option<usize>,
    pub digests: Vec<SourceHashDigest>,
    pub parser_version: Option<String>,
    pub parser_warnings: Vec<String>,
    pub signature: Option<SourceSignatureMetadata>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OfficialModelSnapshot {
    pub pricing: OfficialModelPricing,
    pub lifecycle: Option<ModelLifecycleMetadata>,
    pub context_window: Option<ContextWindowMetadata>,
    pub prompt_caching: Option<PromptCachingPolicyMetadata>,
    pub processing_modes: Vec<ProcessingModePricingMetadata>,
    pub tool_pricing: Vec<OfficialToolPricing>,
    pub release_notes: Vec<ReleaseNoteSnapshot>,
    pub source_integrity: Option<SourceIntegrityMetadata>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OfficialModelMetadataRecord {
    pub provider: String,
    pub model_id: String,
    pub model_label: String,
    pub lifecycle: Option<ModelLifecycleMetadata>,
    pub context_window: Option<ContextWindowMetadata>,
    pub prompt_caching: Option<PromptCachingPolicyMetadata>,
    pub processing_modes: Vec<ProcessingModePricingMetadata>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExchangeRateRecord {
    pub provider: String,
    pub source_slug: String,
    pub base_currency: String,
    pub quote_currency: String,
    pub rate: f64,
    pub upstream_provider: Option<String>,
    pub observed_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatusSnapshotRecord {
    pub provider: String,
    pub source_slug: String,
    pub page_url: String,
    pub snapshot: ProviderStatus,
}

#[derive(Debug, Clone)]
pub struct OfficialSyncOptions {
    pub openai_admin_key: Option<String>,
    pub openai_lookback_days: i64,
    pub agent_status_config: AgentStatusConfig,
}

impl Default for OfficialSyncOptions {
    fn default() -> Self {
        Self {
            openai_admin_key: None,
            openai_lookback_days: 30,
            agent_status_config: AgentStatusConfig::default(),
        }
    }
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

pub const OPENAI_MODELS_SOURCE: OfficialContentSourceDef = OfficialContentSourceDef {
    slug: "openai_models_docs",
    provider: "openai",
    url: "https://developers.openai.com/api/docs/models",
    kind: OfficialSourceKind::ModelCatalog,
    format: OfficialSourceFormat::Html,
    authority: OfficialSourceAuthority::ProviderDocs,
    cadence: OfficialSourceCadence::Daily,
    priority: 100,
};

pub const ANTHROPIC_MODELS_SOURCE: OfficialContentSourceDef = OfficialContentSourceDef {
    slug: "anthropic_models_docs",
    provider: "anthropic",
    url: "https://platform.claude.com/docs/en/about-claude/models/overview",
    kind: OfficialSourceKind::ModelCatalog,
    format: OfficialSourceFormat::Html,
    authority: OfficialSourceAuthority::ProviderDocs,
    cadence: OfficialSourceCadence::Daily,
    priority: 100,
};

pub const OPENAI_CHANGELOG_SOURCE: OfficialContentSourceDef = OfficialContentSourceDef {
    slug: "openai_api_changelog",
    provider: "openai",
    url: "https://developers.openai.com/api/docs/changelog",
    kind: OfficialSourceKind::ReleaseNotes,
    format: OfficialSourceFormat::Html,
    authority: OfficialSourceAuthority::ProviderReleaseNotes,
    cadence: OfficialSourceCadence::Daily,
    priority: 100,
};

pub const ANTHROPIC_RELEASE_NOTES_SOURCE: OfficialContentSourceDef = OfficialContentSourceDef {
    slug: "anthropic_api_release_notes",
    provider: "anthropic",
    url: "https://platform.claude.com/docs/en/release-notes/overview",
    kind: OfficialSourceKind::ReleaseNotes,
    format: OfficialSourceFormat::Html,
    authority: OfficialSourceAuthority::ProviderReleaseNotes,
    cadence: OfficialSourceCadence::Daily,
    priority: 100,
};

pub const CONTENT_SOURCES: &[OfficialContentSourceDef] = &[
    OPENAI_MODELS_SOURCE,
    ANTHROPIC_MODELS_SOURCE,
    OPENAI_CHANGELOG_SOURCE,
    ANTHROPIC_RELEASE_NOTES_SOURCE,
];

pub const OPENAI_STATUS_SOURCE: StatusSourceDef = StatusSourceDef {
    slug: "openai_status",
    provider: "openai",
    page_url: "https://status.openai.com",
    summary_url: "https://status.openai.com/api/v2/status.json",
    incidents_url: Some("https://status.openai.com/api/v2/incidents.json"),
    vendor: StatusVendor::IncidentIo,
    format: OfficialSourceFormat::Json,
    authority: OfficialSourceAuthority::ProviderStatus,
    cadence: OfficialSourceCadence::Realtime,
    priority: 100,
};

pub const ANTHROPIC_STATUS_SOURCE: StatusSourceDef = StatusSourceDef {
    slug: "anthropic_status",
    provider: "anthropic",
    page_url: "https://status.claude.com",
    summary_url: "https://status.claude.com/api/v2/summary.json",
    incidents_url: None,
    vendor: StatusVendor::Statuspage,
    format: OfficialSourceFormat::Json,
    authority: OfficialSourceAuthority::ProviderStatus,
    cadence: OfficialSourceCadence::Realtime,
    priority: 100,
};

pub const STATUS_SOURCES: &[StatusSourceDef] = &[OPENAI_STATUS_SOURCE, ANTHROPIC_STATUS_SOURCE];

pub const FRANKFURTER_EXCHANGE_RATE_SOURCE: ExchangeRateSourceDef = ExchangeRateSourceDef {
    slug: "frankfurter_usd_latest",
    provider: "frankfurter",
    url: "https://api.frankfurter.dev/v1/latest?from=USD",
    base_currency: "USD",
    quote_currency: None,
    format: OfficialSourceFormat::Json,
    authority: OfficialSourceAuthority::AggregatorApi,
    cadence: OfficialSourceCadence::Daily,
    upstream_provider: Some("ecb"),
    priority: 100,
};

pub const EXCHANGE_RATE_SOURCES: &[ExchangeRateSourceDef] = &[FRANKFURTER_EXCHANGE_RATE_SOURCE];

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

#[derive(Debug, Clone, PartialEq, Serialize)]
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
pub struct OfficialSyncRunRecord {
    pub fetched_at: String,
    pub source_slug: String,
    pub source_kind: String,
    pub source_url: String,
    pub provider: String,
    pub authority: String,
    pub format: String,
    pub cadence: String,
    pub status: String,
    pub http_status: Option<i64>,
    pub content_type: String,
    pub etag: String,
    pub last_modified: String,
    pub raw_body: String,
    pub normalized_body: String,
    pub error_text: String,
    pub parser_version: String,
    pub raw_body_sha256: String,
    pub normalized_body_sha256: String,
    pub extracted_sha256: String,
}

#[derive(Debug, Clone)]
pub struct OfficialExtractedRecord {
    pub source_slug: String,
    pub provider: String,
    pub record_type: String,
    pub record_key: String,
    pub model_id: String,
    pub effective_at: String,
    pub payload_json: String,
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
    pub metadata_runs: usize,
    pub metadata_records: usize,
    pub changed_models: Vec<String>,
    pub repriced_turns: usize,
    pub repriced_sessions: usize,
    pub pricing_version: Option<String>,
}

#[derive(Debug, Clone)]
struct FetchedSourceBody {
    http_status: u16,
    content_type: String,
    etag: String,
    last_modified: String,
    raw_body: String,
    normalized_body: String,
}

pub fn sync_pricing(conn: &Connection, options: &OfficialSyncOptions) -> Result<PricingSyncSummary> {
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
                let records = build_records_for_content_source(
                    source,
                    &fetched.normalized_body,
                    &fetched_at,
                );
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
            _ => unreachable!(),
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
