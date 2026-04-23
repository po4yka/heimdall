use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow, bail};

use crate::agent_status::models::ProviderStatus;
use crate::analytics::blocks::identify_blocks;
use crate::analytics::depletion::{
    billing_block_signal, build_depletion_forecast, primary_window_signal, secondary_window_signal,
};
use crate::analytics::predictive::compute_predictive_insights;
use crate::analytics::quota::compute_quota_suggestions;
use crate::models::{
    LIVE_PROVIDERS_CONTRACT_VERSION, LiveFloatPercentiles, LiveHistoricalEnvelope,
    LiveIntegerPercentiles, LiveLimitHitAnalysis, LivePredictiveBurnRate, LivePredictiveInsights,
    LiveProviderHistoryResponse, LiveProviderIdentity, LiveProviderSnapshot,
    LiveProviderSourceAttempt, LiveProviderStatus, LiveProvidersResponse, LiveQuotaSuggestionLevel,
    LiveQuotaSuggestions, LocalNotificationCondition, LocalNotificationState,
    MOBILE_SNAPSHOT_CONTRACT_VERSION, MobileProviderHistorySeries, MobileSnapshotEnvelope,
    MobileSnapshotFreshness, MobileSnapshotTotals, ProviderCostSummary,
};
use crate::oauth::credentials;
use crate::oauth::models::{BudgetInfo, Identity, Plan, UsageWindowsResponse, WindowInfo};
use crate::scanner::db;
use crate::server::api::{
    AppState, refresh_agent_status, refresh_community_signal, refresh_usage_windows,
};
use crate::status_aggregator::models::{CommunitySignal, SignalLevel};
use crate::tz::TzParams;

pub mod codex;

const LIVE_PROVIDER_CACHE_SECS: u64 = 60;
const ALL_PROVIDERS: [&str; 2] = ["claude", "codex"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseScope {
    All,
    ProviderOnly,
}

impl ResponseScope {
    fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::ProviderOnly => "provider",
        }
    }
}

#[derive(Debug, Clone)]
struct CodexLiveResolution {
    available: bool,
    source_used: String,
    last_attempted_source: Option<String>,
    resolved_via_fallback: bool,
    refresh_duration_ms: u64,
    source_attempts: Vec<LiveProviderSourceAttempt>,
    identity: Option<LiveProviderIdentity>,
    primary: Option<crate::models::LiveRateWindow>,
    secondary: Option<crate::models::LiveRateWindow>,
    credits: Option<f64>,
    error: Option<String>,
    auth: Option<codex::CodexAuth>,
    credential_store: codex::ResolvedCodexCredentialStore,
    bootstrap_error: Option<String>,
}

pub async fn load_snapshots(
    state: &Arc<AppState>,
    requested_provider: Option<&str>,
    scope: ResponseScope,
    force_refresh: bool,
    startup: bool,
) -> Result<LiveProvidersResponse> {
    let requested_provider = normalize_provider(requested_provider)?.map(str::to_string);
    load_snapshots_with_fetcher(
        state,
        requested_provider.clone(),
        scope,
        force_refresh,
        startup,
        |provider, scope, force_refresh, startup| async move {
            fetch_live_provider_response(state, provider.as_deref(), scope, force_refresh, startup)
                .await
        },
    )
    .await
}

async fn load_snapshots_with_fetcher<F, Fut>(
    state: &Arc<AppState>,
    requested_provider: Option<String>,
    scope: ResponseScope,
    force_refresh: bool,
    startup: bool,
    fetcher: F,
) -> Result<LiveProvidersResponse>
where
    F: Fn(Option<String>, ResponseScope, bool, bool) -> Fut,
    Fut: Future<Output = Result<LiveProvidersResponse>>,
{
    if startup {
        if let Some(cached) = cached_response(state).await {
            return Ok(filter_response(
                &cached,
                requested_provider.as_deref(),
                scope,
                true,
            ));
        }
        if let Some(cached) = cached_response_any(state).await {
            return Ok(filter_response(
                &cached,
                requested_provider.as_deref(),
                scope,
                true,
            ));
        }

        return fetcher(requested_provider, scope, force_refresh, true).await;
    }

    if !force_refresh && let Some(cached) = cached_response(state).await {
        return Ok(filter_response(
            &cached,
            requested_provider.as_deref(),
            scope,
            true,
        ));
    }

    let mut waited_for_refresh = false;
    let _refresh_guard = match state.live_provider_refresh_lock.try_lock() {
        Ok(guard) => guard,
        Err(_) => {
            if !force_refresh && let Some(cached) = cached_response_any(state).await {
                return Ok(filter_response(
                    &cached,
                    requested_provider.as_deref(),
                    scope,
                    true,
                ));
            }
            waited_for_refresh = true;
            state.live_provider_refresh_lock.lock().await
        }
    };

    if !force_refresh && let Some(cached) = cached_response(state).await {
        return Ok(filter_response(
            &cached,
            requested_provider.as_deref(),
            scope,
            true,
        ));
    }

    if waited_for_refresh && let Some(cached) = cached_response_any(state).await {
        return Ok(filter_response(
            &cached,
            requested_provider.as_deref(),
            scope,
            true,
        ));
    }

    let response = fetcher(requested_provider.clone(), scope, force_refresh, false).await?;
    update_cache_after_fetch(state, requested_provider.as_deref(), scope, &response).await;
    Ok(response)
}

pub async fn load_provider_cost_summary(
    state: &Arc<AppState>,
    provider: &str,
) -> Result<ProviderCostSummary> {
    let provider = normalize_provider(Some(provider))?
        .ok_or_else(|| anyhow!("missing provider"))?
        .to_string();
    let db_path = state.db_path.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db::open_db(&db_path)?;
        db::init_db(&conn)?;
        provider_cost_summary(&conn, &provider)
    })
    .await
    .map_err(anyhow::Error::from)?
}

pub async fn load_provider_history(
    state: &Arc<AppState>,
    provider: &str,
) -> Result<LiveProviderHistoryResponse> {
    let provider =
        normalize_provider(Some(provider))?.ok_or_else(|| anyhow!("missing provider"))?;
    let summary = load_provider_cost_summary(state, provider).await?;
    Ok(LiveProviderHistoryResponse {
        provider: provider.to_string(),
        summary,
    })
}

pub async fn load_mobile_snapshot(state: &Arc<AppState>) -> Result<MobileSnapshotEnvelope> {
    let response = load_snapshots(state, None, ResponseScope::All, false, false).await?;
    let providers = response.providers.clone();
    let provider_names = providers
        .iter()
        .map(|provider| provider.provider.clone())
        .collect::<Vec<_>>();
    let db_path = state.db_path.clone();

    let (history_90d, totals) = tokio::task::spawn_blocking(move || {
        let conn = db::open_db(&db_path)?;
        db::init_db(&conn)?;
        build_mobile_history_and_totals(&conn, &providers, &provider_names)
    })
    .await
    .map_err(anyhow::Error::from)??;

    Ok(build_mobile_snapshot(response, history_90d, totals))
}

