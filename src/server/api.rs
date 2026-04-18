use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum::response::sse::{Event, KeepAlive, Sse};
use serde_json::Value;
use tokio::sync::{Mutex, RwLock, broadcast};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::tz::TzParams;

use crate::config::WebhookConfig;
use crate::oauth;
use crate::oauth::models::UsageWindowsResponse;
use crate::openai;
use crate::scanner;
use crate::scanner::db;
use crate::webhooks::{self, WebhookState};

pub struct AppState {
    pub db_path: PathBuf,
    pub projects_dirs: Option<Vec<PathBuf>>,
    pub oauth_enabled: bool,
    pub oauth_refresh_interval: u64,
    pub oauth_cache: RwLock<Option<(Instant, UsageWindowsResponse)>>,
    pub openai_enabled: bool,
    pub openai_admin_key_env: String,
    pub openai_refresh_interval: u64,
    pub openai_lookback_days: i64,
    pub openai_cache: RwLock<Option<(Instant, crate::models::OpenAiReconciliation)>>,
    pub db_lock: Mutex<()>,
    pub webhook_state: Mutex<WebhookState>,
    pub webhook_config: WebhookConfig,
    /// Phase 20: broadcast channel for SSE scan_completed events.
    /// The watcher fires into this channel; SSE subscribers receive the events.
    pub scan_event_tx: broadcast::Sender<String>,
}

pub async fn api_data(
    State(state): State<Arc<AppState>>,
    Query(tz): Query<TzParams>,
) -> Result<Json<Value>, StatusCode> {
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
            Some(fetch_openai_reconciliation(&state, openai_local_cost_nanos).await);
    }

    maybe_send_cost_threshold_webhook(&state, &result).await;

    let value = serde_json::to_value(result).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(value))
}

async fn fetch_openai_reconciliation(
    state: &Arc<AppState>,
    local_cost_nanos: i64,
) -> crate::models::OpenAiReconciliation {
    {
        let cache = state.openai_cache.read().await;
        if let Some((fetched_at, ref data)) = *cache
            && fetched_at.elapsed().as_secs() < state.openai_refresh_interval
        {
            return data.clone();
        }
    }

    let estimated_local_cost = local_cost_nanos as f64 / 1_000_000_000.0;
    let reconciliation = match std::env::var(&state.openai_admin_key_env) {
        Ok(admin_key) if !admin_key.trim().is_empty() => {
            openai::fetch_org_usage_reconciliation(
                admin_key.trim(),
                state.openai_lookback_days,
                estimated_local_cost,
            )
            .await
        }
        _ => crate::models::OpenAiReconciliation {
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

    {
        let mut cache = state.openai_cache.write().await;
        *cache = Some((Instant::now(), reconciliation.clone()));
    }

    reconciliation
}

pub async fn api_rescan(State(state): State<Arc<AppState>>) -> Result<Json<Value>, StatusCode> {
    let _db_guard = state.db_lock.lock().await;
    let db_path = state.db_path.clone();
    let projects_dirs = state.projects_dirs.clone();

    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        // Atomic rescan: write to temp, then rename
        let temp_path = db_path.with_extension("db.tmp");
        cleanup_sqlite_files(&temp_path)?;
        let scan_result = scanner::scan(projects_dirs, &temp_path, false)?;
        if temp_path.exists() {
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
) -> Result<Json<Value>, StatusCode> {
    if !state.oauth_enabled {
        let value = serde_json::to_value(UsageWindowsResponse::unavailable())
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        return Ok(Json(value));
    }

    // Check cache
    {
        let cache = state.oauth_cache.read().await;
        if let Some((fetched_at, ref data)) = *cache
            && fetched_at.elapsed().as_secs() < state.oauth_refresh_interval
        {
            let value =
                serde_json::to_value(data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            return Ok(Json(value));
        }
    }

    // Cache miss or expired: fetch fresh data
    let resp = oauth::poll_usage().await;

    if let Some(session) = resp.session.as_ref() {
        maybe_send_session_webhook(&state, session.used_percent, session.resets_in_minutes).await;
    }

    // Update cache
    {
        let mut cache = state.oauth_cache.write().await;
        *cache = Some((Instant::now(), resp.clone()));
    }

    let value = serde_json::to_value(resp).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(value))
}

pub async fn api_health() -> &'static str {
    "ok"
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
) -> Result<Json<Value>, StatusCode> {
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
pub async fn api_stream(State(state): State<Arc<AppState>>) -> impl axum::response::IntoResponse {
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

    Sse::new(event_stream).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
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
