use claude_usage_tracker::analytics;
use claude_usage_tracker::archive;
use claude_usage_tracker::scrape;
use claude_usage_tracker::config;
use claude_usage_tracker::currency;
use claude_usage_tracker::db as db_mod;
use claude_usage_tracker::export;
use claude_usage_tracker::hook;
use claude_usage_tracker::jq as jq_mod;
use claude_usage_tracker::litellm;
use claude_usage_tracker::locale;
#[cfg(feature = "mcp")]
use claude_usage_tracker::mcp;
use claude_usage_tracker::menubar;
use claude_usage_tracker::optimizer;
use claude_usage_tracker::pricing;
use claude_usage_tracker::pricing_defs;
use claude_usage_tracker::pricing_sync;
use claude_usage_tracker::scanner;
use claude_usage_tracker::scheduler;
use claude_usage_tracker::server;
use claude_usage_tracker::statusline;
use claude_usage_tracker::usage_monitor;

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "claude-usage-tracker",
    version,
    about = "Local analytics dashboard for Claude Code and Codex usage"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan JSONL files and update the database
    Scan {
        #[arg(long)]
        projects_dir: Option<PathBuf>,
        #[arg(long)]
        db_path: Option<PathBuf>,
    },
    /// Show today's usage summary
    Today {
        #[arg(long)]
        db_path: Option<PathBuf>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// jq-style filter applied to the JSON output (implies --json)
        #[arg(long, value_name = "FILTER")]
        jq: Option<String>,
        /// Map a project slug to a human-readable name (repeatable).
        /// Format: "slug=Display Name". Merges into / overrides config's [project_aliases].
        #[arg(long = "project-alias", value_parser = parse_project_alias)]
        project_aliases: Vec<(String, String)>,
        /// Show per-model breakdown sub-rows under each provider total
        #[arg(long)]
        breakdown: bool,
        /// Locale for date formatting (BCP-47, e.g., "ja-JP", "de-DE").
        /// Defaults to $LANG or "en-US".
        #[arg(long)]
        locale: Option<String>,
        /// Narrow layout: drop cache columns and condense model lists.
        /// Useful for screenshots and <100 column terminals.
        #[arg(long)]
        compact: bool,
    },
    /// Show all-time statistics
    Stats {
        #[arg(long)]
        db_path: Option<PathBuf>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// jq-style filter applied to the JSON output (implies --json)
        #[arg(long, value_name = "FILTER")]
        jq: Option<String>,
        /// Map a project slug to a human-readable name (repeatable).
        /// Format: "slug=Display Name". Merges into / overrides config's [project_aliases].
        #[arg(long = "project-alias", value_parser = parse_project_alias)]
        project_aliases: Vec<(String, String)>,
        /// Show per-model breakdown sub-rows under each provider total
        #[arg(long)]
        breakdown: bool,
        /// Locale for date formatting (BCP-47, e.g., "ja-JP", "de-DE").
        /// Defaults to $LANG or "en-US".
        #[arg(long)]
        locale: Option<String>,
        /// Narrow layout: drop cache columns and condense model lists.
        /// Useful for screenshots and <100 column terminals.
        #[arg(long)]
        compact: bool,
    },
    /// Scan + start web dashboard
    Dashboard {
        #[arg(long)]
        projects_dir: Option<PathBuf>,
        #[arg(long)]
        db_path: Option<PathBuf>,
        #[arg(long, default_value = "localhost")]
        host: String,
        #[arg(long, default_value = "8080")]
        port: u16,
        /// Enable file-watcher auto-refresh: re-scan whenever .jsonl files change
        #[arg(long, default_value_t = false)]
        watch: bool,
        /// Do not automatically open the dashboard in a browser
        #[arg(long, default_value_t = false)]
        no_open: bool,
        /// Start background polling so remote monitoring data warms without browser traffic
        #[arg(long, default_value_t = false)]
        background_poll: bool,
    },
    /// Export aggregated usage to CSV / JSON / JSONL
    Export {
        #[arg(long)]
        db_path: Option<PathBuf>,
        /// Output format: csv | json | jsonl
        #[arg(long, default_value = "csv")]
        format: String,
        /// Time window: today | week | month | year | all
        #[arg(long, default_value = "all")]
        period: String,
        /// Output file path ("-" for stdout)
        #[arg(long)]
        output: PathBuf,
        /// Restrict to a single provider (claude | codex | xcode | ...)
        #[arg(long)]
        provider: Option<String>,
        /// Restrict to a single project_name
        #[arg(long)]
        project: Option<String>,
        /// jq-style filter applied to each JSON/JSONL record (implies --format=json/jsonl)
        #[arg(long, value_name = "FILTER")]
        jq: Option<String>,
        /// Map a project slug to a human-readable name (repeatable).
        /// Format: "slug=Display Name". Merges into / overrides config's [project_aliases].
        #[arg(long = "project-alias", value_parser = parse_project_alias)]
        project_aliases: Vec<(String, String)>,
        /// Locale for date formatting (BCP-47, e.g., "ja-JP", "de-DE").
        /// Defaults to $LANG or "en-US".
        #[arg(long)]
        locale: Option<String>,
    },
    /// Print SwiftBar-formatted menubar widget showing today's cost
    Menubar {
        #[arg(long)]
        db_path: Option<PathBuf>,
    },
    /// Pricing data management
    Pricing {
        #[command(subcommand)]
        action: PricingAction,
    },
    /// Manage the platform-native scheduled scan job
    Scheduler {
        #[command(subcommand)]
        action: SchedulerAction,
    },
    /// Capture Claude `/usage` snapshots and manage their scheduled collection
    UsageMonitor {
        #[command(subcommand)]
        action: UsageMonitorAction,
    },
    /// Analyse usage data and report waste findings (Phase 6)
    Optimize {
        #[arg(long)]
        db_path: Option<PathBuf>,
        /// Output format: text | json
        #[arg(long, default_value = "text")]
        format: String,
        /// jq-style filter applied to the JSON output (implies --format=json)
        #[arg(long, value_name = "FILTER")]
        jq: Option<String>,
    },
    /// Manage the local chat-backup archive (Phase 1: CLI snapshots only)
    Archive {
        #[command(subcommand)]
        action: ArchiveAction,
    },
    /// Import an Anthropic or OpenAI account-export ZIP into the archive
    ImportExport {
        /// Path to the ZIP. Required unless --watch is set.
        zip: Option<PathBuf>,
        /// Watch this directory and import any new ZIPs as they land
        #[arg(long)]
        watch: Option<PathBuf>,
        /// Override the archive root
        #[arg(long)]
        archive_root: Option<PathBuf>,
        /// JSON output
        #[arg(long)]
        json: bool,
    },
    /// Manage the heimdall-hook real-time PreToolUse ingest hook
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },
    /// Manage the always-on dashboard daemon (macOS only)
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Database management utilities
    Db {
        #[command(subcommand)]
        action: DbAction,
    },
    /// Show Claude 5-hour billing blocks with burn rate and end-of-block projection
    Blocks {
        /// Narrow layout: drop cache columns and condense model lists.
        /// Useful for screenshots and <100 column terminals.
        #[arg(long)]
        compact: bool,
        /// Path to SQLite DB file
        #[arg(long)]
        db_path: Option<PathBuf>,
        /// Session block duration in hours (0 < h <= 168); overrides config default.
        /// When absent, uses --provider lookup or [blocks.session_length_hours] from config.
        #[arg(long, value_parser = parse_session_length)]
        session_length: Option<f64>,
        /// Provider hint for resolving the default session length when
        /// --session-length is absent (looks up [blocks.session_length_by_provider]).
        #[arg(long)]
        provider: Option<String>,
        /// Only show the currently active block
        #[arg(long)]
        active: bool,
        /// Emit JSON instead of a human-readable table
        #[arg(long)]
        json: bool,
        /// Token quota for the active billing block: a number, or "max" to use the historical peak.
        #[arg(long, value_parser = parse_token_limit)]
        token_limit: Option<TokenLimit>,
        /// jq-style filter applied to the JSON output (implies --json)
        #[arg(long, value_name = "FILTER")]
        jq: Option<String>,
        /// Suppress gap rows between activity blocks (shown by default)
        #[arg(long)]
        no_gaps: bool,
        /// Locale for date formatting (BCP-47, e.g., "ja-JP", "de-DE").
        /// Defaults to $LANG or "en-US".
        #[arg(long)]
        locale: Option<String>,
    },
    /// Aggregated usage by ISO calendar week
    Weekly {
        /// Narrow layout: drop cache columns and condense model lists.
        /// Useful for screenshots and <100 column terminals.
        #[arg(long)]
        compact: bool,
        /// Path to SQLite DB file
        #[arg(long)]
        db_path: Option<PathBuf>,
        /// Week start day (sunday|monday|tuesday|wednesday|thursday|friday|saturday)
        #[arg(long, default_value = "monday", value_parser = parse_weekday)]
        start_of_week: chrono::Weekday,
        /// Emit JSON instead of a human-readable table
        #[arg(long)]
        json: bool,
        /// Include per-model breakdown sub-rows under each week
        #[arg(long)]
        breakdown: bool,
        /// jq-style filter applied to the JSON output (implies --json)
        #[arg(long, value_name = "FILTER")]
        jq: Option<String>,
        /// Map a project slug to a human-readable name (repeatable).
        /// Format: "slug=Display Name". Merges into / overrides config's [project_aliases].
        #[arg(long = "project-alias", value_parser = parse_project_alias)]
        project_aliases: Vec<(String, String)>,
        /// Locale for date formatting (BCP-47, e.g., "ja-JP", "de-DE").
        /// Defaults to $LANG or "en-US".
        #[arg(long)]
        locale: Option<String>,
    },
    /// Emit a Claude Code status line from the PostToolUse hook JSON on stdin
    Statusline {
        /// Max seconds since last render before recomputing
        #[arg(long, default_value_t = 30)]
        refresh_interval: u64,
        /// Which cost to display: auto (prefer hook), local, hook, or both
        #[arg(long, default_value = "auto", value_parser = parse_cost_source)]
        cost_source: statusline::StatuslineCostSource,
        /// Skip any potential network calls (currency, LiteLLM); purely local
        #[arg(long)]
        offline: bool,
        /// Path to SQLite DB file
        #[arg(long)]
        db_path: Option<PathBuf>,
        /// Render the burn-rate tier indicator: off | bracket (default) | emoji | both
        #[arg(long, default_value = "bracket", value_parser = parse_visual_burn_rate)]
        visual_burn_rate: statusline::VisualBurnRate,
    },
    /// Manage the statusline PostToolUse hook entry in ~/.claude/settings.json
    StatuslineHook {
        #[command(subcommand)]
        action: StatuslineHookAction,
    },
    /// Run the MCP server or manage its installation
    #[cfg(feature = "mcp")]
    Mcp {
        #[command(subcommand)]
        action: McpAction,
    },
    /// Utilities for Heimdall's config file
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Scrape claude.ai or chatgpt.com private APIs using copy-pasted cookies
    Scrape {
        #[command(subcommand)]
        action: ScrapeAction,
    },
    /// Manage the companion bearer token used by the browser extension and the scrape CLI
    CompanionToken {
        #[command(subcommand)]
        action: CompanionTokenAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Emit the JSON schema for Heimdall's config to stdout
    Schema,
    /// Print the resolved effective config (for debugging)
    Show {
        /// Output format: toml | json
        #[arg(long, default_value = "toml", value_parser = ["toml", "json"])]
        format: String,
    },
}