fn build_mobile_snapshot(
    response: LiveProvidersResponse,
    history_90d: Vec<MobileProviderHistorySeries>,
    totals: MobileSnapshotTotals,
) -> MobileSnapshotEnvelope {
    let freshest_provider_refresh = response
        .providers
        .iter()
        .map(|provider| provider.last_refresh.clone())
        .max();
    let oldest_provider_refresh = response
        .providers
        .iter()
        .map(|provider| provider.last_refresh.clone())
        .min();
    let stale_providers = response
        .providers
        .iter()
        .filter(|provider| provider.stale)
        .map(|provider| provider.provider.clone())
        .collect::<Vec<_>>();
    let generated_at = chrono::Utc::now().to_rfc3339();

    MobileSnapshotEnvelope {
        contract_version: MOBILE_SNAPSHOT_CONTRACT_VERSION,
        generated_at: generated_at.clone(),
        source_device: source_device_name(),
        providers: response.providers,
        history_90d,
        totals,
        freshness: MobileSnapshotFreshness {
            newest_provider_refresh: freshest_provider_refresh,
            oldest_provider_refresh,
            has_stale_providers: !stale_providers.is_empty(),
            stale_providers,
        },
    }
}

fn build_mobile_history_and_totals(
    conn: &rusqlite::Connection,
    providers: &[LiveProviderSnapshot],
    provider_names: &[String],
) -> Result<(Vec<MobileProviderHistorySeries>, MobileSnapshotTotals)> {
    let start_90d = (chrono::Utc::now().date_naive() - chrono::Duration::days(89)).to_string();

    let mut history_90d = Vec::with_capacity(provider_names.len());
    let mut totals = MobileSnapshotTotals::default();

    for provider in providers {
        totals.today_tokens += provider.cost_summary.today_tokens;
        totals.today_cost_usd += provider.cost_summary.today_cost_usd;
        totals
            .today_breakdown
            .accumulate(&provider.cost_summary.today_breakdown);
    }

    for provider in provider_names {
        let (cost_90d_nanos, tokens_90d, breakdown_90d) =
            db::get_provider_cost_summary_since(conn, provider, &start_90d)?;
        let daily = db::get_provider_daily_cost_history_since(conn, provider, &start_90d)?;

        totals.last_90_days_tokens += tokens_90d;
        totals.last_90_days_cost_usd += cost_90d_nanos as f64 / 1_000_000_000.0;
        totals.last_90_days_breakdown.accumulate(&breakdown_90d);

        history_90d.push(MobileProviderHistorySeries {
            provider: provider.clone(),
            daily,
            total_tokens: tokens_90d,
            total_cost_usd: cost_90d_nanos as f64 / 1_000_000_000.0,
        });
    }

    Ok((history_90d, totals))
}

