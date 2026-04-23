pub mod api;
pub mod assets;
#[cfg(test)]
mod tests;
pub mod tz;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::{get, post};
use tokio::sync::{Mutex, RwLock};

use crate::config::{AgentStatusConfig, AggregatorConfig, WebhookConfig};
use crate::webhooks::WebhookState;
use api::AppState;

type BackgroundPollFuture =
    Pin<Box<dyn std::future::Future<Output = Result<(), StatusCode>> + Send>>;
type BackgroundPollRefresh = Box<dyn Fn() -> BackgroundPollFuture + Send + Sync>;

pub struct ServeOptions {
    pub host: String,
    pub port: u16,
    pub db_path: PathBuf,
    pub projects_dirs: Option<Vec<PathBuf>>,
    pub oauth_enabled: bool,
    pub oauth_refresh_interval: u64,
    pub openai_enabled: bool,
    pub openai_admin_key_env: String,
    pub openai_refresh_interval: u64,
    pub openai_lookback_days: i64,
    pub webhook_config: WebhookConfig,
    /// Phase 20: enable file-watcher auto-refresh (started with `--watch`).
    pub watch: bool,
    /// Start background polling so remote monitoring data warms at login.
    pub background_poll: bool,
    /// Agent status monitoring config.
    pub agent_status_config: AgentStatusConfig,
    /// Community signal aggregator config (opt-in, off by default).
    pub aggregator_config: AggregatorConfig,
    /// Token quota for the billing-blocks dashboard endpoint (from config [blocks.token_limit]).
    pub blocks_token_limit: Option<i64>,
    /// Effective session length in hours for /api/billing-blocks (Phase 13).
    pub session_length_hours: f64,
    /// Phase 11: project slug -> display name map, populated once from config at startup.
    pub project_aliases: std::collections::HashMap<String, String>,
}

pub(crate) fn build_state(
    options: &ServeOptions,
    scan_event_tx: tokio::sync::broadcast::Sender<String>,
) -> Arc<AppState> {
    Arc::new(AppState {
        db_path: options.db_path.clone(),
        projects_dirs: options.projects_dirs.clone(),
        oauth_enabled: options.oauth_enabled,
        oauth_refresh_interval: options.oauth_refresh_interval,
        oauth_cache: RwLock::new(None),
        oauth_refresh_lock: Mutex::new(()),
        openai_enabled: options.openai_enabled,
        openai_admin_key_env: options.openai_admin_key_env.clone(),
        openai_refresh_interval: options.openai_refresh_interval,
        openai_lookback_days: options.openai_lookback_days,
        openai_cache: RwLock::new(None),
        openai_refresh_lock: Mutex::new(()),
        db_lock: Mutex::new(()),
        webhook_state: Mutex::new(WebhookState::default()),
        webhook_config: options.webhook_config.clone(),
        scan_event_tx,
        agent_status_config: options.agent_status_config.clone(),
        agent_status_cache: RwLock::new(None),
        agent_status_refresh_lock: Mutex::new(()),
        aggregator_config: options.aggregator_config.clone(),
        aggregator_cache: RwLock::new(None),
        aggregator_refresh_lock: Mutex::new(()),
        blocks_token_limit: options.blocks_token_limit,
        session_length_hours: options.session_length_hours,
        project_aliases: options.project_aliases.clone(),
        live_provider_cache: RwLock::new(None),
        live_provider_refresh_lock: Mutex::new(()),
    })
}

pub(crate) fn build_router(state: Arc<AppState>) -> Router {
    let dashboard_html = assets::render_dashboard();

    Router::new()
        .route(
            "/",
            get({
                let html = dashboard_html.clone();
                move || async { Html(html) }
            }),
        )
        .route(
            "/index.html",
            get({
                let html = dashboard_html;
                move || async { Html(html) }
            }),
        )
        .route(
            "/monitor",
            get({
                let html = assets::render_dashboard();
                move || async { Html(html) }
            }),
        )
        .route(
            "/favicon.ico",
            get(|| async { axum::http::StatusCode::NO_CONTENT }),
        )
        .route("/api/data", get(api::api_data))
        .route("/api/rescan", post(api::api_rescan))
        .route("/api/usage-windows", get(api::api_usage_windows))
        .route("/api/claude-usage", get(api::api_claude_usage))
        .route("/api/health", get(api::api_health))
        .route("/api/heatmap", get(api::api_heatmap))
        .route("/api/stream", get(api::api_stream))
        .route("/api/agent-status", get(api::api_agent_status))
        .route("/api/community-signal", get(api::api_community_signal))
        .route("/api/billing-blocks", get(api::api_billing_blocks))
        .route("/api/context-window", get(api::api_context_window))
        .route("/api/live-providers", get(api::api_live_providers))
        .route(
            "/api/live-providers/refresh",
            post(api::api_live_provider_refresh),
        )
        .route(
            "/api/live-providers/history",
            get(api::api_live_provider_history),
        )
        .route("/api/live-monitor", get(api::api_live_monitor))
        .route("/api/mobile-snapshot", get(api::api_mobile_snapshot))
        .route(
            "/api/cost-reconciliation",
            get(api::api_cost_reconciliation),
        )
        .with_state(state)
}