#[derive(Subcommand)]
enum ArchiveAction {
    /// Take a content-addressed snapshot of every provider's source files
    Snapshot {
        /// Override the archive root (default: ~/.heimdall/archive)
        #[arg(long)]
        archive_root: Option<PathBuf>,
        /// Emit JSON instead of a human-readable report
        #[arg(long)]
        json: bool,
    },
    /// List existing snapshots
    List {
        #[arg(long)]
        archive_root: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Show the manifest of a snapshot
    Show {
        /// Snapshot ID (format: 2026-04-28T080000.000000Z)
        snapshot_id: String,
        #[arg(long)]
        archive_root: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Restore a snapshot's contents into a fresh directory
    Restore {
        snapshot_id: String,
        /// Destination directory (default: ./heimdall-restore-<id>)
        #[arg(long)]
        to: Option<PathBuf>,
        #[arg(long)]
        archive_root: Option<PathBuf>,
    },
    /// Keep only the most recent N snapshots, GC unreferenced objects
    Prune {
        #[arg(long, default_value_t = 30)]
        keep_last: usize,
        #[arg(long)]
        archive_root: Option<PathBuf>,
    },
    /// Verify object integrity and rebuild the index
    Verify {
        #[arg(long)]
        archive_root: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum ScrapeAction {
    /// Scrape claude.ai
    Claude {
        /// sessionKey cookie value (`sk-ant-sid01-...`).
        /// Falls back to env `HEIMDALL_CLAUDE_SESSION_KEY`.
        #[arg(long)]
        session_key: Option<String>,
        /// cf_clearance cookie value (optional but usually required).
        /// Falls back to env `HEIMDALL_CLAUDE_CF_CLEARANCE`.
        #[arg(long)]
        cf_clearance: Option<String>,
        /// User-Agent matching the browser that issued the cookies.
        /// Falls back to env `HEIMDALL_USER_AGENT`.
        #[arg(long)]
        user_agent: Option<String>,
        /// Override archive root
        #[arg(long)]
        archive_root: Option<PathBuf>,
        /// JSON output
        #[arg(long)]
        json: bool,
    },
    /// Scrape chatgpt.com
    Chatgpt {
        /// Falls back to env `HEIMDALL_CHATGPT_SESSION_TOKEN`.
        #[arg(long)]
        session_token: Option<String>,
        /// Falls back to env `HEIMDALL_CHATGPT_ACCESS_TOKEN`.
        #[arg(long)]
        access_token: Option<String>,
        /// Falls back to env `HEIMDALL_CHATGPT_CF_CLEARANCE`.
        #[arg(long)]
        cf_clearance: Option<String>,
        /// User-Agent matching the browser that issued the cookies.
        /// Falls back to env `HEIMDALL_USER_AGENT`.
        #[arg(long)]
        user_agent: Option<String>,
        #[arg(long)]
        archive_root: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum CompanionTokenAction {
    /// Print the current bearer token (creates one if missing)
    Show,
    /// Generate a fresh bearer token (any prior pair-up must be repeated)
    Rotate,
}

/// Parse the MCP transport string.
#[cfg(feature = "mcp")]
fn parse_mcp_transport(s: &str) -> std::result::Result<mcp::McpTransport, String> {
    s.parse()
}

#[cfg(feature = "mcp")]
#[derive(Subcommand)]
enum McpAction {
    /// Start the MCP server (default: stdio transport)
    Serve {
        /// Transport: stdio | http
        #[arg(long, default_value = "stdio", value_parser = parse_mcp_transport)]
        transport: mcp::McpTransport,
        /// Bind host for HTTP transport
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Bind port for HTTP transport
        #[arg(long, default_value_t = 8081)]
        port: u16,
        /// Path to SQLite DB file
        #[arg(long)]
        db_path: Option<PathBuf>,
    },
    /// Install the Heimdall MCP server into a client's mcp.json
    Install {
        /// Target client: claude-code | claude-desktop | cursor
        #[arg(long, default_value = "claude-code")]
        client: String,
    },
    /// Remove the Heimdall MCP server entry from a client's mcp.json
    Uninstall {
        /// Target client: claude-code | claude-desktop | cursor
        #[arg(long, default_value = "claude-code")]
        client: String,
    },
    /// Show install status for a client
    Status {
        /// Target client: claude-code | claude-desktop | cursor
        #[arg(long, default_value = "claude-code")]
        client: String,
    },
}

/// Token quota specification for `--token-limit`.
#[derive(Debug, Clone)]
enum TokenLimit {
    /// A fixed absolute token count.
    Absolute(i64),
    /// Resolve to the highest token count seen across all historical blocks.
    HistoricalMax,
}

#[derive(Subcommand)]
enum StatuslineHookAction {
    /// Install the statusLine entry into ~/.claude/settings.json
    Install,
    /// Remove the statusLine entry from ~/.claude/settings.json
    Uninstall,
    /// Show whether the statusLine entry is present
    Status,
}

/// Parse a `slug=Display Name` pair for `--project-alias`.
pub(crate) fn parse_project_alias(s: &str) -> Result<(String, String), String> {
    let (k, v) = s
        .split_once('=')
        .ok_or_else(|| format!("expected 'slug=Display Name', got: {s}"))?;
    let k = k.trim().to_string();
    let v = v.trim().to_string();
    if k.is_empty() {
        return Err(format!("project alias key is empty in: {s}"));
    }
    if v.is_empty() {
        return Err(format!("project alias value is empty in: {s}"));
    }
    Ok((k, v))
}

fn parse_cost_source(s: &str) -> Result<statusline::StatuslineCostSource, String> {
    statusline::StatuslineCostSource::parse(s)
}

fn parse_visual_burn_rate(s: &str) -> Result<statusline::VisualBurnRate, String> {
    statusline::VisualBurnRate::parse(s)
}

fn parse_token_limit(s: &str) -> Result<TokenLimit, String> {
    if s.eq_ignore_ascii_case("max") {
        return Ok(TokenLimit::HistoricalMax);
    }
    match s.parse::<i64>() {
        Ok(n) if n > 0 => Ok(TokenLimit::Absolute(n)),
        Ok(_) => Err(format!("token-limit must be a positive integer, got: {s}")),
        Err(_) => Err(format!(
            "invalid token-limit value '{s}': expected a positive integer or \"max\""
        )),
    }
}

/// Parse a session-length value: must be a float in (0, 168].
fn parse_session_length(s: &str) -> Result<f64, String> {
    match s.parse::<f64>() {
        Ok(h) if h > 0.0 && h <= 168.0 => Ok(h),
        Ok(h) => Err(format!(
            "session-length must be > 0 and <= 168 hours, got: {h}"
        )),
        Err(_) => Err(format!(
            "invalid session-length '{s}': expected a number of hours"
        )),
    }
}

/// Parse a weekday name (case-insensitive) into a `chrono::Weekday`.
fn parse_weekday(s: &str) -> Result<chrono::Weekday, String> {
    match s.to_ascii_lowercase().as_str() {
        "monday" | "mon" => Ok(chrono::Weekday::Mon),
        "tuesday" | "tue" => Ok(chrono::Weekday::Tue),
        "wednesday" | "wed" => Ok(chrono::Weekday::Wed),
        "thursday" | "thu" => Ok(chrono::Weekday::Thu),
        "friday" | "fri" => Ok(chrono::Weekday::Fri),
        "saturday" | "sat" => Ok(chrono::Weekday::Sat),
        "sunday" | "sun" => Ok(chrono::Weekday::Sun),
        _ => Err(format!(
            "unknown weekday '{s}': expected sunday|monday|tuesday|wednesday|thursday|friday|saturday"
        )),
    }
}

/// Map a `chrono::Weekday` to the 0=Sunday … 6=Saturday encoding used by `TzParams`.
fn weekday_to_u8(day: chrono::Weekday) -> u8 {
    match day {
        chrono::Weekday::Sun => 0,
        chrono::Weekday::Mon => 1,
        chrono::Weekday::Tue => 2,
        chrono::Weekday::Wed => 3,
        chrono::Weekday::Thu => 4,
        chrono::Weekday::Fri => 5,
        chrono::Weekday::Sat => 6,
    }
}

#[derive(Subcommand)]
enum DbAction {
    /// Destructively reset the database (deletes the SQLite file).
    ///
    /// Interactive: prompts for confirmation by typing "rebuild".
    /// Non-interactive (CI, pipes): requires --yes flag.
    Reset {
        /// Override the database path
        #[arg(long)]
        db_path: Option<PathBuf>,
        /// Skip the interactive prompt and proceed non-interactively
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum DaemonAction {
    /// Install the always-on dashboard daemon into ~/Library/LaunchAgents (macOS only)
    Install,
    /// Remove the dashboard daemon plist and unregister it from launchd
    Uninstall,
    /// Show whether the daemon is installed and running
    Status,
}

#[derive(Subcommand)]
enum HookAction {
    /// Install the heimdall-hook entry into ~/.claude/settings.json
    Install,
    /// Remove the heimdall-hook entry from ~/.claude/settings.json
    Uninstall,
    /// Show whether the hook entry is present
    Status,
}

#[derive(Subcommand)]
enum PricingAction {
    /// Fetch the LiteLLM model catalogue and cache it locally
    Refresh {
        /// Override the default cache path (~/.cache/heimdall/litellm_pricing.json)
        #[arg(long)]
        cache_path: Option<PathBuf>,
    },
    /// Fetch official provider pricing sources, keep raw snapshots, and reprice turns on change
    Sync {
        /// Override the database path used to persist pricing history and repriced turns
        #[arg(long)]
        db_path: Option<PathBuf>,
    },
    /// Install a platform-native scheduled official pricing sync job
    Install {
        /// How often to run: hourly or daily (default: daily)
        #[arg(long, default_value = "daily")]
        interval: String,
        /// Override the database path used by the scheduled job
        #[arg(long)]
        db_path: Option<PathBuf>,
    },
    /// Remove the scheduled pricing sync job
    Uninstall,
    /// Show the current pricing sync scheduler status
    Status,
}

#[derive(Subcommand)]
enum SchedulerAction {
    /// Install a platform-native scheduled scan job
    Install {
        /// How often to run: hourly or daily (default: hourly)
        #[arg(long, default_value = "hourly")]
        interval: String,
        /// Override the database path used by the scheduled job
        #[arg(long)]
        db_path: Option<PathBuf>,
        /// Also install the daily archive snapshot job
        #[arg(long)]
        include_archive: bool,
    },
    /// Remove the scheduled scan job
    Uninstall,
    /// Show the current scheduler status
    Status,
}

#[derive(Subcommand)]
enum UsageMonitorAction {
    /// Capture one Claude `/usage` snapshot immediately
    Capture {
        /// Override the database path used by the capture
        #[arg(long)]
        db_path: Option<PathBuf>,
    },
    /// Install a platform-native scheduled Claude `/usage` capture job
    Install {
        /// How often to run: hourly or daily (defaults to config or daily)
        #[arg(long)]
        interval: Option<String>,
        /// Override the database path used by the scheduled job
        #[arg(long)]
        db_path: Option<PathBuf>,
    },
    /// Remove the scheduled Claude `/usage` capture job
    Uninstall,
    /// Show the current Claude `/usage` scheduler status
    Status,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cfg = config::load_config_resolved();
    apply_pricing_overrides(&cfg);
    maybe_load_litellm(&cfg);

    // Resolve merged configs (commands.* overrides flat defaults).
    let resolved_blocks = cfg.resolved_blocks();
    let resolved_statusline = cfg.resolved_statusline();
    // Resolve session length before any partial moves of `cfg` fields below.
    let cfg_blocks_session_length = cfg.resolved_session_length(None, None);

    // Extract config values before match (avoids partial move issues)
    let cfg_db = cfg.db_path;
    let cfg_dirs = cfg.projects_dirs;
    let cfg_host = cfg.host;
    let cfg_port = cfg.port;
    let cfg_oauth_enabled = cfg.oauth.enabled;
    let cfg_oauth_refresh = cfg.oauth.refresh_interval;
    let cfg_claude_usage_monitor = cfg.claude_usage_monitor.clone();
    let cfg_claude_admin_enabled = cfg.claude_admin.enabled;
    let cfg_claude_admin_key_env = cfg.claude_admin.admin_key_env;
    let cfg_claude_admin_refresh_interval = cfg.claude_admin.refresh_interval;
    let cfg_claude_admin_lookback_days = cfg.claude_admin.lookback_days;
    let cfg_webhooks = cfg.webhooks;
    let cfg_openai_enabled = cfg.openai.enabled;
    let cfg_openai_admin_key_env = cfg.openai.admin_key_env;
    let cfg_openai_refresh_interval = cfg.openai.refresh_interval;
    let cfg_openai_lookback_days = cfg.openai.lookback_days;
    let cfg_display_currency = cfg.display.currency.unwrap_or_else(|| "USD".into());
    let cfg_display_locale = cfg.display.locale;
    let cfg_display_compact = cfg.display.compact.unwrap_or(false);
    let cfg_agent_status = cfg.agent_status;
    let cfg_aggregator = cfg.aggregator;
    let cfg_blocks_token_limit = resolved_blocks.token_limit;
    let cfg_blocks_session_length_by_provider = resolved_blocks.session_length_by_provider.clone();
    let cfg_statusline_low = resolved_statusline.context_low_threshold;
    let cfg_statusline_medium = resolved_statusline.context_medium_threshold;
    let cfg_burn_rate_normal_max = resolved_statusline.burn_rate_normal_max;
    let cfg_burn_rate_moderate_max = resolved_statusline.burn_rate_moderate_max;
    let cfg_project_aliases = cfg.project_aliases.clone();

    let default_db = |cli_db: Option<PathBuf>| -> PathBuf {
        cli_db
            .or_else(|| cfg_db.clone())
            .unwrap_or_else(scanner::default_db_path)
    };
    let default_dirs = |cli_dir: Option<PathBuf>| -> Option<Vec<PathBuf>> {
        if let Some(d) = cli_dir {
            return Some(vec![d]);
        }
        if !cfg_dirs.is_empty() {
            return Some(cfg_dirs.clone());
        }
        None
    };

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            projects_dir,
            db_path,
        } => {
            let db = default_db(db_path);
            let dirs = default_dirs(projects_dir);
            scanner::scan(dirs, &db, true)?;
        }
        Commands::Today {
            db_path,
            json,
            jq,
            project_aliases,
            breakdown,
            locale,
            compact,
        } => {
            let db = default_db(db_path);
            let mut aliases = cfg_project_aliases.clone();
            for (k, v) in project_aliases {
                aliases.insert(k, v);
            }
            let loc = locale::resolve_locale(locale.as_deref(), cfg_display_locale.as_deref());
            let effective_compact = compact || cfg_display_compact;
            cmd_today(
                &db,
                json,
                breakdown,
                jq.as_deref(),
                &aliases,
                loc,
                effective_compact,
            )?;
        }
        Commands::Stats {
            db_path,
            json,
            jq,
            project_aliases,
            breakdown,
            locale,
            compact,
        } => {
            let db = default_db(db_path);
            let mut aliases = cfg_project_aliases.clone();
            for (k, v) in project_aliases {
                aliases.insert(k, v);
            }
            let loc = locale::resolve_locale(locale.as_deref(), cfg_display_locale.as_deref());
            let effective_compact = compact || cfg_display_compact;
            cmd_stats(
                &db,
                json,
                breakdown,
                &cfg_display_currency,
                jq.as_deref(),
                &aliases,
                loc,
                effective_compact,
            )?;
        }
        Commands::Dashboard {
            projects_dir,
            db_path,
            host,
            port,
            watch,
            no_open,
            background_poll,
        } => {
            cmd_dashboard(
                default_db(db_path),
                default_dirs(projects_dir),
                host,
                port,
                watch,
                no_open,
                background_poll,
                cfg_host,
                cfg_port,
                cfg_oauth_enabled,
                cfg_oauth_refresh,
                cfg_claude_admin_enabled,
                cfg_claude_admin_key_env,
                cfg_claude_admin_refresh_interval,
                cfg_claude_admin_lookback_days,
                cfg_openai_enabled,
                cfg_openai_admin_key_env,
                cfg_openai_refresh_interval,
                cfg_openai_lookback_days,
                cfg_webhooks,
                cfg_agent_status,
                cfg_aggregator,
                cfg_blocks_token_limit,
                cfg_blocks_session_length,
                cfg_project_aliases.clone(),
            )?;
        }
        Commands::Export {
            db_path,
            format,
            period,
            output,
            provider,
            project,
            jq,
            project_aliases,
            locale: _,
        } => {
            let db = default_db(db_path);
            let mut aliases = cfg_project_aliases.clone();
            for (k, v) in project_aliases {
                aliases.insert(k, v);
            }
            let opts = export::ExportOptions {
                format: format.parse()?,
                period: period.parse()?,
                output,
                provider,
                project,
                jq,
                project_aliases: aliases,
            };
            let n = export::run_export(&db, &opts)?;
            // When writing to stdout (`-`), suppress the status message so it
            // doesn't pollute the data stream.  Otherwise emit to stderr.
            if opts.output != std::path::Path::new("-") {
                eprintln!("Exported {} rows to {}", n, opts.output.display());
            }
        }
        Commands::Menubar { db_path } => {
            let db = default_db(db_path);
            let output = menubar::run_menubar(&db)?;
            print!("{}", output);
        }
        Commands::Pricing { action } => {
            cmd_pricing(
                action,
                &default_db(None),
                &pricing_defs::OfficialSyncOptions {
                    openai_admin_key: if cfg_openai_enabled {
                        std::env::var(&cfg_openai_admin_key_env).ok()
                    } else {
                        None
                    },
                    openai_lookback_days: cfg_openai_lookback_days,
                    agent_status_config: cfg_agent_status.clone(),
                },
            )?;
        }
        Commands::Scheduler { action } => {
            cmd_scheduler(action, &default_db(None))?;
        }
        Commands::UsageMonitor { action } => {
            cmd_usage_monitor(action, &default_db(None), &cfg_claude_usage_monitor)?;
        }
        Commands::Optimize {
            db_path,
            format,
            jq,
        } => {
            let db = default_db(db_path);
            cmd_optimize(&db, &format, jq.as_deref())?;
        }
        Commands::Archive { action } => {
            use archive::{Archive, ArchiveLock};
            match action {
                ArchiveAction::Snapshot { archive_root, json } => {
                    let root = archive_root.unwrap_or_else(archive::default_root);
                    let _lock = ArchiveLock::acquire(&root)?;
                    let archive_handle = Archive::at(root)?;
                    let providers = scanner::providers::all();
                    let id = archive_handle.snapshot(&providers)?;
                    let metas = archive_handle.list()?;
                    let latest = metas.into_iter().find(|m| m.snapshot_id == id);
                    if json {
                        println!("{}", serde_json::to_string_pretty(&latest)?);
                    } else if let Some(m) = latest {
                        println!(
                            "snapshot {}: {} files, {} bytes",
                            m.snapshot_id, m.total_files, m.total_bytes
                        );
                    }
                }
                ArchiveAction::List { archive_root, json } => {
                    let root = archive_root.unwrap_or_else(archive::default_root);
                    let archive_handle = Archive::at(root)?;
                    let metas = archive_handle.list()?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&metas)?);
                    } else {
                        for m in metas {
                            println!(
                                "{}  {} files  {} bytes",
                                m.snapshot_id, m.total_files, m.total_bytes
                            );
                        }
                    }
                }
                ArchiveAction::Show {
                    snapshot_id,
                    archive_root,
                    json,
                } => {
                    let root = archive_root.unwrap_or_else(archive::default_root);
                    let archive_handle = Archive::at(root)?;
                    let manifest = archive_handle.show(&snapshot_id)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&manifest)?);
                    } else {
                        println!(
                            "{} ({} providers)",
                            manifest.snapshot_id,
                            manifest.providers.len()
                        );
                        for p in &manifest.providers {
                            println!("  {}: {} files", p.name, p.files.len());
                        }
                    }
                }
                ArchiveAction::Restore {
                    snapshot_id,
                    to,
                    archive_root,
                } => {
                    let root = archive_root.unwrap_or_else(archive::default_root);
                    let archive_handle = Archive::at(root)?;
                    let dest = to.unwrap_or_else(|| {
                        std::path::PathBuf::from(format!("heimdall-restore-{snapshot_id}"))
                    });
                    archive_handle.restore(&snapshot_id, &dest)?;
                    println!("restored {} -> {}", snapshot_id, dest.display());
                }
                ArchiveAction::Prune {
                    keep_last,
                    archive_root,
                } => {
                    let root = archive_root.unwrap_or_else(archive::default_root);
                    let _lock = ArchiveLock::acquire(&root)?;
                    let archive_handle = Archive::at(root)?;
                    let (snaps, objs) = archive_handle.prune(keep_last)?;
                    println!("pruned {} snapshots, {} objects", snaps, objs);
                }
                ArchiveAction::Verify { archive_root } => {
                    let root = archive_root.unwrap_or_else(archive::default_root);
                    let archive_handle = Archive::at(root)?;
                    let report = archive_handle.verify()?;
                    println!(
                        "verified {} objects across {} manifests; {} issues",
                        report.objects_checked,
                        report.manifests_checked,
                        report.corrupt_objects.len()
                    );
                    for line in &report.corrupt_objects {
                        eprintln!("  {line}");
                    }
                    if !report.corrupt_objects.is_empty() {
                        std::process::exit(2);
                    }
                }
            }
        }
        Commands::ImportExport {
            zip,
            watch,
            archive_root,
            json,
        } => {
            let root = archive_root.unwrap_or_else(archive::default_root);
            if let Some(watch_dir) = watch {
                archive::imports::watch::run_watch(&root, &watch_dir)?;
                return Ok(());
            }
            let zip = zip.ok_or_else(|| {
                anyhow::anyhow!("either <zip> argument or --watch <dir> is required")
            })?;
            let report = archive::imports::import_zip(&root, &zip)?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "import_id": report.import_id,
                        "vendor": report.vendor.slug(),
                        "conversation_count": report.conversation_count,
                        "parse_warnings": report.parse_warnings,
                        "root": report.root.display().to_string(),
                    }))?
                );
            } else {
                println!(
                    "imported {} {} conversations into {}",
                    report.conversation_count,
                    report.vendor.slug(),
                    report.root.display()
                );
                if !report.parse_warnings.is_empty() {
                    eprintln!(
                        "  {} warnings written to parse-errors.json",
                        report.parse_warnings.len()
                    );
                }
            }
        }
        Commands::Hook { action } => {
            cmd_hook(action)?;
        }
        Commands::Daemon { action } => {
            cmd_daemon(action)?;
        }
        Commands::Db { action } => match action {
            DbAction::Reset { db_path, yes } => {
                let db = default_db(db_path);
                db_mod::cmd_db_reset(&db, yes)?;
            }
        },
        Commands::Blocks {
            db_path,
            session_length,
            provider,
            active,
            json,
            token_limit,
            jq,
            no_gaps,
            locale,
            compact: _,
        } => {
            let db = default_db(db_path);
            // Resolution order: CLI flag > provider-specific config > flat config > 5.0.
            // We replicate the same logic as `Config::resolved_session_length` using the
            // pre-extracted snapshots (cfg is partially moved above the match).
            let session_hours = if let Some(h) = session_length {
                h
            } else if let Some(p) = provider.as_deref() {
                if let Some(&h) = cfg_blocks_session_length_by_provider.get(p) {
                    if h > 0.0 && h <= 168.0 {
                        h
                    } else {
                        tracing::warn!(
                            "invalid session_length {h} for provider '{p}' from config, falling back to 5.0"
                        );
                        5.0
                    }
                } else {
                    cfg_blocks_session_length
                }
            } else {
                cfg_blocks_session_length
            };
            let loc = locale::resolve_locale(locale.as_deref(), cfg_display_locale.as_deref());
            cmd_blocks(
                &db,
                session_hours,
                active,
                json,
                token_limit,
                jq.as_deref(),
                !no_gaps,
                loc,
            )?;
        }
        Commands::Weekly {
            db_path,
            start_of_week,
            json,
            breakdown,
            jq,
            project_aliases,
            locale,
            compact,
        } => {
            let db = default_db(db_path);
            let mut aliases = cfg_project_aliases.clone();
            for (k, v) in project_aliases {
                aliases.insert(k, v);
            }
            let loc = locale::resolve_locale(locale.as_deref(), cfg_display_locale.as_deref());
            let effective_compact = compact || cfg_display_compact;
            cmd_weekly(
                &db,
                start_of_week,
                json,
                breakdown,
                jq.as_deref(),
                &aliases,
                loc,
                effective_compact,
            )?;
        }
        Commands::Statusline {
            refresh_interval,
            cost_source,
            offline,
            db_path,
            visual_burn_rate,
        } => {
            let opts = statusline::StatuslineOpts {
                refresh_interval,
                cost_source,
                offline,
                db_path,
                context_low_threshold: cfg_statusline_low,
                context_medium_threshold: cfg_statusline_medium,
                burn_rate_normal_max: cfg_burn_rate_normal_max,
                burn_rate_moderate_max: cfg_burn_rate_moderate_max,
                visual_burn_rate,
            };
            statusline::run(&opts);
        }
        Commands::StatuslineHook { action } => {
            cmd_statusline_hook(action)?;
        }
        #[cfg(feature = "mcp")]
        Commands::Mcp { action } => {
            cmd_mcp(action, &default_db)?;
        }
        Commands::Config { action } => {
            cmd_config(action)?;
        }
        Commands::Scrape { action } => {
            let rt = tokio::runtime::Runtime::new()?;
            match action {
                ScrapeAction::Claude {
                    session_key,
                    cf_clearance,
                    user_agent,
                    archive_root,
                    json,
                } => {
                    const DEFAULT_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15";
                    let session_key = session_key
                        .or_else(|| std::env::var("HEIMDALL_CLAUDE_SESSION_KEY").ok())
                        .ok_or_else(|| anyhow::anyhow!(
                            "--session-key (or HEIMDALL_CLAUDE_SESSION_KEY) required"
                        ))?;
                    let cf_clearance = cf_clearance
                        .or_else(|| std::env::var("HEIMDALL_CLAUDE_CF_CLEARANCE").ok());
                    let user_agent = user_agent
                        .or_else(|| std::env::var("HEIMDALL_USER_AGENT").ok())
                        .unwrap_or_else(|| DEFAULT_UA.to_string());
                    let root = archive_root.unwrap_or_else(archive::default_root);
                    let report =
                        rt.block_on(scrape_claude_run(&session_key, cf_clearance, &user_agent, &root))?;
                    print_scrape_report(&report, json)?;
                }
                ScrapeAction::Chatgpt {
                    session_token,
                    access_token,
                    cf_clearance,
                    user_agent,
                    archive_root,
                    json,
                } => {
                    const DEFAULT_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15";
                    let session_token = session_token
                        .or_else(|| std::env::var("HEIMDALL_CHATGPT_SESSION_TOKEN").ok())
                        .ok_or_else(|| anyhow::anyhow!(
                            "--session-token (or HEIMDALL_CHATGPT_SESSION_TOKEN) required"
                        ))?;
                    let access_token = access_token
                        .or_else(|| std::env::var("HEIMDALL_CHATGPT_ACCESS_TOKEN").ok())
                        .ok_or_else(|| anyhow::anyhow!(
                            "--access-token (or HEIMDALL_CHATGPT_ACCESS_TOKEN) required"
                        ))?;
                    let cf_clearance = cf_clearance
                        .or_else(|| std::env::var("HEIMDALL_CHATGPT_CF_CLEARANCE").ok());
                    let user_agent = user_agent
                        .or_else(|| std::env::var("HEIMDALL_USER_AGENT").ok())
                        .unwrap_or_else(|| DEFAULT_UA.to_string());
                    let root = archive_root.unwrap_or_else(archive::default_root);
                    let report = rt.block_on(scrape_chatgpt_run(
                        &session_token,
                        &access_token,
                        cf_clearance,
                        &user_agent,
                        &root,
                    ))?;
                    print_scrape_report(&report, json)?;
                }
            }
        }
        Commands::CompanionToken { action } => {
            let path = archive::companion_token::default_path();
            match action {
                CompanionTokenAction::Show => {
                    let t = archive::companion_token::read_or_init(&path)?;
                    println!("{}", t.as_hex());
                }
                CompanionTokenAction::Rotate => {
                    let t = archive::companion_token::rotate(&path)?;
                    println!("{}", t.as_hex());
                }
            }
        }
    }
    Ok(())
}

