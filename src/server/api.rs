use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::config::WebhookConfig;
use crate::oauth;
use crate::oauth::models::UsageWindowsResponse;
use crate::scanner;
use crate::scanner::db;

pub struct AppState {
    pub db_path: PathBuf,
    pub projects_dirs: Option<Vec<PathBuf>>,
    pub oauth_enabled: bool,
    pub oauth_refresh_interval: u64,
    pub oauth_cache: RwLock<Option<(Instant, UsageWindowsResponse)>>,
    #[allow(dead_code)]
    pub webhook_config: WebhookConfig,
}

pub async fn api_data(State(state): State<Arc<AppState>>) -> Result<Json<Value>, StatusCode> {
    let db_path = state.db_path.clone();
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let conn = db::open_db(&db_path)?;
        db::init_db(&conn)?;
        db::get_dashboard_data(&conn)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let value = serde_json::to_value(result).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(value))
}

pub async fn api_rescan(State(state): State<Arc<AppState>>) -> Result<Json<Value>, StatusCode> {
    let db_path = state.db_path.clone();
    let projects_dirs = state.projects_dirs.clone();

    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        // Atomic rescan: write to temp, then rename
        let temp_path = db_path.with_extension("db.tmp");
        let scan_result = scanner::scan(projects_dirs, &temp_path, false)?;
        if temp_path.exists() {
            std::fs::rename(&temp_path, &db_path)?;
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
