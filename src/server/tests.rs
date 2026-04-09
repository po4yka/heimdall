#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::Arc;
    use tempfile::TempDir;

    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::response::Html;
    use axum::routing::{get, post};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::config::WebhookConfig;
    use crate::scanner;
    use crate::server::api::{AppState, api_data, api_health, api_rescan};
    use crate::server::assets;

    fn make_assistant(session_id: &str, input: i64, output: i64, msg_id: &str) -> String {
        let mut msg = serde_json::json!({
            "model": "claude-sonnet-4-6",
            "usage": {
                "input_tokens": input,
                "output_tokens": output,
                "cache_read_input_tokens": 0,
                "cache_creation_input_tokens": 0,
            },
            "content": [],
        });
        if !msg_id.is_empty() {
            msg["id"] = serde_json::json!(msg_id);
        }
        serde_json::json!({
            "type": "assistant",
            "sessionId": session_id,
            "timestamp": "2026-04-08T10:00:00Z",
            "cwd": "/home/user/project",
            "message": msg,
        })
        .to_string()
    }

    fn setup_test_db(tmp: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
        let projects = tmp.path().join("projects").join("user").join("proj");
        std::fs::create_dir_all(&projects).unwrap();

        let filepath = projects.join("sess.jsonl");
        let mut f = std::fs::File::create(&filepath).unwrap();
        writeln!(
            f,
            "{}",
            serde_json::json!({"type": "user", "sessionId": "s1", "timestamp": "2026-04-08T09:00:00Z", "cwd": "/home/user/project"})
        )
        .unwrap();
        writeln!(f, "{}", make_assistant("s1", 1000, 500, "msg-1")).unwrap();

        let db_path = tmp.path().join("usage.db");
        let parent = tmp.path().join("projects");
        scanner::scan(Some(vec![parent.clone()]), &db_path, false).unwrap();

        (db_path, parent)
    }

    fn test_app(db_path: std::path::PathBuf, projects_dir: std::path::PathBuf) -> Router {
        let state = Arc::new(AppState {
            db_path,
            projects_dirs: Some(vec![projects_dir]),
            oauth_enabled: false,
            oauth_refresh_interval: 60,
            oauth_cache: tokio::sync::RwLock::new(None),
            webhook_config: WebhookConfig::default(),
        });
        let html = assets::render_dashboard();

        Router::new()
            .route(
                "/",
                get({
                    let h = html.clone();
                    move || async { Html(h) }
                }),
            )
            .route("/api/data", get(api_data))
            .route("/api/rescan", post(api_rescan))
            .route("/api/health", get(api_health))
            .route(
                "/api/usage-windows",
                get(crate::server::api::api_usage_windows),
            )
            .with_state(state)
    }

    #[tokio::test]
    async fn test_index_returns_html() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app(db_path, projects);

        let resp = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(ct.contains("text/html"));
    }

    #[tokio::test]
    async fn test_api_data_returns_json() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app(db_path, projects);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/data")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(data.get("all_models").is_some());
        assert!(data.get("sessions_all").is_some());
        assert!(data.get("daily_by_model").is_some());
        assert!(data.get("generated_at").is_some());
    }

    #[tokio::test]
    async fn test_api_data_has_correct_values() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app(db_path, projects);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/data")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let models = data["all_models"].as_array().unwrap();
        assert!(
            models
                .iter()
                .any(|m| m.as_str() == Some("claude-sonnet-4-6"))
        );

        let sessions = data["sessions_all"].as_array().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0]["input"].as_i64().unwrap(), 1000);
        assert_eq!(sessions[0]["output"].as_i64().unwrap(), 500);
    }

    #[tokio::test]
    async fn test_api_rescan_returns_json() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app(db_path, projects);

        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/rescan")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(data.get("new").is_some());
        assert!(data.get("updated").is_some());
        assert!(data.get("skipped").is_some());
    }

    #[tokio::test]
    async fn test_api_health() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app(db_path, projects);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body.as_ref(), b"ok");
    }

    #[tokio::test]
    async fn test_404_for_unknown_path() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app(db_path, projects);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_api_usage_windows_disabled() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app(db_path, projects); // oauth_enabled=false
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/usage-windows")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(data["available"], false);
    }

    #[tokio::test]
    async fn test_api_data_has_subagent_summary() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app(db_path, projects);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/data")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(data.get("subagent_summary").is_some());
        assert!(data.get("entrypoint_breakdown").is_some());
        assert!(data.get("service_tiers").is_some());
    }

    #[test]
    fn test_render_dashboard_has_structure() {
        let html = assets::render_dashboard();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("apexcharts"));
        assert!(html.contains("Claude"));
    }

    #[test]
    fn test_render_dashboard_has_xss_protection() {
        let html = assets::render_dashboard();
        assert!(html.contains("function esc("));
    }
}