/// When config says `source = "litellm"`, load whatever is currently on disk
/// into the pricing lookup map. If the cache is missing or stale, spawn a
/// background thread that fetches and writes a fresh copy — startup is never
/// blocked.
fn maybe_load_litellm(cfg: &config::Config) {
    if !cfg.pricing_source.is_litellm() {
        return;
    }
    let cache = litellm::cache_path();
    let refresh_hours = cfg.pricing_source.effective_refresh_hours();

    // Load whatever is currently on disk (may be None if cache absent).
    let maybe_snapshot = litellm::read_cache(&cache);
    let needs_refresh = match &maybe_snapshot {
        None => {
            tracing::info!("litellm pricing: cache missing, refresh in background");
            true
        }
        Some(snap) => {
            let age_h = snap.age_hours();
            let model_count = snap.entries.len();
            tracing::info!(
                "litellm pricing: {} models loaded from cache (age: {:.1}h)",
                model_count,
                age_h
            );
            age_h > refresh_hours as f64
        }
    };

    // Install the current on-disk map synchronously.
    if let Some(snap) = maybe_snapshot {
        let map = pricing::load_litellm_cache_from_snapshot(snap);
        if !map.is_empty() {
            pricing::set_litellm_map(map);
        }
    }

    // Kick off background refresh if needed (non-blocking).
    if needs_refresh {
        let cache_clone = cache.clone();
        std::thread::spawn(move || {
            if let Some(fresh) = litellm::fetch_live() {
                let count = fresh.entries.len();
                match litellm::write_cache(&cache_clone, &fresh) {
                    Ok(()) => tracing::info!(
                        "litellm pricing: background refresh complete ({} models)",
                        count
                    ),
                    Err(e) => tracing::warn!("litellm pricing: background write failed: {}", e),
                }
            } else {
                tracing::warn!("litellm pricing: background fetch failed");
            }
        });
    }
}

/// Convert config pricing overrides into the pricing module's runtime overrides.
fn apply_pricing_overrides(cfg: &config::Config) {
    if cfg.pricing.is_empty() {
        return;
    }
    let overrides: HashMap<String, pricing::ModelPricing> = cfg
        .pricing
        .iter()
        .map(|(name, p)| {
            // For cache rates, default to standard multipliers if not specified
            let cache_write = p.cache_write.unwrap_or(p.input * 1.25);
            let cache_read = p.cache_read.unwrap_or(p.input * 0.1);
            (
                name.clone(),
                pricing::ModelPricing {
                    input: p.input,
                    output: p.output,
                    cache_write,
                    cache_read,
                    threshold_tokens: None,
                    input_above_threshold: None,
                    output_above_threshold: None,
                },
            )
        })
        .collect();
    tracing::info!("Loaded {} pricing override(s) from config", overrides.len());
    pricing::set_overrides(overrides);
}

