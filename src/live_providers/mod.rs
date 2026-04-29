use std::future::Future;
use std::sync::Arc;

use anyhow::{Result, anyhow, bail};

use crate::agent_status::models::ProviderStatus;
use crate::models::{
    LIVE_PROVIDERS_CONTRACT_VERSION, LiveProviderHistoryResponse, LiveProviderStatus,
    LiveProvidersResponse, MOBILE_SNAPSHOT_CONTRACT_VERSION, MobileProviderHistorySeries,
    MobileSnapshotEnvelope, MobileSnapshotFreshness, MobileSnapshotTotals, ProviderCostSummary,
};
use crate::oauth::models::UsageWindowsResponse;
use crate::scanner::db;
use crate::server::api::{
    AppState, refresh_agent_status, refresh_community_signal, refresh_usage_windows,
};
use crate::status_aggregator::models::CommunitySignal;

pub mod cache;
pub mod claude;
pub mod codex;
pub mod conditions;
pub mod quota_estimator;

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
        // Readiness gate: `startup=true` is contracted as a fast probe. It
        // returns any full cached response (fresh or stale) when one exists,
        // and otherwise a synthetic empty envelope — never the fetcher path.
        // The fetcher's `build_claude_snapshot` performs `load_all_turns` and
        // analytics that routinely take 5-30 seconds on a populated DB; that's
        // fine for background refresh but unacceptable for a 5-second probe
        // timeout. Subsequent (non-startup) requests walk the full path and
        // populate `live_provider_cache` for the next readiness probe.
        if let Some(cached) = cache::cached_response(state).await {
            return Ok(cache::filter_response(
                &cached,
                requested_provider.as_deref(),
                scope,
                true,
            ));
        }
        if let Some(cached) = cache::cached_response_any(state).await {
            return Ok(cache::filter_response(
                &cached,
                requested_provider.as_deref(),
                scope,
                true,
            ));
        }

        return Ok(LiveProvidersResponse {
            contract_version: LIVE_PROVIDERS_CONTRACT_VERSION,
            fetched_at: chrono::Utc::now().to_rfc3339(),
            requested_provider,
            response_scope: scope.as_str().to_string(),
            cache_hit: true,
            ..Default::default()
        });
    }

    if !force_refresh && let Some(cached) = cache::cached_response(state).await {
        return Ok(cache::filter_response(
            &cached,
            requested_provider.as_deref(),
            scope,
            true,
        ));
    }

    // Stale-while-revalidate: if we have any cached entry (even expired) and no
    // refresh is already in flight, return the stale response immediately and
    // kick off a background refresh so the *next* request sees fresh data.
    if !force_refresh && let Some(stale) = cache::cached_response_any(state).await {
        match state.live_provider_refresh_lock.try_lock() {
            Ok(_guard) => {
                // Lock acquired; drop the guard and let the background task
                // re-acquire it so it serialises correctly with any future
                // concurrent foreground requests.
                drop(_guard);
                let bg_state = Arc::clone(state);
                let bg_provider = requested_provider.clone();
                tokio::spawn(async move {
                    let _guard = bg_state.live_provider_refresh_lock.lock().await;
                    // Re-check: another task may have refreshed while we waited.
                    if cache::cached_response(&bg_state).await.is_some() {
                        return;
                    }
                    match fetch_live_provider_response(
                        &bg_state,
                        bg_provider.as_deref(),
                        scope,
                        false,
                        false,
                    )
                    .await
                    {
                        Ok(response) => {
                            cache::update_cache_after_fetch(
                                &bg_state,
                                bg_provider.as_deref(),
                                scope,
                                &response,
                            )
                            .await;
                        }
                        Err(err) => {
                            tracing::warn!("background live-provider refresh failed: {:#}", err);
                        }
                    }
                });
                return Ok(cache::filter_response(
                    &stale,
                    requested_provider.as_deref(),
                    scope,
                    true,
                ));
            }
            Err(_) => {
                // A refresh is already in flight; return stale immediately
                // rather than blocking the caller.
                return Ok(cache::filter_response(
                    &stale,
                    requested_provider.as_deref(),
                    scope,
                    true,
                ));
            }
        }
    }

    // Cold cache (no entry at all) — foreground fetch.
    let mut waited_for_refresh = false;
    let _refresh_guard = match state.live_provider_refresh_lock.try_lock() {
        Ok(guard) => guard,
        Err(_) => {
            waited_for_refresh = true;
            state.live_provider_refresh_lock.lock().await
        }
    };

    if !force_refresh && let Some(cached) = cache::cached_response(state).await {
        return Ok(cache::filter_response(
            &cached,
            requested_provider.as_deref(),
            scope,
            true,
        ));
    }

    if waited_for_refresh && let Some(cached) = cache::cached_response_any(state).await {
        return Ok(cache::filter_response(
            &cached,
            requested_provider.as_deref(),
            scope,
            true,
        ));
    }

    let response = fetcher(requested_provider.clone(), scope, force_refresh, false).await?;
    cache::update_cache_after_fetch(state, requested_provider.as_deref(), scope, &response).await;
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

