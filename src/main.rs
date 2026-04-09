mod config;
mod models;
mod oauth;
mod pricing;
mod scanner;
mod server;
mod webhooks;

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "claude-usage-tracker",
    version,
    about = "Local analytics dashboard for Claude Code usage"
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
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cfg = config::load_config();
    apply_pricing_overrides(&cfg);

    // Extract config values before match (avoids partial move issues)
    let cfg_db = cfg.db_path;
    let cfg_dirs = cfg.projects_dirs;
    let cfg_host = cfg.host;
    let cfg_port = cfg.port;
    let cfg_oauth_enabled = cfg.oauth.enabled;
    let cfg_oauth_refresh = cfg.oauth.refresh_interval;
    let cfg_webhooks = cfg.webhooks;

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
            cmd_stats(&db, json)?;
        }
        Commands::Dashboard {
            projects_dir,
            db_path,
            host,
            port,
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
            rt.block_on(server::serve(
                host_env,
                port_env,
                db,
                dirs,
                cfg_oauth_enabled,
                cfg_oauth_refresh,
                cfg_webhooks,
            ))?;
        }
    }
    Ok(())
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

#[cfg(test)]
mod cli_tests;

fn cmd_today(db_path: &std::path::Path, json_output: bool) -> Result<()> {
    if !db_path.exists() {
        anyhow::bail!("Database not found. Run: claude-usage-tracker scan");
    }
    let conn = scanner::db::open_db(db_path)?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut stmt = conn.prepare(
        "SELECT COALESCE(model, 'unknown') as model,
                SUM(input_tokens) as inp, SUM(output_tokens) as out,
                SUM(cache_read_tokens) as cr, SUM(cache_creation_tokens) as cc,
                COUNT(*) as turns
         FROM turns WHERE substr(timestamp, 1, 10) = ?1
         GROUP BY model ORDER BY inp + out DESC",
    )?;

    let rows: Vec<(String, i64, i64, i64, i64, i64)> = stmt
        .query_map([&today], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => { tracing::warn!("Failed to read row: {}", e); None }
        })
        .collect();

    if json_output {
        let models: Vec<serde_json::Value> = rows
            .iter()
            .map(|(model, inp, out, cr, cc, turns)| {
                let cost = pricing::calc_cost(model, *inp, *out, *cr, *cc);
                serde_json::json!({
                    "model": model, "turns": turns,
                    "input_tokens": inp, "output_tokens": out,
                    "cache_read_tokens": cr, "cache_creation_tokens": cc,
                    "estimated_cost": (cost * 10000.0).round() / 10000.0,
                })
            })
            .collect();
        let total_cost: f64 = rows
            .iter()
            .map(|(m, i, o, cr, cc, _)| pricing::calc_cost(m, *i, *o, *cr, *cc))
            .sum();
        let output = serde_json::json!({
            "date": today,
            "models": models,
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
    for (model, inp, out, cr, cc, turns) in &rows {
        let cost = pricing::calc_cost(model, *inp, *out, *cr, *cc);
        total_cost += cost;
        println!(
            "  {:<30}  turns={:<4}  in={:<8}  out={:<8}  cost={}",
            model,
            turns,
            pricing::fmt_tokens(*inp),
            pricing::fmt_tokens(*out),
            pricing::fmt_cost(cost)
        );
    }

    println!("{}", "-".repeat(70));
    println!("  Est. total cost: {}", pricing::fmt_cost(total_cost));
    println!();
    Ok(())
}

fn cmd_stats(db_path: &std::path::Path, json_output: bool) -> Result<()> {
    if !db_path.exists() {
        anyhow::bail!("Database not found. Run: claude-usage-tracker scan");
    }
    let conn = scanner::db::open_db(db_path)?;

    let (sessions, first, last): (i64, Option<String>, Option<String>) = conn.query_row(
        "SELECT COUNT(*), MIN(first_timestamp), MAX(last_timestamp) FROM sessions",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;

    let (inp, out, cr, cc, turns): (i64, i64, i64, i64, i64) = conn.query_row(
        "SELECT COALESCE(SUM(input_tokens),0), COALESCE(SUM(output_tokens),0),
                COALESCE(SUM(cache_read_tokens),0), COALESCE(SUM(cache_creation_tokens),0),
                COUNT(*) FROM turns",
        [],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        },
    )?;

    let mut stmt = conn.prepare(
        "SELECT COALESCE(model,'unknown'), SUM(input_tokens), SUM(output_tokens),
                SUM(cache_read_tokens), SUM(cache_creation_tokens), COUNT(*),
                COUNT(DISTINCT session_id)
         FROM turns GROUP BY model ORDER BY SUM(input_tokens+output_tokens) DESC",
    )?;
    let by_model: Vec<(String, i64, i64, i64, i64, i64, i64)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        })?
        .filter_map(|r| match r {
            Ok(val) => Some(val),
            Err(e) => { tracing::warn!("Failed to read row: {}", e); None }
        })
        .collect();

    let total_cost: f64 = by_model
        .iter()
        .map(|(m, i, o, cr, cc, _, _)| pricing::calc_cost(m, *i, *o, *cr, *cc))
        .sum();

    if json_output {
        let models: Vec<serde_json::Value> = by_model
            .iter()
            .map(|(model, mi, mo, mcr, mcc, mt, ms)| {
                let cost = pricing::calc_cost(model, *mi, *mo, *mcr, *mcc);
                serde_json::json!({
                    "model": model, "sessions": ms, "turns": mt,
                    "input_tokens": mi, "output_tokens": mo,
                    "cache_read_tokens": mcr, "cache_creation_tokens": mcc,
                    "estimated_cost": (cost * 10000.0).round() / 10000.0,
                })
            })
            .collect();
        let f = |s: &Option<String>| {
            s.as_deref()
                .unwrap_or("")
                .chars()
                .take(10)
                .collect::<String>()
        };
        let output = serde_json::json!({
            "period": { "from": f(&first), "to": f(&last) },
            "total_sessions": sessions,
            "total_turns": turns,
            "total_input_tokens": inp,
            "total_output_tokens": out,
            "total_cache_read_tokens": cr,
            "total_cache_creation_tokens": cc,
            "total_estimated_cost": (total_cost * 10000.0).round() / 10000.0,
            "by_model": models,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    println!();
    println!("{}", "=".repeat(70));
    println!("  Claude Code Usage - All-Time Statistics");
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
        "  Cache read:       {:<12}  (cheaper than input)",
        pricing::fmt_tokens(cr)
    );
    println!(
        "  Cache creation:   {:<12}  (premium on input)",
        pricing::fmt_tokens(cc)
    );
    println!();
    println!("  Est. total cost:  {}", pricing::fmt_cost(total_cost));
    println!("{}", "-".repeat(70));

    println!("  By Model:");
    for (model, mi, mo, mcr, mcc, mt, ms) in &by_model {
        let cost = pricing::calc_cost(model, *mi, *mo, *mcr, *mcc);
        println!(
            "    {:<30}  sessions={:<4}  turns={:<6}  in={:<8}  out={:<8}  cost={}",
            model,
            ms,
            pricing::fmt_tokens(*mt),
            pricing::fmt_tokens(*mi),
            pricing::fmt_tokens(*mo),
            pricing::fmt_cost(cost)
        );
    }
    println!("{}", "=".repeat(70));
    println!();
    Ok(())
}
