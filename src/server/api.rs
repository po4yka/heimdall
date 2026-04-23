use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use axum::extract::{Query, Request, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::response::sse::{Event, KeepAlive, Sse};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::{Mutex, RwLock, broadcast};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::tz::TzParams;

use crate::agent_status;
use crate::agent_status::models::AgentStatusSnapshot;
use crate::config::{AgentStatusConfig, AggregatorConfig, WebhookConfig};
use crate::live_providers;
use crate::models::ClaudeUsageResponse;
use crate::models::DepletionForecast;
use crate::models::LIVE_MONITOR_CONTRACT_VERSION;
use crate::models::LiveMonitorBillingBlock;
use crate::models::LiveMonitorBurnRate;
use crate::models::LiveMonitorContextWindow;
use crate::models::LiveMonitorFreshness;
use crate::models::LiveMonitorProjection;
use crate::models::LiveMonitorProvider;
use crate::models::LiveMonitorQuota;
use crate::models::LiveMonitorResponse;
use crate::models::LiveProviderHistoryResponse;
use crate::models::LiveProvidersResponse;
use crate::models::LiveQuotaSuggestionLevel;
use crate::models::LiveQuotaSuggestions;
use crate::models::MobileSnapshotEnvelope;
use crate::models::OpenAiReconciliation;
use crate::models::TokenBreakdown;
use crate::oauth;
use crate::oauth::models::UsageWindowsResponse;
use crate::openai;
use crate::scanner;
use crate::scanner::db;
use crate::status_aggregator;
use crate::status_aggregator::models::CommunitySignal;
use crate::webhooks::{self, WebhookState};

pub struct AppState {
    pub db_path: PathBuf,
    pub projects_dirs: Option<Vec<PathBuf>>,
    pub oauth_enabled: bool,
    pub oauth_refresh_interval: u64,
    pub oauth_cache: RwLock<Option<(Instant, UsageWindowsResponse)>>,
    pub oauth_refresh_lock: Mutex<()>,
    pub openai_enabled: bool,
    pub openai_admin_key_env: String,
    pub openai_refresh_interval: u64,
    pub openai_lookback_days: i64,
    pub openai_cache: RwLock<Option<(Instant, OpenAiReconciliation)>>,
    pub openai_refresh_lock: Mutex<()>,
    pub db_lock: Mutex<()>,
    pub webhook_state: Mutex<WebhookState>,
    pub webhook_config: WebhookConfig,
    /// Phase 20: broadcast channel for SSE scan_completed events.
    /// The watcher fires into this channel; SSE subscribers receive the events.
    pub scan_event_tx: broadcast::Sender<String>,
    /// Agent status monitoring config and cache.
    pub agent_status_config: AgentStatusConfig,
    /// Cache: (fetched_at, snapshot, claude_etag).
    pub agent_status_cache: RwLock<Option<(Instant, AgentStatusSnapshot, Option<String>)>>,
    pub agent_status_refresh_lock: Mutex<()>,
    /// Community-signal aggregator config (opt-in, off by default).
    pub aggregator_config: AggregatorConfig,
    /// Community-signal cache: (fetched_at, signal).
    pub aggregator_cache: RwLock<Option<(Instant, CommunitySignal)>>,
    pub aggregator_refresh_lock: Mutex<()>,
    /// Token quota for the /api/billing-blocks endpoint (from config [blocks.token_limit]).
    pub blocks_token_limit: Option<i64>,
    /// Effective session length in hours for /api/billing-blocks (from config, Phase 13).
    pub session_length_hours: f64,
    /// Phase 11: project slug -> display name map, populated once at startup.
    pub project_aliases: std::collections::HashMap<String, String>,
    /// Cached live-provider snapshots for native menu consumers.
    pub live_provider_cache: RwLock<Option<(Instant, LiveProvidersResponse)>>,
    pub live_provider_refresh_lock: Mutex<()>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BillingBlockViewResponse {
    pub start: String,
    pub end: String,
    pub first_timestamp: String,
    pub last_timestamp: String,
    pub tokens: TokenBreakdown,
    pub cost_nanos: i64,
    pub models: Vec<String>,
    pub is_active: bool,
    pub is_gap: bool,
    pub kind: String,
    pub entry_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burn_rate: Option<LiveMonitorBurnRate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection: Option<LiveMonitorProjection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota: Option<LiveMonitorQuota>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BillingBlocksApiResponse {
    pub session_length_hours: f64,
    pub token_limit: Option<i64>,
    pub historical_max_tokens: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_suggestions: Option<LiveQuotaSuggestions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depletion_forecast: Option<DepletionForecast>,
    pub blocks: Vec<BillingBlockViewResponse>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct ContextWindowApiResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_input_tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pct: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<String>,
}

pub async fn api_data(
    State(state): State<Arc<AppState>>,
    Query(tz): Query<TzParams>,
    request: Request,
) -> Result<Json<Value>, StatusCode> {
    enforce_loopback_request(&request)?;
    let _db_guard = state.db_lock.lock().await;
    let db_path = state.db_path.clone();
    let openai_lookback_days = state.openai_lookback_days;
    let openai_start_date = (chrono::Utc::now().date_naive()
        - chrono::Duration::days(openai_lookback_days.saturating_sub(1)))
    .to_string();
    let (mut result, openai_local_cost_nanos) =
        tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
            let conn = db::open_db(&db_path)?;
            db::init_db(&conn)?;
            let data = db::get_dashboard_data(&conn, tz)?;
            let local_cost_nanos =
                db::get_provider_estimated_cost_nanos_since(&conn, "codex", &openai_start_date)?;
            Ok((data, local_cost_nanos))
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if state.openai_enabled {
        result.openai_reconciliation =
            Some(refresh_openai_reconciliation(&state, Some(openai_local_cost_nanos)).await);
    }

    maybe_send_cost_threshold_webhook(&state, &result).await;

    // Phase 11: apply project aliases — resolve display_name on every row.
    // Aliases are pre-loaded into AppState at startup; no per-request disk read.
    let aliases = &state.project_aliases;
    for row in &mut result.daily_by_project {
        row.display_name = aliases
            .get(&row.project)
            .map(|s| s.as_str())
            .unwrap_or(&row.project)
            .to_string();
    }
    for row in &mut result.sessions_all {
        row.display_name = aliases
            .get(&row.project)
            .map(|s| s.as_str())
            .unwrap_or(&row.project)
            .to_string();
    }

    let value = serde_json::to_value(result).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(value))
}

pub(crate) async fn refresh_openai_reconciliation(
    state: &Arc<AppState>,
    local_cost_nanos: Option<i64>,
) -> OpenAiReconciliation {
    let admin_key = std::env::var(&state.openai_admin_key_env).ok();
    refresh_openai_reconciliation_with(
        state,
        local_cost_nanos,
        admin_key,
        |key, days, cost| async move {
            openai::fetch_org_usage_reconciliation(key.trim(), days, cost).await
        },
    )
    .await
}

pub(crate) async fn refresh_openai_reconciliation_with<F, Fut>(
    state: &Arc<AppState>,
    local_cost_nanos: Option<i64>,
    admin_key: Option<String>,
    fetcher: F,
) -> OpenAiReconciliation
where
    F: FnOnce(String, i64, f64) -> Fut,
    Fut: std::future::Future<Output = OpenAiReconciliation>,
{
    {
        let cache = state.openai_cache.read().await;
        if let Some((fetched_at, ref data)) = *cache
            && fetched_at.elapsed().as_secs() < state.openai_refresh_interval
        {
            return data.clone();
        }
    }

    let _refresh_guard = state.openai_refresh_lock.lock().await;
    {
        let cache = state.openai_cache.read().await;
        if let Some((fetched_at, ref data)) = *cache
            && fetched_at.elapsed().as_secs() < state.openai_refresh_interval
        {
            return data.clone();
        }
    }

    let local_cost_nanos = match local_cost_nanos {
        Some(nanos) => nanos,
        None => fetch_openai_local_cost_nanos(state).await.unwrap_or(0),
    };
    let estimated_local_cost = local_cost_nanos as f64 / 1_000_000_000.0;
    let reconciliation = match admin_key {
        Some(admin_key) if !admin_key.trim().is_empty() => {
            fetcher(admin_key, state.openai_lookback_days, estimated_local_cost).await
        }
        _ => OpenAiReconciliation {
            available: false,
            lookback_days: state.openai_lookback_days,
            start_date: (chrono::Utc::now().date_naive()
                - chrono::Duration::days(state.openai_lookback_days.saturating_sub(1)))
            .to_string(),
            end_date: chrono::Utc::now().date_naive().to_string(),
            estimated_local_cost,
            api_usage_cost: 0.0,
            api_input_tokens: 0,
            api_output_tokens: 0,
            api_cached_input_tokens: 0,
            api_requests: 0,
            delta_cost: 0.0,
            error: Some(format!(
                "Set {} to enable OpenAI organization usage reconciliation.",
                state.openai_admin_key_env
            )),
        },
    };

    let cached = {
        let cache = state.openai_cache.read().await;
        cache.as_ref().map(|(_, data)| data.clone())
    };
    let to_store = if reconciliation.available {
        reconciliation.clone()
    } else {
        cached.unwrap_or_else(|| reconciliation.clone())
    };

    {
        let mut cache = state.openai_cache.write().await;
        *cache = Some((Instant::now(), to_store.clone()));
    }

    to_store
}

async fn fetch_openai_local_cost_nanos(state: &Arc<AppState>) -> Result<i64, StatusCode> {
    let db_path = state.db_path.clone();
    let openai_start_date = (chrono::Utc::now().date_naive()
        - chrono::Duration::days(state.openai_lookback_days.saturating_sub(1)))
    .to_string();

    tokio::task::spawn_blocking(move || -> anyhow::Result<i64> {
        let conn = db::open_db(&db_path)?;
        db::init_db(&conn)?;
        let local_cost_nanos =
            db::get_provider_estimated_cost_nanos_since(&conn, "codex", &openai_start_date)?;
        Ok(local_cost_nanos)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn api_rescan(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<Value>, StatusCode> {
    if let Some(addr) = request
        .extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|info| info.0)
        && !addr.ip().is_loopback()
    {
        return Err(StatusCode::FORBIDDEN);
    }

    let _db_guard = state.db_lock.lock().await;
    let db_path = state.db_path.clone();
    let projects_dirs = state.projects_dirs.clone();

    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        // Atomic rescan: write to temp, then rename
        let temp_path = db_path.with_extension("db.tmp");
        cleanup_sqlite_files(&temp_path)?;
        let scan_result = scanner::scan(projects_dirs, &temp_path, false)?;
        if temp_path.exists() {
            preserve_runtime_history(&temp_path, &db_path)?;
            replace_sqlite_files(&temp_path, &db_path)?;
        }
        Ok(scan_result)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let value = serde_json::to_value(result).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(value))
}

pub async fn api_usage_windows(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<Value>, StatusCode> {
    enforce_loopback_request(&request)?;
    let resp = refresh_usage_windows(&state).await;
    let value = serde_json::to_value(resp).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(value))
}

pub(crate) async fn refresh_usage_windows(state: &Arc<AppState>) -> UsageWindowsResponse {
    refresh_usage_windows_with(state, || async { oauth::poll_usage().await }).await
}

pub(crate) async fn refresh_usage_windows_with<F, Fut>(
    state: &Arc<AppState>,
    fetcher: F,
) -> UsageWindowsResponse
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = UsageWindowsResponse>,
{
    if !state.oauth_enabled {
        return UsageWindowsResponse::unavailable();
    }

    {
        let cache = state.oauth_cache.read().await;
        if let Some((fetched_at, ref data)) = *cache
            && fetched_at.elapsed().as_secs() < state.oauth_refresh_interval
        {
            return data.clone();
        }
    }

    let _refresh_guard = state.oauth_refresh_lock.lock().await;
    {
        let cache = state.oauth_cache.read().await;
        if let Some((fetched_at, ref data)) = *cache
            && fetched_at.elapsed().as_secs() < state.oauth_refresh_interval
        {
            return data.clone();
        }
    }

    let resp = fetcher().await;
    if let Some(session) = resp.session.as_ref() {
        maybe_send_session_webhook(state, session.used_percent, session.resets_in_minutes).await;
    }

    let cached = {
        let cache = state.oauth_cache.read().await;
        cache.as_ref().map(|(_, data)| data.clone())
    };
    let to_store = if resp.available {
        resp.clone()
    } else {
        cached.unwrap_or_else(|| resp.clone())
    };

    {
        let mut cache = state.oauth_cache.write().await;
        *cache = Some((Instant::now(), to_store.clone()));
    }

    to_store
}

pub async fn api_claude_usage(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<Value>, StatusCode> {
    enforce_loopback_request(&request)?;
    let db_path = state.db_path.clone();
    let response = tokio::task::spawn_blocking(move || -> anyhow::Result<ClaudeUsageResponse> {
        let conn = db::open_db(&db_path)?;
        db::init_db(&conn)?;
        db::get_latest_claude_usage_response(&conn)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let value = serde_json::to_value(response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(value))
}

pub async fn api_health() -> &'static str {
    "ok"
}

#[derive(Debug, serde::Deserialize)]
pub struct LiveProviderQuery {
    pub provider: Option<String>,
    pub scope: Option<String>,
    pub startup: Option<bool>,
}

pub async fn api_live_providers(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LiveProviderQuery>,
    request: Request,
) -> Result<Json<LiveProvidersResponse>, StatusCode> {
    enforce_loopback_request(&request)?;
    let scope = parse_live_provider_scope(query.scope.as_deref())?;
    let response = live_providers::load_snapshots(
        &state,
        query.provider.as_deref(),
        scope,
        false,
        query.startup.unwrap_or(false),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(response))
}

pub async fn api_live_provider_refresh(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LiveProviderQuery>,
    request: Request,
) -> Result<Json<LiveProvidersResponse>, StatusCode> {
    enforce_loopback_request(&request)?;
    let scope = parse_live_provider_scope(query.scope.as_deref())?;
    if query.provider.is_none() || scope == live_providers::ResponseScope::All {
        let mut cache = state.live_provider_cache.write().await;
        *cache = None;
    }
    let response =
        live_providers::load_snapshots(&state, query.provider.as_deref(), scope, true, false)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(response))
}

pub async fn api_live_provider_history(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LiveProviderQuery>,
    request: Request,
) -> Result<Json<LiveProviderHistoryResponse>, StatusCode> {
    enforce_loopback_request(&request)?;
    let provider = query.provider.unwrap_or_else(|| "claude".into());
    let summary = live_providers::load_provider_history(&state, &provider)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(summary))
}

pub async fn api_mobile_snapshot(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<MobileSnapshotEnvelope>, StatusCode> {
    enforce_loopback_request(&request)?;
    let snapshot = live_providers::load_mobile_snapshot(&state)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(snapshot))
}

pub async fn api_live_monitor(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<LiveMonitorResponse>, StatusCode> {
    enforce_loopback_request(&request)?;
    let snapshots = live_providers::load_snapshots(
        &state,
        None,
        live_providers::ResponseScope::All,
        false,
        false,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let billing_blocks = load_billing_blocks_response(&state).await?;
    let context_window = load_context_window_response(&state).await?;
    Ok(Json(build_live_monitor_response(
        snapshots,
        billing_blocks,
        context_window,
    )))
}

fn build_live_monitor_response(
    snapshots: LiveProvidersResponse,
    billing_blocks: BillingBlocksApiResponse,
    context_window: ContextWindowApiResponse,
) -> LiveMonitorResponse {
    let now = chrono::Utc::now();
    let active_block = billing_blocks
        .blocks
        .iter()
        .find(|block| block.is_active)
        .cloned();
    let context_window_detail = build_live_monitor_context_window(&context_window);

    let providers = snapshots
        .providers
        .iter()
        .map(|snapshot| {
            let warnings =
                build_monitor_warnings(snapshot, active_block.as_ref(), &context_window_detail);
            LiveMonitorProvider {
                provider: snapshot.provider.clone(),
                title: provider_title(&snapshot.provider).to_string(),
                visual_state: monitor_visual_state(snapshot, &warnings).to_string(),
                source_label: format!("Source: {}", snapshot.source_used),
                warnings,
                identity_label: monitor_identity_label(snapshot),
                primary: snapshot.primary.clone(),
                secondary: snapshot.secondary.clone(),
                today_cost_usd: snapshot.cost_summary.today_cost_usd,
                projected_weekly_spend_usd: projected_weekly_spend(snapshot),
                last_refresh: snapshot.last_refresh.clone(),
                last_refresh_label: format!(
                    "Updated {}",
                    relative_refresh_label(&snapshot.last_refresh, now)
                ),
                active_block: if snapshot.provider == "claude" {
                    active_block
                        .as_ref()
                        .map(|block| live_monitor_billing_block(block.clone()))
                } else {
                    None
                },
                context_window: if snapshot.provider == "claude" {
                    context_window_detail.clone()
                } else {
                    None
                },
                recent_session: snapshot.cost_summary.recent_sessions.first().cloned(),
                quota_suggestions: if snapshot.provider == "claude" {
                    billing_blocks.quota_suggestions.clone()
                } else {
                    None
                },
                depletion_forecast: live_monitor_depletion_forecast(
                    snapshot,
                    active_block.as_ref(),
                ),
            }
        })
        .collect::<Vec<_>>();

    let newest_provider_refresh = snapshots
        .providers
        .iter()
        .map(|provider| provider.last_refresh.clone())
        .max();
    let oldest_provider_refresh = snapshots
        .providers
        .iter()
        .map(|provider| provider.last_refresh.clone())
        .min();
    let stale_providers = snapshots
        .providers
        .iter()
        .filter(|provider| provider.stale)
        .map(|provider| provider.provider.clone())
        .collect::<Vec<_>>();
    let global_issue = monitor_global_issue(&providers);

    LiveMonitorResponse {
        contract_version: LIVE_MONITOR_CONTRACT_VERSION,
        generated_at: now.to_rfc3339(),
        default_focus: "all".into(),
        global_issue,
        freshness: LiveMonitorFreshness {
            newest_provider_refresh,
            oldest_provider_refresh,
            has_stale_providers: !stale_providers.is_empty(),
            stale_providers,
            refresh_state: if providers
                .iter()
                .any(|provider| provider.visual_state == "error")
            {
                "attention".into()
            } else if providers
                .iter()
                .any(|provider| provider.visual_state == "stale")
            {
                "stale".into()
            } else {
                "current".into()
            },
        },
        providers,
    }
}

fn provider_title(provider: &str) -> &'static str {
    match provider {
        "claude" => "Claude",
        "codex" => "Codex",
        _ => "Provider",
    }
}

fn relative_refresh_label(iso: &str, now: chrono::DateTime<chrono::Utc>) -> String {
    chrono::DateTime::parse_from_rfc3339(iso)
        .ok()
        .map(|ts| ts.with_timezone(&chrono::Utc))
        .map(|ts| {
            let delta = now.signed_duration_since(ts);
            if delta.num_minutes() <= 0 {
                "just now".to_string()
            } else if delta.num_minutes() < 60 {
                format!("{}m ago", delta.num_minutes())
            } else if delta.num_hours() < 24 {
                format!("{}h ago", delta.num_hours())
            } else {
                format!("{}d ago", delta.num_days())
            }
        })
        .unwrap_or_else(|| iso.to_string())
}

fn projected_weekly_spend(snapshot: &crate::models::LiveProviderSnapshot) -> Option<f64> {
    let recent_days = snapshot
        .cost_summary
        .daily
        .iter()
        .rev()
        .take(7)
        .map(|point| point.cost_usd)
        .collect::<Vec<_>>();
    if !recent_days.is_empty() {
        let avg = recent_days.iter().sum::<f64>() / recent_days.len() as f64;
        return Some(avg * 7.0);
    }
    (snapshot.cost_summary.today_cost_usd > 0.0)
        .then_some(snapshot.cost_summary.today_cost_usd * 7.0)
}

fn monitor_identity_label(snapshot: &crate::models::LiveProviderSnapshot) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(identity) = snapshot.identity.as_ref() {
        if let Some(plan) = identity.plan.as_ref() {
            parts.push(plan.clone());
        }
        if let Some(login_method) = identity.login_method.as_ref() {
            parts.push(login_method.clone());
        }
        if let Some(account_email) = identity.account_email.as_ref() {
            parts.push(account_email.clone());
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}

fn push_warning(warnings: &mut Vec<String>, warning: Option<String>) {
    if let Some(warning) = warning.filter(|warning| !warning.trim().is_empty())
        && !warnings.iter().any(|existing| existing == &warning)
    {
        warnings.push(warning);
    }
}

fn build_monitor_warnings(
    snapshot: &crate::models::LiveProviderSnapshot,
    active_block: Option<&BillingBlockViewResponse>,
    context_window: &Option<LiveMonitorContextWindow>,
) -> Vec<String> {
    let mut warnings = Vec::new();
    push_warning(&mut warnings, snapshot.error.clone());
    if snapshot.auth.requires_relogin || !snapshot.auth.is_authenticated {
        push_warning(&mut warnings, Some("Authentication needs attention".into()));
    }
    if let Some(status) = snapshot.status.as_ref()
        && status.indicator != "none"
    {
        push_warning(
            &mut warnings,
            Some(format!(
                "{} status: {}",
                provider_title(&snapshot.provider),
                status.description
            )),
        );
    }
    if snapshot.provider == "claude"
        && let Some(block) = active_block
        && let Some(quota) = block.quota.as_ref()
        && quota.projected_severity != "ok"
    {
        push_warning(
            &mut warnings,
            Some(format!(
                "Billing block projected {}",
                quota.projected_severity
            )),
        );
    }
    if snapshot.provider == "claude"
        && let Some(context_window) = context_window.as_ref()
        && context_window.severity != "ok"
    {
        push_warning(
            &mut warnings,
            Some(format!("Context window {}", context_window.severity)),
        );
    }
    warnings
}

fn monitor_visual_state(
    snapshot: &crate::models::LiveProviderSnapshot,
    warnings: &[String],
) -> &'static str {
    if snapshot.error.is_some()
        || snapshot.auth.requires_relogin
        || !snapshot.auth.is_source_compatible
    {
        return "error";
    }
    if let Some(status) = snapshot.status.as_ref()
        && matches!(status.indicator.as_str(), "critical" | "major")
    {
        return "incident";
    }
    if snapshot.stale {
        return "stale";
    }
    if !warnings.is_empty()
        || snapshot
            .status
            .as_ref()
            .is_some_and(|status| matches!(status.indicator.as_str(), "minor" | "maintenance"))
    {
        return "degraded";
    }
    "healthy"
}

fn monitor_global_issue(providers: &[LiveMonitorProvider]) -> Option<String> {
    let error_count = providers
        .iter()
        .filter(|provider| provider.visual_state == "error")
        .count();
    if error_count > 0 {
        return Some(format!(
            "{} provider{} need attention",
            error_count,
            if error_count == 1 { "" } else { "s" }
        ));
    }
    let stale_count = providers
        .iter()
        .filter(|provider| provider.visual_state == "stale")
        .count();
    if stale_count > 0 {
        return Some(format!(
            "{} provider{} using stale data",
            stale_count,
            if stale_count == 1 { "" } else { "s" }
        ));
    }
    providers
        .iter()
        .find_map(|provider| provider.warnings.first().cloned())
}

fn build_live_monitor_context_window(
    response: &ContextWindowApiResponse,
) -> Option<LiveMonitorContextWindow> {
    if response.enabled == Some(false) {
        return None;
    }
    Some(LiveMonitorContextWindow {
        total_input_tokens: response.total_input_tokens?,
        context_window_size: response.context_window_size?,
        pct: response.pct?,
        severity: response.severity.clone()?,
        session_id: response.session_id.clone(),
        captured_at: response.captured_at.clone(),
    })
}

fn live_quota_suggestions(
    suggestions: crate::analytics::quota::QuotaSuggestions,
) -> LiveQuotaSuggestions {
    LiveQuotaSuggestions {
        sample_count: suggestions.sample_count,
        recommended_key: suggestions.recommended_key,
        levels: suggestions
            .levels
            .into_iter()
            .map(|level| LiveQuotaSuggestionLevel {
                key: level.key,
                label: level.label,
                limit_tokens: level.limit_tokens,
            })
            .collect(),
        note: suggestions.note,
    }
}

fn live_monitor_depletion_forecast(
    snapshot: &crate::models::LiveProviderSnapshot,
    active_block: Option<&BillingBlockViewResponse>,
) -> Option<DepletionForecast> {
    use crate::analytics::depletion::{
        build_depletion_forecast, primary_window_signal, secondary_window_signal,
    };

    let mut signals = Vec::new();

    if snapshot.provider == "claude"
        && let Some(signal) = active_block.and_then(billing_block_depletion_signal)
    {
        signals.push(signal);
    }
    if let Some(window) = snapshot.primary.as_ref() {
        signals.push(primary_window_signal(
            window.used_percent,
            Some(100.0 - window.used_percent),
            window.resets_in_minutes,
            None,
            window.resets_at.clone(),
        ));
    }
    if let Some(window) = snapshot.secondary.as_ref() {
        signals.push(secondary_window_signal(
            window.used_percent,
            Some(100.0 - window.used_percent),
            window.resets_in_minutes,
            None,
            window.resets_at.clone(),
        ));
    }

    build_depletion_forecast(signals)
}

fn billing_block_depletion_signal(
    block: &BillingBlockViewResponse,
) -> Option<crate::models::DepletionForecastSignal> {
    use crate::analytics::depletion::billing_block_signal;

    let quota = block.quota.as_ref()?;
    Some(billing_block_signal(
        "Billing block",
        quota.current_pct * 100.0,
        Some(quota.projected_pct * 100.0),
        Some(quota.remaining_tokens),
        Some(100.0 - (quota.current_pct * 100.0)),
        Some(block.end.clone()),
    ))
}

fn live_monitor_billing_block(block: BillingBlockViewResponse) -> LiveMonitorBillingBlock {
    LiveMonitorBillingBlock {
        start: block.start,
        end: block.end,
        first_timestamp: block.first_timestamp,
        last_timestamp: block.last_timestamp,
        tokens: block.tokens,
        cost_nanos: block.cost_nanos,
        entry_count: block.entry_count,
        burn_rate: block.burn_rate,
        projection: block.projection,
        quota: block.quota,
    }
}

fn parse_live_provider_scope(
    scope: Option<&str>,
) -> Result<live_providers::ResponseScope, StatusCode> {
    match scope.unwrap_or("all") {
        "all" => Ok(live_providers::ResponseScope::All),
        "provider" => Ok(live_providers::ResponseScope::ProviderOnly),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

fn enforce_loopback_request(request: &Request) -> Result<(), StatusCode> {
    if let Some(addr) = request
        .extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|info| info.0)
        && !addr.ip().is_loopback()
    {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

/// `GET /api/agent-status` — returns the latest upstream provider health snapshot.
///
/// When `agent_status_config.enabled` is false, returns an empty snapshot
/// without making any network calls.
pub async fn api_agent_status(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<Value>, StatusCode> {
    enforce_loopback_request(&request)?;
    let snapshot = refresh_agent_status(&state).await?;
    let value = serde_json::to_value(&snapshot).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(value))
}

pub(crate) async fn refresh_agent_status(
    state: &Arc<AppState>,
) -> Result<AgentStatusSnapshot, StatusCode> {
    refresh_agent_status_with(state, |config, cached_etag| async move {
        tokio::task::spawn_blocking(move || agent_status::poll(&config, cached_etag.as_deref()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    })
    .await
}

pub(crate) async fn refresh_agent_status_with<F, Fut>(
    state: &Arc<AppState>,
    fetcher: F,
) -> Result<AgentStatusSnapshot, StatusCode>
where
    F: FnOnce(AgentStatusConfig, Option<String>) -> Fut,
    Fut: std::future::Future<Output = Result<(AgentStatusSnapshot, Option<String>), StatusCode>>,
{
    if !state.agent_status_config.enabled {
        return Ok(AgentStatusSnapshot::default());
    }

    {
        let cache = state.agent_status_cache.read().await;
        if let Some((fetched_at, ref data, _)) = *cache
            && fetched_at.elapsed().as_secs() < state.agent_status_config.refresh_interval
        {
            return Ok(data.clone());
        }
    }

    let _refresh_guard = state.agent_status_refresh_lock.lock().await;
    {
        let cache = state.agent_status_cache.read().await;
        if let Some((fetched_at, ref data, _)) = *cache
            && fetched_at.elapsed().as_secs() < state.agent_status_config.refresh_interval
        {
            return Ok(data.clone());
        }
    }

    let config = state.agent_status_config.clone();
    let cached_etag = {
        let cache = state.agent_status_cache.read().await;
        cache.as_ref().and_then(|(_, _, etag)| etag.clone())
    };
    let (mut snapshot, new_etag) = fetcher(config, cached_etag).await?;
    let is_fresh = snapshot.claude.is_some() || snapshot.openai.is_some();

    if is_fresh {
        persist_agent_status_snapshot(state, &mut snapshot).await;
        maybe_send_agent_status_webhooks(state, &snapshot).await;
        let mut cache = state.agent_status_cache.write().await;
        *cache = Some((Instant::now(), snapshot.clone(), new_etag));
        return Ok(snapshot);
    }

    let cached = {
        let cache = state.agent_status_cache.read().await;
        cache
            .as_ref()
            .map(|(_, data, etag)| (data.clone(), etag.clone()))
    };
    if let Some((snapshot, etag)) = cached {
        let mut cache = state.agent_status_cache.write().await;
        *cache = Some((Instant::now(), snapshot.clone(), etag));
        return Ok(snapshot);
    }

    let mut cache = state.agent_status_cache.write().await;
    *cache = Some((Instant::now(), snapshot.clone(), None));
    Ok(snapshot)
}

async fn persist_agent_status_snapshot(state: &Arc<AppState>, snapshot: &mut AgentStatusSnapshot) {
    let db_path = state.db_path.clone();
    let mut snapshot_out = std::mem::take(snapshot);
    let updated = tokio::task::spawn_blocking(move || {
        if let Ok(conn) = db::open_db(&db_path) {
            let _ = db::init_db(&conn);
            let ts_epoch = chrono::Utc::now().timestamp();

            if let Some(ref claude) = snapshot_out.claude {
                let samples: Vec<(String, String, String)> = claude
                    .components
                    .iter()
                    .map(|c| (c.id.clone(), c.name.clone(), c.status.clone()))
                    .collect();
                let _ = db::insert_agent_status_samples(&conn, "claude", &samples, ts_epoch);
            }

            if let Some(ref openai) = snapshot_out.openai {
                let samples: Vec<(String, String, String)> = openai
                    .components
                    .iter()
                    .map(|c| (c.id.clone(), c.name.clone(), c.status.clone()))
                    .collect();
                let _ = db::insert_agent_status_samples(&conn, "openai", &samples, ts_epoch);
            }

            if let Err(e) = db::prune_agent_status_history(&conn, 90) {
                tracing::debug!("agent_status prune error: {}", e);
            }

            if let Some(ref mut claude) = snapshot_out.claude {
                for c in &mut claude.components {
                    let id = c.id.clone();
                    c.uptime_30d = db::uptime_pct(&conn, "claude", &id, 30).ok().flatten();
                    c.uptime_7d = db::uptime_pct(&conn, "claude", &id, 7).ok().flatten();
                }
            }

            if let Some(ref mut openai) = snapshot_out.openai {
                for c in &mut openai.components {
                    let id = c.id.clone();
                    c.uptime_30d = db::uptime_pct(&conn, "openai", &id, 30).ok().flatten();
                    c.uptime_7d = db::uptime_pct(&conn, "openai", &id, 7).ok().flatten();
                }
            }
        }
        snapshot_out
    })
    .await
    .unwrap_or_else(|_| std::mem::take(snapshot));
    *snapshot = updated;
}

async fn maybe_send_agent_status_webhooks(state: &Arc<AppState>, snapshot: &AgentStatusSnapshot) {
    let mut webhook_state = state.webhook_state.lock().await;

    if let Some(ref claude) = snapshot.claude
        && let Some(event) = webhooks::agent_status_transition_event(
            &state.webhook_config,
            &mut webhook_state.claude_degraded,
            "Claude",
            &claude.indicator,
            claude.active_incidents.len(),
        )
    {
        webhooks::notify_if_configured(&state.webhook_config, event);
    }

    if let Some(ref openai) = snapshot.openai
        && let Some(event) = webhooks::agent_status_transition_event(
            &state.webhook_config,
            &mut webhook_state.openai_degraded,
            "OpenAI",
            &openai.indicator,
            openai.active_incidents.len(),
        )
    {
        webhooks::notify_if_configured(&state.webhook_config, event);
    }
}

/// Phase 13: Query params for the heatmap endpoint.
///
/// `tz_offset_min` and `week_starts_on` mirror `TzParams` fields directly
/// (serde `flatten` does not compose with axum's `Query` extractor).
#[derive(Debug, serde::Deserialize, Default)]
pub struct HeatmapParams {
    #[serde(default)]
    pub period: Option<String>,
    #[serde(default)]
    pub tz_offset_min: Option<i32>,
    #[serde(default)]
    pub week_starts_on: Option<u8>,
}

/// Phase 13: JSON response shape for `GET /api/heatmap`.
#[derive(Debug, serde::Serialize)]
pub struct HeatmapResponse {
    pub cells: Vec<crate::models::HeatmapCell>,
    pub max_cost_nanos: i64,
    pub max_call_count: i64,
    /// Count of distinct calendar days with non-zero spend.
    pub active_days: i64,
    /// Total cost nanos for the period (used by the caller for avg/day).
    pub total_cost_nanos: i64,
    pub period: String,
    /// The resolved tz offset (0 when absent — UTC default).
    pub tz_offset_min: i32,
}

/// `GET /api/heatmap?period=<period>&tz_offset_min=<n>&week_starts_on=<n>`
///
/// Returns a 7×24 activity heatmap for the requested period.
/// `period` defaults to `"month"` when absent.
/// `tz_offset_min` defaults to 0 (UTC) when absent.
pub async fn api_heatmap(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HeatmapParams>,
    request: Request,
) -> Result<Json<Value>, StatusCode> {
    enforce_loopback_request(&request)?;
    let _db_guard = state.db_lock.lock().await;
    let db_path = state.db_path.clone();
    let period = params.period.unwrap_or_else(|| "month".to_string());
    let period_clone = period.clone();
    let tz = TzParams {
        tz_offset_min: params.tz_offset_min,
        week_starts_on: params.week_starts_on,
    };

    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = db::open_db(&db_path)?;
        db::init_db(&conn)?;
        let cells = db::get_heatmap(&conn, &period_clone, tz)?;
        let (total_cost_nanos, active_days) =
            db::active_period_avg_cost_nanos(&conn, &period_clone, tz)?;
        Ok((cells, total_cost_nanos, active_days))
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (cells, total_cost_nanos, active_days) = result;
    let max_cost_nanos = cells.iter().map(|c| c.cost_nanos).max().unwrap_or(0);
    let max_call_count = cells.iter().map(|c| c.call_count).max().unwrap_or(0);
    let tz_offset_min = tz.normalized_offset_min();

    let resp = HeatmapResponse {
        cells,
        max_cost_nanos,
        max_call_count,
        active_days,
        total_cost_nanos,
        period,
        tz_offset_min,
    };

    let value = serde_json::to_value(resp).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(value))
}

/// Phase 20: SSE endpoint — emits `event: scan_completed` whenever the
/// file-watcher triggers a background re-scan.
///
/// Clients connect to `GET /api/stream` and receive a keep-alive ping every
/// 15 seconds plus an event after each watcher-triggered scan.
///
/// Event body JSON: `{ "type": "scan_completed", "ts": "<iso8601>" }`
pub async fn api_stream(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    enforce_loopback_request(&request)?;
    let rx = state.scan_event_tx.subscribe();
    let broadcast_stream = BroadcastStream::new(rx);

    let event_stream = broadcast_stream.filter_map(|res| {
        // Ignore lagged-receiver errors by turning them into None (skip).
        res.ok().map(|payload| {
            Ok::<Event, std::convert::Infallible>(
                Event::default().event("scan_completed").data(payload),
            )
        })
    });

    Ok(Sse::new(event_stream).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    ))
}

/// `GET /api/billing-blocks` — returns all billing blocks with optional quota metadata.
///
/// Response shape:
/// ```json
/// {
///   "session_length_hours": 5.0,
///   "token_limit": 1000000,          // from config [blocks.token_limit], or null
///   "historical_max_tokens": 823412, // always computed
///   "blocks": [ ... ]
/// }
/// ```
pub async fn api_billing_blocks(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<Value>, StatusCode> {
    enforce_loopback_request(&request)?;
    let response = load_billing_blocks_response(&state).await?;
    let value = serde_json::to_value(response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(value))
}

pub(crate) async fn load_billing_blocks_response(
    state: &Arc<AppState>,
) -> Result<BillingBlocksApiResponse, StatusCode> {
    use crate::analytics::blocks::{
        calculate_burn_rate, identify_blocks_with_gaps, project_block_usage,
    };
    use crate::analytics::burn_rate::{self as br, BurnRateConfig};
    use crate::analytics::depletion::{billing_block_signal, build_depletion_forecast};
    use crate::analytics::quota::{compute_quota, compute_quota_suggestions};

    let db_path = state.db_path.clone();
    let token_limit = state.blocks_token_limit;
    let session_hours = state.session_length_hours;

    tokio::task::spawn_blocking(move || -> anyhow::Result<BillingBlocksApiResponse> {
        let conn = db::open_db(&db_path)?;
        db::init_db(&conn)?;

        let turns = db::load_all_turns(&conn)?;
        let now = chrono::Utc::now();
        let blocks = identify_blocks_with_gaps(&turns, session_hours, now, true);
        let historical_max_tokens = blocks
            .iter()
            .filter(|block| !block.is_gap)
            .map(|block| block.tokens.total())
            .max()
            .unwrap_or(0);

        let quota_suggestions = compute_quota_suggestions(&blocks).map(live_quota_suggestions);
        let depletion_forecast = token_limit.and_then(|limit| {
            blocks
                .iter()
                .find(|block| block.is_active && !block.is_gap)
                .and_then(|block| {
                    let rate = calculate_burn_rate(block, now);
                    let projection = project_block_usage(block, rate, now);
                    compute_quota(block, &projection, limit).map(|quota| {
                        build_depletion_forecast([billing_block_signal(
                            "Billing block",
                            quota.current_pct * 100.0,
                            Some(quota.projected_pct * 100.0),
                            Some(quota.remaining_tokens),
                            Some(100.0 - (quota.current_pct * 100.0)),
                            Some(block.end.to_rfc3339()),
                        )])
                    })
                })
                .flatten()
        });

        let blocks = blocks
            .iter()
            .map(|block| {
                let rate = if block.is_active {
                    calculate_burn_rate(block, now)
                } else {
                    None
                };
                let projection = project_block_usage(block, rate, now);

                BillingBlockViewResponse {
                    start: block.start.to_rfc3339(),
                    end: block.end.to_rfc3339(),
                    first_timestamp: block.first_timestamp.to_rfc3339(),
                    last_timestamp: block.last_timestamp.to_rfc3339(),
                    tokens: TokenBreakdown {
                        input: block.tokens.input,
                        output: block.tokens.output,
                        cache_read: block.tokens.cache_read,
                        cache_creation: block.tokens.cache_creation,
                        reasoning_output: block.tokens.reasoning_output,
                    },
                    cost_nanos: block.cost_nanos,
                    models: block.models.clone(),
                    is_active: block.is_active,
                    is_gap: block.is_gap,
                    kind: block.kind.to_string(),
                    entry_count: block.entry_count,
                    burn_rate: rate.map(|rate| LiveMonitorBurnRate {
                        tokens_per_min: rate.tokens_per_min,
                        cost_per_hour_nanos: rate.cost_per_hour_nanos,
                        tier: Some(
                            match br::tier(rate.tokens_per_min, &BurnRateConfig::default()) {
                                br::BurnRateTier::Normal => "normal",
                                br::BurnRateTier::Moderate => "moderate",
                                br::BurnRateTier::High => "high",
                            }
                            .into(),
                        ),
                    }),
                    projection: block.is_active.then_some(LiveMonitorProjection {
                        projected_cost_nanos: projection.projected_cost_nanos,
                        projected_tokens: projection.projected_tokens as i64,
                    }),
                    quota: token_limit
                        .filter(|_| block.is_active)
                        .and_then(|limit| compute_quota(block, &projection, limit))
                        .map(|quota| LiveMonitorQuota {
                            limit_tokens: quota.limit_tokens,
                            used_tokens: quota.used_tokens,
                            projected_tokens: quota.projected_tokens,
                            current_pct: quota.current_pct,
                            projected_pct: quota.projected_pct,
                            remaining_tokens: quota.remaining_tokens,
                            current_severity: match quota.current_severity {
                                crate::analytics::quota::Severity::Ok => "ok",
                                crate::analytics::quota::Severity::Warn => "warn",
                                crate::analytics::quota::Severity::Danger => "danger",
                            }
                            .into(),
                            projected_severity: match quota.projected_severity {
                                crate::analytics::quota::Severity::Ok => "ok",
                                crate::analytics::quota::Severity::Warn => "warn",
                                crate::analytics::quota::Severity::Danger => "danger",
                            }
                            .into(),
                        }),
                }
            })
            .collect();

        Ok(BillingBlocksApiResponse {
            session_length_hours: session_hours,
            token_limit,
            historical_max_tokens,
            quota_suggestions,
            depletion_forecast,
            blocks,
        })
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// `GET /api/context-window` — returns the most recent context-window snapshot
/// from `live_events`, or `{"enabled": false}` when no rows have context data.
///
/// Response (populated):
/// ```json
/// {
///   "total_input_tokens": 45231,
///   "context_window_size": 200000,
///   "pct": 0.2262,
///   "severity": "ok",
///   "session_id": "claude:abc",
///   "model": "claude-sonnet-4-6",
///   "captured_at": "2026-04-18T10:00:00Z"
/// }
/// ```
pub async fn api_context_window(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<Value>, StatusCode> {
    enforce_loopback_request(&request)?;
    let response = load_context_window_response(&state).await?;
    let value = serde_json::to_value(response).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(value))
}

pub(crate) async fn load_context_window_response(
    state: &Arc<AppState>,
) -> Result<ContextWindowApiResponse, StatusCode> {
    use crate::analytics::quota::severity_for_pct;
    use rusqlite::OptionalExtension;

    let db_path = state.db_path.clone();

    tokio::task::spawn_blocking(move || -> anyhow::Result<ContextWindowApiResponse> {
        let conn = db::open_db(&db_path)?;
        db::init_db(&conn)?;

        struct Row {
            context_input_tokens: i64,
            context_window_size: i64,
            session_id: Option<String>,
            captured_at: String,
        }

        let result: Option<Row> = {
            let mut stmt = conn.prepare(
                "SELECT context_input_tokens, context_window_size, session_id, received_at
                 FROM live_events
                 WHERE context_input_tokens IS NOT NULL AND context_window_size > 0
                 ORDER BY received_at DESC LIMIT 1",
            )?;
            stmt.query_row([], |r| {
                Ok(Row {
                    context_input_tokens: r.get(0)?,
                    context_window_size: r.get(1)?,
                    session_id: r.get(2)?,
                    captured_at: r.get(3)?,
                })
            })
            .optional()
            .map_err(anyhow::Error::from)?
        };

        Ok(match result {
            None => ContextWindowApiResponse {
                enabled: Some(false),
                ..Default::default()
            },
            Some(row) => {
                let pct = row.context_input_tokens as f64 / row.context_window_size as f64;
                ContextWindowApiResponse {
                    enabled: None,
                    total_input_tokens: Some(row.context_input_tokens),
                    context_window_size: Some(row.context_window_size),
                    pct: Some(pct),
                    severity: Some(
                        match severity_for_pct(pct) {
                            crate::analytics::quota::Severity::Ok => "ok",
                            crate::analytics::quota::Severity::Warn => "warn",
                            crate::analytics::quota::Severity::Danger => "danger",
                        }
                        .into(),
                    ),
                    session_id: row.session_id,
                    captured_at: Some(row.captured_at),
                }
            }
        })
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// `GET /api/cost-reconciliation?period=<day|week|month>`
///
/// Returns hook-reported vs. local-estimate cost totals by day for the
/// requested rolling period. When no hook costs have ever been recorded,
/// returns `{ "enabled": false }`.
///
/// Full response shape:
/// ```json
/// {
///   "enabled": true,
///   "period": "month",
///   "hook_total_nanos": 12345000000,
///   "local_total_nanos": 14567000000,
///   "divergence_pct": 0.15,
///   "breakdown": [
///     { "day": "2026-04-01", "hook_nanos": 5000000000, "local_nanos": 6000000000 },
///     ...
///   ]
/// }
/// ```
pub async fn api_cost_reconciliation(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CostReconciliationParams>,
    request: Request,
) -> Result<Json<Value>, StatusCode> {
    enforce_loopback_request(&request)?;
    let _db_guard = state.db_lock.lock().await;
    let db_path = state.db_path.clone();
    let period = params.period.clone();

    let value = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = db::open_db(&db_path)?;
        db::init_db(&conn)?;

        // Check if any hook costs exist at all.
        let hook_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM live_events WHERE hook_reported_cost_nanos IS NOT NULL",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        if hook_count == 0 {
            return Ok(serde_json::json!({ "enabled": false }));
        }

        // Determine rolling period bounds (last N days including today).
        let days_back: i64 = match period.as_str() {
            "day" => 1,
            "week" => 7,
            _ => 30, // "month" default
        };

        let now = chrono::Utc::now();
        let cutoff = (now - chrono::Duration::days(days_back)).to_rfc3339();

        // Sum hook_reported_cost_nanos by date from live_events.
        let mut hook_by_day: std::collections::HashMap<String, i64> =
            std::collections::HashMap::new();
        {
            let mut stmt = conn.prepare(
                "SELECT date(received_at) AS day,
                        COALESCE(SUM(hook_reported_cost_nanos), 0) AS nanos
                 FROM live_events
                 WHERE hook_reported_cost_nanos IS NOT NULL
                   AND received_at >= ?1
                 GROUP BY day
                 ORDER BY day",
            )?;
            let rows = stmt.query_map(rusqlite::params![cutoff], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            })?;
            for row in rows {
                let (day, nanos) = row?;
                hook_by_day.insert(day, nanos);
            }
        }

        // Sum estimated_cost_nanos by date from turns.
        let mut local_by_day: std::collections::HashMap<String, i64> =
            std::collections::HashMap::new();
        {
            let mut stmt = conn.prepare(
                "SELECT date(timestamp) AS day,
                        COALESCE(SUM(estimated_cost_nanos), 0) AS nanos
                 FROM turns
                 WHERE timestamp >= ?1
                 GROUP BY day
                 ORDER BY day",
            )?;
            let rows = stmt.query_map(rusqlite::params![cutoff], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            })?;
            for row in rows {
                let (day, nanos) = row?;
                local_by_day.insert(day, nanos);
            }
        }

        // Build unified day list (union of both maps).
        let mut all_days: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        all_days.extend(hook_by_day.keys().cloned());
        all_days.extend(local_by_day.keys().cloned());

        let breakdown: Vec<Value> = all_days
            .iter()
            .map(|day| {
                let hook_nanos = hook_by_day.get(day).copied().unwrap_or(0);
                let local_nanos = local_by_day.get(day).copied().unwrap_or(0);
                serde_json::json!({
                    "day": day,
                    "hook_nanos": hook_nanos,
                    "local_nanos": local_nanos,
                })
            })
            .collect();

        let hook_total_nanos: i64 = hook_by_day.values().sum();
        let local_total_nanos: i64 = local_by_day.values().sum();

        let divergence_pct = if local_total_nanos > 0 {
            (hook_total_nanos - local_total_nanos) as f64 / local_total_nanos as f64
        } else {
            0.0
        };

        Ok(serde_json::json!({
            "enabled": true,
            "period": period,
            "hook_total_nanos": hook_total_nanos,
            "local_total_nanos": local_total_nanos,
            "divergence_pct": divergence_pct,
            "breakdown": breakdown,
        }))
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(value))
}

#[derive(serde::Deserialize, Default)]
pub struct CostReconciliationParams {
    /// day | week | month
    #[serde(default = "default_reconciliation_period")]
    pub period: String,
}

fn default_reconciliation_period() -> String {
    "month".to_string()
}

fn sqlite_sidecar_paths(path: &std::path::Path) -> [PathBuf; 2] {
    [
        PathBuf::from(format!("{}-wal", path.to_string_lossy())),
        PathBuf::from(format!("{}-shm", path.to_string_lossy())),
    ]
}

fn cleanup_sqlite_files(path: &std::path::Path) -> std::io::Result<()> {
    for candidate in std::iter::once(path.to_path_buf()).chain(sqlite_sidecar_paths(path)) {
        match std::fs::remove_file(&candidate) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

fn replace_sqlite_files(
    temp_path: &std::path::Path,
    db_path: &std::path::Path,
) -> std::io::Result<()> {
    cleanup_sqlite_files(db_path)?;
    std::fs::rename(temp_path, db_path)?;

    for (src, dst) in sqlite_sidecar_paths(temp_path)
        .into_iter()
        .zip(sqlite_sidecar_paths(db_path))
    {
        if src.exists() {
            std::fs::rename(src, dst)?;
        }
    }
    Ok(())
}

fn preserve_runtime_history(
    temp_path: &std::path::Path,
    db_path: &std::path::Path,
) -> anyhow::Result<()> {
    if !db_path.exists() {
        return Ok(());
    }

    let live_conn = db::open_db(db_path)?;
    db::init_db(&live_conn)?;
    drop(live_conn);

    let conn = db::open_db(temp_path)?;
    db::init_db(&conn)?;

    conn.execute(
        "ATTACH DATABASE ?1 AS live_db",
        [db_path.to_string_lossy().as_ref()],
    )?;

    conn.execute_batch(
        "INSERT OR IGNORE INTO live_events
             (dedup_key, received_at, session_id, tool_name, cost_usd_nanos,
              input_tokens, output_tokens, raw_json, context_input_tokens,
              context_window_size, hook_reported_cost_nanos)
         SELECT dedup_key, received_at, session_id, tool_name, cost_usd_nanos,
                input_tokens, output_tokens, raw_json, context_input_tokens,
                context_window_size, hook_reported_cost_nanos
         FROM live_db.live_events;

         INSERT OR IGNORE INTO agent_status_history
             (ts_epoch, provider, component_id, component_name, status)
         SELECT ts_epoch, provider, component_id, component_name, status
         FROM live_db.agent_status_history;",
    )?;

    conn.execute_batch(
        "INSERT OR IGNORE INTO claude_usage_runs
             (id, captured_at, status, exit_code, stdout_raw, stderr_raw, invocation_mode, period, parser_version, error_summary)
         SELECT id, captured_at, status, exit_code, stdout_raw, stderr_raw, invocation_mode, period, parser_version, error_summary
         FROM live_db.claude_usage_runs;

         INSERT OR IGNORE INTO claude_usage_factors
             (id, run_id, factor_key, display_label, percent, description, advice_text, display_order)
         SELECT id, run_id, factor_key, display_label, percent, description, advice_text, display_order
         FROM live_db.claude_usage_factors;",
    )?;

    conn.execute(
        "INSERT INTO rate_window_history
             (timestamp, window_type, used_percent, resets_at, source_kind, source_path)
         SELECT lr.timestamp,
                lr.window_type,
                lr.used_percent,
                lr.resets_at,
                COALESCE(lr.source_kind, 'oauth'),
                COALESCE(lr.source_path, '')
         FROM live_db.rate_window_history lr
         WHERE COALESCE(lr.source_kind, 'oauth') = 'oauth'
           AND NOT EXISTS (
               SELECT 1
               FROM rate_window_history cur
               WHERE cur.timestamp = lr.timestamp
                 AND cur.window_type = lr.window_type
                 AND ABS(cur.used_percent - lr.used_percent) < 0.000001
                 AND (
                     (cur.resets_at IS NULL AND lr.resets_at IS NULL)
                     OR cur.resets_at = lr.resets_at
                 )
                 AND cur.source_kind = COALESCE(lr.source_kind, 'oauth')
                 AND cur.source_path = COALESCE(lr.source_path, '')
           )",
        [],
    )?;

    conn.execute_batch("DETACH DATABASE live_db;")?;
    Ok(())
}

async fn maybe_send_session_webhook(
    state: &Arc<AppState>,
    used_percent: f64,
    resets_in_minutes: Option<i64>,
) {
    let mut webhook_state = state.webhook_state.lock().await;
    if let Some(event) = webhooks::session_transition_event(
        &state.webhook_config,
        &mut webhook_state,
        used_percent,
        resets_in_minutes,
    ) {
        webhooks::notify_if_configured(&state.webhook_config, event);
    }
}

async fn maybe_send_cost_threshold_webhook(
    state: &Arc<AppState>,
    data: &crate::models::DashboardData,
) {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let daily_cost: f64 = data
        .daily_by_model
        .iter()
        .filter(|row| row.day == today)
        .map(|row| row.cost)
        .sum();

    let mut webhook_state = state.webhook_state.lock().await;
    if let Some(event) = webhooks::cost_threshold_event(
        &state.webhook_config,
        &mut webhook_state,
        &today,
        daily_cost,
    ) {
        webhooks::notify_if_configured(&state.webhook_config, event);
    }
}

/// `GET /api/community-signal` — returns the latest crowd-sourced signal snapshot.
///
/// When `aggregator_config.enabled` is false, returns `{"enabled": false}` with
/// status 200 and makes no network calls.
///
/// When enabled, uses a TTL cache (default 300s). On cache miss, fetches fresh
/// data from the configured backend (StatusGator), fires divergence webhooks if
/// the crowd signal spikes while the official indicator is below Major, and
/// updates the cache.
pub async fn api_community_signal(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<Value>, StatusCode> {
    enforce_loopback_request(&request)?;
    let Some(signal) = refresh_community_signal(&state).await? else {
        return Ok(Json(serde_json::json!({ "enabled": false })));
    };

    let value = serde_json::to_value(&signal).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(value))
}

pub(crate) async fn refresh_community_signal(
    state: &Arc<AppState>,
) -> Result<Option<CommunitySignal>, StatusCode> {
    if !state.aggregator_config.enabled {
        return Ok(None);
    }

    {
        let cache = state.aggregator_cache.read().await;
        if let Some((fetched_at, ref data)) = *cache
            && fetched_at.elapsed().as_secs() < state.aggregator_config.refresh_interval
        {
            return Ok(Some(data.clone()));
        }
    }

    let _refresh_guard = state.aggregator_refresh_lock.lock().await;
    {
        let cache = state.aggregator_cache.read().await;
        if let Some((fetched_at, ref data)) = *cache
            && fetched_at.elapsed().as_secs() < state.aggregator_config.refresh_interval
        {
            return Ok(Some(data.clone()));
        }
    }

    let config = state.aggregator_config.clone();
    let signal = tokio::task::spawn_blocking(move || status_aggregator::poll(&config))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    maybe_send_community_spike_webhooks(state, &signal).await;

    let cached = {
        let cache = state.aggregator_cache.read().await;
        cache.as_ref().map(|(_, data)| data.clone())
    };
    let to_store = if signal.enabled {
        signal.clone()
    } else {
        cached.unwrap_or_else(|| signal.clone())
    };

    {
        let mut cache = state.aggregator_cache.write().await;
        *cache = Some((Instant::now(), to_store.clone()));
    }

    Ok(Some(to_store))
}

/// Fire `community_signal_spike` webhook events when the crowd signal is at
/// Spike level while the official agent-status indicator is below Major.
///
/// Deduplication: per-provider boolean on `WebhookState` — fires only on the
/// `false → true` transition, then silences until the crowd normalises or the
/// official page catches up.
async fn maybe_send_community_spike_webhooks(state: &Arc<AppState>, signal: &CommunitySignal) {
    use crate::status_aggregator::models::SignalLevel;

    // Read the current official agent-status indicator from the cached snapshot.
    let (claude_official_major, openai_official_major) = {
        let cache = state.agent_status_cache.read().await;
        let snap = cache.as_ref().map(|(_, s, _)| s.clone());
        let claude_major = snap
            .as_ref()
            .and_then(|s| s.claude.as_ref())
            .map(|p| p.indicator.is_alert_worthy())
            .unwrap_or(false);
        let openai_major = snap
            .as_ref()
            .and_then(|s| s.openai.as_ref())
            .map(|p| p.indicator.is_alert_worthy())
            .unwrap_or(false);
        (claude_major, openai_major)
    };

    let claude_spike = signal.claude.iter().any(|s| s.level == SignalLevel::Spike);
    let openai_spike = signal.openai.iter().any(|s| s.level == SignalLevel::Spike);

    let mut webhook_state = state.webhook_state.lock().await;

    if let Some(event) = webhooks::community_signal_spike_event(
        &state.webhook_config,
        &mut webhook_state.claude_community_spike,
        "Claude",
        claude_spike,
        claude_official_major,
    ) {
        webhooks::notify_if_configured(&state.webhook_config, event);
    }

    if let Some(event) = webhooks::community_signal_spike_event(
        &state.webhook_config,
        &mut webhook_state.openai_community_spike,
        "OpenAI",
        openai_spike,
        openai_official_major,
    ) {
        webhooks::notify_if_configured(&state.webhook_config, event);
    }
}
