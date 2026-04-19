/// `statusline` subcommand — PostToolUse hook that emits a compact status line.
///
/// Flow:
/// 1. Parse stdin JSON (5 s timeout).
/// 2. Warm cache hit → print cached line, exit 0.
/// 3. Acquire PID lock (1 s timeout). On failure → print "..." and exit 0.
/// 4. Compute stats from DB.
/// 5. Render line.
/// 6. Write cache, release lock.
/// 7. Print line.
///
/// Exit 0 in every code path — never block Claude Code.
pub mod cache;
pub mod compute;
pub mod context_window;
pub mod input;
pub mod install;
pub mod render;

use std::path::{Path, PathBuf};
use std::time::Duration;

use tracing::{debug, warn};

use crate::config::load_config_resolved;
use crate::scanner::default_db_path;
use crate::statusline::cache::{
    CacheEntry, acquire_lock, is_fresh, read_cache, transcript_mtime, write_cache,
};
use crate::statusline::compute::{CostSource, compute};
use crate::statusline::input::{HookInput, parse_stdin_with_timeout};
use crate::statusline::render::render_status_line_with_opts;

pub use compute::CostSource as StatuslineCostSource;
pub use install::{
    StatuslineActionResult, StatuslineStatus, install, install_into, status, status_from,
    uninstall, uninstall_from,
};

// ── Visual burn-rate style ────────────────────────────────────────────────────

/// Controls how the burn-rate tier is rendered in the statusline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualBurnRate {
    /// No tier indicator rendered.
    Off,
    /// Bracketed text only: `[NORMAL]` / `[WARN]` / `[CRIT]`.
    Bracket,
    /// Emoji only: `🟢` / `⚠️` / `🚨`.
    Emoji,
    /// Both emoji and bracket: `🟢 [NORMAL]` / `⚠️ [WARN]` / `🚨 [CRIT]`.
    Both,
}

impl VisualBurnRate {
    /// Parse from a CLI string value.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "off" => Ok(Self::Off),
            "bracket" => Ok(Self::Bracket),
            "emoji" => Ok(Self::Emoji),
            "both" => Ok(Self::Both),
            other => Err(format!(
                "invalid visual-burn-rate '{}': expected one of off, bracket, emoji, both",
                other
            )),
        }
    }
}

// ── Options ───────────────────────────────────────────────────────────────────

pub struct StatuslineOpts {
    pub refresh_interval: u64,
    pub cost_source: CostSource,
    pub offline: bool,
    pub db_path: Option<PathBuf>,
    pub context_low_threshold: f64,
    pub context_medium_threshold: f64,
    pub burn_rate_normal_max: f64,
    pub burn_rate_moderate_max: f64,
    pub visual_burn_rate: VisualBurnRate,
}

// ── Entry point ───────────────────────────────────────────────────────────────

/// Run the statusline subcommand. Always exits cleanly; never panics.
pub fn run(opts: &StatuslineOpts) {
    match run_inner(opts) {
        Ok(line) => {
            println!("{}", line);
        }
        Err(e) => {
            // Emit detail to stderr/tracing so operators can diagnose without
            // leaking paths or internal state to Claude Code's status bar.
            warn!("statusline error: {:#}", e);
            // Print a sanitized sentinel — never expose internal error text to
            // the status bar where it could leak paths or sensitive info.
            println!("heimdall: [error]");
        }
    }
}

fn run_inner(opts: &StatuslineOpts) -> anyhow::Result<String> {
    let stdin_timeout = Duration::from_secs(5);
    let input = parse_stdin_with_timeout(stdin_timeout)?;
    run_inner_with_input(opts, &input, Duration::from_secs(1))
}

fn run_inner_with_input(
    opts: &StatuslineOpts,
    input: &HookInput,
    lock_timeout: Duration,
) -> anyhow::Result<String> {
    let ttl = Duration::from_secs(opts.refresh_interval);

    // NOTE: --offline is a forward-compat contract.  The current compute()
    // path is already network-free (no LiteLLM fetch, no OAuth call).  The
    // flag is honored trivially — it is a no-op today but will gate any future
    // network call added to this code path.
    debug!(
        offline = opts.offline,
        "statusline compute offline={}", opts.offline
    );

    let transcript_path = Path::new(&input.transcript_path);
    let current_mtime = transcript_mtime(transcript_path).unwrap_or(0);

    // 2. Warm cache check.
    if let Some(entry) = read_cache()
        && is_fresh(&entry, &input.session_id, transcript_path, ttl)
    {
        return Ok(entry.output);
    }

    // 3. Acquire lock — on failure print a stub rather than blocking.
    let _lock = match acquire_lock(lock_timeout) {
        Ok(g) => g,
        Err(_) => {
            return Ok("...".to_string());
        }
    };

    // 4. Compute stats.
    let db_path = resolve_db_path(opts.db_path.as_deref());
    let stats = compute(&db_path, input, opts.cost_source)?;

    // 5. Render.
    let render_opts = crate::statusline::render::RenderOpts {
        context_low_threshold: opts.context_low_threshold,
        context_medium_threshold: opts.context_medium_threshold,
        burn_rate: crate::analytics::burn_rate::BurnRateConfig {
            normal_max: opts.burn_rate_normal_max,
            moderate_max: opts.burn_rate_moderate_max,
        },
        visual_burn_rate: opts.visual_burn_rate,
        cost_source: opts.cost_source,
    };
    let line = render_status_line_with_opts(&stats, &render_opts);

    // 6. Write cache.
    let entry = CacheEntry {
        session_id: input.session_id.clone(),
        computed_at: chrono::Utc::now(),
        transcript_mtime_secs: current_mtime,
        output: line.clone(),
    };
    if let Err(e) = write_cache(&entry) {
        warn!("statusline: failed to write cache: {}", e);
    }

    Ok(line)
}