pub async fn serve(options: ServeOptions) -> anyhow::Result<()> {
    // Phase 20: broadcast channel for SSE scan_completed events.
    // Capacity 16: enough to buffer events for slow subscribers.
    let (scan_event_tx, _scan_event_rx) = tokio::sync::broadcast::channel::<String>(16);

    let state = build_state(&options, scan_event_tx.clone());
    let app = build_router(state.clone());
    let addr = format!("{}:{}", options.host, options.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    if options.background_poll {
        start_background_pollers(state.clone());
    }

    // Phase 20: start file-watcher if --watch was requested.
    // The watcher lives for the lifetime of the server; it is dropped when
    // the server exits.
    let _file_watcher = if options.watch {
        let db_path = options.db_path.clone();
        let projects_dirs = options.projects_dirs.clone();
        let scan_tx = scan_event_tx.clone();
        let scan_lock = crate::scanner::watcher::new_scan_lock();

        // Determine directories to watch: ~/.claude/projects/ plus any
        // explicit projects_dirs override.
        let mut watch_paths: Vec<PathBuf> = Vec::new();
        if let Some(ref dirs) = projects_dirs {
            watch_paths.extend(dirs.iter().cloned());
        } else {
            let claude_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".claude");
            watch_paths.push(claude_dir);
        }
        watch_paths.retain(|p| p.exists());

        if watch_paths.is_empty() {
            tracing::warn!("file-watcher: no watchable directories found; --watch has no effect");
            None
        } else {
            let watcher = crate::scanner::watcher::FileWatcher::start(
                watch_paths,
                Box::new(move || {
                    // Guard: only one scan at a time.
                    let Ok(_guard) = scan_lock.try_lock() else {
                        tracing::debug!("file-watcher: scan already running, skipping");
                        return;
                    };
                    tracing::info!("file-watcher: triggered background re-scan");
                    match crate::scanner::scan(projects_dirs.clone(), &db_path, false) {
                        Ok(result) => {
                            let ts = chrono::Utc::now().to_rfc3339();
                            let payload = serde_json::json!({
                                "type": "scan_completed",
                                "ts": ts,
                                "new": result.new,
                                "updated": result.updated,
                            })
                            .to_string();
                            // Best-effort broadcast; ignore errors if no subscribers.
                            let _ = scan_tx.send(payload);
                            tracing::info!(
                                "file-watcher: scan complete ({} new, {} updated)",
                                result.new,
                                result.updated
                            );
                        }
                        Err(e) => {
                            tracing::warn!("file-watcher: scan failed: {}", e);
                        }
                    }
                }),
            );
            match watcher {
                Ok(w) => {
                    tracing::info!("file-watcher: active (2s debounce)");
                    Some(w)
                }
                Err(e) => {
                    tracing::warn!("file-watcher: failed to start: {}", e);
                    None
                }
            }
        }
    } else {
        None
    };
    tracing::info!("Dashboard running at http://{}", addr);
    eprintln!("Dashboard running at http://{addr}");
    eprintln!("Press Ctrl+C to stop.");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

pub(crate) fn start_background_pollers(state: Arc<AppState>) {
    start_background_pollers_with(state, |name, interval_secs, refresh| {
        std::mem::drop(spawn_background_loop(name, interval_secs, refresh));
    });
}

pub(crate) fn start_background_pollers_with<F>(state: Arc<AppState>, mut spawn: F)
where
    F: FnMut(&'static str, u64, BackgroundPollRefresh),
{
    if state.oauth_enabled {
        let state = state.clone();
        spawn(
            "oauth usage",
            state.oauth_refresh_interval,
            Box::new(move || {
                let state = state.clone();
                Box::pin(async move {
                    let _ = api::refresh_usage_windows(&state).await;
                    Ok(())
                })
            }),
        );
    }

    if state.agent_status_config.enabled {
        let state = state.clone();
        spawn(
            "agent status",
            state.agent_status_config.refresh_interval,
            Box::new(move || {
                let state = state.clone();
                Box::pin(async move {
                    let _ = api::refresh_agent_status(&state).await?;
                    Ok(())
                })
            }),
        );
    }

    if state.aggregator_config.enabled {
        let state = state.clone();
        spawn(
            "community signal",
            state.aggregator_config.refresh_interval,
            Box::new(move || {
                let state = state.clone();
                Box::pin(async move {
                    let _ = api::refresh_community_signal(&state).await?;
                    Ok(())
                })
            }),
        );
    }

    if state.openai_enabled {
        let state = state.clone();
        spawn(
            "OpenAI reconciliation",
            state.openai_refresh_interval,
            Box::new(move || {
                let state = state.clone();
                Box::pin(async move {
                    let _ = api::refresh_openai_reconciliation(&state, None).await;
                    Ok(())
                })
            }),
        );
    }
}

pub(crate) fn spawn_background_loop<F, Fut>(
    name: &'static str,
    interval_secs: u64,
    refresh: F,
) -> tokio::task::JoinHandle<()>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), axum::http::StatusCode>> + Send + 'static,
{
    tokio::spawn(async move {
        let interval_secs = interval_secs.max(1);
        tokio::time::sleep(background_poll_startup_delay(name, interval_secs)).await;
        loop {
            if let Err(status) = refresh().await {
                tracing::warn!("background poller '{}' failed: {}", name, status);
            }
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    })
}

pub(crate) fn background_poll_startup_delay(name: &str, interval_secs: u64) -> Duration {
    const BASE_DELAY_SECS: u64 = 5;
    const JITTER_WINDOW_SECS: u64 = 5;

    let interval_secs = interval_secs.max(1);
    let jitter = name
        .bytes()
        .fold(14_695_981_039_346_656_037_u64, |acc, byte| {
            (acc ^ u64::from(byte)).wrapping_mul(1_099_511_628_211)
        })
        % JITTER_WINDOW_SECS;
    Duration::from_secs((BASE_DELAY_SECS + jitter).min(interval_secs))
}
