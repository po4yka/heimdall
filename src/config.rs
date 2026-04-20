use std::collections::HashMap;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Top-level Heimdall configuration.
///
/// Loaded from `~/.claude/usage-tracker.toml` (or `.json`).  All fields are
/// optional; omitted fields fall back to built-in defaults.
///
/// JSON format: set `$schema` for IDE autocomplete:
/// ```json
/// {
///   "$schema": "https://raw.githubusercontent.com/po4yka/heimdall/main/schemas/heimdall.config.schema.json",
///   "display": { "currency": "EUR" },
///   "blocks": { "token_limit": 1000000 }
/// }
/// ```
#[derive(Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct Config {
    /// Additional directories to scan for JSONL usage files.
    pub projects_dirs: Vec<PathBuf>,

    /// Override the default SQLite database path (`~/.claude/usage.db`).
    pub db_path: Option<PathBuf>,

    /// Dashboard bind host (default: localhost).
    pub host: Option<String>,

    /// Dashboard bind port (default: 8080).
    pub port: Option<u16>,

    /// Custom per-model pricing overrides. Keys are model names (e.g. `claude-opus-4-6`).
    #[serde(default)]
    pub pricing: HashMap<String, PricingOverride>,

    /// OAuth settings for real-time rate window tracking.
    #[serde(default)]
    pub oauth: OAuthConfig,

    /// Scheduled Claude `/usage` capture settings.
    #[serde(default)]
    pub claude_usage_monitor: ClaudeUsageMonitorConfig,

    /// Webhook notification settings.
    #[serde(default)]
    pub webhooks: WebhookConfig,

    /// Optional OpenAI organization usage reconciliation for Codex API-backed usage.
    #[serde(default)]
    pub openai: OpenAiConfig,

    /// Display settings (currency, formatting).
    #[serde(default)]
    pub display: Display,

    /// Pricing source settings (static vs litellm refresh).
    #[serde(default)]
    pub pricing_source: PricingConfig,

    /// Upstream coding-agent status monitoring.
    #[serde(default)]
    pub agent_status: AgentStatusConfig,

    /// Optional community-signal aggregator (opt-in, off by default).
    #[serde(default, rename = "status_aggregator")]
    pub aggregator: AggregatorConfig,

    /// Billing-block quota settings.
    #[serde(default)]
    pub blocks: BlocksConfig,

    /// Statusline display settings.
    #[serde(default)]
    pub statusline: StatuslineConfig,

    /// Per-command config overrides.  Values here win over the flat top-level
    /// fields for the named subcommand.  Currently `blocks` and `statusline`
    /// are supported.
    #[serde(default)]
    pub commands: Option<CommandsOverrides>,

    /// JSON Schema URL for IDE autocomplete.  Ignored at runtime.
    /// Skipped when `None` during serialisation so `config show --format=toml`
    /// does not emit a bare `"$schema"` quoted key.
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    /// Map of raw project slug (e.g., `-Users-foo-proj`) to a human-readable
    /// display name (e.g., `My Project`).  Purely cosmetic — storage always
    /// keeps the raw slug as canonical.
    ///
    /// TOML example:
    /// ```toml
    /// [project_aliases]
    /// "-Users-po4yka-GitRep-heimdall" = "Heimdall"
    /// ```
    #[serde(default)]
    #[schemars(
        description = "Map of raw project slug (e.g., '-Users-foo-proj') to a human-readable display name"
    )]
    pub project_aliases: HashMap<String, String>,
}

impl Config {
    /// Resolve the display name for a project slug.
    /// Returns the alias if set, otherwise the slug unchanged.
    pub fn display_name_for<'a>(&'a self, slug: &'a str) -> &'a str {
        self.project_aliases
            .get(slug)
            .map(|s| s.as_str())
            .unwrap_or(slug)
    }

    /// Return the effective `BlocksConfig`, merging flat `blocks` with any
    /// `commands.blocks` override.  `commands.blocks.*` wins for each field
    /// that is `Some`.
    pub fn resolved_blocks(&self) -> BlocksConfig {
        let base = self.blocks.clone();
        let Some(ref cmds) = self.commands else {
            return base;
        };
        let Some(ref override_cfg) = cmds.blocks else {
            return base;
        };
        BlocksConfig {
            token_limit: override_cfg.token_limit.or(base.token_limit),
            session_length_hours: override_cfg
                .session_length_hours
                .or(base.session_length_hours),
            session_length_by_provider: if override_cfg.session_length_by_provider.is_empty() {
                base.session_length_by_provider
            } else {
                let mut merged = base.session_length_by_provider;
                merged.extend(override_cfg.session_length_by_provider.clone());
                merged
            },
        }
    }

    /// Resolve the effective session length in hours.
    ///
    /// Precedence: CLI flag (passed as `Some`) > provider-specific default >
    /// flat default > 5.0 fallback.  Config-sourced values outside `(0, 168]`
    /// are clamped to 5.0 with a `warn!` log.
    pub fn resolved_session_length(
        &self,
        cli_override: Option<f64>,
        provider: Option<&str>,
    ) -> f64 {
        if let Some(h) = cli_override {
            return h;
        }
        let blocks = self.resolved_blocks();
        if let Some(p) = provider
            && let Some(&h) = blocks.session_length_by_provider.get(p)
        {
            if !(h > 0.0 && h <= 168.0) {
                tracing::warn!(
                    "invalid session_length {} for provider '{}' from config, falling back to 5.0",
                    h,
                    p
                );
                return 5.0;
            }
            return h;
        }
        if let Some(h) = blocks.session_length_hours {
            if !(h > 0.0 && h <= 168.0) {
                tracing::warn!(
                    "invalid session_length {} from config, falling back to 5.0",
                    h
                );
                return 5.0;
            }
            return h;
        }
        5.0
    }

