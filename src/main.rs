use claude_usage_tracker::config;
use claude_usage_tracker::currency;
use claude_usage_tracker::db as db_mod;
use claude_usage_tracker::export;
use claude_usage_tracker::hook;
use claude_usage_tracker::litellm;
use claude_usage_tracker::menubar;
use claude_usage_tracker::optimizer;
use claude_usage_tracker::pricing;
use claude_usage_tracker::scanner;
use claude_usage_tracker::scheduler;
use claude_usage_tracker::server;

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
    },
    /// Show all-time statistics
    Stats {
        #[arg(long)]
        db_path: Option<PathBuf>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
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
        /// Output file path
        #[arg(long)]
        output: PathBuf,
        /// Restrict to a single provider (claude | codex | xcode | ...)
        #[arg(long)]
        provider: Option<String>,
        /// Restrict to a single project_name
        #[arg(long)]
        project: Option<String>,
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
    /// Analyse usage data and report waste findings (Phase 6)
    Optimize {
        #[arg(long)]
        db_path: Option<PathBuf>,
        /// Output format: text | json
        #[arg(long, default_value = "text")]
        format: String,
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
    },
    /// Remove the scheduled scan job
    Uninstall,
    /// Show the current scheduler status
    Status,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cfg = config::load_config();
    apply_pricing_overrides(&cfg);
    maybe_load_litellm(&cfg);

    // Extract config values before match (avoids partial move issues)
    let cfg_db = cfg.db_path;
    let cfg_dirs = cfg.projects_dirs;
    let cfg_host = cfg.host;
    let cfg_port = cfg.port;
    let cfg_oauth_enabled = cfg.oauth.enabled;
    let cfg_oauth_refresh = cfg.oauth.refresh_interval;
    let cfg_webhooks = cfg.webhooks;
    let cfg_openai_enabled = cfg.openai.enabled;
    let cfg_openai_admin_key_env = cfg.openai.admin_key_env;
    let cfg_openai_refresh_interval = cfg.openai.refresh_interval;
    let cfg_openai_lookback_days = cfg.openai.lookback_days;
    let cfg_display_currency = cfg.display.currency.unwrap_or_else(|| "USD".into());
    let cfg_agent_status = cfg.agent_status;
    let cfg_aggregator = cfg.aggregator;

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
        Commands::Today { db_path, json } => {
            let db = default_db(db_path);
            cmd_today(&db, json)?;
        }
        Commands::Stats { db_path, json } => {
            let db = default_db(db_path);
            cmd_stats(&db, json, &cfg_display_currency)?;
        }
        Commands::Dashboard {
            projects_dir,
            db_path,
            host,
            port,
            watch,
        } => {
            let db = default_db(db_path);
            let dirs = default_dirs(projects_dir);

            eprintln!("Running scan first...");
            scanner::scan(dirs.clone(), &db, true)?;

            let host_env = std::env::var("HOST")
                .ok()
                .or_else(|| cfg_host.clone())
                .unwrap_or(host);
            let port_env = std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .or(cfg_port)
                .unwrap_or(port);

            let url = format!("http://{}:{}", host_env, port_env);
            let _ = open::that(&url);

            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(server::serve(server::ServeOptions {
                host: host_env,
                port: port_env,
                db_path: db,
                projects_dirs: dirs,
                oauth_enabled: cfg_oauth_enabled,
                oauth_refresh_interval: cfg_oauth_refresh,
                openai_enabled: cfg_openai_enabled,
                openai_admin_key_env: cfg_openai_admin_key_env,
                openai_refresh_interval: cfg_openai_refresh_interval,
                openai_lookback_days: cfg_openai_lookback_days,
                webhook_config: cfg_webhooks,
                watch,
                agent_status_config: cfg_agent_status,
                aggregator_config: cfg_aggregator,
            }))?;
        }
        Commands::Export {
            db_path,
            format,
            period,
            output,
            provider,
            project,
        } => {
            let db = default_db(db_path);
            let opts = export::ExportOptions {
                format: format.parse()?,
                period: period.parse()?,
                output,
                provider,
                project,
            };
            let n = export::run_export(&db, &opts)?;
            eprintln!("Exported {} rows to {}", n, opts.output.display());
        }
        Commands::Menubar { db_path } => {
            let db = default_db(db_path);
            let output = menubar::run_menubar(&db)?;
            print!("{}", output);
        }
        Commands::Pricing {
            action: PricingAction::Refresh { cache_path },
        } => {
            let path = cache_path.unwrap_or_else(litellm::cache_path);
            match litellm::run_refresh(&path) {
                Ok((count, written)) => {
                    println!("Fetched {} models, cached at {}", count, written.display());
                }
                Err(reason) => {
                    eprintln!("Refresh failed: {}", reason);
                    std::process::exit(1);
                }
            }
        }
        Commands::Scheduler { action } => {
            cmd_scheduler(action, &default_db(None))?;
        }
        Commands::Optimize { db_path, format } => {
            let db = default_db(db_path);
            cmd_optimize(&db, &format)?;
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

fn cmd_optimize(db_path: &std::path::Path, format: &str) -> Result<()> {
    use optimizer::Severity;

    if !db_path.exists() {
        anyhow::bail!(
            "Database not found at {}. Run: claude-usage-tracker scan",
            db_path.display()
        );
    }

    let report = optimizer::run_optimize(db_path)?;

    match format.to_ascii_lowercase().as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&report)?);
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
        SchedulerAction::Install { interval, db_path } => {
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
        }
        SchedulerAction::Uninstall => {
            sched.uninstall()?;
            println!("Uninstalled: scheduled scan job removed");
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
                HookActionResult::AlreadyInstalled { binary_path } => {
                    println!("Already installed (no change made)");
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

#[cfg(test)]
mod cli_tests;

type TodayModelRow = (
    String,
    String,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    String,
    String,
);
type StatsModelRow = (
    String,
    String,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    i64,
    String,
    String,
);
type ProviderRollup = (i64, i64, i64, i64, i64, i64, i64);

fn cmd_today(db_path: &std::path::Path, json_output: bool) -> Result<()> {
    if !db_path.exists() {
        anyhow::bail!("Database not found. Run: claude-usage-tracker scan");
    }
    let conn = scanner::db::open_db(db_path)?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut stmt = conn.prepare(
        "SELECT provider, COALESCE(model, 'unknown') as model,
                SUM(input_tokens) as inp, SUM(output_tokens) as out,
                SUM(cache_read_tokens) as cr, SUM(cache_creation_tokens) as cc,
                SUM(reasoning_output_tokens) as ro,
                COUNT(*) as turns,
                COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos,
                CASE
                    WHEN SUM(CASE WHEN cost_confidence = 'low' THEN 1 ELSE 0 END) > 0 THEN 'low'
                    WHEN SUM(CASE WHEN cost_confidence = 'medium' THEN 1 ELSE 0 END) > 0 THEN 'medium'
                    ELSE 'high'
                END as cost_confidence,
                CASE
                    WHEN COUNT(DISTINCT billing_mode) = 1 THEN MAX(billing_mode)
                    ELSE 'mixed'
                END as billing_mode
         FROM turns WHERE substr(timestamp, 1, 10) = ?1
         GROUP BY provider, model ORDER BY inp + out DESC",
    )?;

    let rows: Vec<TodayModelRow> = stmt
        .query_map([&today], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
                row.get(10)?,
            ))
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                tracing::warn!("Failed to read row: {}", e);
                None
            }
        })
        .collect();

    if json_output {
        let by_provider: Vec<serde_json::Value> = {
            let mut stmt = conn.prepare(
                "SELECT provider, COUNT(*) as turns,
                        COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0),
                        COALESCE(SUM(cache_read_tokens), 0), COALESCE(SUM(cache_creation_tokens), 0),
                        COALESCE(SUM(reasoning_output_tokens), 0),
                        COALESCE(SUM(estimated_cost_nanos), 0)
                 FROM turns
                 WHERE substr(timestamp, 1, 10) = ?1
                 GROUP BY provider
                 ORDER BY turns DESC",
            )?;
            stmt.query_map([&today], |row| {
                let provider: String = row.get(0)?;
                Ok(serde_json::json!({
                    "provider": provider,
                    "turns": row.get::<_, i64>(1)?,
                    "input_tokens": row.get::<_, i64>(2)?,
                    "output_tokens": row.get::<_, i64>(3)?,
                    "cache_read_tokens": row.get::<_, i64>(4)?,
                    "cache_creation_tokens": row.get::<_, i64>(5)?,
                    "reasoning_output_tokens": row.get::<_, i64>(6)?,
                    "estimated_cost": row.get::<_, i64>(7)? as f64 / 1_000_000_000.0,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect()
        };
        let confidence_breakdown: Vec<serde_json::Value> = {
            let mut stmt = conn.prepare(
                "SELECT COALESCE(cost_confidence, 'low') as cost_confidence,
                        COUNT(*) as turns,
                        COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
                 FROM turns
                 WHERE substr(timestamp, 1, 10) = ?1
                 GROUP BY cost_confidence
                 ORDER BY turns DESC",
            )?;
            stmt.query_map([&today], |row| {
                Ok(serde_json::json!({
                    "cost_confidence": row.get::<_, String>(0)?,
                    "turns": row.get::<_, i64>(1)?,
                    "estimated_cost": row.get::<_, i64>(2)? as f64 / 1_000_000_000.0,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect()
        };
        let billing_mode_breakdown: Vec<serde_json::Value> = {
            let mut stmt = conn.prepare(
                "SELECT COALESCE(billing_mode, 'estimated_local') as billing_mode,
                        COUNT(*) as turns,
                        COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
                 FROM turns
                 WHERE substr(timestamp, 1, 10) = ?1
                 GROUP BY billing_mode
                 ORDER BY turns DESC",
            )?;
            stmt.query_map([&today], |row| {
                Ok(serde_json::json!({
                    "billing_mode": row.get::<_, String>(0)?,
                    "turns": row.get::<_, i64>(1)?,
                    "estimated_cost": row.get::<_, i64>(2)? as f64 / 1_000_000_000.0,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect()
        };
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
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    println!();
    println!("{}", "-".repeat(70));
    println!("  Today's Usage  ({})", today);
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
    for (
        provider,
        model,
        inp,
        out,
        cr,
        cc,
        _ro,
        turns,
        cost_nanos,
        cost_confidence,
        billing_mode,
    ) in &rows
    {
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

    println!("{}", "-".repeat(70));
    println!("  Est. total cost: {}", pricing::fmt_cost(total_cost));
    println!("  By Provider:");
    for (provider, (turns, input, output, cache_read, cache_creation, cost)) in provider_totals {
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
    println!();
    Ok(())
}

fn cmd_stats(db_path: &std::path::Path, json_output: bool, display_currency: &str) -> Result<()> {
    if !db_path.exists() {
        anyhow::bail!("Database not found. Run: claude-usage-tracker scan");
    }
    let conn = scanner::db::open_db(db_path)?;

    let (sessions, first, last): (i64, Option<String>, Option<String>) = conn.query_row(
        "SELECT COUNT(*), MIN(first_timestamp), MAX(last_timestamp) FROM sessions",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

    let (inp, out, cr, cc, ro, turns): (i64, i64, i64, i64, i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(input_tokens),0), COALESCE(SUM(output_tokens),0),
                COALESCE(SUM(cache_read_tokens),0), COALESCE(SUM(cache_creation_tokens),0),
                COALESCE(SUM(reasoning_output_tokens),0), COUNT(*) FROM turns",
        [],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        },
    )?;

    let mut stmt = conn.prepare(
        "SELECT provider, COALESCE(model,'unknown'), SUM(input_tokens), SUM(output_tokens),
                SUM(cache_read_tokens), SUM(cache_creation_tokens), SUM(reasoning_output_tokens), COUNT(*),
                COUNT(DISTINCT session_id), COALESCE(SUM(estimated_cost_nanos), 0),
                CASE
                    WHEN SUM(CASE WHEN cost_confidence = 'low' THEN 1 ELSE 0 END) > 0 THEN 'low'
                    WHEN SUM(CASE WHEN cost_confidence = 'medium' THEN 1 ELSE 0 END) > 0 THEN 'medium'
                    ELSE 'high'
                END as cost_confidence,
                CASE
                    WHEN COUNT(DISTINCT billing_mode) = 1 THEN MAX(billing_mode)
                    ELSE 'mixed'
                END as billing_mode
         FROM turns GROUP BY provider, model ORDER BY SUM(input_tokens+output_tokens) DESC",
    )?;
    let by_model: Vec<StatsModelRow> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
                row.get(8)?,
                row.get(9)?,
                row.get(10)?,
                row.get(11)?,
            ))
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => {
                tracing::warn!("Failed to read row: {}", e);
                None
            }
        })
        .collect();

    let total_cost: f64 = by_model
        .iter()
        .map(|(_, _, _, _, _, _, _, _, _, cost_nanos, _, _)| *cost_nanos as f64 / 1_000_000_000.0)
        .sum();

    if json_output {
        let by_provider: Vec<serde_json::Value> = {
            let mut stmt = conn.prepare(
                "SELECT provider,
                        COUNT(DISTINCT session_id), COUNT(*),
                        COALESCE(SUM(input_tokens),0), COALESCE(SUM(output_tokens),0),
                        COALESCE(SUM(cache_read_tokens),0), COALESCE(SUM(cache_creation_tokens),0),
                        COALESCE(SUM(reasoning_output_tokens),0), COALESCE(SUM(estimated_cost_nanos),0)
                 FROM turns
                 GROUP BY provider
                 ORDER BY COUNT(*) DESC",
            )?;
            stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "provider": row.get::<_, String>(0)?,
                    "sessions": row.get::<_, i64>(1)?,
                    "turns": row.get::<_, i64>(2)?,
                    "input_tokens": row.get::<_, i64>(3)?,
                    "output_tokens": row.get::<_, i64>(4)?,
                    "cache_read_tokens": row.get::<_, i64>(5)?,
                    "cache_creation_tokens": row.get::<_, i64>(6)?,
                    "reasoning_output_tokens": row.get::<_, i64>(7)?,
                    "estimated_cost": row.get::<_, i64>(8)? as f64 / 1_000_000_000.0,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect()
        };
        let confidence_breakdown: Vec<serde_json::Value> = {
            let mut stmt = conn.prepare(
                "SELECT COALESCE(cost_confidence, 'low') as cost_confidence,
                        COUNT(*) as turns,
                        COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
                 FROM turns
                 GROUP BY cost_confidence
                 ORDER BY turns DESC",
            )?;
            stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "cost_confidence": row.get::<_, String>(0)?,
                    "turns": row.get::<_, i64>(1)?,
                    "estimated_cost": row.get::<_, i64>(2)? as f64 / 1_000_000_000.0,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect()
        };
        let billing_mode_breakdown: Vec<serde_json::Value> = {
            let mut stmt = conn.prepare(
                "SELECT COALESCE(billing_mode, 'estimated_local') as billing_mode,
                        COUNT(*) as turns,
                        COALESCE(SUM(estimated_cost_nanos), 0) as cost_nanos
                 FROM turns
                 GROUP BY billing_mode
                 ORDER BY turns DESC",
            )?;
            stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "billing_mode": row.get::<_, String>(0)?,
                    "turns": row.get::<_, i64>(1)?,
                    "estimated_cost": row.get::<_, i64>(2)? as f64 / 1_000_000_000.0,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect()
        };
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
        let one_shot_rate: Option<f64> = conn
            .query_row(
                "SELECT AVG(CAST(one_shot AS REAL)) FROM sessions WHERE one_shot IS NOT NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or(None);

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
            "one_shot_rate": one_shot_rate,
            "by_provider": by_provider,
            "confidence_breakdown": confidence_breakdown,
            "billing_mode_breakdown": billing_mode_breakdown,
            "by_model": models,
        });
        if let Some(dc) = display_currency_value {
            output["display_currency"] = dc;
        }
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    println!();
    println!("{}", "=".repeat(70));
    println!("  Usage - All-Time Statistics");
    println!("{}", "=".repeat(70));
    let f = |s: &Option<String>| {
        s.as_deref()
            .unwrap_or("")
            .chars()
            .take(10)
            .collect::<String>()
    };
    println!("  Period:           {} to {}", f(&first), f(&last));
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
    println!("{}", "=".repeat(70));
    println!();
    Ok(())
}