fn source_device_name() -> String {
    std::env::var("HEIMDALL_SOURCE_DEVICE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .or_else(|| std::env::var("COMPUTERNAME").ok())
        .unwrap_or_else(|| "unknown-device".to_string())
}

async fn fetch_live_provider_response(
    state: &Arc<AppState>,
    requested_provider: Option<&str>,
    scope: ResponseScope,
    force_refresh: bool,
    startup: bool,
) -> Result<LiveProvidersResponse> {
    let providers_to_build: Vec<&str> = match (requested_provider, scope, force_refresh) {
        (Some(provider), ResponseScope::ProviderOnly, _) => vec![provider],
        _ => ALL_PROVIDERS.to_vec(),
    };

    let agent_status = if startup {
        cached_agent_status(state).await
    } else if providers_to_build
        .iter()
        .any(|provider| *provider == "claude" || *provider == "codex")
    {
        refresh_agent_status(state).await.ok()
    } else {
        None
    };
    let claude_usage = if startup && providers_to_build.contains(&"claude") {
        Some(cached_or_unavailable_claude_usage(state).await)
    } else if providers_to_build.contains(&"claude") {
        Some(refresh_usage_windows(state).await)
    } else {
        None
    };
    let community_signal = if startup {
        cached_community_signal(state).await
    } else if state.aggregator_config.enabled {
        refresh_community_signal(state).await.ok().flatten()
    } else {
        None
    };

    let mut providers = Vec::with_capacity(providers_to_build.len());
    for provider in providers_to_build.iter().copied() {
        match provider {
            "claude" => {
                let snapshot = build_claude_snapshot(
                    state,
                    claude_usage
                        .as_ref()
                        .ok_or_else(|| anyhow!("missing Claude usage snapshot"))?,
                    agent_status
                        .as_ref()
                        .and_then(|status| status.claude.as_ref()),
                    state.session_length_hours,
                )
                .await?;
                providers.push(snapshot);
            }
            "codex" => {
                let snapshot = if startup {
                    build_codex_bootstrap_snapshot(
                        state,
                        agent_status
                            .as_ref()
                            .and_then(|status| status.openai.as_ref()),
                    )
                    .await?
                } else {
                    build_codex_snapshot(
                        state,
                        agent_status
                            .as_ref()
                            .and_then(|status| status.openai.as_ref()),
                    )
                    .await?
                };
                providers.push(snapshot);
            }
            other => bail!("unsupported live provider: {}", other),
        }
    }
    let local_notification_state = build_local_notification_state(
        state,
        agent_status.as_ref(),
        claude_usage.as_ref(),
        community_signal.as_ref(),
    )
    .await?;

    Ok(LiveProvidersResponse {
        contract_version: LIVE_PROVIDERS_CONTRACT_VERSION,
        providers,
        fetched_at: chrono::Utc::now().to_rfc3339(),
        requested_provider: requested_provider.map(ToOwned::to_owned),
        response_scope: scope.as_str().to_string(),
        cache_hit: false,
        refreshed_providers: if force_refresh {
            providers_to_build
                .iter()
                .map(|provider| (*provider).to_string())
                .collect()
        } else {
            Vec::new()
        },
        local_notification_state: Some(local_notification_state),
    })
}

async fn cached_agent_status(
    state: &Arc<AppState>,
) -> Option<crate::agent_status::models::AgentStatusSnapshot> {
    let cache = state.agent_status_cache.read().await;
    cache.as_ref().map(|(_, snapshot, _)| snapshot.clone())
}

async fn cached_or_unavailable_claude_usage(state: &Arc<AppState>) -> UsageWindowsResponse {
    let cache = state.oauth_cache.read().await;
    cache
        .as_ref()
        .map(|(_, response)| response.clone())
        .unwrap_or_else(UsageWindowsResponse::unavailable)
}

async fn cached_community_signal(state: &Arc<AppState>) -> Option<CommunitySignal> {
    if !state.aggregator_config.enabled {
        return None;
    }

    let cache = state.aggregator_cache.read().await;
    cache.as_ref().map(|(_, signal)| signal.clone())
}

async fn build_local_notification_state(
    state: &Arc<AppState>,
    agent_status: Option<&crate::agent_status::models::AgentStatusSnapshot>,
    claude_usage: Option<&UsageWindowsResponse>,
    community_signal: Option<&CommunitySignal>,
) -> Result<LocalNotificationState> {
    let generated_at = chrono::Utc::now().to_rfc3339();
    let cost_threshold_usd = state.webhook_config.cost_threshold;
    let mut conditions = Vec::new();

    if let Some(session) = claude_usage.and_then(|usage| usage.session.as_ref()) {
        let depleted = session.used_percent >= 99.99;
        conditions.push(LocalNotificationCondition {
            id: "claude-session-depleted".into(),
            kind: "session_depleted".into(),
            provider: Some("claude".into()),
            service_label: "Claude".into(),
            is_active: depleted,
            activation_title: "Claude session depleted".into(),
            activation_body: match session.resets_in_minutes {
                Some(minutes) => {
                    format!("Claude session is depleted. Resets in {minutes} minutes.")
                }
                None => "Claude session is depleted.".into(),
            },
            recovery_title: Some("Claude session restored".into()),
            recovery_body: Some("Claude session capacity is available again.".into()),
            day_key: None,
        });
    }

    if let Some(status) = agent_status.and_then(|snapshot| snapshot.claude.as_ref()) {
        conditions.push(provider_status_condition(
            "claude-service-degraded",
            "claude",
            "Claude",
            status,
        ));
    }
    if let Some(status) = agent_status.and_then(|snapshot| snapshot.openai.as_ref()) {
        conditions.push(provider_status_condition(
            "codex-service-degraded",
            "codex",
            "OpenAI",
            status,
        ));
    }

    if state.aggregator_config.enabled
        && let Some(signal) = community_signal
        && signal.enabled
    {
        let claude_major = agent_status
            .and_then(|snapshot| snapshot.claude.as_ref())
            .map(|status| status.indicator.is_alert_worthy())
            .unwrap_or(false);
        let codex_major = agent_status
            .and_then(|snapshot| snapshot.openai.as_ref())
            .map(|status| status.indicator.is_alert_worthy())
            .unwrap_or(false);

        conditions.push(community_spike_condition(
            "claude-community-spike",
            "claude",
            "Claude",
            &signal.claude,
            claude_major,
        ));
        conditions.push(community_spike_condition(
            "codex-community-spike",
            "codex",
            "OpenAI",
            &signal.openai,
            codex_major,
        ));
    }

    if let Some(threshold) = cost_threshold_usd {
        let (day_key, daily_cost_usd) = load_today_cost_snapshot(state).await?;
        conditions.push(LocalNotificationCondition {
            id: "daily-cost-threshold".into(),
            kind: "daily_cost_threshold".into(),
            provider: None,
            service_label: "Heimdall".into(),
            is_active: daily_cost_usd > threshold,
            activation_title: "Daily cost threshold crossed".into(),
            activation_body: format!(
                "Today's Heimdall cost reached ${daily_cost_usd:.2}, above the configured ${threshold:.2} threshold."
            ),
            recovery_title: None,
            recovery_body: None,
            day_key: Some(day_key),
        });
    }

    Ok(LocalNotificationState {
        generated_at,
        cost_threshold_usd,
        conditions,
    })
}

fn provider_status_condition(
    id: &str,
    provider: &str,
    service_label: &str,
    status: &ProviderStatus,
) -> LocalNotificationCondition {
    let active = status.indicator.is_alert_worthy();
    let activation_title = if provider == "codex" {
        "Codex service degraded"
    } else {
        "Claude service degraded"
    };
    let recovery_title = if provider == "codex" {
        "Codex service restored"
    } else {
        "Claude service restored"
    };

    LocalNotificationCondition {
        id: id.into(),
        kind: "provider_degraded".into(),
        provider: Some(provider.into()),
        service_label: service_label.into(),
        is_active: active,
        activation_title: activation_title.into(),
        activation_body: if provider == "codex" {
            format!(
                "OpenAI status is degraded for Codex. {}",
                status.description
            )
        } else {
            format!("Claude status is degraded. {}", status.description)
        },
        recovery_title: Some(recovery_title.into()),
        recovery_body: Some(if provider == "codex" {
            "OpenAI status returned to a non-alert state for Codex.".into()
        } else {
            "Claude status returned to a non-alert state.".into()
        }),
        day_key: None,
    }
}

fn community_spike_condition(
    id: &str,
    provider: &str,
    service_label: &str,
    signals: &[crate::status_aggregator::models::ServiceSignal],
    official_is_major: bool,
) -> LocalNotificationCondition {
    let is_spike = signals
        .iter()
        .any(|signal| signal.level == SignalLevel::Spike);
    let is_active = is_spike && !official_is_major;
    let activation_title = if provider == "codex" {
        "Codex community spike detected"
    } else {
        "Claude community spike detected"
    };

    LocalNotificationCondition {
        id: id.into(),
        kind: "community_spike".into(),
        provider: Some(provider.into()),
        service_label: service_label.into(),
        is_active,
        activation_title: activation_title.into(),
        activation_body: if provider == "codex" {
            "OpenAI community reports spiked while official status remains below major.".into()
        } else {
            "Claude community reports spiked while official status remains below major.".into()
        },
        recovery_title: None,
        recovery_body: None,
        day_key: None,
    }
}

async fn load_today_cost_snapshot(state: &Arc<AppState>) -> Result<(String, f64)> {
    let db_path = state.db_path.clone();
    tokio::task::spawn_blocking(move || -> Result<(String, f64)> {
        let conn = db::open_db(&db_path)?;
        db::init_db(&conn)?;

        let now = chrono::Local::now();
        let tz = TzParams {
            tz_offset_min: Some(now.offset().local_minus_utc() / 60),
            week_starts_on: None,
        };
        let today = now.format("%Y-%m-%d").to_string();
        let data = db::get_dashboard_data(&conn, tz)?;
        let daily_cost_usd = data
            .daily_by_model
            .iter()
            .filter(|row| row.day == today)
            .map(|row| row.cost)
            .sum();

        Ok((today, daily_cost_usd))
    })
    .await
    .map_err(|err| anyhow!("failed to load daily cost snapshot: {}", err))?
}

async fn cached_response(state: &Arc<AppState>) -> Option<LiveProvidersResponse> {
    let cache = state.live_provider_cache.read().await;
    match &*cache {
        Some((fetched_at, cached))
            if fetched_at.elapsed() < Duration::from_secs(LIVE_PROVIDER_CACHE_SECS) =>
        {
            Some(cached.clone())
        }
        _ => None,
    }
}

async fn cached_response_any(state: &Arc<AppState>) -> Option<LiveProvidersResponse> {
    let cache = state.live_provider_cache.read().await;
    cache.as_ref().map(|(_, cached)| cached.clone())
}

async fn update_cache_after_fetch(
    state: &Arc<AppState>,
    requested_provider: Option<&str>,
    scope: ResponseScope,
    response: &LiveProvidersResponse,
) {
    let mut cache = state.live_provider_cache.write().await;

    if is_full_response(response) {
        *cache = Some((Instant::now(), cacheable_response(response)));
        return;
    }

    if requested_provider.is_some()
        && scope == ResponseScope::ProviderOnly
        && let Some((fetched_at, cached)) = &mut *cache
    {
        merge_provider_snapshot(cached, response);
        *fetched_at = Instant::now();
    }
}

fn is_full_response(response: &LiveProvidersResponse) -> bool {
    response.providers.len() == ALL_PROVIDERS.len()
        && response
            .providers
            .iter()
            .all(|snapshot| ALL_PROVIDERS.contains(&snapshot.provider.as_str()))
}

fn cacheable_response(response: &LiveProvidersResponse) -> LiveProvidersResponse {
    let mut cached = response.clone();
    cached.requested_provider = None;
    cached.response_scope = ResponseScope::All.as_str().to_string();
    cached.cache_hit = false;
    cached.refreshed_providers.clear();
    cached
}

fn merge_provider_snapshot(base: &mut LiveProvidersResponse, update: &LiveProvidersResponse) {
    for snapshot in &update.providers {
        if let Some(existing) = base
            .providers
            .iter_mut()
            .find(|candidate| candidate.provider == snapshot.provider)
        {
            *existing = snapshot.clone();
        } else {
            base.providers.push(snapshot.clone());
        }
    }
    sort_snapshots(&mut base.providers);
    base.fetched_at = chrono::Utc::now().to_rfc3339();
    base.local_notification_state = update.local_notification_state.clone();
}

fn filter_response(
    response: &LiveProvidersResponse,
    requested_provider: Option<&str>,
    scope: ResponseScope,
    cache_hit: bool,
) -> LiveProvidersResponse {
    let providers = match (requested_provider, scope) {
        (Some(provider), ResponseScope::ProviderOnly) => response
            .providers
            .iter()
            .filter(|snapshot| snapshot.provider == provider)
            .cloned()
            .collect(),
        _ => response.providers.clone(),
    };

    LiveProvidersResponse {
        contract_version: response.contract_version,
        providers,
        fetched_at: response.fetched_at.clone(),
        requested_provider: requested_provider.map(ToOwned::to_owned),
        response_scope: scope.as_str().to_string(),
        cache_hit,
        refreshed_providers: if cache_hit {
            Vec::new()
        } else {
            response.refreshed_providers.clone()
        },
        local_notification_state: response.local_notification_state.clone(),
    }
}

fn sort_snapshots(snapshots: &mut [LiveProviderSnapshot]) {
    snapshots.sort_by_key(|snapshot| match snapshot.provider.as_str() {
        "claude" => 0,
        "codex" => 1,
        _ => 2,
    });
}

fn normalize_provider(provider: Option<&str>) -> Result<Option<&'static str>> {
    match provider {
        None => Ok(None),
        Some("claude") => Ok(Some("claude")),
        Some("codex") => Ok(Some("codex")),
        Some(other) => bail!("unsupported live provider: {}", other),
    }
}