pub async fn load_provider_cost_summary_tz(
    state: &Arc<AppState>,
    provider: &str,
    tz: crate::tz::TzParams,
) -> Result<ProviderCostSummary> {
    let provider = normalize_provider(Some(provider))?
        .ok_or_else(|| anyhow!("missing provider"))?
        .to_string();
    let db_path = state.db_path.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db::open_db(&db_path)?;
        db::init_db(&conn)?;
        provider_cost_summary_tz(&conn, &provider, tz)
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
    providers: &[crate::models::LiveProviderSnapshot],
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

/// Window-type identifiers used both as a series key on the dashboard chart
/// and as `rate_window_history.window_type` in SQLite.
const WINDOW_FIVE_HOUR: &str = "five_hour";
const WINDOW_SEVEN_DAY: &str = "seven_day";
const WINDOW_SEVEN_DAY_OPUS: &str = "seven_day_opus";
const WINDOW_SEVEN_DAY_SONNET: &str = "seven_day_sonnet";
const WINDOW_CODEX_PRIMARY: &str = "codex_primary";
const WINDOW_CODEX_SECONDARY: &str = "codex_secondary";

/// Persist quota snapshots for the current response on a fire-and-forget task.
/// Failure to record is non-fatal: the historical chart simply skips that
/// data point. We never want to block the live-provider response on disk IO.
fn record_subscription_quota_snapshots(state: &Arc<AppState>, response: &LiveProvidersResponse) {
    use crate::live_providers::quota_estimator::estimate_window_cap;
    use crate::models::LiveProviderSnapshot;
    use crate::scanner::db::{RateWindowSnapshotInsert, record_rate_window_snapshot};

    struct WindowSpec<'a> {
        provider: &'a str,
        window_type: &'a str,
        used_percent: f64,
        resets_at: Option<String>,
        plan: Option<String>,
        window_seconds: i64,
        model_pattern: Option<&'a str>,
    }

    fn collect_specs<'a>(snapshot: &'a LiveProviderSnapshot) -> Vec<WindowSpec<'a>> {
        let plan = snapshot.identity.as_ref().and_then(|i| i.plan.clone());
        let mut specs = Vec::new();
        match snapshot.provider.as_str() {
            "claude" => {
                if let Some(window) = snapshot.primary.as_ref() {
                    specs.push(WindowSpec {
                        provider: "claude",
                        window_type: WINDOW_FIVE_HOUR,
                        used_percent: window.used_percent,
                        resets_at: window.resets_at.clone(),
                        plan: plan.clone(),
                        window_seconds: 5 * 3600,
                        model_pattern: None,
                    });
                }
                if let Some(window) = snapshot.secondary.as_ref() {
                    specs.push(WindowSpec {
                        provider: "claude",
                        window_type: WINDOW_SEVEN_DAY,
                        used_percent: window.used_percent,
                        resets_at: window.resets_at.clone(),
                        plan: plan.clone(),
                        window_seconds: 7 * 86_400,
                        model_pattern: None,
                    });
                }
                if let Some(window) = snapshot.tertiary.as_ref() {
                    // Tertiary is the more-specific of the Opus/Sonnet
                    // windows; we cannot tell from the snapshot alone which
                    // one Anthropic selected, so we record both with the
                    // matching observed-token filter and let the
                    // estimator's confidence floor weed out the noise.
                    let used = window.used_percent;
                    let resets = window.resets_at.clone();
                    specs.push(WindowSpec {
                        provider: "claude",
                        window_type: WINDOW_SEVEN_DAY_OPUS,
                        used_percent: used,
                        resets_at: resets.clone(),
                        plan: plan.clone(),
                        window_seconds: 7 * 86_400,
                        model_pattern: Some("%opus%"),
                    });
                    specs.push(WindowSpec {
                        provider: "claude",
                        window_type: WINDOW_SEVEN_DAY_SONNET,
                        used_percent: used,
                        resets_at: resets,
                        plan: plan.clone(),
                        window_seconds: 7 * 86_400,
                        model_pattern: Some("%sonnet%"),
                    });
                }
            }
            "codex" => {
                if let Some(window) = snapshot.primary.as_ref() {
                    specs.push(WindowSpec {
                        provider: "codex",
                        window_type: WINDOW_CODEX_PRIMARY,
                        used_percent: window.used_percent,
                        resets_at: window.resets_at.clone(),
                        plan: plan.clone(),
                        window_seconds: window.window_minutes.unwrap_or(5 * 60) * 60,
                        model_pattern: None,
                    });
                }
                if let Some(window) = snapshot.secondary.as_ref() {
                    specs.push(WindowSpec {
                        provider: "codex",
                        window_type: WINDOW_CODEX_SECONDARY,
                        used_percent: window.used_percent,
                        resets_at: window.resets_at.clone(),
                        plan,
                        window_seconds: window.window_minutes.unwrap_or(7 * 24 * 60) * 60,
                        model_pattern: None,
                    });
                }
            }
            _ => {}
        }
        specs
    }

    let mut all_specs: Vec<WindowSpec<'_>> = Vec::new();
    for snapshot in &response.providers {
        if !snapshot.available {
            continue;
        }
        all_specs.extend(collect_specs(snapshot));
    }
    if all_specs.is_empty() {
        return;
    }

    // Snapshot owned data for the spawned task.
    struct OwnedSpec {
        provider: String,
        window_type: String,
        used_percent: f64,
        resets_at: Option<String>,
        plan: Option<String>,
        window_seconds: i64,
        model_pattern: Option<String>,
    }
    let owned: Vec<OwnedSpec> = all_specs
        .into_iter()
        .map(|s| OwnedSpec {
            provider: s.provider.to_string(),
            window_type: s.window_type.to_string(),
            used_percent: s.used_percent,
            resets_at: s.resets_at,
            plan: s.plan,
            window_seconds: s.window_seconds,
            model_pattern: s.model_pattern.map(|p| p.to_string()),
        })
        .collect();
    let db_path = state.db_path.clone();

    tokio::task::spawn_blocking(move || {
        use crate::scanner::db::{init_db, observed_tokens_for_window, open_db};
        let conn = match open_db(&db_path) {
            Ok(c) => c,
            Err(err) => {
                tracing::warn!("subscription_quota: open_db failed: {err}");
                return;
            }
        };
        if let Err(err) = init_db(&conn) {
            tracing::warn!("subscription_quota: init_db failed: {err}");
            return;
        }
        for spec in &owned {
            let observed = observed_tokens_for_window(
                &conn,
                &spec.provider,
                spec.window_seconds,
                spec.model_pattern.as_deref(),
                spec.resets_at.as_deref(),
            )
            .unwrap_or(0);
            let estimate = estimate_window_cap(spec.used_percent, observed);
            let snap = RateWindowSnapshotInsert {
                provider: &spec.provider,
                window_type: &spec.window_type,
                used_percent: spec.used_percent,
                resets_at: spec.resets_at.as_deref(),
                plan: spec.plan.as_deref(),
                observed_tokens: Some(observed),
                estimated_cap_tokens: estimate.map(|e| e.estimated_cap_tokens),
                confidence: estimate.map(|e| e.confidence),
                source_kind: "oauth",
            };
            if let Err(err) = record_rate_window_snapshot(&conn, &snap) {
                tracing::warn!(
                    "subscription_quota: record snapshot {}/{} failed: {err}",
                    spec.provider,
                    spec.window_type
                );
            }
        }
    });
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
                let snapshot = claude::build_claude_snapshot(
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
                    codex::build_codex_bootstrap_snapshot(
                        state,
                        agent_status
                            .as_ref()
                            .and_then(|status| status.openai.as_ref()),
                    )
                    .await?
                } else {
                    codex::build_codex_snapshot(
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
    let local_notification_state = conditions::build_local_notification_state(
        state,
        agent_status.as_ref(),
        claude_usage.as_ref(),
        community_signal.as_ref(),
    )
    .await?;

    let response = LiveProvidersResponse {
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
    };
    if !startup {
        record_subscription_quota_snapshots(state, &response);
    }
    Ok(response)
}

async fn cached_agent_status(
    state: &Arc<AppState>,
) -> Option<crate::agent_status::models::AgentStatusSnapshot> {
    let cache = state.agent_status_cache.read().await;
    cache.as_ref().map(|(_, snapshot, _)| snapshot.clone())
}

async fn cached_or_unavailable_claude_usage(state: &Arc<AppState>) -> UsageWindowsResponse {
    let cache = state.oauth_cache.read().await;
    if let Some((_, response)) = cache.as_ref() {
        return response.clone();
    }
    drop(cache);

    let admin_cache = state.claude_admin_cache.read().await;
    admin_cache
        .as_ref()
        .map(|(_, summary)| UsageWindowsResponse::from_admin_fallback(summary.clone()))
        .unwrap_or_else(UsageWindowsResponse::unavailable)
}

async fn cached_community_signal(state: &Arc<AppState>) -> Option<CommunitySignal> {
    if !state.aggregator_config.enabled {
        return None;
    }

    let cache = state.aggregator_cache.read().await;
    cache.as_ref().map(|(_, signal)| signal.clone())
}

fn normalize_provider(provider: Option<&str>) -> Result<Option<&'static str>> {
    match provider {
        None => Ok(None),
        Some("claude") => Ok(Some("claude")),
        Some("codex") => Ok(Some("codex")),
        Some(other) => bail!("unsupported live provider: {}", other),
    }
}