/// Apply a jq filter to `value` and print the result, or exit 2 on error.
///
/// - Empty result → no output, exit 0.
/// - Single result → println, exit 0.
/// - Multiple results → one line each, exit 0.
/// - Error → eprintln to stderr, std::process::exit(2).
fn apply_jq_and_print(value: &serde_json::Value, filter: &str) {
    match jq_mod::apply(value, filter) {
        Ok(jq_mod::JqResult::Empty) => {}
        Ok(jq_mod::JqResult::Single(s)) => println!("{s}"),
        Ok(jq_mod::JqResult::Multiple(vs)) => {
            for v in vs {
                println!("{v}");
            }
        }
        Err(e) => {
            eprintln!("jq error: {e}");
            std::process::exit(2);
        }
    }
}

fn cmd_optimize(db_path: &std::path::Path, format: &str, jq: Option<&str>) -> Result<()> {
    use optimizer::Severity;

    if !db_path.exists() {
        anyhow::bail!(
            "Database not found at {}. Run: claude-usage-tracker scan",
            db_path.display()
        );
    }

    let report = optimizer::run_optimize(db_path)?;

    // --jq implies JSON output.
    let effective_format = if jq.is_some() { "json" } else { format };

    match effective_format.to_ascii_lowercase().as_str() {
        "json" => {
            let value = serde_json::to_value(&report)?;
            if let Some(filter) = jq {
                apply_jq_and_print(&value, filter);
            } else {
                println!("{}", serde_json::to_string_pretty(&value)?);
            }
        }
        _ => {
            println!();
            println!("{}", "=".repeat(70));
            println!("  Optimize Report  --  Grade: {}", report.grade);
            println!("{}", "=".repeat(70));

            if report.findings.is_empty() {
                println!("  No waste findings. Configuration looks clean.");
            } else {
                let waste_usd = report.total_monthly_waste_nanos as f64 / 1_000_000_000.0;
                println!(
                    "  {} finding(s)  |  Est. monthly waste: {}",
                    report.findings.len(),
                    pricing::fmt_cost(waste_usd),
                );
                println!("{}", "-".repeat(70));
                for f in &report.findings {
                    let sev = match f.severity {
                        Severity::Low => "[low]   ",
                        Severity::Medium => "[medium]",
                        Severity::High => "[HIGH]  ",
                    };
                    println!("  {} {}", sev, f.title);
                    println!("          {}", f.detail);
                    if f.estimated_monthly_waste_nanos > 0 {
                        let w = f.estimated_monthly_waste_nanos as f64 / 1_000_000_000.0;
                        println!("          Est. monthly waste: {}", pricing::fmt_cost(w));
                    }
                    println!();
                }
            }
            println!("{}", "=".repeat(70));
            println!();
        }
    }
    Ok(())
}