async fn build_claude_snapshot(
    state: &Arc<AppState>,
    usage: &UsageWindowsResponse,
    status: Option<&ProviderStatus>,
    session_length_hours: f64,
) -> Result<LiveProviderSnapshot> {
    let started_at = Instant::now();
    let db_path = state.db_path.clone();
    let blocks_token_limit = state.blocks_token_limit;
    let usage_clone = usage.clone();
    let status = status.cloned();
    let env = std::env::vars().collect::<Vec<_>>();
    let resolved_auth = credentials::resolve_auth(&env);
    let auth = resolved_auth.health.clone();
    let resolved_identity = resolved_auth.identity.clone();
    tokio::task::spawn_blocking(move || {
        use crate::analytics::blocks::{calculate_burn_rate, project_block_usage};
        use crate::analytics::quota::compute_quota;

        let conn = db::open_db(&db_path)?;
        db::init_db(&conn)?;
        let claude_usage = db::get_latest_claude_usage_response(&conn)?.latest_snapshot;
        let cost_summary = provider_cost_summary(&conn, "claude")?;
        let turns = db::load_all_turns(&conn)?;
        let blocks = identify_blocks(&turns, session_length_hours);
        let now = chrono::Utc::now();
        let active_projection = blocks
            .iter()
            .find(|block| block.is_active && !block.is_gap)
            .map(|block| {
                let projection = project_block_usage(block, calculate_burn_rate(block, now), now);
                projection.projected_tokens as i64
            });
        let quota_suggestions =
            compute_quota_suggestions(&blocks, blocks_token_limit).map(live_quota_suggestions);
        let primary = usage_clone.session.as_ref().map(window_to_live);
        let secondary = usage_clone.weekly.as_ref().map(window_to_live);
        let depletion_forecast = {
            let mut signals = Vec::new();
            if let Some(limit) = blocks_token_limit
                && let Some(active_block) =
                    blocks.iter().find(|block| block.is_active && !block.is_gap)
            {
                let projection =
                    project_block_usage(active_block, calculate_burn_rate(active_block, now), now);
                if let Some(quota) = compute_quota(active_block, &projection, limit) {
                    signals.push(billing_block_signal(
                        "Billing block",
                        quota.current_pct * 100.0,
                        Some(quota.projected_pct * 100.0),
                        Some(quota.remaining_tokens),
                        Some(100.0 - (quota.current_pct * 100.0)),
                        Some(active_block.end.to_rfc3339()),
                    ));
                }
            }
            if let Some(window) = primary.as_ref() {
                signals.push(primary_window_signal(
                    window.used_percent,
                    Some(100.0 - window.used_percent),
                    window.resets_in_minutes,
                    None,
                    window.resets_at.clone(),
                ));
            }
            if let Some(window) = secondary.as_ref() {
                signals.push(secondary_window_signal(
                    window.used_percent,
                    Some(100.0 - window.used_percent),
                    window.resets_in_minutes,
                    None,
                    window.resets_at.clone(),
                ));
            }
            build_depletion_forecast(signals)
        };
        let predictive_insights =
            compute_predictive_insights(&blocks, blocks_token_limit, active_projection, now)
                .map(live_predictive_insights);

        let mut source_attempts = Vec::new();
        if usage_clone.available {
            source_attempts.push(LiveProviderSourceAttempt {
                source: "oauth".into(),
                outcome: "success".into(),
                message: None,
            });
        } else {
            source_attempts.push(LiveProviderSourceAttempt {
                source: "oauth".into(),
                outcome: if usage_clone.error.is_some() {
                    "error".into()
                } else {
                    "unavailable".into()
                },
                message: usage_clone.error.clone(),
            });
            if claude_usage.is_some() {
                source_attempts.push(LiveProviderSourceAttempt {
                    source: "local".into(),
                    outcome: "success".into(),
                    message: Some("using latest stored /usage factors".into()),
                });
            }
        }

        let source_used = if usage_clone.available {
            "oauth"
        } else if claude_usage.is_some() {
            "local"
        } else {
            "unavailable"
        };
        let last_attempted_source = source_attempts.last().map(|attempt| attempt.source.clone());
        let resolved_via_fallback = source_used == "local";

        Ok(LiveProviderSnapshot {
            provider: "claude".into(),
            available: usage_clone.available || claude_usage.is_some(),
            source_used: source_used.into(),
            last_attempted_source,
            resolved_via_fallback,
            refresh_duration_ms: started_at.elapsed().as_millis() as u64,
            source_attempts,
            identity: usage_clone
                .identity
                .as_ref()
                .map(identity_to_live)
                .or_else(|| resolved_identity.as_ref().map(identity_to_live)),
            primary,
            secondary,
            tertiary: usage_clone
                .weekly_opus
                .as_ref()
                .or(usage_clone.weekly_sonnet.as_ref())
                .map(window_to_live),
            credits: usage_clone.budget.as_ref().map(budget_to_credits),
            status: status.as_ref().map(status_to_live),
            auth,
            cost_summary,
            claude_usage,
            quota_suggestions,
            depletion_forecast,
            predictive_insights,
            last_refresh: chrono::Utc::now().to_rfc3339(),
            stale: !usage_clone.available,
            error: if source_used == "unavailable" {
                usage_clone.error.clone()
            } else {
                None
            },
        })
    })
    .await
    .map_err(anyhow::Error::from)?
}

