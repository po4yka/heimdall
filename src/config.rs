use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::debug;

/// Configuration loaded from ~/.claude/usage-tracker.toml
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub projects_dirs: Vec<PathBuf>,
    pub db_path: Option<PathBuf>,
    pub host: Option<String>,
    pub port: Option<u16>,

    /// Custom pricing overrides. Keys are model names (e.g., "claude-opus-4-6"),
    /// values override the built-in rates.
    #[serde(default)]
    pub pricing: HashMap<String, PricingOverride>,

    /// OAuth settings for real-time rate window tracking.
    #[serde(default)]
    pub oauth: OAuthConfig,

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
    /// TOML section: [pricing_source]
    #[serde(default)]
    pub pricing_source: PricingConfig,

    /// Upstream coding-agent status monitoring.
    /// TOML section: [agent_status]
    #[serde(default)]
    pub agent_status: AgentStatusConfig,

    /// Optional community-signal aggregator (opt-in, off by default).
    /// TOML section: [status_aggregator]
    #[serde(default, rename = "status_aggregator")]
    pub aggregator: AggregatorConfig,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Minor,
    #[default]
    Major,
    Critical,
}

/// Configuration for upstream coding-agent status monitoring.
#[derive(Debug, Clone, Deserialize)]
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

fn default_aggregator_api_key_env() -> String {
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
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct AggregatorConfig {
    /// Enable community signal polling (default: false — explicit opt-in required).
    pub enabled: bool,
    /// Backend provider name (default: "statusgator").
    #[serde(default = "default_aggregator_provider")]
    pub provider: String,
    /// Environment variable that holds the API key (key never stored in TOML).
    #[serde(default = "default_aggregator_api_key_env")]
    pub api_key_env: String,
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
            api_key_env: default_aggregator_api_key_env(),
            refresh_interval: default_aggregator_refresh_interval(),
            claude_services: default_claude_services(),
            openai_services: default_openai_services(),
            spike_webhook: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PricingOverride {
    pub input: f64,
    pub output: f64,
    #[serde(default)]
    pub cache_write: Option<f64>,
    #[serde(default)]
    pub cache_read: Option<f64>,
}

/// Display settings — controls how costs are rendered to the user.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Display {
    /// ISO 4217 currency code for cost display (default: USD, no conversion).
    pub currency: Option<String>,
}

impl Default for Display {
    fn default() -> Self {
        Self {
            currency: Some("USD".into()),
        }
    }
}

/// Pricing source settings — controls where model pricing data is loaded from.
#[derive(Debug, Clone, Default, Deserialize)]
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

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("usage-tracker.toml")
}

/// Resolve the config file path using priority order:
/// 1. `$HEIMDALL_CONFIG` environment variable
/// 2. `~/.config/heimdall/config.toml`
/// 3. `~/.claude/usage-tracker.toml`
/// 4. Returns `None` if none of the above exist (callers use defaults).
///
/// Ported from Claude-Guardian's `_find_config_path()` pattern.
pub fn resolve_config_path() -> Option<PathBuf> {
    // 1. Explicit env override
    if let Ok(env_path) = std::env::var("HEIMDALL_CONFIG") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Some(p);
        }
    }

    // 2. XDG-style ~/.config/heimdall/config.toml
    if let Some(home) = dirs::home_dir() {
        let xdg = home.join(".config").join("heimdall").join("config.toml");
        if xdg.exists() {
            return Some(xdg);
        }

        // 3. Legacy ~/.claude/usage-tracker.toml
        let legacy = home.join(".claude").join("usage-tracker.toml");
        if legacy.exists() {
            return Some(legacy);
        }
    }

    // 4. No config found — callers use bundled defaults
    None
}

/// Load config from the default path, or return defaults if not found.
pub fn load_config() -> Config {
    load_config_from(&config_path())
}

/// Load config using the dual-config resolver (for the hook binary).
/// Applies the priority: HEIMDALL_CONFIG > ~/.config/heimdall/config.toml
/// > ~/.claude/usage-tracker.toml > bundled defaults.
pub fn load_config_resolved() -> Config {
    match resolve_config_path() {
        Some(path) => load_config_from(&path),
        None => Config::default(),
    }
}

/// Load config from a specific path, or return defaults if not found.
pub fn load_config_from(path: &Path) -> Config {
    match std::fs::read_to_string(path) {
        Ok(contents) => match toml::from_str::<Config>(&contents) {
            Ok(config) => {
                debug!("Loaded config from {}", path.display());
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
        },
        Err(_) => Config::default(),
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
        assert_eq!(config.aggregator.api_key_env, "STATUSGATOR_API_KEY");
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
api_key_env = "MY_SG_KEY"
refresh_interval = 600
claude_services = ["claude-ai"]
openai_services = ["openai"]
spike_webhook = false
"#
        )
        .unwrap();
        let config = load_config_from(&path);
        assert!(config.aggregator.enabled);
        assert_eq!(config.aggregator.api_key_env, "MY_SG_KEY");
        assert_eq!(config.aggregator.refresh_interval, 600);
        assert_eq!(config.aggregator.claude_services, vec!["claude-ai"]);
        assert!(!config.aggregator.spike_webhook);
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
        assert_eq!(config.aggregator.api_key_env, "STATUSGATOR_API_KEY");
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
}