// ---------------------------------------------------------------------------
// Shared helpers used by multiple submodules
// ---------------------------------------------------------------------------

/// Compute the cost summary for a single provider. Used by both
/// `load_provider_cost_summary` (orchestration) and `claude::build_claude_snapshot`.
pub(crate) fn provider_cost_summary(
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
    let daily_by_model =
        db::get_provider_daily_by_model(conn, provider, &start_date).unwrap_or_default();
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
        daily_by_model,
    })
}

pub(crate) fn provider_cost_summary_tz(
    conn: &rusqlite::Connection,
    provider: &str,
    tz: crate::tz::TzParams,
) -> Result<ProviderCostSummary> {
    let offset = chrono::Duration::minutes(tz.normalized_offset_min() as i64);
    let local_today = (chrono::Utc::now() + offset).date_naive();
    let today = local_today.to_string();
    let start_date = (local_today - chrono::Duration::days(29)).to_string();
    let (today_cost_nanos, today_tokens, today_breakdown) =
        db::get_provider_cost_summary_since_tz(conn, provider, &today, tz)?;
    let (last_30_cost_nanos, last_30_tokens, last_30_days_breakdown) =
        db::get_provider_cost_summary_since_tz(conn, provider, &start_date, tz)?;
    let daily = db::get_provider_daily_cost_history_since_tz(conn, provider, &start_date, tz)?;
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
    let daily_by_model =
        db::get_provider_daily_by_model(conn, provider, &start_date).unwrap_or_default();
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
        daily_by_model,
    })
}

