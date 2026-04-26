use crate::agent_status::models::ProviderStatus;
use crate::config::AgentStatusConfig;
use serde::Serialize;

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