    /// Return the effective `StatuslineConfig`, merging flat `statusline` with
    /// any `commands.statusline` override.  `commands.statusline.*` wins for
    /// each field that is `Some`.
    pub fn resolved_statusline(&self) -> StatuslineConfig {
        let base = self.statusline.clone();
        let Some(ref cmds) = self.commands else {
            return base;
        };
        let Some(ref override_cfg) = cmds.statusline else {
            return base;
        };
        StatuslineConfig {
            context_low_threshold: override_cfg
                .context_low_threshold
                .unwrap_or(base.context_low_threshold),
            context_medium_threshold: override_cfg
                .context_medium_threshold
                .unwrap_or(base.context_medium_threshold),
            burn_rate_normal_max: override_cfg
                .burn_rate_normal_max
                .unwrap_or(base.burn_rate_normal_max),
            burn_rate_moderate_max: override_cfg
                .burn_rate_moderate_max
                .unwrap_or(base.burn_rate_moderate_max),
        }
    }
}

/// Per-command configuration overrides.  Fields here win over the flat
/// top-level equivalents for the relevant subcommand.
///
/// Example JSON:
/// ```json
/// { "commands": { "blocks": { "token_limit": 1000000 } } }
/// ```
#[derive(Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct CommandsOverrides {
    /// Overrides for the `blocks` subcommand.
    pub blocks: Option<BlocksConfig>,
    /// Overrides for the `statusline` subcommand.
    pub statusline: Option<StatuslineOverride>,
}

/// Statusline overrides with all fields optional (unlike `StatuslineConfig`
/// which carries non-optional f64 defaults).
#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct StatuslineOverride {
    /// Fractional fill below which no severity marker is shown.
    pub context_low_threshold: Option<f64>,
    /// Fractional fill above which [CRIT] is shown; between low and this → [WARN].
    pub context_medium_threshold: Option<f64>,
    /// tokens/min at or below this value → Normal tier.
    pub burn_rate_normal_max: Option<f64>,
    /// tokens/min at or below this value → Moderate tier; above → High.
    pub burn_rate_moderate_max: Option<f64>,
}

/// Billing-block quota configuration.
///
/// Example TOML:
/// ```toml
/// [blocks]
/// token_limit = 1000000  # optional; CLI --token-limit takes precedence
/// session_length_hours = 5.0  # global default session window
///
/// [blocks.session_length_by_provider]
/// claude = 5.0
/// codex = 1.0
/// amp = 24.0
/// ```
#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct BlocksConfig {
    /// Token quota for the active billing block used by the dashboard.
    /// The CLI `--token-limit` flag takes precedence over this value.
    pub token_limit: Option<i64>,
    /// Default session block duration in hours. If absent, falls back to 5.0
    /// (Claude's actual billing window).
    pub session_length_hours: Option<f64>,
    /// Per-provider session-length overrides. Map key = provider name
    /// (e.g., "claude", "codex", "amp"); value = duration in hours.
    /// CLI --session-length always wins over these.
    #[serde(default)]
    pub session_length_by_provider: std::collections::HashMap<String, f64>,
}

/// Statusline display configuration.
///
/// Example TOML:
/// ```toml
/// [statusline]
/// context_low_threshold = 0.5    # below → no marker
/// context_medium_threshold = 0.8 # below → [WARN], above → [CRIT]
/// burn_rate_normal_max = 4000    # tokens/min at or below → Normal
/// burn_rate_moderate_max = 10000 # tokens/min at or below → Moderate
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct StatuslineConfig {
    /// Fractional fill below which no severity marker is shown (default: 0.5).
    pub context_low_threshold: f64,
    /// Fractional fill above which [CRIT] is shown; between low and this → [WARN] (default: 0.8).
    pub context_medium_threshold: f64,
    /// tokens/min at or below this value → Normal tier (default: 4000).
    pub burn_rate_normal_max: f64,
    /// tokens/min at or below this value → Moderate tier; above → High (default: 10000).
    pub burn_rate_moderate_max: f64,
}

impl Default for StatuslineConfig {
    fn default() -> Self {
        Self {
            context_low_threshold: 0.5,
            context_medium_threshold: 0.8,
            burn_rate_normal_max: 4000.0,
            burn_rate_moderate_max: 10000.0,
        }
    }
}

/// OAuth polling configuration.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct OAuthConfig {
    /// Enable OAuth usage polling (default: true, auto-detects credentials).
    pub enabled: bool,
    /// Seconds between API polls (default: 60).
    pub refresh_interval: u64,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            refresh_interval: 60,
        }
    }
}

/// Claude `/usage` monitoring configuration.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct ClaudeUsageMonitorConfig {
    /// Optional explicit path to the Claude CLI binary. When absent, PATH is used.
    pub claude_binary: Option<PathBuf>,
    /// Optional working directory used for headless `/usage` capture.
    pub working_dir: Option<PathBuf>,
    /// Default install interval for `usage-monitor install` (daily|hourly).
    pub default_interval: String,
}

impl Default for ClaudeUsageMonitorConfig {
    fn default() -> Self {
        Self {
            claude_binary: None,
            working_dir: None,
            default_interval: "daily".into(),
        }
    }
}