fn resolve_db_path(cli_db: Option<&Path>) -> PathBuf {
    if let Some(p) = cli_db {
        return p.to_path_buf();
    }
    let cfg = load_config_resolved();
    cfg.db_path.unwrap_or_else(default_db_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::db::{init_db, open_db};
    use crate::statusline::input::{ContextWindow, HookCost};
    use std::fs;
    use std::sync::{Mutex, MutexGuard};
    use tempfile::TempDir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct CacheEnvGuard {
        _lock: MutexGuard<'static, ()>,
    }

    impl CacheEnvGuard {
        fn new(path: &Path) -> Self {
            let lock = match ENV_LOCK.lock() {
                Ok(lock) => lock,
                Err(poisoned) => poisoned.into_inner(),
            };
            // SAFETY: tests serialize all XDG_CACHE_HOME mutation through a
            // process-wide mutex, so no concurrent environment access occurs.
            unsafe {
                std::env::set_var("XDG_CACHE_HOME", path);
            }
            Self { _lock: lock }
        }
    }

    impl Drop for CacheEnvGuard {
        fn drop(&mut self) {
            // SAFETY: paired with `CacheEnvGuard::new`, still under the same
            // process-wide mutex while the guard is alive.
            unsafe {
                std::env::remove_var("XDG_CACHE_HOME");
            }
        }
    }

    fn make_opts(db_path: PathBuf, refresh_interval: u64) -> StatuslineOpts {
        StatuslineOpts {
            refresh_interval,
            cost_source: CostSource::Auto,
            offline: true,
            db_path: Some(db_path),
            context_low_threshold: 0.5,
            context_medium_threshold: 0.8,
            burn_rate_normal_max: 100.0,
            burn_rate_moderate_max: 250.0,
            visual_burn_rate: VisualBurnRate::Bracket,
        }
    }

    fn make_input(transcript_path: &Path) -> HookInput {
        HookInput {
            session_id: "session-1".to_string(),
            transcript_path: transcript_path.display().to_string(),
            model: Some("claude-sonnet-4-6".to_string()),
            cost: Some(HookCost {
                total_cost_usd: 0.25,
                total_duration_ms: None,
                total_api_duration_ms: None,
            }),
            context_window: Some(ContextWindow {
                total_input_tokens: Some(45_231),
                context_window_size: Some(200_000),
            }),
        }
    }

    fn init_test_db(dir: &TempDir) -> PathBuf {
        let db_path = dir.path().join("heimdall.db");
        let conn = open_db(&db_path).expect("open db");
        init_db(&conn).expect("init db");
        drop(conn);
        db_path
    }

    #[test]
    fn run_inner_writes_cache_and_reuses_fresh_output_before_lock() {
        let cache_dir = TempDir::new().expect("cache dir");
        let _cache_env = CacheEnvGuard::new(cache_dir.path());
        let work_dir = TempDir::new().expect("work dir");
        let db_path = init_test_db(&work_dir);
        let transcript_path = work_dir.path().join("transcript.jsonl");
        fs::write(&transcript_path, "{\"type\":\"assistant\"}\n").expect("write transcript");

        let opts = make_opts(db_path, 60);
        let input = make_input(&transcript_path);

        let first =
            run_inner_with_input(&opts, &input, Duration::from_millis(20)).expect("first run");
        assert!(first.contains("claude-sonnet-4-6"));
        assert!(first.contains("$0.2500"));
        assert!(first.contains("45.2K tokens (23%)"));

        let cached = read_cache().expect("cache entry");
        assert_eq!(cached.session_id, input.session_id);
        assert_eq!(cached.output, first);

        let _lock = acquire_lock(Duration::from_millis(20)).expect("held lock");
        let second =
            run_inner_with_input(&opts, &input, Duration::from_millis(20)).expect("cache hit");
        assert_eq!(second, first);
    }

    #[test]
    fn run_inner_returns_stub_when_lock_is_held_and_cache_misses() {
        let cache_dir = TempDir::new().expect("cache dir");
        let _cache_env = CacheEnvGuard::new(cache_dir.path());
        let work_dir = TempDir::new().expect("work dir");
        let db_path = init_test_db(&work_dir);
        let transcript_path = work_dir.path().join("transcript.jsonl");
        fs::write(&transcript_path, "{\"type\":\"assistant\"}\n").expect("write transcript");

        let opts = make_opts(db_path, 60);
        let input = make_input(&transcript_path);
        let _lock = acquire_lock(Duration::from_millis(20)).expect("held lock");

        let line =
            run_inner_with_input(&opts, &input, Duration::from_millis(10)).expect("lock fallback");
        assert_eq!(line, "...");
        assert!(read_cache().is_none());
    }
}