/// Convert a `ProviderStatus` to its live wire representation.
/// Used by both `claude::build_claude_snapshot` and
/// `codex::build_codex_snapshot` / `codex::build_codex_bootstrap_snapshot`.
pub(crate) fn status_to_live(status: &ProviderStatus) -> LiveProviderStatus {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LiveProviderAuth;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    fn fixture_response(provider: &str) -> LiveProvidersResponse {
        LiveProvidersResponse {
            contract_version: LIVE_PROVIDERS_CONTRACT_VERSION,
            providers: vec![crate::models::LiveProviderSnapshot {
                provider: provider.to_string(),
                available: true,
                source_used: "oauth".into(),
                last_attempted_source: Some("oauth".into()),
                resolved_via_fallback: false,
                refresh_duration_ms: 1,
                source_attempts: vec![crate::models::LiveProviderSourceAttempt {
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
                claude_admin: None,
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
            claude_admin_enabled: false,
            claude_admin_key_env: "ANTHROPIC_ADMIN_KEY".into(),
            claude_admin_refresh_interval: 300,
            claude_admin_lookback_days: 30,
            claude_admin_cache: tokio::sync::RwLock::new(None),
            claude_admin_refresh_lock: tokio::sync::Mutex::new(()),
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
            *cache = Some((Instant::now() - std::time::Duration::from_secs(300), {
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

        assert!(started.elapsed() < std::time::Duration::from_millis(50));
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
            *cache = Some((Instant::now() - std::time::Duration::from_secs(300), {
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
    async fn startup_mode_with_cold_caches_returns_synthetic_response_without_fetch() {
        let state = test_state();
        let counter = Arc::new(AtomicUsize::new(0));

        let response =
            load_snapshots_with_fetcher(&state, None, ResponseScope::All, false, true, {
                let counter = counter.clone();
                move |_, _, _, _| {
                    let counter = counter.clone();
                    async move {
                        counter.fetch_add(1, Ordering::SeqCst);
                        Ok(fixture_response("claude"))
                    }
                }
            })
            .await
            .unwrap();

        assert_eq!(
            counter.load(Ordering::SeqCst),
            0,
            "cold-cache startup must not invoke fetcher"
        );
        assert_eq!(response.contract_version, LIVE_PROVIDERS_CONTRACT_VERSION);
        assert!(response.providers.is_empty());
        assert!(response.cache_hit);
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

        let resolution = codex::resolve_codex_live_data_with(
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
        let resolution = codex::resolve_codex_live_data_with(
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

    #[tokio::test]
    async fn codex_rpc_fails_oauth_succeeds_skips_cli_pty() {
        // RPC fails, auth IS present so OAuth runs and wins; cli-pty must
        // never be invoked.  Closes the gap between the two existing tests
        // (rpc-wins + auth-absent rpc-fails) which never exercise oauth.
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

        let resolution = codex::resolve_codex_live_data_with(
            bootstrap,
            |_| {
                Box::pin(async move {
                    Ok(codex::CodexUsageResponse {
                        plan_type: Some("plus".into()),
                        rate_limit: None,
                        credits: None,
                    })
                })
            },
            |_| Err(anyhow!("rpc failed")),
            |_| Err(anyhow!("cli-pty must not run when oauth wins")),
        )
        .await;

        assert!(resolution.available);
        assert_eq!(resolution.source_used, "oauth");
        assert!(resolution.resolved_via_fallback);
        assert_eq!(resolution.last_attempted_source.as_deref(), Some("oauth"));
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
                .any(|attempt| attempt.source == "oauth" && attempt.outcome == "success")
        );
        assert!(
            resolution
                .source_attempts
                .iter()
                .all(|attempt| attempt.source != "cli-pty"),
            "cli-pty must not appear in attempts when oauth wins; found {:?}",
            resolution.source_attempts
        );
    }

    #[tokio::test]
    async fn codex_full_waterfall_rpc_oauth_pty_lands_on_cli_pty() {
        // RPC fails, OAuth fails, cli-pty wins — exercises every rung of the
        // ladder in one test, with auth present so OAuth is actually attempted
        // (the existing test sets `auth: None`, which short-circuits OAuth).
        let auth = codex::CodexAuth {
            access_token: "token".into(),
            refresh_token: None,
            id_token: None,
            account_id: None,
            auth_mode: Some("chatgpt".into()),
        };
        let bootstrap = codex::CodexBootstrapAuth {
            auth: Some(auth),
            credential_store: codex::ResolvedCodexCredentialStore::AutoKeyring,
            auth_file_path: std::path::PathBuf::from("/tmp/codex-auth.json"),
            load_error: None,
        };

        let resolution = codex::resolve_codex_live_data_with(
            bootstrap,
            |_| Box::pin(async { Err(anyhow!("oauth 503")) }),
            |_| Err(anyhow!("rpc failed")),
            |_| {
                Ok(codex::CliStatusSnapshot {
                    credits: Some(7.5),
                    primary: Some(crate::models::LiveRateWindow {
                        used_percent: 33.0,
                        resets_at: None,
                        resets_in_minutes: None,
                        window_minutes: None,
                        reset_label: None,
                    }),
                    secondary: None,
                })
            },
        )
        .await;

        assert!(resolution.available);
        assert_eq!(resolution.source_used, "cli-pty");
        assert!(resolution.resolved_via_fallback);
        assert_eq!(resolution.last_attempted_source.as_deref(), Some("cli-pty"));
        // All three rungs visible in attempts, in chain order.
        let chain: Vec<&str> = resolution
            .source_attempts
            .iter()
            .map(|a| a.source.as_str())
            .collect();
        assert_eq!(chain, vec!["cli-rpc", "oauth", "cli-pty"]);
        assert_eq!(resolution.source_attempts[0].outcome, "error");
        assert_eq!(resolution.source_attempts[1].outcome, "error");
        assert_eq!(resolution.source_attempts[2].outcome, "success");
    }

    #[tokio::test]
    async fn codex_all_sources_fail_returns_unavailable() {
        // Defensive: when every rung fails, snapshot stays unavailable, error
        // surfaces, and the recorded attempt chain is intact for diagnostics.
        let auth = codex::CodexAuth {
            access_token: "token".into(),
            refresh_token: None,
            id_token: None,
            account_id: None,
            auth_mode: Some("chatgpt".into()),
        };
        let bootstrap = codex::CodexBootstrapAuth {
            auth: Some(auth),
            credential_store: codex::ResolvedCodexCredentialStore::AutoKeyring,
            auth_file_path: std::path::PathBuf::from("/tmp/codex-auth.json"),
            load_error: None,
        };

        let resolution = codex::resolve_codex_live_data_with(
            bootstrap,
            |_| Box::pin(async { Err(anyhow!("oauth 401")) }),
            |_| Err(anyhow!("rpc failed")),
            |_| Err(anyhow!("cli-pty timeout")),
        )
        .await;

        assert!(!resolution.available);
        assert_eq!(resolution.source_used, "unavailable");
        assert!(!resolution.resolved_via_fallback);
        assert!(resolution.error.is_some());
        let chain: Vec<&str> = resolution
            .source_attempts
            .iter()
            .map(|a| a.source.as_str())
            .collect();
        assert_eq!(chain, vec!["cli-rpc", "oauth", "cli-pty"]);
        assert!(
            resolution
                .source_attempts
                .iter()
                .all(|a| a.outcome == "error")
        );
    }

    #[tokio::test]
    async fn stale_cache_returns_immediately_and_triggers_background_refresh() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let state = test_state();

        // Stale timestamp: 120s old, well past LIVE_PROVIDER_CACHE_SECS (60s).
        {
            let mut cache = state.live_provider_cache.write().await;
            *cache = Some((Instant::now() - std::time::Duration::from_secs(120), {
                let mut response = fixture_response("claude");
                response.response_scope = "all".into();
                response.requested_provider = None;
                response
            }));
        }

        // Counter incremented if the *foreground* fetcher path is accidentally
        // invoked. In the stale-while-revalidate path it must stay zero.
        let foreground_calls = Arc::new(AtomicUsize::new(0));

        let started = Instant::now();
        let response =
            load_snapshots_with_fetcher(&state, None, ResponseScope::All, false, false, {
                let foreground_calls = foreground_calls.clone();
                move |_, _, _, _| {
                    let foreground_calls = foreground_calls.clone();
                    async move {
                        foreground_calls.fetch_add(1, Ordering::SeqCst);
                        // Return a valid response so the test doesn't error if
                        // this path is ever reached during debugging.
                        let mut r = fixture_response("claude");
                        r.response_scope = "all".into();
                        r.requested_provider = None;
                        Ok(r)
                    }
                }
            })
            .await
            .unwrap();

        // The stale response must come back immediately — the foreground path
        // must never block on the background fetch.
        assert!(
            started.elapsed() < std::time::Duration::from_millis(50),
            "stale response took too long: {:?}",
            started.elapsed()
        );
        assert!(
            response.cache_hit,
            "stale response must be reported as a cache hit"
        );
        assert_eq!(
            foreground_calls.load(Ordering::SeqCst),
            0,
            "foreground fetcher must not be called when a stale cache entry exists"
        );

        // The cache entry must still exist (stale entry was not evicted).
        let cache = state.live_provider_cache.read().await;
        assert!(
            cache.is_some(),
            "cache must not be cleared by the stale-while-revalidate path"
        );
    }
}