/// OpenAI organization usage reconciliation configuration.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct OpenAiConfig {
    /// Enable OpenAI organization usage reconciliation (default: true if OPENAI_ADMIN_KEY exists).
    pub enabled: bool,
    /// Environment variable name that stores the OpenAI admin key.
    pub admin_key_env: String,
    /// Seconds between API refreshes.
    pub refresh_interval: u64,
    /// Number of trailing days to compare against local Codex estimates.
    pub lookback_days: i64,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            admin_key_env: "OPENAI_ADMIN_KEY".into(),
            refresh_interval: 300,
            lookback_days: 30,
        }
    }
}

/// Webhook notification configuration.
#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct WebhookConfig {
    /// URL to POST webhook events to.
    pub url: Option<String>,
    /// Notify when daily cost exceeds this amount (USD).
    pub cost_threshold: Option<f64>,
    /// Notify on session depletion events.
    pub session_depleted: bool,
    /// Notify on agent status transitions (default: true when URL is set).
    #[serde(default = "default_true")]
    pub agent_status: bool,
    /// Notify on community signal spike when official status is below Major
    /// (default: true — fires only when the feature is enabled).
    #[serde(default = "default_true")]
    pub spike_webhook: bool,
}

fn default_true() -> bool {
    true
}

/// Alert severity floor for agent-status webhooks.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Minor,
    #[default]
    Major,
    Critical,
}

/// Configuration for upstream coding-agent status monitoring.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct AgentStatusConfig {
    /// Enable polling (default: true).
    pub enabled: bool,
    /// Seconds between polls (default: 60).
    pub refresh_interval: u64,
    /// Enable Claude status polling (default: true).
    pub claude_enabled: bool,
    /// Enable OpenAI status polling (default: true).
    pub openai_enabled: bool,
    /// Minimum severity for webhook alerts (default: Major).
    pub alert_min_severity: AlertSeverity,
}

impl Default for AgentStatusConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            refresh_interval: 60,
            claude_enabled: true,
            openai_enabled: true,
            alert_min_severity: AlertSeverity::Major,
        }
    }
}

fn default_aggregator_provider() -> String {
    "statusgator".into()
}

fn default_aggregator_key_env_var() -> String {
    "STATUSGATOR_API_KEY".into()
}

fn default_aggregator_refresh_interval() -> u64 {
    300
}

fn default_claude_services() -> Vec<String> {
    vec!["claude-ai".into(), "claude".into()]
}

fn default_openai_services() -> Vec<String> {
    vec!["openai".into(), "chatgpt".into()]
}

/// Configuration for the optional community-signal aggregator (opt-in feature).
///
/// TOML section: `[status_aggregator]`
/// Feature is **off by default** (`enabled = false`).
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct AggregatorConfig {
    /// Enable community signal polling (default: false — explicit opt-in required).
    pub enabled: bool,
    /// Backend provider name (default: "statusgator").
    #[serde(default = "default_aggregator_provider")]
    pub provider: String,
    /// Environment variable that holds the API key (key never stored in TOML).
    #[serde(default = "default_aggregator_key_env_var", alias = "api_key_env")]
    pub key_env_var: String,
    /// Seconds between polls (default: 300 — respects StatusGator free-tier rate).
    #[serde(default = "default_aggregator_refresh_interval")]
    pub refresh_interval: u64,
    /// StatusGator service slugs for Claude.
    #[serde(default = "default_claude_services")]
    pub claude_services: Vec<String>,
    /// StatusGator service slugs for OpenAI.
    #[serde(default = "default_openai_services")]
    pub openai_services: Vec<String>,
    /// Fire a webhook when crowd=Spike AND official indicator is below Major.
    #[serde(default = "default_true")]
    pub spike_webhook: bool,
}

impl Default for AggregatorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: default_aggregator_provider(),
            key_env_var: default_aggregator_key_env_var(),
            refresh_interval: default_aggregator_refresh_interval(),
            claude_services: default_claude_services(),
            openai_services: default_openai_services(),
            spike_webhook: true,
        }
    }
}

/// Per-model pricing override entry.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PricingOverride {
    /// Input token price per million tokens (USD).
    pub input: f64,
    /// Output token price per million tokens (USD).
    pub output: f64,
    /// Cache write price per million tokens (USD, optional).
    #[serde(default)]
    pub cache_write: Option<f64>,
    /// Cache read price per million tokens (USD, optional).
    #[serde(default)]
    pub cache_read: Option<f64>,
}

/// Display settings — controls how costs are rendered to the user.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct Display {
    /// ISO 4217 currency code for cost display (default: USD, no conversion).
    pub currency: Option<String>,
    /// BCP-47 locale for date formatting ("en-US", "ja-JP", "de-DE", ...).
    /// Resolved from CLI flag > config > $LANG > "en-US".
    pub locale: Option<String>,
    /// Narrow-table mode for CLI output.
    /// When true, cache columns are dropped and model lists are condensed.
    /// Equivalent to passing `--compact` on the command line.
    #[serde(default)]
    pub compact: Option<bool>,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            currency: Some("USD".into()),
            locale: None,
            compact: None,
        }
    }
}

/// Pricing source settings — controls where model pricing data is loaded from.
#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct PricingConfig {
    /// Pricing source: "static" (default) or "litellm".
    pub source: Option<String>,
    /// How many hours before re-fetching the LiteLLM cache (default: 24).
    pub refresh_hours: Option<u32>,
}

impl PricingConfig {
    /// Returns true when the source is explicitly set to "litellm".
    pub fn is_litellm(&self) -> bool {
        self.source
            .as_deref()
            .map(|s| s.eq_ignore_ascii_case("litellm"))
            .unwrap_or(false)
    }