async fn build_codex_snapshot(
    state: &Arc<AppState>,
    status: Option<&ProviderStatus>,
) -> Result<LiveProviderSnapshot> {
    let cost_summary = load_provider_cost_summary(state, "codex").await?;
    let env = std::env::vars().collect::<Vec<_>>();
    let config_facts = codex::load_config_facts(&env);
    let bootstrap = codex::resolve_bootstrap_auth(&env, &config_facts);
    // Wrap fetch_oauth_usage with a single-shot refresh retry: if the initial
    // call fails with an auth-looking error and we have a refresh_token, ask
    // Codex's OAuth provider for a new access token, persist it back to
    // auth.json only when the resolved credential backend is file-backed, and
    // retry the fetch once with the new creds.
    let env_for_refresh = env.clone();
    let resolution = resolve_codex_live_data_with(
        bootstrap,
        move |auth| {
            let env_for_refresh = env_for_refresh.clone();
            Box::pin(async move {
                match codex::fetch_oauth_usage(auth).await {
                    Ok(response) => Ok(response),
                    Err(err) => {
                        let Some(refresh) = auth.refresh_token.as_deref() else {
                            return Err(err);
                        };
                        if !codex::looks_like_oauth_auth_error(&err) {
                            return Err(err);
                        }
                        match codex::refresh_oauth_token(refresh).await {
                            Ok(tokens) => {
                                let refreshed = codex::apply_refreshed_tokens(auth, &tokens);
                                let config_facts = codex::load_config_facts(&env_for_refresh);
                                let bootstrap =
                                    codex::resolve_bootstrap_auth(&env_for_refresh, &config_facts);
                                if bootstrap.credential_store.persists_to_file()
                                    && let Err(persist_err) =
                                        codex::persist_refreshed_tokens_to_disk(
                                            &env_for_refresh,
                                            &tokens,
                                        )
                                {
                                    tracing::warn!("Codex refresh persist failed: {persist_err:#}");
                                }
                                codex::fetch_oauth_usage(&refreshed).await
                            }
                            Err(refresh_err) => {
                                tracing::warn!("Codex token refresh failed: {refresh_err:#}");
                                Err(err)
                            }
                        }
                    }
                }
            })
        },
        codex::fetch_rpc_snapshot,
        codex::fetch_cli_status,
    )
    .await;
    let auth = codex::build_auth_health(
        &env,
        &config_facts,
        codex::CodexAuthHealthInput {
            credential_store: resolution.credential_store,
            auth: resolution.auth.as_ref(),
            identity: resolution.identity.as_ref(),
            available: resolution.available,
            bootstrap_error: resolution.bootstrap_error.as_deref(),
            error: resolution.error.as_deref(),
        },
    );
    let depletion_forecast = {
        let mut signals = Vec::new();
        if let Some(window) = resolution.primary.as_ref() {
            signals.push(primary_window_signal(
                window.used_percent,
                Some(100.0 - window.used_percent),
                window.resets_in_minutes,
                None,
                window.resets_at.clone(),
            ));
        }
        if let Some(window) = resolution.secondary.as_ref() {
            signals.push(secondary_window_signal(
                window.used_percent,
                Some(100.0 - window.used_percent),
                window.resets_in_minutes,
                None,
                window.resets_at.clone(),
            ));
        }
        build_depletion_forecast(signals)
    };

    Ok(LiveProviderSnapshot {
        provider: "codex".into(),
        available: resolution.available,
        source_used: resolution.source_used,
        last_attempted_source: resolution.last_attempted_source,
        resolved_via_fallback: resolution.resolved_via_fallback,
        refresh_duration_ms: resolution.refresh_duration_ms,
        source_attempts: resolution.source_attempts,
        identity: resolution.identity,
        primary: resolution.primary,
        secondary: resolution.secondary,
        tertiary: None,
        credits: resolution.credits,
        status: status.map(status_to_live),
        auth,
        cost_summary,
        claude_usage: None,
        quota_suggestions: None,
        depletion_forecast,
        predictive_insights: None,
        last_refresh: chrono::Utc::now().to_rfc3339(),
        stale: !resolution.available,
        error: resolution.error,
    })
}

async fn build_codex_bootstrap_snapshot(
    state: &Arc<AppState>,
    status: Option<&ProviderStatus>,
) -> Result<LiveProviderSnapshot> {
    let started_at = Instant::now();
    let cost_summary = load_provider_cost_summary(state, "codex").await?;
    let env = std::env::vars().collect::<Vec<_>>();
    let config_facts = codex::load_config_facts(&env);
    let bootstrap = codex::resolve_bootstrap_auth(&env, &config_facts);
    let identity = bootstrap.auth.as_ref().and_then(codex::decode_identity);
    let source_used = if bootstrap.auth.is_some() {
        "bootstrap"
    } else {
        "unavailable"
    };
    let source_attempts = if let Some(message) = bootstrap.load_error.clone() {
        vec![LiveProviderSourceAttempt {
            source: "oauth-auth".into(),
            outcome: "unavailable".into(),
            message: Some(message),
        }]
    } else {
        Vec::new()
    };
    let auth = codex::build_auth_health(
        &env,
        &config_facts,
        codex::CodexAuthHealthInput {
            credential_store: bootstrap.credential_store,
            auth: bootstrap.auth.as_ref(),
            identity: identity.as_ref(),
            available: false,
            bootstrap_error: bootstrap.load_error.as_deref(),
            error: None,
        },
    );

    Ok(LiveProviderSnapshot {
        provider: "codex".into(),
        available: false,
        source_used: source_used.into(),
        last_attempted_source: source_attempts.last().map(|attempt| attempt.source.clone()),
        resolved_via_fallback: false,
        refresh_duration_ms: started_at.elapsed().as_millis() as u64,
        source_attempts,
        identity,
        primary: None,
        secondary: None,
        tertiary: None,
        credits: None,
        status: status.map(status_to_live),
        auth,
        cost_summary,
        claude_usage: None,
        quota_suggestions: None,
        depletion_forecast: None,
        predictive_insights: None,
        last_refresh: chrono::Utc::now().to_rfc3339(),
        stale: true,
        error: None,
    })
}