fn cmd_scheduler(action: SchedulerAction, default_db: &std::path::Path) -> Result<()> {
    use scheduler::{InstallStatus, Interval};
    use std::str::FromStr;

    let sched = scheduler::current();

    match action {
        SchedulerAction::Install {
            interval,
            db_path,
            include_archive,
        } => {
            let interval = Interval::from_str(&interval)?;
            let bin_path = scheduler::resolve_bin_path()?;
            let db = db_path.unwrap_or_else(|| default_db.to_path_buf());
            sched.install(interval, &bin_path, &db)?;
            // Report status after install.
            match sched.status()? {
                InstallStatus::Installed {
                    next_run_hint,
                    config_path,
                } => {
                    if let Some(path) = config_path {
                        println!("Installed: {} ({})", next_run_hint, path.display());
                    } else {
                        println!("Installed: {}", next_run_hint);
                    }
                }
                InstallStatus::NotInstalled => {
                    println!("Installed (status unknown)");
                }
                InstallStatus::UnsupportedPlatform(plat) => {
                    eprintln!("Unsupported platform: {}", plat);
                    std::process::exit(1);
                }
            }
            if include_archive {
                let archive_sched = scheduler::current_for(scheduler::ARCHIVE_JOB);
                archive_sched.install(scheduler::Interval::Daily, &bin_path, &db)?;
                println!("scheduler: archive job installed (daily)");
            }
        }
        SchedulerAction::Uninstall => {
            sched.uninstall()?;
            println!("Uninstalled: scheduled scan job removed");
            let archive_sched = scheduler::current_for(scheduler::ARCHIVE_JOB);
            archive_sched.uninstall()?;
        }
        SchedulerAction::Status => match sched.status()? {
            InstallStatus::Installed {
                next_run_hint,
                config_path,
            } => {
                if let Some(path) = config_path {
                    println!("Installed: {} ({})", next_run_hint, path.display());
                } else {
                    println!("Installed: {}", next_run_hint);
                }
            }
            InstallStatus::NotInstalled => {
                println!("Not installed");
            }
            InstallStatus::UnsupportedPlatform(plat) => {
                println!("Unsupported platform: {}", plat);
            }
        },
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_dashboard(
    db: std::path::PathBuf,
    dirs: Option<Vec<std::path::PathBuf>>,
    host: String,
    port: u16,
    watch: bool,
    no_open: bool,
    background_poll: bool,
    cfg_host: Option<String>,
    cfg_port: Option<u16>,
    cfg_oauth_enabled: bool,
    cfg_oauth_refresh: u64,
    cfg_claude_admin_enabled: bool,
    cfg_claude_admin_key_env: String,
    cfg_claude_admin_refresh_interval: u64,
    cfg_claude_admin_lookback_days: i64,
    cfg_openai_enabled: bool,
    cfg_openai_admin_key_env: String,
    cfg_openai_refresh_interval: u64,
    cfg_openai_lookback_days: i64,
    cfg_webhooks: config::WebhookConfig,
    cfg_agent_status: config::AgentStatusConfig,
    cfg_aggregator: config::AggregatorConfig,
    cfg_blocks_token_limit: Option<i64>,
    cfg_blocks_session_length: f64,
    cfg_project_aliases: std::collections::HashMap<String, String>,
) -> Result<()> {
    // In interactive mode (no --background-poll) we block until the
    // initial scan completes so the browser opens to populated data.
    // In background mode (menu-bar app), defer the scan so the HTTP
    // listener comes up immediately — readiness probes cannot wait
    // for a multi-second JSONL walk.
    if !background_poll {
        eprintln!("Running scan first...");
        scanner::scan(dirs.clone(), &db, true)?;
    }

    let host_env = std::env::var("HOST").ok().or(cfg_host).unwrap_or(host);
    let port_env = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .or(cfg_port)
        .unwrap_or(port);

    let url = format!("http://{}:{}", host_env, port_env);
    if !no_open {
        let _ = open::that(&url);
    }

    let rt = tokio::runtime::Runtime::new()?;
    let scan_dirs = dirs.clone();
    let scan_db = db.clone();
    rt.block_on(async move {
        if background_poll {
            tokio::task::spawn_blocking(move || {
                if let Err(e) = scanner::scan(scan_dirs, &scan_db, true) {
                    tracing::warn!(error = ?e, "background initial scan failed");
                }
            });
        }
        server::serve(server::ServeOptions {
            host: host_env,
            port: port_env,
            db_path: db,
            projects_dirs: dirs,
            oauth_enabled: cfg_oauth_enabled,
            oauth_refresh_interval: cfg_oauth_refresh,
            claude_admin_enabled: cfg_claude_admin_enabled,
            claude_admin_key_env: cfg_claude_admin_key_env,
            claude_admin_refresh_interval: cfg_claude_admin_refresh_interval,
            claude_admin_lookback_days: cfg_claude_admin_lookback_days,
            openai_enabled: cfg_openai_enabled,
            openai_admin_key_env: cfg_openai_admin_key_env,
            openai_refresh_interval: cfg_openai_refresh_interval,
            openai_lookback_days: cfg_openai_lookback_days,
            webhook_config: cfg_webhooks,
            watch,
            background_poll,
            agent_status_config: cfg_agent_status,
            aggregator_config: cfg_aggregator,
            blocks_token_limit: cfg_blocks_token_limit,
            session_length_hours: cfg_blocks_session_length,
            project_aliases: cfg_project_aliases,
        })
        .await
    })?;
    Ok(())
}

fn cmd_pricing(
    action: PricingAction,
    default_db: &std::path::Path,
    sync_options: &pricing_defs::OfficialSyncOptions,
) -> Result<()> {
    use scheduler::{InstallStatus, Interval, PRICING_SYNC_JOB};
    use std::str::FromStr;

    match action {
        PricingAction::Refresh { cache_path } => {
            let path = cache_path.unwrap_or_else(litellm::cache_path);
            match litellm::run_refresh(&path) {
                Ok((count, written)) => {
                    println!("Fetched {} models, cached at {}", count, written.display());
                }
                Err(reason) => {
                    anyhow::bail!("Refresh failed: {}", reason);
                }
            }
        }
        PricingAction::Sync { db_path } => {
            let db = db_path.unwrap_or_else(|| default_db.to_path_buf());
            let conn = scanner::db::open_db(&db)?;
            scanner::db::init_db(&conn)?;
            let summary = pricing_sync::sync_pricing(&conn, sync_options)?;

            println!(
                "Pricing sync complete: {} / {} official sources parsed",
                summary.successful_sources, summary.total_sources
            );
            println!(
                "Stored {} sync run(s) and {} extracted record(s)",
                summary.metadata_runs, summary.metadata_records
            );
            if summary.changed_models.is_empty() {
                println!("No effective catalog changes detected");
            } else {
                println!(
                    "Detected {} pricing change(s): {}",
                    summary.changed_models.len(),
                    summary.changed_models.join(", ")
                );
                println!(
                    "Repriced {} turn(s) across {} session(s)",
                    summary.repriced_turns, summary.repriced_sessions
                );
                if let Some(version) = summary.pricing_version {
                    println!("Applied pricing version: {}", version);
                }
            }
        }
        PricingAction::Install { interval, db_path } => {
            let sched = scheduler::current_for(PRICING_SYNC_JOB);
            let interval = Interval::from_str(&interval)?;
            let bin_path = scheduler::resolve_bin_path()?;
            let db = db_path.unwrap_or_else(|| default_db.to_path_buf());
            sched.install(interval, &bin_path, &db)?;
            match sched.status()? {
                InstallStatus::Installed {
                    next_run_hint,
                    config_path,
                } => {
                    if let Some(path) = config_path {
                        println!("Installed: {} ({})", next_run_hint, path.display());
                    } else {
                        println!("Installed: {}", next_run_hint);
                    }
                }
                InstallStatus::NotInstalled => println!("Installed (status unknown)"),
                InstallStatus::UnsupportedPlatform(plat) => {
                    eprintln!("Unsupported platform: {}", plat);
                    std::process::exit(1);
                }
            }
        }
        PricingAction::Uninstall => {
            let sched = scheduler::current_for(PRICING_SYNC_JOB);
            sched.uninstall()?;
            println!("Uninstalled: scheduled pricing sync job removed");
        }
        PricingAction::Status => {
            let sched = scheduler::current_for(PRICING_SYNC_JOB);
            match sched.status()? {
                InstallStatus::Installed {
                    next_run_hint,
                    config_path,
                } => {
                    if let Some(path) = config_path {
                        println!("Installed: {} ({})", next_run_hint, path.display());
                    } else {
                        println!("Installed: {}", next_run_hint);
                    }
                }
                InstallStatus::NotInstalled => println!("Not installed"),
                InstallStatus::UnsupportedPlatform(plat) => {
                    eprintln!("Unsupported platform: {}", plat);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

fn cmd_usage_monitor(
    action: UsageMonitorAction,
    default_db: &std::path::Path,
    cfg: &config::ClaudeUsageMonitorConfig,
) -> Result<()> {
    use scheduler::{InstallStatus, Interval, USAGE_MONITOR_JOB};
    use std::str::FromStr;

    match action {
        UsageMonitorAction::Capture { db_path } => {
            let db = db_path.unwrap_or_else(|| default_db.to_path_buf());
            let result = usage_monitor::capture_snapshot(&usage_monitor::CaptureOptions {
                db_path: db,
                claude_binary: cfg.claude_binary.clone(),
                working_dir: cfg.working_dir.clone(),
            })?;
            match result.status.as_str() {
                "success" => {
                    println!(
                        "Captured Claude /usage snapshot: run {} (success)",
                        result.run_id
                    );
                }
                "unparsed" => {
                    anyhow::bail!(
                        "Claude /usage capture stored raw output but could not parse it (run {})",
                        result.run_id
                    );
                }
                _ => {
                    anyhow::bail!("Claude /usage capture failed (run {})", result.run_id);
                }
            }
        }
        UsageMonitorAction::Install { interval, db_path } => {
            let sched = scheduler::current_for(USAGE_MONITOR_JOB);
            let interval_str = interval.unwrap_or_else(|| cfg.default_interval.clone());
            let interval = Interval::from_str(&interval_str)?;
            let bin_path = scheduler::resolve_bin_path()?;
            let db = db_path.unwrap_or_else(|| default_db.to_path_buf());
            sched.install(interval, &bin_path, &db)?;
            match sched.status()? {
                InstallStatus::Installed {
                    next_run_hint,
                    config_path,
                } => {
                    if let Some(path) = config_path {
                        println!("Installed: {} ({})", next_run_hint, path.display());
                    } else {
                        println!("Installed: {}", next_run_hint);
                    }
                }
                InstallStatus::NotInstalled => println!("Installed (status unknown)"),
                InstallStatus::UnsupportedPlatform(plat) => {
                    eprintln!("Unsupported platform: {}", plat);
                    std::process::exit(1);
                }
            }
        }
        UsageMonitorAction::Uninstall => {
            let sched = scheduler::current_for(USAGE_MONITOR_JOB);
            sched.uninstall()?;
            println!("Uninstalled: scheduled Claude /usage capture job removed");
        }
        UsageMonitorAction::Status => {
            let sched = scheduler::current_for(USAGE_MONITOR_JOB);
            match sched.status()? {
                InstallStatus::Installed {
                    next_run_hint,
                    config_path,
                } => {
                    if let Some(path) = config_path {
                        println!("Installed: {} ({})", next_run_hint, path.display());
                    } else {
                        println!("Installed: {}", next_run_hint);
                    }
                }
                InstallStatus::NotInstalled => println!("Not installed"),
                InstallStatus::UnsupportedPlatform(plat) => {
                    println!("Unsupported platform: {}", plat);
                }
            }
        }
    }
    Ok(())
}

fn cmd_hook(action: HookAction) -> Result<()> {
    use hook::install::{
        HookActionResult, HookStatus, install, resolve_hook_binary_path, status, uninstall,
    };

    match action {
        HookAction::Install => {
            let bin = resolve_hook_binary_path()?;
            match install(&bin)? {
                HookActionResult::Installed { binary_path } => {
                    println!("Installed: heimdall-hook entry added to ~/.claude/settings.json");
                    println!("  binary: {}", binary_path.display());
                }
                HookActionResult::Updated { binary_path } => {
                    println!("Updated: heimdall-hook entry refreshed in ~/.claude/settings.json");
                    println!("  binary: {}", binary_path.display());
                }
                _ => {}
            }
        }
        HookAction::Uninstall => match uninstall()? {
            HookActionResult::Uninstalled => {
                println!("Uninstalled: heimdall-hook entry removed from ~/.claude/settings.json");
            }
            HookActionResult::NothingToUninstall => {
                println!("Nothing to uninstall: no heimdall-hook entry found");
            }
            _ => {}
        },
        HookAction::Status => match status()? {
            HookStatus::Present { binary_path } => {
                println!("Installed");
                println!("  binary: {}", binary_path);
            }
            HookStatus::Absent => {
                println!("Not installed");
                println!("  Run: claude-usage-tracker hook install");
            }
        },
    }
    Ok(())
}

fn cmd_statusline_hook(action: StatuslineHookAction) -> Result<()> {
    use statusline::install::{
        StatuslineActionResult, StatuslineStatus, install as sl_install, status as sl_status,
        uninstall as sl_uninstall,
    };

    match action {
        StatuslineHookAction::Install => match sl_install()? {
            StatuslineActionResult::Installed => {
                println!("Installed: statusLine entry added to ~/.claude/settings.json");
                println!("  command: claude-usage-tracker statusline");
            }
            StatuslineActionResult::Updated => {
                println!("Updated: statusLine entry refreshed in ~/.claude/settings.json");
                println!("  command: claude-usage-tracker statusline");
            }
            _ => {}
        },
        StatuslineHookAction::Uninstall => match sl_uninstall()? {
            StatuslineActionResult::Uninstalled => {
                println!("Uninstalled: statusLine entry removed from ~/.claude/settings.json");
            }
            StatuslineActionResult::NothingToUninstall => {
                println!("Nothing to uninstall: no heimdall statusLine entry found");
            }
            _ => {}
        },
        StatuslineHookAction::Status => match sl_status()? {
            StatuslineStatus::Present { command } => {
                println!("Installed");
                println!("  command: {}", command);
            }
            StatuslineStatus::Absent => {
                println!("Not installed");
                println!("  Run: claude-usage-tracker statusline-hook install");
            }
        },
    }
    Ok(())
}

#[cfg(feature = "mcp")]
fn cmd_mcp(action: McpAction, default_db: &dyn Fn(Option<PathBuf>) -> PathBuf) -> Result<()> {
    use mcp::install::{McpInstallResult, McpInstallStatus};

    match action {
        McpAction::Serve {
            transport,
            host,
            port,
            db_path,
        } => {
            let db = default_db(db_path);
            let rt = tokio::runtime::Runtime::new()?;
            match transport {
                mcp::McpTransport::Stdio => {
                    rt.block_on(mcp::run_stdio(db))?;
                }
                mcp::McpTransport::Http => {
                    rt.block_on(mcp::run_http(&host, port, db))?;
                }
            }
        }
        McpAction::Install { client } => match mcp::install::install(&client)? {
            McpInstallResult::Installed { path } => {
                println!("Installed: heimdall MCP server added to {}", path.display());
            }
            McpInstallResult::Updated { path } => {
                println!(
                    "Updated: heimdall MCP server refreshed in {}",
                    path.display()
                );
            }
            _ => {}
        },
        McpAction::Uninstall { client } => match mcp::install::uninstall(&client)? {
            McpInstallResult::Uninstalled { path } => {
                println!(
                    "Uninstalled: heimdall entry removed from {}",
                    path.display()
                );
            }
            McpInstallResult::NothingToUninstall => {
                println!("Nothing to uninstall: no heimdall entry found (or user-customized)");
            }
            _ => {}
        },
        McpAction::Status { client } => match mcp::install::status(&client)? {
            McpInstallStatus::Installed { path } => {
                println!("Installed: {}", path.display());
            }
            McpInstallStatus::Customized { path } => {
                println!(
                    "Customized: heimdall entry present but not installed by us: {}",
                    path.display()
                );
            }
            McpInstallStatus::Absent => {
                println!("Not installed");
                println!("  Run: claude-usage-tracker mcp install");
            }
        },
    }
    Ok(())
}

fn cmd_daemon(action: DaemonAction) -> Result<()> {
    use scheduler::daemon::current_daemon_scheduler;
    use scheduler::{InstallStatus, resolve_bin_path};

    let sched = current_daemon_scheduler();

    match action {
        DaemonAction::Install => {
            let bin = resolve_bin_path()?;
            sched.install(&bin)?;
            match sched.status()? {
                InstallStatus::Installed {
                    next_run_hint,
                    config_path,
                } => {
                    println!("Installed: {}", next_run_hint);
                    if let Some(p) = config_path {
                        println!("  plist: {}", p.display());
                    }
                }
                InstallStatus::NotInstalled => {
                    println!("Installed (status unknown)");
                }
                InstallStatus::UnsupportedPlatform(plat) => {
                    eprintln!(
                        "daemon subcommand is currently macOS-only; \
                         Linux systemd and Windows Service support is deferred (platform: {})",
                        plat
                    );
                    std::process::exit(1);
                }
            }
        }
        DaemonAction::Uninstall => {
            sched.uninstall()?;
            println!("Uninstalled: dashboard daemon removed");
        }
        DaemonAction::Status => match sched.status()? {
            InstallStatus::Installed {
                next_run_hint,
                config_path,
            } => {
                println!("Installed: {}", next_run_hint);
                if let Some(p) = config_path {
                    println!("  plist: {}", p.display());
                }
            }
            InstallStatus::NotInstalled => {
                println!("Not installed");
            }
            InstallStatus::UnsupportedPlatform(_) => {
                eprintln!(
                    "daemon subcommand is currently macOS-only; \
                     Linux systemd and Windows Service support is deferred"
                );
                std::process::exit(1);
            }
        },
    }
    Ok(())
}

// CI guarantees `schemas/heimdall.config.schema.json` matches the output of
// `cargo run -- config schema`; see the rust-stable job in
// .github/workflows/ci.yml.  Regenerate after editing config.rs and commit
// the schema alongside the code change.

fn cmd_config(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Schema => {
            let schema = schemars::schema_for!(config::Config);
            println!("{}", serde_json::to_string_pretty(&schema)?);
        }
        ConfigAction::Show { format } => {
            let cfg = config::load_config_resolved();
            match format.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(&cfg)?);
                }
                _ => {
                    // toml (default)
                    match toml::to_string_pretty(&cfg) {
                        Ok(s) => print!("{}", s),
                        Err(e) => {
                            eprintln!(
                                "Warning: could not serialize to TOML: {}. Falling back to JSON.",
                                e
                            );
                            println!("{}", serde_json::to_string_pretty(&cfg)?);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod cli_tests;

use scanner::db::{StatsModelRow, TodayModelRow};
type ProviderRollup = (i64, i64, i64, i64, i64, i64, i64);

pub(crate) fn cmd_today(
    db_path: &std::path::Path,
    json_output: bool,
    breakdown: bool,
    jq: Option<&str>,
    _aliases: &HashMap<String, String>,
    display_locale: chrono::Locale,
    compact: bool,
) -> Result<()> {
    if !db_path.exists() {
        anyhow::bail!("Database not found. Run: claude-usage-tracker scan");
    }
    let conn = scanner::db::open_db(db_path)?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let rows: Vec<TodayModelRow> = scanner::db::query_today_model_rows(&conn, &today)?;

    if json_output || jq.is_some() {
        let by_provider: Vec<serde_json::Value> =
            scanner::db::query_today_provider_breakdown(&conn, &today)?
                .into_iter()
                .map(|(provider, turns, inp, out, cr, cc, ro, cost_nanos)| {
                    serde_json::json!({
                        "provider": provider,
                        "turns": turns,
                        "input_tokens": inp,
                        "output_tokens": out,
                        "cache_read_tokens": cr,
                        "cache_creation_tokens": cc,
                        "reasoning_output_tokens": ro,
                        "estimated_cost": cost_nanos as f64 / 1_000_000_000.0,
                    })
                })
                .collect();
        let confidence_breakdown: Vec<serde_json::Value> =
            scanner::db::query_today_confidence_breakdown(&conn, &today)?
                .into_iter()
                .map(|(conf, turns, cost_nanos)| {
                    serde_json::json!({
                        "cost_confidence": conf,
                        "turns": turns,
                        "estimated_cost": cost_nanos as f64 / 1_000_000_000.0,
                    })
                })
                .collect();
        let billing_mode_breakdown: Vec<serde_json::Value> =
            scanner::db::query_today_billing_mode_breakdown(&conn, &today)?
                .into_iter()
                .map(|(mode, turns, cost_nanos)| {
                    serde_json::json!({
                        "billing_mode": mode,
                        "turns": turns,
                        "estimated_cost": cost_nanos as f64 / 1_000_000_000.0,
                    })
                })
                .collect();
        let models: Vec<serde_json::Value> = rows
            .iter()
            .map(
                |(
                    provider,
                    model,
                    inp,
                    out,
                    cr,
                    cc,
                    ro,
                    turns,
                    cost_nanos,
                    cost_confidence,
                    billing_mode,
                )| {
                    serde_json::json!({
                        "provider": provider, "model": model, "turns": turns,
                        "input_tokens": inp, "output_tokens": out,
                        "cache_read_tokens": cr, "cache_creation_tokens": cc,
                        "reasoning_output_tokens": ro,
                        "estimated_cost": *cost_nanos as f64 / 1_000_000_000.0,
                        "cost_confidence": cost_confidence,
                        "billing_mode": billing_mode,
                    })
                },
            )
            .collect();
        let total_cost: f64 = rows
            .iter()
            .map(|(_, _, _, _, _, _, _, _, cost_nanos, _, _)| *cost_nanos as f64 / 1_000_000_000.0)
            .sum();
        let output = serde_json::json!({
            "date": today,
            "models": models,
            "by_provider": by_provider,
            "confidence_breakdown": confidence_breakdown,
            "billing_mode_breakdown": billing_mode_breakdown,
            "total_estimated_cost": (total_cost * 10000.0).round() / 10000.0,
        });
        if let Some(filter) = jq {
            apply_jq_and_print(&output, filter);
        } else {
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        return Ok(());
    }

    if !compact
        && std::io::IsTerminal::is_terminal(&std::io::stdout())
        && std::env::var("COLUMNS")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .is_some_and(|c| c < 100)
    {
        eprintln!("(narrow terminal detected; try --compact)");
    }

    let today_display =
        locale::format_naive_date(chrono::Local::now().date_naive(), display_locale);
    println!();
    println!("{}", "-".repeat(70));
    println!("  Today's Usage  ({})", today_display);
    println!("{}", "-".repeat(70));

    if rows.is_empty() {
        println!("  No usage recorded today.");
        println!();
        return Ok(());
    }

    let mut total_cost = 0.0;
    let mut provider_totals: std::collections::BTreeMap<String, (i64, i64, i64, i64, i64, f64)> =
        std::collections::BTreeMap::new();
    let mut confidence_totals: std::collections::BTreeMap<String, (i64, f64)> =
        std::collections::BTreeMap::new();
    let mut billing_mode_totals: std::collections::BTreeMap<String, (i64, f64)> =
        std::collections::BTreeMap::new();
    // Group rows by provider for breakdown rendering (preserves insertion order via BTreeMap).
    let mut rows_by_provider: std::collections::BTreeMap<String, Vec<&TodayModelRow>> =
        std::collections::BTreeMap::new();
    for row in &rows {
        let (
            provider,
            _model,
            inp,
            out,
            cr,
            cc,
            _ro,
            turns,
            cost_nanos,
            cost_confidence,
            billing_mode,
        ) = row;
        let cost = *cost_nanos as f64 / 1_000_000_000.0;
        total_cost += cost;
        let entry = provider_totals
            .entry(provider.clone())
            .or_insert((0, 0, 0, 0, 0, 0.0));
        entry.0 += *turns;
        entry.1 += *inp;
        entry.2 += *out;
        entry.3 += *cr;
        entry.4 += *cc;
        entry.5 += cost;
        let confidence_entry = confidence_totals
            .entry(cost_confidence.clone())
            .or_insert((0, 0.0));
        confidence_entry.0 += *turns;
        confidence_entry.1 += cost;
        let billing_entry = billing_mode_totals
            .entry(billing_mode.clone())
            .or_insert((0, 0.0));
        billing_entry.0 += *turns;
        billing_entry.1 += cost;
        rows_by_provider
            .entry(provider.clone())
            .or_default()
            .push(row);
    }

    /// Truncate a string to at most `max_chars` characters at a char boundary.
    fn truncate_model(s: &str, max_chars: usize) -> &str {
        if s.len() <= max_chars {
            s
        } else {
            let end = s
                .char_indices()
                .nth(max_chars)
                .map(|(i, _)| i)
                .unwrap_or(s.len());
            &s[..end]
        }
    }

    if breakdown {
        for (provider, prov_rows) in &rows_by_provider {
            let (p_turns, p_inp, p_out, _p_cr, _p_cc, p_cost) = provider_totals
                .get(provider)
                .copied()
                .unwrap_or((0, 0, 0, 0, 0, 0.0));
            if prov_rows.len() == 1 {
                let (
                    _,
                    model,
                    inp,
                    out,
                    _cr,
                    _cc,
                    _ro,
                    turns,
                    cost_nanos,
                    cost_confidence,
                    billing_mode,
                ) = prov_rows[0];
                let cost = *cost_nanos as f64 / 1_000_000_000.0;
                if compact {
                    println!(
                        "  {:<8}  {:<20}  turns={:<4}  in={:<8}  out={:<8}  cost={}",
                        provider,
                        truncate_model(model, 20),
                        turns,
                        pricing::fmt_tokens(*inp),
                        pricing::fmt_tokens(*out),
                        pricing::fmt_cost(cost),
                    );
                } else {
                    println!(
                        "  {:<8}  {:<30}  turns={:<4}  in={:<8}  out={:<8}  cost={}  conf={}  mode={}",
                        provider,
                        model,
                        turns,
                        pricing::fmt_tokens(*inp),
                        pricing::fmt_tokens(*out),
                        pricing::fmt_cost(cost),
                        cost_confidence,
                        billing_mode,
                    );
                }
            } else {
                println!(
                    "  {:<8}  ({} models){:<21}  turns={:<4}  in={:<8}  out={:<8}  cost={}",
                    provider,
                    prov_rows.len(),
                    "",
                    p_turns,
                    pricing::fmt_tokens(p_inp),
                    pricing::fmt_tokens(p_out),
                    pricing::fmt_cost(p_cost),
                );
                for (
                    _,
                    model,
                    inp,
                    out,
                    _cr,
                    _cc,
                    _ro,
                    turns,
                    cost_nanos,
                    cost_confidence,
                    billing_mode,
                ) in prov_rows
                {
                    let cost = *cost_nanos as f64 / 1_000_000_000.0;
                    if compact {
                        println!(
                            "  \u{2514}\u{2500} {:<20}  turns={:<4}  in={:<8}  out={:<8}  cost={}",
                            truncate_model(model, 20),
                            turns,
                            pricing::fmt_tokens(*inp),
                            pricing::fmt_tokens(*out),
                            pricing::fmt_cost(cost),
                        );
                    } else {
                        println!(
                            "  \u{2514}\u{2500} {:<28}  turns={:<4}  in={:<8}  out={:<8}  cost={}  conf={}  mode={}",
                            model,
                            turns,
                            pricing::fmt_tokens(*inp),
                            pricing::fmt_tokens(*out),
                            pricing::fmt_cost(cost),
                            cost_confidence,
                            billing_mode,
                        );
                    }
                }
            }
        }
    } else {
        for (
            provider,
            model,
            inp,
            out,
            _cr,
            _cc,
            _ro,
            turns,
            cost_nanos,
            cost_confidence,
            billing_mode,
        ) in &rows
        {
            let cost = *cost_nanos as f64 / 1_000_000_000.0;
            if compact {
                println!(
                    "  {:<8}  {:<20}  turns={:<4}  in={:<8}  out={:<8}  cost={}",
                    provider,
                    truncate_model(model, 20),
                    turns,
                    pricing::fmt_tokens(*inp),
                    pricing::fmt_tokens(*out),
                    pricing::fmt_cost(cost),
                );
            } else {
                println!(
                    "  {:<8}  {:<30}  turns={:<4}  in={:<8}  out={:<8}  cost={}  conf={}  mode={}",
                    provider,
                    model,
                    turns,
                    pricing::fmt_tokens(*inp),
                    pricing::fmt_tokens(*out),
                    pricing::fmt_cost(cost),
                    cost_confidence,
                    billing_mode,
                );
            }
        }
    }

    println!("{}", "-".repeat(70));
    println!("  Est. total cost: {}", pricing::fmt_cost(total_cost));
    if !compact {
        println!("  By Provider:");
        for (provider, (turns, input, output, cache_read, cache_creation, cost)) in provider_totals
        {
            println!(
                "    {:<8}  turns={:<6}  in={:<8}  out={:<8}  cached={:<8}  cache_write={:<8}  cost={}",
                provider,
                pricing::fmt_tokens(turns),
                pricing::fmt_tokens(input),
                pricing::fmt_tokens(output),
                pricing::fmt_tokens(cache_read),
                pricing::fmt_tokens(cache_creation),
                pricing::fmt_cost(cost)
            );
        }
        println!("  By Confidence:");
        for (confidence, (turns, cost)) in confidence_totals {
            println!(
                "    {:<8}  turns={:<6}  cost={}",
                confidence,
                pricing::fmt_tokens(turns),
                pricing::fmt_cost(cost)
            );
        }
        println!("  By Billing Mode:");
        for (billing_mode, (turns, cost)) in billing_mode_totals {
            println!(
                "    {:<18}  turns={:<6}  cost={}",
                billing_mode,
                pricing::fmt_tokens(turns),
                pricing::fmt_cost(cost)
            );
        }
    }
    println!();
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_weekly(
    db_path: &std::path::Path,
    start_of_week: chrono::Weekday,
    json_output: bool,
    breakdown: bool,
    jq: Option<&str>,
    _aliases: &HashMap<String, String>,
    display_locale: chrono::Locale,
    compact: bool,
) -> Result<()> {
    if !db_path.exists() {
        anyhow::bail!("Database not found. Run: claude-usage-tracker scan");
    }
    let tz = claude_usage_tracker::tz::TzParams {
        tz_offset_min: None,
        week_starts_on: Some(weekday_to_u8(start_of_week)),
    };
    let conn = scanner::db::open_db(db_path)?;
    let rows = scanner::db::sum_by_week(&conn, tz)?;

    // Group by week.
    let weeks: Vec<String> = rows
        .iter()
        .map(|r| r.week.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut by_week: std::collections::HashMap<String, Vec<&scanner::db::WeekRow>> =
        std::collections::HashMap::new();
    for r in &rows {
        by_week.entry(r.week.clone()).or_default().push(r);
    }

    let sow_str = format!("{}", start_of_week).to_lowercase();

    if json_output || jq.is_some() {
        let weeks_json: Vec<serde_json::Value> = weeks
            .iter()
            .map(|week| {
                let week_rows = by_week.get(week).cloned().unwrap_or_default();
                let total_cost_nanos: i64 = week_rows.iter().map(|r| r.cost_nanos).sum();
                let total_input: i64 = week_rows.iter().map(|r| r.input_tokens).sum();
                let total_output: i64 = week_rows.iter().map(|r| r.output_tokens).sum();

                // Aggregate by provider.
                let mut prov_map: std::collections::HashMap<String, (i64, i64, i64)> =
                    std::collections::HashMap::new();
                for r in &week_rows {
                    let e = prov_map.entry(r.provider.clone()).or_default();
                    e.0 += r.input_tokens + r.output_tokens;
                    e.1 += r.turns;
                    e.2 += r.cost_nanos;
                }
                let by_provider: Vec<serde_json::Value> = {
                    let mut pv: Vec<_> = prov_map.into_iter().collect();
                    pv.sort_by_key(|b| std::cmp::Reverse(b.1.0));
                    pv.iter()
                        .map(|(prov, (tokens, turns, cost))| {
                            serde_json::json!({
                                "provider": prov,
                                "turns": turns,
                                "tokens": tokens,
                                "estimated_cost": *cost as f64 / 1_000_000_000.0,
                            })
                        })
                        .collect()
                };

                let models: Vec<serde_json::Value> = week_rows
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "model": r.model,
                            "provider": r.provider,
                            "turns": r.turns,
                            "input_tokens": r.input_tokens,
                            "output_tokens": r.output_tokens,
                            "cache_read_tokens": r.cache_read_tokens,
                            "cache_creation_tokens": r.cache_creation_tokens,
                            "reasoning_output_tokens": r.reasoning_output_tokens,
                            "estimated_cost": r.cost_nanos as f64 / 1_000_000_000.0,
                        })
                    })
                    .collect();

                serde_json::json!({
                    "week": week,
                    "models": models,
                    "by_provider": by_provider,
                    "total_input_tokens": total_input,
                    "total_output_tokens": total_output,
                    "total_estimated_cost": total_cost_nanos as f64 / 1_000_000_000.0,
                })
            })
            .collect();

        let out = serde_json::json!({
            "start_of_week": sow_str,
            "weeks": weeks_json,
        });
        if let Some(filter) = jq {
            apply_jq_and_print(&out, filter);
        } else {
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
    } else {
        println!("Weekly usage summary (start-of-week: {})\n", sow_str);
        for week in &weeks {
            let week_rows = by_week.get(week).cloned().unwrap_or_default();
            let total_cost_nanos: i64 = week_rows.iter().map(|r| r.cost_nanos).sum();
            let total_input: i64 = week_rows.iter().map(|r| r.input_tokens).sum();
            let total_output: i64 = week_rows.iter().map(|r| r.output_tokens).sum();
            let total_turns: i64 = week_rows.iter().map(|r| r.turns).sum();
            let week_label = locale::format_week_label(week, display_locale);

            println!(
                "Week {}  turns={}  in={}  out={}  cost={}",
                week_label,
                pricing::fmt_tokens(total_turns),
                pricing::fmt_tokens(total_input),
                pricing::fmt_tokens(total_output),
                pricing::fmt_cost(total_cost_nanos as f64 / 1_000_000_000.0)
            );

            if breakdown {
                for r in &week_rows {
                    if compact {
                        let model_trunc = if r.model.len() <= 20 {
                            r.model.as_str()
                        } else {
                            let end = r
                                .model
                                .char_indices()
                                .nth(20)
                                .map(|(i, _)| i)
                                .unwrap_or(r.model.len());
                            &r.model[..end]
                        };
                        println!(
                            "  \u{2514}\u{2500} {:<20}  in={}  out={}  cost={}",
                            model_trunc,
                            pricing::fmt_tokens(r.input_tokens),
                            pricing::fmt_tokens(r.output_tokens),
                            pricing::fmt_cost(r.cost_nanos as f64 / 1_000_000_000.0)
                        );
                    } else {
                        println!(
                            "  \u{2514}\u{2500} {:<40}  in={}  out={}  cost={}",
                            r.model,
                            pricing::fmt_tokens(r.input_tokens),
                            pricing::fmt_tokens(r.output_tokens),
                            pricing::fmt_cost(r.cost_nanos as f64 / 1_000_000_000.0)
                        );
                    }
                }
            }
        }
        println!();
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_stats(
    db_path: &std::path::Path,
    json_output: bool,
    breakdown: bool,
    display_currency: &str,
    jq: Option<&str>,
    _aliases: &HashMap<String, String>,
    display_locale: chrono::Locale,
    compact: bool,
) -> Result<()> {
    if !db_path.exists() {
        anyhow::bail!("Database not found. Run: claude-usage-tracker scan");
    }
    let conn = scanner::db::open_db(db_path)?;

    let (sessions, first, last) = scanner::db::query_stats_session_window(&conn)?;

    let (inp, out, cr, cc, ro, turns, total_credits_opt) =
        scanner::db::query_stats_token_totals(&conn)?;

    let by_model: Vec<StatsModelRow> = scanner::db::query_stats_by_model(&conn)?;

    let total_cost: f64 = by_model
        .iter()
        .map(|(_, _, _, _, _, _, _, _, _, cost_nanos, _, _)| *cost_nanos as f64 / 1_000_000_000.0)
        .sum();

    if json_output || jq.is_some() {
        let by_provider: Vec<serde_json::Value> = scanner::db::query_stats_by_provider(&conn)?
            .into_iter()
            .map(
                |(provider, sessions_n, turns_n, inp_n, out_n, cr_n, cc_n, ro_n, cost_nanos)| {
                    serde_json::json!({
                        "provider": provider,
                        "sessions": sessions_n,
                        "turns": turns_n,
                        "input_tokens": inp_n,
                        "output_tokens": out_n,
                        "cache_read_tokens": cr_n,
                        "cache_creation_tokens": cc_n,
                        "reasoning_output_tokens": ro_n,
                        "estimated_cost": cost_nanos as f64 / 1_000_000_000.0,
                    })
                },
            )
            .collect();
        let confidence_breakdown: Vec<serde_json::Value> =
            scanner::db::query_stats_confidence_breakdown(&conn)?
                .into_iter()
                .map(|(conf, turns_n, cost_nanos)| {
                    serde_json::json!({
                        "cost_confidence": conf,
                        "turns": turns_n,
                        "estimated_cost": cost_nanos as f64 / 1_000_000_000.0,
                    })
                })
                .collect();
        let billing_mode_breakdown: Vec<serde_json::Value> =
            scanner::db::query_stats_billing_mode_breakdown(&conn)?
                .into_iter()
                .map(|(mode, turns_n, cost_nanos)| {
                    serde_json::json!({
                        "billing_mode": mode,
                        "turns": turns_n,
                        "estimated_cost": cost_nanos as f64 / 1_000_000_000.0,
                    })
                })
                .collect();
        let models: Vec<serde_json::Value> = by_model
            .iter()
            .map(
                |(
                    provider,
                    model,
                    mi,
                    mo,
                    mcr,
                    mcc,
                    mro,
                    mt,
                    ms,
                    cost_nanos,
                    cost_confidence,
                    billing_mode,
                )| {
                    serde_json::json!({
                        "provider": provider, "model": model, "sessions": ms, "turns": mt,
                        "input_tokens": mi, "output_tokens": mo,
                        "cache_read_tokens": mcr, "cache_creation_tokens": mcc,
                        "reasoning_output_tokens": mro,
                        "estimated_cost": *cost_nanos as f64 / 1_000_000_000.0,
                        "cost_confidence": cost_confidence,
                        "billing_mode": billing_mode,
                    })
                },
            )
            .collect();
        // one_shot_rate: AVG(one_shot) across sessions where one_shot IS NOT NULL.
        // Returns None when no classifiable sessions exist (all NULL).
        let one_shot_rate: Option<f64> =
            scanner::db::query_stats_oneshot_avg(&conn).unwrap_or(None);

        let f = |s: &Option<String>| {
            s.as_deref()
                .unwrap_or("")
                .chars()
                .take(10)
                .collect::<String>()
        };

        // Build display_currency block — only present when currency != "USD".
        // No network calls during tests; uses convert_with_snapshot internally.
        let display_currency_value: Option<serde_json::Value> = if display_currency != "USD" {
            let result = currency::convert_from_usd(total_cost, display_currency);
            let age = result.source.age_hours();
            Some(serde_json::json!({
                "code": result.currency,
                "total_cost_display": result.amount,
                "rate_source": result.source.as_str(),
                "rate_age_hours": age,
            }))
        } else {
            None
        };

        let mut output = serde_json::json!({
            "period": { "from": f(&first), "to": f(&last) },
            "total_sessions": sessions,
            "total_turns": turns,
            "total_input_tokens": inp,
            "total_output_tokens": out,
            "total_cache_read_tokens": cr,
            "total_cache_creation_tokens": cc,
            "total_reasoning_output_tokens": ro,
            "total_estimated_cost": (total_cost * 10000.0).round() / 10000.0,
            "total_credits": total_credits_opt,
            "one_shot_rate": one_shot_rate,
            "by_provider": by_provider,
            "confidence_breakdown": confidence_breakdown,
            "billing_mode_breakdown": billing_mode_breakdown,
            "by_model": models,
        });
        if let Some(dc) = display_currency_value {
            output["display_currency"] = dc;
        }
        if let Some(filter) = jq {
            apply_jq_and_print(&output, filter);
        } else {
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        return Ok(());
    }

    if !compact
        && std::io::IsTerminal::is_terminal(&std::io::stdout())
        && std::env::var("COLUMNS")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .is_some_and(|c| c < 100)
    {
        eprintln!("(narrow terminal detected; try --compact)");
    }

    println!();
    println!("{}", "=".repeat(70));
    println!("  Usage - All-Time Statistics");
    println!("{}", "=".repeat(70));
    let fmt_period_date = |s: &Option<String>| -> String {
        let iso = s
            .as_deref()
            .unwrap_or("")
            .chars()
            .take(10)
            .collect::<String>();
        if let Ok(d) = chrono::NaiveDate::parse_from_str(&iso, "%Y-%m-%d") {
            locale::format_naive_date(d, display_locale)
        } else {
            iso
        }
    };
    println!(
        "  Period:           {} to {}",
        fmt_period_date(&first),
        fmt_period_date(&last)
    );
    println!("  Total sessions:   {}", sessions);
    println!("  Total turns:      {}", pricing::fmt_tokens(turns));
    println!();
    println!(
        "  Input tokens:     {:<12}  (raw prompt tokens)",
        pricing::fmt_tokens(inp)
    );
    println!(
        "  Output tokens:    {:<12}  (generated tokens)",
        pricing::fmt_tokens(out)
    );
    println!(
        "  Cached input:     {:<12}  (cheaper than input)",
        pricing::fmt_tokens(cr)
    );
    println!(
        "  Cache creation:   {:<12}  (premium on input)",
        pricing::fmt_tokens(cc)
    );
    println!(
        "  Reasoning output: {:<12}  (included in output totals)",
        pricing::fmt_tokens(ro)
    );
    println!();
    println!("  Est. total cost:  {}", pricing::fmt_cost(total_cost));
    println!("{}", "-".repeat(70));

    println!("  By Provider:");
    let mut by_provider: std::collections::BTreeMap<String, ProviderRollup> =
        std::collections::BTreeMap::new();
    let mut by_confidence: std::collections::BTreeMap<String, (i64, f64)> =
        std::collections::BTreeMap::new();
    let mut by_billing_mode: std::collections::BTreeMap<String, (i64, f64)> =
        std::collections::BTreeMap::new();
    for (
        provider,
        _model,
        mi,
        mo,
        mcr,
        mcc,
        mro,
        mt,
        _ms,
        cost_nanos,
        cost_confidence,
        billing_mode,
    ) in &by_model
    {
        let cost = *cost_nanos as f64 / 1_000_000_000.0;
        let entry = by_provider
            .entry(provider.clone())
            .or_insert((0, 0, 0, 0, 0, 0, 0));
        entry.0 += *mt;
        entry.1 += *mi;
        entry.2 += *mo;
        entry.3 += *mcr;
        entry.4 += *mcc;
        entry.5 += *mro;
        entry.6 += *cost_nanos;
        let confidence_entry = by_confidence
            .entry(cost_confidence.clone())
            .or_insert((0, 0.0));
        confidence_entry.0 += *mt;
        confidence_entry.1 += cost;
        let billing_entry = by_billing_mode
            .entry(billing_mode.clone())
            .or_insert((0, 0.0));
        billing_entry.0 += *mt;
        billing_entry.1 += cost;
    }
    for (
        provider,
        (turns, input, output, cache_read, cache_creation, reasoning_output, cost_nanos),
    ) in by_provider
    {
        println!(
            "    {:<8}  turns={:<6}  in={:<8}  out={:<8}  cached={:<8}  reasoning={:<8}  cost={}",
            provider,
            pricing::fmt_tokens(turns),
            pricing::fmt_tokens(input),
            pricing::fmt_tokens(output),
            pricing::fmt_tokens(cache_read),
            pricing::fmt_tokens(reasoning_output),
            pricing::fmt_cost(cost_nanos as f64 / 1_000_000_000.0)
        );
        if cache_creation > 0 {
            println!(
                "             cache_write={}",
                pricing::fmt_tokens(cache_creation)
            );
        }
    }
    println!("{}", "-".repeat(70));
    println!("  By Confidence:");
    for (confidence, (turns, cost)) in by_confidence {
        println!(
            "    {:<8}  turns={:<6}  cost={}",
            confidence,
            pricing::fmt_tokens(turns),
            pricing::fmt_cost(cost)
        );
    }
    println!("  By Billing Mode:");
    for (billing_mode, (turns, cost)) in by_billing_mode {
        println!(
            "    {:<18}  turns={:<6}  cost={}",
            billing_mode,
            pricing::fmt_tokens(turns),
            pricing::fmt_cost(cost)
        );
    }
    println!("{}", "-".repeat(70));
    println!("  By Model:");

    /// Truncate a string to at most `max_chars` characters at a char boundary.
    fn truncate_model_stats(s: &str, max_chars: usize) -> &str {
        if s.len() <= max_chars {
            s
        } else {
            let end = s
                .char_indices()
                .nth(max_chars)
                .map(|(i, _)| i)
                .unwrap_or(s.len());
            &s[..end]
        }
    }

    if breakdown {
        // Group by provider for breakdown rendering.
        let mut rows_by_provider: std::collections::BTreeMap<String, Vec<&StatsModelRow>> =
            std::collections::BTreeMap::new();
        for row in &by_model {
            rows_by_provider.entry(row.0.clone()).or_default().push(row);
        }
        for (provider, prov_rows) in &rows_by_provider {
            if prov_rows.len() == 1 {
                let (
                    _,
                    model,
                    mi,
                    mo,
                    _mcr,
                    _mcc,
                    _mro,
                    mt,
                    ms,
                    cost_nanos,
                    cost_confidence,
                    billing_mode,
                ) = prov_rows[0];
                if compact {
                    println!(
                        "    {:<8}  {:<20}  turns={:<6}  in={:<8}  out={:<8}  cost={}",
                        provider,
                        truncate_model_stats(model, 20),
                        pricing::fmt_tokens(*mt),
                        pricing::fmt_tokens(*mi),
                        pricing::fmt_tokens(*mo),
                        pricing::fmt_cost(*cost_nanos as f64 / 1_000_000_000.0),
                    );
                } else {
                    println!(
                        "    {:<8}  {:<30}  sessions={:<4}  turns={:<6}  in={:<8}  out={:<8}  cost={}  conf={}  mode={}",
                        provider,
                        model,
                        ms,
                        pricing::fmt_tokens(*mt),
                        pricing::fmt_tokens(*mi),
                        pricing::fmt_tokens(*mo),
                        pricing::fmt_cost(*cost_nanos as f64 / 1_000_000_000.0),
                        cost_confidence,
                        billing_mode
                    );
                }
            } else {
                let p_turns: i64 = prov_rows.iter().map(|r| r.7).sum();
                let p_inp: i64 = prov_rows.iter().map(|r| r.2).sum();
                let p_out: i64 = prov_rows.iter().map(|r| r.3).sum();
                let p_sessions: i64 = prov_rows.iter().map(|r| r.8).sum();
                let p_cost_nanos: i64 = prov_rows.iter().map(|r| r.9).sum();
                println!(
                    "    {:<8}  ({} models){:<21}  sessions={:<4}  turns={:<6}  in={:<8}  out={:<8}  cost={}",
                    provider,
                    prov_rows.len(),
                    "",
                    p_sessions,
                    pricing::fmt_tokens(p_turns),
                    pricing::fmt_tokens(p_inp),
                    pricing::fmt_tokens(p_out),
                    pricing::fmt_cost(p_cost_nanos as f64 / 1_000_000_000.0),
                );
                for (
                    _,
                    model,
                    mi,
                    mo,
                    _mcr,
                    _mcc,
                    _mro,
                    mt,
                    ms,
                    cost_nanos,
                    cost_confidence,
                    billing_mode,
                ) in prov_rows
                {
                    if compact {
                        println!(
                            "    \u{2514}\u{2500} {:<20}  turns={:<6}  in={:<8}  out={:<8}  cost={}",
                            truncate_model_stats(model, 20),
                            pricing::fmt_tokens(*mt),
                            pricing::fmt_tokens(*mi),
                            pricing::fmt_tokens(*mo),
                            pricing::fmt_cost(*cost_nanos as f64 / 1_000_000_000.0),
                        );
                    } else {
                        println!(
                            "    \u{2514}\u{2500} {:<28}  sessions={:<4}  turns={:<6}  in={:<8}  out={:<8}  cost={}  conf={}  mode={}",
                            model,
                            ms,
                            pricing::fmt_tokens(*mt),
                            pricing::fmt_tokens(*mi),
                            pricing::fmt_tokens(*mo),
                            pricing::fmt_cost(*cost_nanos as f64 / 1_000_000_000.0),
                            cost_confidence,
                            billing_mode
                        );
                    }
                }
            }
        }
    } else {
        for (
            provider,
            model,
            mi,
            mo,
            _mcr,
            _mcc,
            _mro,
            mt,
            ms,
            cost_nanos,
            cost_confidence,
            billing_mode,
        ) in &by_model
        {
            if compact {
                println!(
                    "    {:<8}  {:<20}  turns={:<6}  in={:<8}  out={:<8}  cost={}",
                    provider,
                    truncate_model_stats(model, 20),
                    pricing::fmt_tokens(*mt),
                    pricing::fmt_tokens(*mi),
                    pricing::fmt_tokens(*mo),
                    pricing::fmt_cost(*cost_nanos as f64 / 1_000_000_000.0),
                );
            } else {
                println!(
                    "    {:<8}  {:<30}  sessions={:<4}  turns={:<6}  in={:<8}  out={:<8}  cost={}  conf={}  mode={}",
                    provider,
                    model,
                    ms,
                    pricing::fmt_tokens(*mt),
                    pricing::fmt_tokens(*mi),
                    pricing::fmt_tokens(*mo),
                    pricing::fmt_cost(*cost_nanos as f64 / 1_000_000_000.0),
                    cost_confidence,
                    billing_mode
                );
            }
        }
    }

    println!("{}", "=".repeat(70));
    println!();
    Ok(())
}

// ── blocks subcommand ─────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn cmd_blocks(
    db_path: &std::path::Path,
    session_hours: f64,
    active_only: bool,
    json_output: bool,
    token_limit: Option<TokenLimit>,
    jq: Option<&str>,
    include_gaps: bool,
    display_locale: chrono::Locale,
) -> anyhow::Result<()> {
    use analytics::blocks::{calculate_burn_rate, identify_blocks_with_gaps, project_block_usage};
    use analytics::quota::compute_quota;

    anyhow::ensure!(
        session_hours > 0.0 && session_hours <= 168.0,
        "session-length must be between 0 and 168 hours"
    );

    let conn = scanner::db::open_db(db_path)?;
    let turns = scanner::db::load_all_turns(&conn)?;
    let now = chrono::Utc::now();
    let mut blocks = identify_blocks_with_gaps(&turns, session_hours, now, include_gaps);

    // Resolve the token limit once before rendering.
    let resolved_limit: Option<i64> = match &token_limit {
        None => None,
        Some(TokenLimit::Absolute(n)) => Some(*n),
        Some(TokenLimit::HistoricalMax) => Some(scanner::db::historical_max_block_tokens(
            &conn,
            session_hours,
        )?),
    };

    if active_only {
        blocks.retain(|b| b.is_active);
    }

    if json_output || jq.is_some() {
        let json_blocks: Vec<serde_json::Value> = blocks
            .iter()
            .map(|b| {
                let rate = if b.is_active {
                    calculate_burn_rate(b, now)
                } else {
                    None
                };
                let proj = project_block_usage(b, rate, now);

                let mut v = serde_json::json!({
                    "start": b.start.to_rfc3339(),
                    "end": b.end.to_rfc3339(),
                    "first_timestamp": b.first_timestamp.to_rfc3339(),
                    "last_timestamp": b.last_timestamp.to_rfc3339(),
                    "tokens": b.tokens,
                    "cost_nanos": b.cost_nanos,
                    "estimated_cost": b.cost_nanos as f64 / 1_000_000_000.0,
                    "models": b.models,
                    "is_active": b.is_active,
                    "is_gap": b.is_gap,
                    "kind": b.kind,
                    "entry_count": b.entry_count,
                });
                if b.is_active {
                    v["burn_rate"] = match rate {
                        Some(r) => serde_json::json!({
                            "tokens_per_min": r.tokens_per_min,
                            "cost_per_hour_nanos": r.cost_per_hour_nanos,
                            "cost_per_hour": r.cost_per_hour_nanos as f64 / 1_000_000_000.0,
                        }),
                        None => serde_json::Value::Null,
                    };
                    v["projection"] = serde_json::json!({
                        "projected_cost_nanos": proj.projected_cost_nanos,
                        "projected_cost": proj.projected_cost_nanos as f64 / 1_000_000_000.0,
                        "projected_tokens": proj.projected_tokens,
                    });
                }
                if let Some(limit) = resolved_limit
                    && let Some(quota) = compute_quota(b, &proj, limit)
                {
                    v["quota"] = serde_json::to_value(quota).unwrap_or(serde_json::Value::Null);
                }
                v
            })
            .collect();
        let json_blocks_value = serde_json::Value::Array(json_blocks);
        if let Some(filter) = jq {
            apply_jq_and_print(&json_blocks_value, filter);
        } else {
            println!("{}", serde_json::to_string_pretty(&json_blocks_value)?);
        }
        return Ok(());
    }

    // ── text output ───────────────────────────────────────────────────────────

    if blocks.is_empty() {
        println!("No billing blocks found.");
        return Ok(());
    }

    let col_w = 24usize;
    let is_tty = std::io::IsTerminal::is_terminal(&std::io::stdout());
    println!();
    println!("BLOCKS (session_length={session_hours:.1}h)");
    println!("{}", "-".repeat(120));
    println!(
        "  {:<col_w$}  {:<col_w$}  {:<10}  {:<12}  {:<10}  {:<30}  STATUS",
        "START", "END", "ELAPSED", "COST", "TOKENS", "MODELS"
    );
    println!("{}", "-".repeat(120));

    for block in &blocks {
        if block.is_gap {
            let gap_dur = block.end - block.start;
            let gap_h = gap_dur.num_hours();
            let gap_m = gap_dur.num_minutes() - gap_h * 60;
            let line = format!("  ({}h {}m gap)", gap_h, gap_m);
            if is_tty {
                println!("\x1b[2m{line}\x1b[0m");
            } else {
                println!("{line}");
            }
            continue;
        }

        let elapsed = now - block.start;
        let elapsed_h = elapsed.num_hours();
        let elapsed_m = elapsed.num_minutes() % 60;
        let elapsed_str = format!("{elapsed_h}h {elapsed_m:02}m");

        let cost_str = pricing::fmt_cost(block.cost_nanos as f64 / 1_000_000_000.0);
        let tokens_str = pricing::fmt_tokens(block.tokens.total());
        let models_str = block.models.join(", ");
        let status = if block.is_active { "ACTIVE" } else { "" };

        let start_display = locale::format_date(block.start, display_locale);
        let end_display = locale::format_date(block.end, display_locale);
        println!(
            "  {:<col_w$}  {:<col_w$}  {:<10}  {:<12}  {:<10}  {:<30}  {}",
            start_display, end_display, elapsed_str, cost_str, tokens_str, models_str, status,
        );

        if block.is_active {
            let rate = calculate_burn_rate(block, now);
            let proj = project_block_usage(block, rate, now);

            let remaining = (block.end - now).max(chrono::Duration::zero());
            let rem_h = remaining.num_hours();
            let rem_m = remaining.num_minutes() % 60;
            let proj_cost = pricing::fmt_cost(proj.projected_cost_nanos as f64 / 1_000_000_000.0);
            let proj_tok = pricing::fmt_tokens(proj.projected_tokens as i64);

            match rate {
                Some(r) => {
                    let cost_per_hr =
                        pricing::fmt_cost(r.cost_per_hour_nanos as f64 / 1_000_000_000.0);
                    let tok_per_min = pricing::fmt_tokens(r.tokens_per_min.round() as i64);
                    println!("      -> BURN RATE:  {tok_per_min} tok/min   {cost_per_hr}/hr");
                }
                None => {
                    println!("      -> BURN RATE:  n/a (single-entry block)");
                }
            }
            println!(
                "      -> PROJECTED:  {rem_h}h {rem_m:02}m remaining   {proj_cost} total   {proj_tok} tokens"
            );

            // Quota lines — only rendered for active blocks in text mode.
            if let Some(limit) = resolved_limit
                && let Some(quota) = compute_quota(block, &proj, limit)
            {
                let remaining_tok = pricing::fmt_tokens(quota.remaining_tokens.abs());
                let remaining_sign = if quota.remaining_tokens < 0 { "-" } else { "" };
                let used_tok = pricing::fmt_tokens(quota.used_tokens);
                let limit_tok = pricing::fmt_tokens(quota.limit_tokens);
                let proj_tok_q = pricing::fmt_tokens(quota.projected_tokens);
                let proj_pct_pct = (quota.projected_pct * 100.0).round() as i64;
                let used_pct_pct = (quota.current_pct * 100.0).round() as i64;
                let sev_str = |s: analytics::quota::Severity| match s {
                    analytics::quota::Severity::Ok => "[OK]",
                    analytics::quota::Severity::Warn => "[WARN]",
                    analytics::quota::Severity::Danger => "[CRIT]",
                };
                println!(
                    "    -> REMAINING:  {remaining_sign}{remaining_tok} tokens   {used_tok} used ({used_pct_pct}% of {limit_tok} limit)   {}",
                    sev_str(quota.current_severity)
                );
                println!(
                    "    -> PROJECTED:  {proj_tok_q} tokens   {proj_pct_pct}% of {limit_tok} limit   {}",
                    sev_str(quota.projected_severity)
                );
            }
        }
    }

    println!("{}", "-".repeat(120));
    println!();
    Ok(())
}

// ── scrape helpers ────────────────────────────────────────────────────────────

async fn scrape_claude_run(
    session_key: &str,
    cf_clearance: Option<String>,
    user_agent: &str,
    archive_root: &std::path::Path,
) -> anyhow::Result<scrape::ScrapeReport> {
    let creds = scrape::claude::Credentials {
        session_key: session_key.to_string(),
        cf_clearance,
        user_agent: user_agent.to_string(),
    };
    let client = scrape::claude::Client::new(&creds)?;
    let mut report = scrape::ScrapeReport {
        vendor: "claude.ai",
        listed: 0,
        written: 0,
        unchanged: 0,
        errors: Vec::new(),
    };
    let orgs = client.list_organizations().await?;
    for org in &orgs {
        let convs = client.list_conversations(&org.uuid).await?;
        for summary in &convs {
            report.listed += 1;
            match client.fetch_conversation(&org.uuid, &summary.uuid).await {
                Ok(payload) => {
                    let conv = archive::web::WebConversation {
                        vendor: "claude.ai".into(),
                        conversation_id: summary.uuid.clone(),
                        captured_at: chrono::Utc::now()
                            .format("%Y-%m-%dT%H%M%S%.6fZ")
                            .to_string(),
                        schema_fingerprint: "claude.ai/v1".into(),
                        payload,
                    };
                    match archive::web::write_web_conversation(archive_root, &conv) {
                        Ok(archive::web::WriteOutcome::Saved { .. }) => report.written += 1,
                        Ok(archive::web::WriteOutcome::Unchanged) => report.unchanged += 1,
                        Err(e) => report.errors.push(format!("{}: {e}", summary.uuid)),
                    }
                }
                Err(e) => report.errors.push(format!("{}: {e}", summary.uuid)),
            }
        }
    }
    Ok(report)
}

async fn scrape_chatgpt_run(
    session_token: &str,
    access_token: &str,
    cf_clearance: Option<String>,
    user_agent: &str,
    archive_root: &std::path::Path,
) -> anyhow::Result<scrape::ScrapeReport> {
    let creds = scrape::chatgpt::Credentials {
        session_token: session_token.to_string(),
        access_token: access_token.to_string(),
        cf_clearance,
        user_agent: user_agent.to_string(),
    };
    let client = scrape::chatgpt::Client::new(&creds)?;
    let mut report = scrape::ScrapeReport {
        vendor: "chatgpt.com",
        listed: 0,
        written: 0,
        unchanged: 0,
        errors: Vec::new(),
    };
    let convs = client.list_conversations(28).await?;
    for summary in &convs {
        report.listed += 1;
        match client.fetch_conversation(&summary.id).await {
            Ok(payload) => {
                let conv = archive::web::WebConversation {
                    vendor: "chatgpt.com".into(),
                    conversation_id: summary.id.clone(),
                    captured_at: chrono::Utc::now()
                        .format("%Y-%m-%dT%H%M%S%.6fZ")
                        .to_string(),
                    schema_fingerprint: "chatgpt.com/v1".into(),
                    payload,
                };
                match archive::web::write_web_conversation(archive_root, &conv) {
                    Ok(archive::web::WriteOutcome::Saved { .. }) => report.written += 1,
                    Ok(archive::web::WriteOutcome::Unchanged) => report.unchanged += 1,
                    Err(e) => report.errors.push(format!("{}: {e}", summary.id)),
                }
            }
            Err(e) => report.errors.push(format!("{}: {e}", summary.id)),
        }
    }
    Ok(report)
}

fn print_scrape_report(r: &scrape::ScrapeReport, json: bool) -> anyhow::Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "vendor": r.vendor,
                "listed": r.listed,
                "written": r.written,
                "unchanged": r.unchanged,
                "errors": r.errors,
            }))?
        );
    } else {
        println!(
            "{}: listed {}, wrote {} new, {} unchanged, {} errors",
            r.vendor,
            r.listed,
            r.written,
            r.unchanged,
            r.errors.len()
        );
        for e in &r.errors {
            eprintln!("  {e}");
        }
    }
    Ok(())
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use clap::Parser as _;

    use super::Cli;
    use super::Commands;
    use super::PricingAction;
    use super::TokenLimit;
    use super::UsageMonitorAction;
    use super::parse_token_limit;
    use super::parse_weekday;
    use super::weekday_to_u8;

    #[test]
    fn parse_token_limit_max_lowercase() {
        let tl = parse_token_limit("max").unwrap();
        assert!(matches!(tl, TokenLimit::HistoricalMax));
    }

    #[test]
    fn parse_token_limit_max_uppercase() {
        let tl = parse_token_limit("MAX").unwrap();
        assert!(matches!(tl, TokenLimit::HistoricalMax));
    }

    #[test]
    fn parse_token_limit_positive_integer() {
        let tl = parse_token_limit("1000000").unwrap();
        assert!(matches!(tl, TokenLimit::Absolute(1_000_000)));
    }

    #[test]
    fn parse_token_limit_bogus_returns_err() {
        assert!(parse_token_limit("bogus").is_err());
    }

    #[test]
    fn parse_token_limit_zero_returns_err() {
        assert!(parse_token_limit("0").is_err());
    }

    #[test]
    fn parse_token_limit_negative_returns_err() {
        assert!(parse_token_limit("-100").is_err());
    }

    #[test]
    fn parse_token_limit_max_mixed_case() {
        assert!(matches!(
            parse_token_limit("Max"),
            Ok(TokenLimit::HistoricalMax)
        ));
        assert!(matches!(
            parse_token_limit("mAx"),
            Ok(TokenLimit::HistoricalMax)
        ));
    }

    // ── parse_weekday tests ───────────────────────────────────────────────────

    #[test]
    fn parse_weekday_full_names() {
        assert_eq!(parse_weekday("monday").unwrap(), chrono::Weekday::Mon);
        assert_eq!(parse_weekday("tuesday").unwrap(), chrono::Weekday::Tue);
        assert_eq!(parse_weekday("wednesday").unwrap(), chrono::Weekday::Wed);
        assert_eq!(parse_weekday("thursday").unwrap(), chrono::Weekday::Thu);
        assert_eq!(parse_weekday("friday").unwrap(), chrono::Weekday::Fri);
        assert_eq!(parse_weekday("saturday").unwrap(), chrono::Weekday::Sat);
        assert_eq!(parse_weekday("sunday").unwrap(), chrono::Weekday::Sun);
    }

    #[test]
    fn parse_weekday_short_names() {
        assert_eq!(parse_weekday("mon").unwrap(), chrono::Weekday::Mon);
        assert_eq!(parse_weekday("fri").unwrap(), chrono::Weekday::Fri);
        assert_eq!(parse_weekday("sun").unwrap(), chrono::Weekday::Sun);
    }

    #[test]
    fn parse_weekday_case_insensitive() {
        assert_eq!(parse_weekday("MONDAY").unwrap(), chrono::Weekday::Mon);
        assert_eq!(parse_weekday("Friday").unwrap(), chrono::Weekday::Fri);
        assert_eq!(parse_weekday("SUN").unwrap(), chrono::Weekday::Sun);
    }

    #[test]
    fn cli_parses_usage_monitor_capture() {
        let cli = Cli::parse_from(["heimdall", "usage-monitor", "capture"]);
        match cli.command {
            Commands::UsageMonitor {
                action: UsageMonitorAction::Capture { db_path },
            } => {
                assert!(db_path.is_none());
            }
            _ => panic!("unexpected command"),
        }
    }

    #[test]
    fn cli_parses_usage_monitor_install_interval() {
        let cli = Cli::parse_from([
            "heimdall",
            "usage-monitor",
            "install",
            "--interval",
            "hourly",
        ]);
        match cli.command {
            Commands::UsageMonitor {
                action: UsageMonitorAction::Install { interval, db_path },
            } => {
                assert_eq!(interval.as_deref(), Some("hourly"));
                assert!(db_path.is_none());
            }
            _ => panic!("unexpected command"),
        }
    }

    #[test]
    fn cli_parses_pricing_sync() {
        let cli = Cli::parse_from(["heimdall", "pricing", "sync"]);
        match cli.command {
            Commands::Pricing {
                action: PricingAction::Sync { db_path },
            } => {
                assert!(db_path.is_none());
            }
            _ => panic!("unexpected command"),
        }
    }

    #[test]
    fn cli_parses_pricing_install_interval() {
        let cli = Cli::parse_from(["heimdall", "pricing", "install", "--interval", "daily"]);
        match cli.command {
            Commands::Pricing {
                action: PricingAction::Install { interval, db_path },
            } => {
                assert_eq!(interval, "daily");
                assert!(db_path.is_none());
            }
            _ => panic!("unexpected command"),
        }
    }

    #[test]
    fn parse_weekday_rejects_bogus() {
        assert!(parse_weekday("weekday").is_err());
        assert!(parse_weekday("").is_err());
        assert!(parse_weekday("0").is_err());
    }

    #[test]
    fn weekday_to_u8_mapping() {
        assert_eq!(weekday_to_u8(chrono::Weekday::Sun), 0);
        assert_eq!(weekday_to_u8(chrono::Weekday::Mon), 1);
        assert_eq!(weekday_to_u8(chrono::Weekday::Sat), 6);
    }

    #[test]
    fn dashboard_cli_flags_default_to_interactive_foreground() {
        let cli = Cli::parse_from(["claude-usage-tracker", "dashboard"]);
        match cli.command {
            Commands::Dashboard {
                watch,
                no_open,
                background_poll,
                ..
            } => {
                assert!(!watch);
                assert!(!no_open);
                assert!(!background_poll);
            }
            other => panic!(
                "expected dashboard command, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn dashboard_cli_accepts_background_flags() {
        let cli = Cli::parse_from([
            "claude-usage-tracker",
            "dashboard",
            "--watch",
            "--no-open",
            "--background-poll",
        ]);
        match cli.command {
            Commands::Dashboard {
                watch,
                no_open,
                background_poll,
                ..
            } => {
                assert!(watch);
                assert!(no_open);
                assert!(background_poll);
            }
            other => panic!(
                "expected dashboard command, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }
}