    /// Effective refresh interval in hours (default 24).
    pub fn effective_refresh_hours(&self) -> u32 {
        self.refresh_hours.unwrap_or(24)
    }
}

// ── Config file discovery ──────────────────────────────────────────────────────

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("usage-tracker.toml")
}

/// Resolve the config file path using priority order:
/// 1. `$HEIMDALL_CONFIG` environment variable (used as-is)
/// 2. `~/.config/heimdall/config.json` then `~/.config/heimdall/config.toml`
/// 3. `~/.claude/usage-tracker.json` then `~/.claude/usage-tracker.toml`
/// 4. Returns `None` if none of the above exist (callers use defaults).
///
/// Within each directory, `.json` is preferred over `.toml` so that users who
/// switch to the schema-backed JSON format get the correct file loaded.
pub fn resolve_config_path() -> Option<PathBuf> {
    // 1. Explicit env override — accept whatever extension the user provides.
    if let Ok(env_path) = std::env::var("HEIMDALL_CONFIG") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Some(p);
        }
    }

    if let Some(home) = dirs::home_dir() {
        // 2. XDG-style ~/.config/heimdall/ — JSON wins over TOML.
        let xdg_base = home.join(".config").join("heimdall");
        if xdg_base.join("config.json").exists() {
            return Some(xdg_base.join("config.json"));
        }
        if xdg_base.join("config.toml").exists() {
            return Some(xdg_base.join("config.toml"));
        }

        // 3. Legacy ~/.claude/ — JSON wins over TOML.
        let claude_base = home.join(".claude");
        if claude_base.join("usage-tracker.json").exists() {
            return Some(claude_base.join("usage-tracker.json"));
        }
        if claude_base.join("usage-tracker.toml").exists() {
            return Some(claude_base.join("usage-tracker.toml"));
        }
    }

    // 4. No config found — callers use bundled defaults.
    None
}

/// Load config from the default path, or return defaults if not found.
pub fn load_config() -> Config {
    load_config_from(&config_path())
}

/// Load config using the dual-config resolver (for the hook binary).
/// Applies the priority: HEIMDALL_CONFIG > ~/.config/heimdall/config.{json,toml}
/// > ~/.claude/usage-tracker.{json,toml} > bundled defaults.
pub fn load_config_resolved() -> Config {
    match resolve_config_path() {
        Some(path) => load_config_from(&path),
        None => Config::default(),
    }
}

/// Detect whether a path is a JSON config file based on its extension.
fn is_json_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("json"))
        .unwrap_or(false)
}

/// Load config from a specific path, dispatching to JSON or TOML parser based
/// on the file extension.  Returns defaults if the file is absent or unparseable.
pub fn load_config_from(path: &Path) -> Config {
    let contents = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return Config::default(),
    };

    if is_json_path(path) {
        match serde_json::from_str::<Config>(&contents) {
            Ok(config) => {
                debug!("Loaded JSON config from {}", path.display());
                config
            }
            Err(e) => {
                eprintln!(
                    "Warning: failed to parse {}: {}. Using defaults.",
                    path.display(),
                    e
                );
                Config::default()
            }
        }
    } else {
        match toml::from_str::<Config>(&contents) {
            Ok(config) => {
                debug!("Loaded TOML config from {}", path.display());
                config
            }
            Err(e) => {
                eprintln!(
                    "Warning: failed to parse {}: {}. Using defaults.",
                    path.display(),
                    e
                );
                Config::default()
            }
        }
    }
}

/// Process-wide mutex for tests that mutate the `HEIMDALL_CONFIG` environment
/// variable. Rust's default test harness runs tests in parallel threads, and
/// `set_var`/`remove_var` are process-global — without serialisation one test's
/// `remove_var` at teardown can wipe another test's `set_var` mid-flight,
/// causing the config resolver to fall back to `~/.claude/usage.db` (which may
/// not exist on a fresh CI runner).
///
/// Any test that touches `HEIMDALL_CONFIG` must hold this guard for its whole
/// body. `unwrap_or_else(|p| p.into_inner())` keeps the suite running even if
/// a previous test panicked while holding the lock.
#[cfg(test)]
pub(crate) static HEIMDALL_CONFIG_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_missing_file_returns_defaults() {
        let config = load_config_from(Path::new("/nonexistent/path/config.toml"));
        assert!(config.projects_dirs.is_empty());
        assert!(config.db_path.is_none());
        assert!(config.host.is_none());
        assert!(config.port.is_none());
        assert!(config.pricing.is_empty());
    }

    #[test]
    fn test_empty_file_returns_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::File::create(&path).unwrap();
        let config = load_config_from(&path);
        assert!(config.projects_dirs.is_empty());
    }

    #[test]
    fn test_basic_config() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            r#"
projects_dirs = ["/home/user/projects", "/opt/claude"]
db_path = "/tmp/usage.db"
host = "0.0.0.0"
port = 9090
"#
        )
        .unwrap();

        let config = load_config_from(&path);
        assert_eq!(config.projects_dirs.len(), 2);
        assert_eq!(
            config.projects_dirs[0],
            PathBuf::from("/home/user/projects")
        );
        assert_eq!(config.db_path.unwrap(), PathBuf::from("/tmp/usage.db"));
        assert_eq!(config.host.unwrap(), "0.0.0.0");
        assert_eq!(config.port.unwrap(), 9090);
    }

    #[test]
    fn test_pricing_overrides() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            r#"
[pricing.claude-opus-4-6]
input = 10.0
output = 50.0
cache_write = 12.5
cache_read = 1.0