fn live_quota_suggestions(
    suggestions: crate::analytics::quota::QuotaSuggestions,
) -> LiveQuotaSuggestions {
    LiveQuotaSuggestions {
        sample_count: suggestions.sample_count,
        population_count: suggestions.population_count,
        recommended_key: suggestions.recommended_key,
        sample_strategy: suggestions.sample_strategy,
        sample_label: suggestions.sample_label,
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

fn live_predictive_insights(
    insights: crate::analytics::predictive::PredictiveInsights,
) -> LivePredictiveInsights {
    LivePredictiveInsights {
        rolling_hour_burn: insights
            .rolling_hour_burn
            .map(|burn| LivePredictiveBurnRate {
                tokens_per_min: burn.tokens_per_min,
                cost_per_hour_nanos: burn.cost_per_hour_nanos,
                coverage_minutes: burn.coverage_minutes,
                tier: burn.tier,
            }),
        historical_envelope: insights
            .historical_envelope
            .map(|envelope| LiveHistoricalEnvelope {
                sample_count: envelope.sample_count,
                tokens: live_integer_percentiles(envelope.tokens),
                cost_usd: live_float_percentiles(envelope.cost_usd),
                turns: live_integer_percentiles(envelope.turns),
            }),
        limit_hit_analysis: insights
            .limit_hit_analysis
            .map(|analysis| LiveLimitHitAnalysis {
                sample_count: analysis.sample_count,
                hit_count: analysis.hit_count,
                hit_rate: analysis.hit_rate,
                threshold_tokens: analysis.threshold_tokens,
                threshold_percent: analysis.threshold_percent,
                active_current_hit: analysis.active_current_hit,
                active_projected_hit: analysis.active_projected_hit,
                risk_level: analysis.risk_level,
                summary_label: analysis.summary_label,
            }),
    }
}

fn live_integer_percentiles(
    percentiles: crate::analytics::predictive::IntegerPercentiles,
) -> LiveIntegerPercentiles {
    LiveIntegerPercentiles {
        average: percentiles.average,
        p50: percentiles.p50,
        p75: percentiles.p75,
        p90: percentiles.p90,
        p95: percentiles.p95,
    }
}

fn live_float_percentiles(
    percentiles: crate::analytics::predictive::FloatPercentiles,
) -> LiveFloatPercentiles {
    LiveFloatPercentiles {
        average: percentiles.average,
        p50: percentiles.p50,
        p75: percentiles.p75,
        p90: percentiles.p90,
        p95: percentiles.p95,
    }
}

async fn resolve_codex_live_data_with<FetchOauth, FetchRpc, FetchCli>(
    bootstrap: codex::CodexBootstrapAuth,
    fetch_oauth: FetchOauth,
    fetch_rpc: FetchRpc,
    fetch_cli: FetchCli,
) -> CodexLiveResolution
where
    FetchOauth: for<'a> Fn(
        &'a codex::CodexAuth,
    ) -> Pin<
        Box<dyn Future<Output = Result<codex::CodexUsageResponse>> + Send + 'a>,
    >,
    FetchRpc: Fn(
        Duration,
    ) -> Result<(
        Option<codex::RpcAccountResponse>,
        codex::RpcRateLimitsResponse,
    )>,
    FetchCli: Fn(Duration) -> Result<codex::CliStatusSnapshot>,
{
    let started_at = Instant::now();
    let mut attempts = Vec::new();
    let mut identity = None::<LiveProviderIdentity>;
    let mut primary = None;
    let mut secondary = None;
    let mut credits = None;
    let resolved_auth = bootstrap.auth.clone();
    let mut source_used = "unavailable".to_string();
    let mut error = bootstrap.load_error.clone();
    let mut available = false;
    let mut last_attempted_source;

    last_attempted_source = Some("cli-rpc".to_string());
    match fetch_rpc(Duration::from_secs(8)) {
        Ok((account, limits)) => {
            attempts.push(LiveProviderSourceAttempt {
                source: "cli-rpc".into(),
                outcome: "success".into(),
                message: Some(format!(
                    "credential backend resolved as {}",
                    bootstrap.credential_store.backend_label()
                )),
            });
            available = true;
            source_used = "cli-rpc".into();
            identity = account.as_ref().and_then(codex::rpc_account_to_identity);
            primary = limits
                .rate_limits
                .primary
                .as_ref()
                .map(codex::rpc_window_to_live);
            secondary = limits
                .rate_limits
                .secondary
                .as_ref()
                .map(codex::rpc_window_to_live);
            credits = limits
                .rate_limits
                .credits
                .as_ref()
                .and_then(codex::rpc_credits_to_f64);
            error = None;
        }
        Err(rpc_error) => {
            attempts.push(LiveProviderSourceAttempt {
                source: "cli-rpc".into(),
                outcome: "error".into(),
                message: Some(rpc_error.to_string()),
            });
            if error.is_none() {
                error = Some(rpc_error.to_string());
            }
        }
    }

    if !available && let Some(codex_auth) = bootstrap.auth.as_ref() {
        last_attempted_source = Some("oauth".to_string());
        if identity.is_none() {
            identity = codex::decode_identity(codex_auth);
        }
        match fetch_oauth(codex_auth).await {
            Ok(response) => {
                attempts.push(LiveProviderSourceAttempt {
                    source: "oauth".into(),
                    outcome: "success".into(),
                    message: None,
                });
                available = true;
                source_used = "oauth".into();
                if let Some(plan_type) = response.plan_type
                    && identity.is_none()
                {
                    identity = Some(LiveProviderIdentity {
                        provider: "codex".into(),
                        account_email: None,
                        account_organization: None,
                        login_method: Some("chatgpt".into()),
                        plan: Some(plan_type),
                    });
                }
                if let Some(rate_limit) = response.rate_limit {
                    primary = rate_limit
                        .primary_window
                        .as_ref()
                        .map(codex::oauth_window_to_live);
                    secondary = rate_limit
                        .secondary_window
                        .as_ref()
                        .map(codex::oauth_window_to_live);
                }
                credits = response
                    .credits
                    .as_ref()
                    .and_then(codex::oauth_credits_to_f64);
                error = None;
            }
            Err(fetch_error) => {
                attempts.push(LiveProviderSourceAttempt {
                    source: "oauth".into(),
                    outcome: "error".into(),
                    message: Some(fetch_error.to_string()),
                });
                error = Some(fetch_error.to_string());
            }
        }
    } else if !available && bootstrap.load_error.is_some() {
        attempts.push(LiveProviderSourceAttempt {
            source: "oauth-auth".into(),
            outcome: "error".into(),
            message: bootstrap.load_error.clone(),
        });
    }

    if !available {
        last_attempted_source = Some("cli-pty".to_string());
        match fetch_cli(Duration::from_secs(8)) {
            Ok(status_snapshot) => {
                attempts.push(LiveProviderSourceAttempt {
                    source: "cli-pty".into(),
                    outcome: "success".into(),
                    message: None,
                });
                available = true;
                source_used = "cli-pty".into();
                primary = status_snapshot.primary;
                secondary = status_snapshot.secondary;
                credits = status_snapshot.credits;
                error = None;
            }
            Err(cli_error) => {
                attempts.push(LiveProviderSourceAttempt {
                    source: "cli-pty".into(),
                    outcome: "error".into(),
                    message: Some(cli_error.to_string()),
                });
                if error.is_none() {
                    error = Some(cli_error.to_string());
                }
            }
        }
    }

    let resolved_via_fallback = available && source_used != "cli-rpc";

    CodexLiveResolution {
        available,
        source_used,
        last_attempted_source,
        resolved_via_fallback,
        refresh_duration_ms: started_at.elapsed().as_millis() as u64,
        source_attempts: attempts,
        identity,
        primary,
        secondary,
        credits,
        error,
        auth: resolved_auth,
        credential_store: bootstrap.credential_store,
        bootstrap_error: bootstrap.load_error,
    }
}

fn window_to_live(window: &WindowInfo) -> crate::models::LiveRateWindow {
    crate::models::LiveRateWindow {
        used_percent: window.used_percent,
        resets_at: window.resets_at.clone(),
        resets_in_minutes: window.resets_in_minutes,
        window_minutes: None,
        reset_label: None,
    }
}

fn budget_to_credits(budget: &BudgetInfo) -> f64 {
    (budget.limit - budget.used).max(0.0)
}

fn identity_to_live(identity: &Identity) -> LiveProviderIdentity {
    LiveProviderIdentity {
        provider: "claude".into(),
        account_email: None,
        account_organization: None,
        login_method: identity.rate_limit_tier.clone(),
        plan: identity.plan.as_ref().map(plan_to_string),
    }
}

fn plan_to_string(plan: &Plan) -> String {
    match plan {
        Plan::Max => "max".into(),
        Plan::Pro => "pro".into(),
        Plan::Team => "team".into(),
        Plan::Enterprise => "enterprise".into(),
    }
}

fn status_to_live(status: &ProviderStatus) -> LiveProviderStatus {
    LiveProviderStatus {
        indicator: match status.indicator {
            crate::agent_status::models::StatusIndicator::None => "none",
            crate::agent_status::models::StatusIndicator::Minor => "minor",
            crate::agent_status::models::StatusIndicator::Major => "major",
            crate::agent_status::models::StatusIndicator::Critical => "critical",
            crate::agent_status::models::StatusIndicator::Maintenance => "maintenance",
            crate::agent_status::models::StatusIndicator::Unknown => "unknown",
        }
        .to_string(),
        description: status.description.clone(),
        page_url: status.page_url.clone(),
    }
}

fn provider_cost_summary(
    conn: &rusqlite::Connection,
    provider: &str,
) -> Result<ProviderCostSummary> {
    let today = chrono::Utc::now().date_naive().to_string();
    let start_date = (chrono::Utc::now().date_naive() - chrono::Duration::days(29)).to_string();
    let (today_cost_nanos, today_tokens, today_breakdown) =
        db::get_provider_cost_summary_since(conn, provider, &today)?;
    let (last_30_cost_nanos, last_30_tokens, last_30_days_breakdown) =
        db::get_provider_cost_summary_since(conn, provider, &start_date)?;
    let daily = db::get_provider_daily_cost_history_since(conn, provider, &start_date)?;
    let cache_hit_rate_today = today_breakdown.cache_hit_rate();
    let cache_hit_rate_30d = last_30_days_breakdown.cache_hit_rate();
    let cache_savings_30d_nanos =
        db::get_provider_cache_savings_nanos_since(conn, provider, &start_date)?;
    let cache_savings_30d_usd = if cache_savings_30d_nanos > 0 {
        Some(cache_savings_30d_nanos as f64 / 1_000_000_000.0)
    } else {
        None
    };

    let by_model = db::get_provider_model_rows(conn, provider, &start_date, 10).unwrap_or_default();
    let by_project =
        db::get_provider_project_rows(conn, provider, &start_date, 10).unwrap_or_default();
    let by_tool = db::get_provider_tool_rows(conn, provider, &start_date, 15).unwrap_or_default();
    let by_mcp = db::get_provider_mcp_rows(conn, provider, &start_date).unwrap_or_default();

    let hourly_activity =
        db::get_provider_hourly_activity(conn, provider, &start_date).unwrap_or_default();
    let activity_heatmap =
        db::get_provider_activity_heatmap(conn, provider, &start_date).unwrap_or_default();
    let recent_sessions = db::get_provider_recent_sessions(conn, provider, 20).unwrap_or_default();
    let subagent_breakdown = db::get_provider_subagent_breakdown(conn, provider, &start_date)
        .ok()
        .flatten();
    let version_breakdown =
        db::get_provider_version_rows(conn, provider, &start_date, 10).unwrap_or_default();

    Ok(ProviderCostSummary {
        today_tokens,
        today_cost_usd: today_cost_nanos as f64 / 1_000_000_000.0,
        last_30_days_tokens: last_30_tokens,
        last_30_days_cost_usd: last_30_cost_nanos as f64 / 1_000_000_000.0,
        daily,
        today_breakdown,
        last_30_days_breakdown,
        cache_hit_rate_today,
        cache_hit_rate_30d,
        cache_savings_30d_usd,
        by_model,
        by_project,
        by_tool,
        by_mcp,
        hourly_activity,
        activity_heatmap,
        recent_sessions,
        subagent_breakdown,
        version_breakdown,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LiveProviderAuth;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn fixture_response(provider: &str) -> LiveProvidersResponse {
        LiveProvidersResponse {
            contract_version: LIVE_PROVIDERS_CONTRACT_VERSION,
            providers: vec![LiveProviderSnapshot {
                provider: provider.to_string(),
                available: true,
                source_used: "oauth".into(),
                last_attempted_source: Some("oauth".into()),
                resolved_via_fallback: false,
                refresh_duration_ms: 1,
                source_attempts: vec![LiveProviderSourceAttempt {
                    source: "oauth".into(),
                    outcome: "success".into(),
                    message: None,
                }],
                identity: None,
                primary: None,
                secondary: None,
                tertiary: None,
                credits: None,
                status: None,
                auth: LiveProviderAuth::default(),
                cost_summary: ProviderCostSummary::default(),
                claude_usage: None,
                quota_suggestions: None,
                depletion_forecast: None,
                predictive_insights: None,
                last_refresh: "2026-01-01T00:00:00Z".into(),
                stale: false,
                error: None,
            }],
            fetched_at: "2026-01-01T00:00:00Z".into(),
            requested_provider: Some(provider.to_string()),
            response_scope: "provider".into(),
            cache_hit: false,
            refreshed_providers: vec![provider.to_string()],
            local_notification_state: None,
        }
    }

    fn test_state() -> Arc<AppState> {
        Arc::new(AppState {
            db_path: std::path::PathBuf::from("/tmp/heimdall-live-provider-tests.db"),
            projects_dirs: None,
            oauth_enabled: false,
            oauth_refresh_interval: 60,
            oauth_cache: tokio::sync::RwLock::new(None),
            oauth_refresh_lock: tokio::sync::Mutex::new(()),
            openai_enabled: false,
            openai_admin_key_env: "OPENAI_ADMIN_KEY".into(),
            openai_refresh_interval: 60,
            openai_lookback_days: 30,
            openai_cache: tokio::sync::RwLock::new(None),
            openai_refresh_lock: tokio::sync::Mutex::new(()),
            db_lock: tokio::sync::Mutex::new(()),
            webhook_state: tokio::sync::Mutex::new(crate::webhooks::WebhookState::default()),
            webhook_config: crate::config::WebhookConfig::default(),
            scan_event_tx: tokio::sync::broadcast::channel::<String>(1).0,
            agent_status_config: crate::config::AgentStatusConfig::default(),
            agent_status_cache: tokio::sync::RwLock::new(None),
            agent_status_refresh_lock: tokio::sync::Mutex::new(()),
            aggregator_config: crate::config::AggregatorConfig::default(),
            aggregator_cache: tokio::sync::RwLock::new(None),
            aggregator_refresh_lock: tokio::sync::Mutex::new(()),
            blocks_token_limit: None,
            session_length_hours: 5.0,
            project_aliases: std::collections::HashMap::new(),
            live_provider_cache: tokio::sync::RwLock::new(None),
            live_provider_refresh_lock: tokio::sync::Mutex::new(()),
        })
    }

    #[tokio::test]
    async fn refresh_invalidates_cached_response() {
        let state = test_state();
        let counter = Arc::new(AtomicUsize::new(0));

        {
            let mut cache = state.live_provider_cache.write().await;
            *cache = Some((Instant::now(), {
                let mut response = fixture_response("claude");
                response.response_scope = "all".into();
                response.requested_provider = None;
                response
            }));
        }

        let initial = load_snapshots_with_fetcher(
            &state,
            Some("claude".to_string()),
            ResponseScope::ProviderOnly,
            false,
            false,
            {
                let counter = counter.clone();
                move |_, _, _, _| {
                    let counter = counter.clone();
                    async move {
                        counter.fetch_add(1, Ordering::SeqCst);
                        Ok(fixture_response("codex"))
                    }
                }
            },
        )
        .await
        .unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 0);
        assert!(initial.cache_hit);
        assert_eq!(initial.providers[0].provider, "claude");

        let refreshed = load_snapshots_with_fetcher(
            &state,
            Some("codex".to_string()),
            ResponseScope::ProviderOnly,
            true,
            false,
            {
                let counter = counter.clone();
                move |_, _, _, _| {
                    let counter = counter.clone();
                    async move {
                        counter.fetch_add(1, Ordering::SeqCst);
                        Ok(fixture_response("codex"))
                    }
                }
            },
        )
        .await
        .unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 1);
        assert!(!refreshed.cache_hit);
        assert_eq!(refreshed.providers[0].provider, "codex");
        let cached = state.live_provider_cache.read().await;
        let cached = cached.as_ref().unwrap().1.clone();
        assert!(
            cached
                .providers
                .iter()
                .any(|snapshot| snapshot.provider == "claude")
        );
        assert!(
            cached
                .providers
                .iter()
                .any(|snapshot| snapshot.provider == "codex")
        );
    }

    #[tokio::test]
    async fn cached_read_does_not_wait_behind_inflight_refresh() {
        let state = test_state();
        {
            let mut cache = state.live_provider_cache.write().await;
            *cache = Some((Instant::now() - Duration::from_secs(300), {
                let mut response = fixture_response("claude");
                response.response_scope = "all".into();
                response.requested_provider = None;
                response
            }));
        }

        let refresh_guard = state.live_provider_refresh_lock.lock().await;

        let started = Instant::now();
        let response = load_snapshots_with_fetcher(
            &state,
            Some("claude".to_string()),
            ResponseScope::ProviderOnly,
            false,
            false,
            |_, _, _, _| async move {
                panic!("cached read should not trigger a fetch while refresh is in flight")
            },
        )
        .await
        .unwrap();

        assert!(started.elapsed() < Duration::from_millis(50));
        assert!(response.cache_hit);
        assert_eq!(response.providers.len(), 1);
        assert_eq!(response.providers[0].provider, "claude");

        drop(refresh_guard);
    }

    #[tokio::test]
    async fn startup_mode_returns_stale_cached_response_without_fetch() {
        let state = test_state();
        {
            let mut cache = state.live_provider_cache.write().await;
            *cache = Some((Instant::now() - Duration::from_secs(300), {
                let mut response = fixture_response("claude");
                response.response_scope = "all".into();
                response.requested_provider = None;
                response
            }));
        }

        let counter = Arc::new(AtomicUsize::new(0));
        let response = load_snapshots_with_fetcher(
            &state,
            Some("claude".to_string()),
            ResponseScope::ProviderOnly,
            false,
            true,
            {
                let counter = counter.clone();
                move |_, _, _, _| {
                    let counter = counter.clone();
                    async move {
                        counter.fetch_add(1, Ordering::SeqCst);
                        Ok(fixture_response("codex"))
                    }
                }
            },
        )
        .await
        .unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 0);
        assert!(response.cache_hit);
        assert_eq!(response.providers.len(), 1);
        assert_eq!(response.providers[0].provider, "claude");
    }

    #[tokio::test]
    async fn codex_rpc_is_primary_when_available() {
        let auth = codex::CodexAuth {
            access_token: "token".into(),
            refresh_token: None,
            id_token: None,
            account_id: None,
            auth_mode: Some("chatgpt".into()),
        };
        let bootstrap = codex::CodexBootstrapAuth {
            auth: Some(auth.clone()),
            credential_store: codex::ResolvedCodexCredentialStore::AutoKeyring,
            auth_file_path: std::path::PathBuf::from("/tmp/codex-auth.json"),
            load_error: None,
        };

        let resolution = resolve_codex_live_data_with(
            bootstrap,
            |_| Box::pin(async { Err(anyhow!("oauth failed")) }),
            |_| {
                Ok((
                    Some(codex::RpcAccountResponse {
                        requires_openai_auth: Some(false),
                        account: Some(codex::RpcAccountDetails::ChatGpt {
                            email: Some("rpc@example.com".into()),
                            plan_type: Some("pro".into()),
                        }),
                    }),
                    codex::RpcRateLimitsResponse {
                        rate_limits: codex::RpcRateLimitSnapshot {
                            primary: Some(codex::RpcRateLimitWindow {
                                used_percent: 42.0,
                                window_duration_mins: Some(300),
                                resets_at: None,
                            }),
                            secondary: None,
                            credits: None,
                        },
                    },
                ))
            },
            |_| Err(anyhow!("cli should not run")),
        )
        .await;

        assert!(resolution.available);
        assert_eq!(resolution.source_used, "cli-rpc");
        assert!(!resolution.resolved_via_fallback);
        assert!(
            resolution
                .source_attempts
                .iter()
                .any(|attempt| attempt.source == "cli-rpc" && attempt.outcome == "success")
        );
        assert!(
            resolution
                .source_attempts
                .iter()
                .all(|attempt| attempt.source != "oauth")
        );
    }

    #[tokio::test]
    async fn codex_rpc_failure_falls_back_to_cli_status() {
        let bootstrap = codex::CodexBootstrapAuth {
            auth: None,
            credential_store: codex::ResolvedCodexCredentialStore::AutoKeyring,
            auth_file_path: std::path::PathBuf::from("/tmp/codex-auth.json"),
            load_error: Some("auth missing".into()),
        };
        let resolution = resolve_codex_live_data_with(
            bootstrap,
            |_| Box::pin(async { Err(anyhow!("oauth should not run")) }),
            |_| Err(anyhow!("rpc failed")),
            |_| {
                Ok(codex::CliStatusSnapshot {
                    credits: Some(12.5),
                    primary: Some(crate::models::LiveRateWindow {
                        used_percent: 12.0,
                        resets_at: None,
                        resets_in_minutes: None,
                        window_minutes: None,
                        reset_label: Some("resets soon".into()),
                    }),
                    secondary: None,
                })
            },
        )
        .await;

        assert!(resolution.available);
        assert_eq!(resolution.source_used, "cli-pty");
        assert!(resolution.resolved_via_fallback);
        assert!(
            resolution
                .source_attempts
                .iter()
                .any(|attempt| attempt.source == "cli-rpc" && attempt.outcome == "error")
        );
        assert!(
            resolution
                .source_attempts
                .iter()
                .any(|attempt| attempt.source == "cli-pty" && attempt.outcome == "success")
        );
    }
}