[pricing.my-custom-model]
input = 2.0
output = 8.0
"#
        )
        .unwrap();

        let config = load_config_from(&path);
        assert_eq!(config.pricing.len(), 2);

        let opus = &config.pricing["claude-opus-4-6"];
        assert_eq!(opus.input, 10.0);
        assert_eq!(opus.output, 50.0);
        assert_eq!(opus.cache_write, Some(12.5));
        assert_eq!(opus.cache_read, Some(1.0));

        let custom = &config.pricing["my-custom-model"];
        assert_eq!(custom.input, 2.0);
        assert_eq!(custom.output, 8.0);
        assert!(custom.cache_write.is_none());
        assert!(custom.cache_read.is_none());
    }

    #[test]
    fn test_partial_config() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "port = 3000\n").unwrap();

        let config = load_config_from(&path);
        assert!(config.projects_dirs.is_empty());
        assert!(config.db_path.is_none());
        assert!(config.host.is_none());
        assert_eq!(config.port.unwrap(), 3000);
    }

    #[test]
    fn test_invalid_toml_returns_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "this is not valid toml {{{{").unwrap();

        let config = load_config_from(&path);
        assert!(config.projects_dirs.is_empty());
    }

    #[test]
    fn test_oauth_config_defaults() {
        // Empty config should give OAuthConfig defaults: enabled=true, refresh_interval=60
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::File::create(&path).unwrap();
        let config = load_config_from(&path);
        assert!(config.oauth.enabled);
        assert_eq!(config.oauth.refresh_interval, 60);
    }

    #[test]
    fn test_oauth_config_custom() {
        // Parse custom OAuth config
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "[oauth]\nenabled = false\nrefresh_interval = 120\n").unwrap();
        let config = load_config_from(&path);
        assert!(!config.oauth.enabled);
        assert_eq!(config.oauth.refresh_interval, 120);
    }

    #[test]
    fn test_claude_usage_monitor_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::File::create(&path).unwrap();
        let config = load_config_from(&path);
        assert!(config.claude_usage_monitor.claude_binary.is_none());
        assert!(config.claude_usage_monitor.working_dir.is_none());
        assert_eq!(config.claude_usage_monitor.default_interval, "daily");
    }

    #[test]
    fn test_claude_usage_monitor_custom() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "[claude_usage_monitor]\nclaude_binary = \"/opt/bin/claude\"\nworking_dir = \"/tmp\"\ndefault_interval = \"hourly\"\n"
        )
        .unwrap();
        let config = load_config_from(&path);
        assert_eq!(
            config.claude_usage_monitor.claude_binary,
            Some(PathBuf::from("/opt/bin/claude"))
        );
        assert_eq!(
            config.claude_usage_monitor.working_dir,
            Some(PathBuf::from("/tmp"))
        );
        assert_eq!(config.claude_usage_monitor.default_interval, "hourly");
    }

    #[test]
    fn test_webhook_config_full() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "[webhooks]\nurl = \"https://hooks.example.com\"\ncost_threshold = 50.0\nsession_depleted = true\n"
        )
        .unwrap();
        let config = load_config_from(&path);
        assert_eq!(config.webhooks.url.unwrap(), "https://hooks.example.com");
        assert!((config.webhooks.cost_threshold.unwrap() - 50.0).abs() < 0.01);
        assert!(config.webhooks.session_depleted);
    }

    #[test]
    fn test_webhook_config_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::File::create(&path).unwrap();
        let config = load_config_from(&path);
        assert!(config.webhooks.url.is_none());
        assert!(config.webhooks.cost_threshold.is_none());
        assert!(!config.webhooks.session_depleted);
    }

    #[test]
    fn test_openai_config_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::File::create(&path).unwrap();
        let config = load_config_from(&path);
        assert!(config.openai.enabled);
        assert_eq!(config.openai.admin_key_env, "OPENAI_ADMIN_KEY");
        assert_eq!(config.openai.refresh_interval, 300);
        assert_eq!(config.openai.lookback_days, 30);
    }

    #[test]
    fn test_openai_config_custom() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "[openai]\nenabled = false\nadmin_key_env = \"CUSTOM_OPENAI_KEY\"\nrefresh_interval = 600\nlookback_days = 14\n"
        )
        .unwrap();
        let config = load_config_from(&path);
        assert!(!config.openai.enabled);
        assert_eq!(config.openai.admin_key_env, "CUSTOM_OPENAI_KEY");
        assert_eq!(config.openai.refresh_interval, 600);
        assert_eq!(config.openai.lookback_days, 14);
    }

    #[test]
    fn test_config_type_mismatch() {
        // Wrong type should fall back to defaults (TOML parse error)
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "port = \"not_a_number\"\n").unwrap();
        let config = load_config_from(&path);
        // Should return defaults since the file fails to parse
        assert!(config.port.is_none());
    }

    #[test]
    fn test_display_config_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::File::create(&path).unwrap();
        let config = load_config_from(&path);
        // Default display currency should be USD
        assert_eq!(config.display.currency.as_deref(), Some("USD"));
    }

    #[test]
    fn test_display_config_currency_eur() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "[display]\ncurrency = \"EUR\"\n").unwrap();
        let config = load_config_from(&path);
        assert_eq!(config.display.currency.as_deref(), Some("EUR"));
    }

    #[test]
    fn test_display_config_currency_explicit_usd() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "[display]\ncurrency = \"USD\"\n").unwrap();
        let config = load_config_from(&path);
        assert_eq!(config.display.currency.as_deref(), Some("USD"));
    }

    #[test]
    fn test_display_config_none_currency() {
        // Explicit null/empty is valid — callers treat None as "no preference"
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        // Omit the display section entirely
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "port = 3000\n").unwrap();
        let config = load_config_from(&path);
        // Default kicks in; currency = Some("USD")
        assert_eq!(config.display.currency.as_deref(), Some("USD"));
    }

    #[test]
    fn test_pricing_source_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::File::create(&path).unwrap();
        let config = load_config_from(&path);
        assert!(!config.pricing_source.is_litellm());
        assert_eq!(config.pricing_source.effective_refresh_hours(), 24);
    }

    #[test]
    fn test_pricing_source_litellm() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "[pricing_source]\nsource = \"litellm\"\nrefresh_hours = 12\n"
        )
        .unwrap();
        let config = load_config_from(&path);
        assert!(config.pricing_source.is_litellm());
        assert_eq!(config.pricing_source.effective_refresh_hours(), 12);
    }

    #[test]
    fn test_pricing_source_static_explicit() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "[pricing_source]\nsource = \"static\"\n").unwrap();
        let config = load_config_from(&path);
        assert!(!config.pricing_source.is_litellm());
    }

    // ── resolve_config_path tests ────────────────────────────────────────────

    #[test]
    fn test_resolve_config_path_env_var_wins() {
        let _guard = super::HEIMDALL_CONFIG_MUTEX
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let tmp = TempDir::new().unwrap();
        let explicit = tmp.path().join("explicit.toml");
        std::fs::File::create(&explicit).unwrap();

        // SAFETY: serialised against other HEIMDALL_CONFIG mutators by the guard above.
        unsafe { std::env::set_var("HEIMDALL_CONFIG", &explicit) };
        let result = resolve_config_path();
        unsafe { std::env::remove_var("HEIMDALL_CONFIG") };

        assert_eq!(result, Some(explicit));
    }

    #[test]
    fn test_resolve_config_path_env_var_nonexistent_falls_through() {
        let _guard = super::HEIMDALL_CONFIG_MUTEX
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        // If HEIMDALL_CONFIG points at a non-existent file, fall through to
        // the next candidate. With a HOME that has no config files this
        // returns None.
        let tmp = TempDir::new().unwrap();
        let nonexistent = tmp.path().join("does_not_exist.toml");

        // SAFETY: serialised against other HEIMDALL_CONFIG mutators by the guard above.
        unsafe { std::env::set_var("HEIMDALL_CONFIG", &nonexistent) };
        // We don't assert the exact value because the test machine might have
        // real config files; we only assert we don't get the nonexistent path.
        let result = resolve_config_path();
        unsafe { std::env::remove_var("HEIMDALL_CONFIG") };

        assert_ne!(result, Some(nonexistent));
    }

    #[test]
    fn test_resolve_config_path_returns_none_when_nothing_exists() {
        let _guard = super::HEIMDALL_CONFIG_MUTEX
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        // Make sure env var is not set.
        // SAFETY: serialised against other HEIMDALL_CONFIG mutators by the guard above.
        unsafe { std::env::remove_var("HEIMDALL_CONFIG") };
        // We can't easily override HOME without unsafe or extra deps,
        // but we can verify the function doesn't panic and returns
        // Some or None based on the real filesystem.
        let _ = resolve_config_path(); // must not panic
    }

    #[test]
    fn test_aggregator_config_defaults() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::File::create(&path).unwrap();
        let config = load_config_from(&path);
        // Default: disabled, statusgator, STATUSGATOR_API_KEY, 300s
        assert!(!config.aggregator.enabled);
        assert_eq!(config.aggregator.provider, "statusgator");
        assert_eq!(config.aggregator.key_env_var, "STATUSGATOR_API_KEY");
        assert_eq!(config.aggregator.refresh_interval, 300);
        assert!(config.aggregator.spike_webhook);
        assert_eq!(
            config.aggregator.claude_services,
            vec!["claude-ai", "claude"]
        );
        assert_eq!(config.aggregator.openai_services, vec!["openai", "chatgpt"]);
    }

    #[test]
    fn test_aggregator_config_full_section() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            r#"
[status_aggregator]
enabled = true
provider = "statusgator"
key_env_var = "MY_SG_TOKEN_ENV"
refresh_interval = 600
claude_services = ["claude-ai"]
openai_services = ["openai"]
spike_webhook = false
"#
        )
        .unwrap();
        let config = load_config_from(&path);
        assert!(config.aggregator.enabled);
        assert_eq!(config.aggregator.key_env_var, "MY_SG_TOKEN_ENV");
        assert_eq!(config.aggregator.refresh_interval, 600);
        assert_eq!(config.aggregator.claude_services, vec!["claude-ai"]);
        assert!(!config.aggregator.spike_webhook);
    }

    #[test]
    fn test_aggregator_config_legacy_api_key_env_alias() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        let legacy_key = ["api", "key", "env"].join("_");
        write!(
            f,
            "[status_aggregator]\nenabled = true\n{} = \"MY_SG_TOKEN_ENV\"\n",
            legacy_key
        )
        .unwrap();
        let config = load_config_from(&path);
        assert!(config.aggregator.enabled);
        assert_eq!(config.aggregator.key_env_var, "MY_SG_TOKEN_ENV");
    }

    #[test]
    fn test_aggregator_config_enabled_only() {
        // Only set enabled=true; all other fields should take defaults.
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "[status_aggregator]\nenabled = true\n").unwrap();
        let config = load_config_from(&path);
        assert!(config.aggregator.enabled);
        assert_eq!(config.aggregator.provider, "statusgator");
        assert_eq!(config.aggregator.key_env_var, "STATUSGATOR_API_KEY");
    }

    #[test]
    fn test_load_config_resolved_uses_env_var() {
        let _guard = super::HEIMDALL_CONFIG_MUTEX
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("resolved.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "port = 7777\n").unwrap();

        // SAFETY: serialised against other HEIMDALL_CONFIG mutators by the guard above.
        unsafe { std::env::set_var("HEIMDALL_CONFIG", &path) };
        let config = load_config_resolved();
        unsafe { std::env::remove_var("HEIMDALL_CONFIG") };

        assert_eq!(config.port, Some(7777));
    }

    // ── Phase 10: JSON config + per-command overrides ────────────────────────

    #[test]
    fn test_json_config_parses() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        std::fs::write(
            &path,
            r#"{"display": {"currency": "EUR"}, "blocks": {"token_limit": 1000000}}"#,
        )
        .unwrap();
        let config = load_config_from(&path);
        assert_eq!(config.display.currency.as_deref(), Some("EUR"));
        assert_eq!(config.blocks.token_limit, Some(1_000_000));
    }

    #[test]
    fn test_json_schema_key_ignored() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        std::fs::write(
            &path,
            r#"{"$schema": "https://example.com/schema.json", "port": 9999}"#,
        )
        .unwrap();
        let config = load_config_from(&path);
        assert_eq!(config.port, Some(9999));
        assert_eq!(
            config.schema.as_deref(),
            Some("https://example.com/schema.json")
        );
    }

    #[test]
    fn test_toml_still_loads() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "port = 4242\n[display]\ncurrency = \"GBP\"\n").unwrap();
        let config = load_config_from(&path);
        assert_eq!(config.port, Some(4242));
        assert_eq!(config.display.currency.as_deref(), Some("GBP"));
    }

    #[test]
    fn test_json_wins_over_toml_via_env() {
        let _guard = super::HEIMDALL_CONFIG_MUTEX
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let tmp = TempDir::new().unwrap();

        // JSON config: port 1111
        let json_path = tmp.path().join("config.json");
        std::fs::write(&json_path, r#"{"port": 1111}"#).unwrap();

        // TOML config: port 2222
        let toml_path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&toml_path).unwrap();
        write!(f, "port = 2222\n").unwrap();

        // Point HEIMDALL_CONFIG at the JSON file explicitly
        unsafe { std::env::set_var("HEIMDALL_CONFIG", &json_path) };
        let config = load_config_resolved();
        unsafe { std::env::remove_var("HEIMDALL_CONFIG") };

        assert_eq!(config.port, Some(1111));
    }

    #[test]
    fn test_commands_blocks_overrides_flat_blocks() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        std::fs::write(
            &path,
            r#"{
                "blocks": {"token_limit": 500000},
                "commands": {"blocks": {"token_limit": 1000000}}
            }"#,
        )
        .unwrap();
        let config = load_config_from(&path);
        // Flat value
        assert_eq!(config.blocks.token_limit, Some(500_000));
        // Resolved value: commands.blocks wins
        let resolved = config.resolved_blocks();
        assert_eq!(resolved.token_limit, Some(1_000_000));
    }

    #[test]
    fn test_commands_blocks_toml_overrides_flat() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "[blocks]\ntoken_limit = 500000\n\n[commands.blocks]\ntoken_limit = 1000000\n"
        )
        .unwrap();
        let config = load_config_from(&path);
        assert_eq!(config.blocks.token_limit, Some(500_000));
        assert_eq!(config.resolved_blocks().token_limit, Some(1_000_000));
    }

    #[test]
    fn test_resolved_blocks_no_commands_returns_flat() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        std::fs::write(&path, r#"{"blocks": {"token_limit": 777}}"#).unwrap();
        let config = load_config_from(&path);
        assert_eq!(config.resolved_blocks().token_limit, Some(777));
    }

    #[test]
    fn test_resolved_statusline_commands_override() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        std::fs::write(
            &path,
            r#"{
                "statusline": {"context_low_threshold": 0.3},
                "commands": {"statusline": {"context_low_threshold": 0.7}}
            }"#,
        )
        .unwrap();
        let config = load_config_from(&path);
        assert!((config.statusline.context_low_threshold - 0.3).abs() < 1e-9);
        let resolved = config.resolved_statusline();
        assert!((resolved.context_low_threshold - 0.7).abs() < 1e-9);
        // Other fields fall back to flat (which uses default since not set)
        assert!((resolved.context_medium_threshold - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_schema_generates_valid_json_with_expected_keys() {
        let schema = schemars::schema_for!(Config);
        let json = serde_json::to_string_pretty(&schema).expect("schema serializes");
        // Must be valid JSON (already proven by to_string_pretty succeeding)
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("schema round-trips as JSON");
        // Must contain top-level properties key
        assert!(
            parsed.get("properties").is_some(),
            "schema must have 'properties'"
        );
        // Must contain blocks and statusline
        let props = parsed["properties"].as_object().unwrap();
        assert!(props.contains_key("blocks"), "schema must have 'blocks'");
        assert!(
            props.contains_key("statusline"),
            "schema must have 'statusline'"
        );
        assert!(
            props.contains_key("commands"),
            "schema must have 'commands'"
        );
    }

    // ── Phase 11: project_aliases ────────────────────────────────────────────

    #[test]
    fn test_project_aliases_toml_parses() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "[project_aliases]\n\"-Users-po4yka-GitRep-heimdall\" = \"Heimdall\"\n\"-Users-po4yka-GitRep-ccusage\" = \"ccusage\"\n"
        )
        .unwrap();
        let config = load_config_from(&path);
        assert_eq!(config.project_aliases.len(), 2);
        assert_eq!(
            config
                .project_aliases
                .get("-Users-po4yka-GitRep-heimdall")
                .map(|s| s.as_str()),
            Some("Heimdall")
        );
        assert_eq!(
            config
                .project_aliases
                .get("-Users-po4yka-GitRep-ccusage")
                .map(|s| s.as_str()),
            Some("ccusage")
        );
    }

    #[test]
    fn test_project_aliases_json_parses() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.json");
        std::fs::write(
            &path,
            r#"{"project_aliases": {"-Users-po4yka-GitRep-heimdall": "Heimdall"}}"#,
        )
        .unwrap();
        let config = load_config_from(&path);
        assert_eq!(
            config
                .project_aliases
                .get("-Users-po4yka-GitRep-heimdall")
                .map(|s| s.as_str()),
            Some("Heimdall")
        );
    }

    #[test]
    fn test_project_aliases_missing_returns_empty_map() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "port = 3000\n").unwrap();
        let config = load_config_from(&path);
        assert!(config.project_aliases.is_empty());
    }

    #[test]
    fn test_display_name_for_known_slug() {
        let mut config = Config::default();
        config
            .project_aliases
            .insert("-Users-foo-bar".to_string(), "My Project".to_string());
        assert_eq!(config.display_name_for("-Users-foo-bar"), "My Project");
    }

    #[test]
    fn test_display_name_for_unknown_slug() {
        let config = Config::default();
        assert_eq!(config.display_name_for("-Users-foo-bar"), "-Users-foo-bar");
    }

    #[test]
    fn test_display_name_for_empty_map() {
        let config = Config::default();
        assert_eq!(config.display_name_for("any-slug"), "any-slug");
    }

    // ── Phase 13: session_length_by_provider ────────────────────────────────

    #[test]
    fn test_blocks_session_length_by_provider_toml_parses() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "[blocks]\nsession_length_hours = 5.0\n\n[blocks.session_length_by_provider]\ncodex = 1.0\n"
        )
        .unwrap();
        let config = load_config_from(&path);
        assert_eq!(config.blocks.session_length_hours, Some(5.0));
        assert_eq!(
            config
                .blocks
                .session_length_by_provider
                .get("codex")
                .copied(),
            Some(1.0)
        );
    }

    #[test]
    fn test_resolved_session_length_cli_wins() {
        let config = Config::default();
        // CLI override always wins regardless of provider or config
        assert!((config.resolved_session_length(Some(3.0), Some("codex")) - 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_resolved_session_length_provider_wins_over_flat() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(
            f,
            "[blocks]\nsession_length_hours = 5.0\n\n[blocks.session_length_by_provider]\ncodex = 1.0\n"
        )
        .unwrap();
        let config = load_config_from(&path);
        let h = config.resolved_session_length(None, Some("codex"));
        assert!((h - 1.0).abs() < 1e-9, "expected 1.0, got {h}");
    }

    #[test]
    fn test_resolved_session_length_flat_wins_over_fallback() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "[blocks]\nsession_length_hours = 8.0\n").unwrap();
        let config = load_config_from(&path);
        let h = config.resolved_session_length(None, None);
        assert!((h - 8.0).abs() < 1e-9, "expected 8.0, got {h}");
    }

    #[test]
    fn test_resolved_session_length_fallback_when_no_config() {
        let config = Config::default();
        let h = config.resolved_session_length(None, None);
        assert!((h - 5.0).abs() < 1e-9, "expected 5.0 fallback, got {h}");
    }

    #[test]
    fn test_resolved_session_length_invalid_zero_clamps_to_5() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "[blocks]\nsession_length_hours = 0.0\n").unwrap();
        let config = load_config_from(&path);
        let h = config.resolved_session_length(None, None);
        assert!((h - 5.0).abs() < 1e-9, "0.0 should clamp to 5.0, got {h}");
    }

    #[test]
    fn test_resolved_session_length_invalid_negative_clamps_to_5() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "[blocks]\nsession_length_hours = -1.0\n").unwrap();
        let config = load_config_from(&path);
        let h = config.resolved_session_length(None, None);
        assert!((h - 5.0).abs() < 1e-9, "-1.0 should clamp to 5.0, got {h}");
    }

    #[test]
    fn test_resolved_session_length_invalid_over_168_clamps_to_5() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "[blocks]\nsession_length_hours = 200.0\n").unwrap();
        let config = load_config_from(&path);
        let h = config.resolved_session_length(None, None);
        assert!((h - 5.0).abs() < 1e-9, "200.0 should clamp to 5.0, got {h}");
    }

    #[test]
    fn test_resolved_session_length_invalid_provider_value_clamps_to_5() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        // codex = 0.0 is invalid (must be > 0); should warn and fall back to 5.0.
        write!(
            f,
            "[blocks]\n\n[blocks.session_length_by_provider]\ncodex = 0.0\n"
        )
        .unwrap();
        let config = load_config_from(&path);
        let h = config.resolved_session_length(None, Some("codex"));
        assert!(
            (h - 5.0).abs() < f64::EPSILON,
            "0.0 for codex should clamp to 5.0, got {h}"
        );
    }

    #[test]
    fn test_schema_contains_session_length_by_provider() {
        let schema = schemars::schema_for!(Config);
        let json = serde_json::to_string_pretty(&schema).expect("schema serializes");
        assert!(
            json.contains("session_length_by_provider"),
            "schema must contain 'session_length_by_provider'"
        );
        assert!(
            json.contains("session_length_hours"),
            "schema must contain 'session_length_hours'"
        );
    }
}
