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
    use crate::webhooks::WebhookState;

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
            openai_enabled: false,
            openai_admin_key_env: "OPENAI_ADMIN_KEY".into(),
            openai_refresh_interval: 300,
            openai_lookback_days: 30,
            openai_cache: tokio::sync::RwLock::new(None),
            db_lock: tokio::sync::Mutex::new(()),
            webhook_state: tokio::sync::Mutex::new(WebhookState::default()),
            webhook_config: WebhookConfig::default(),
            scan_event_tx: tokio::sync::broadcast::channel::<String>(16).0,
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
            .route("/api/heatmap", get(crate::server::api::api_heatmap))
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
        assert!(html.contains("Usage"));
        // Preact mount points wired in by app.tsx.
        assert!(html.contains("header-mount"));
        assert!(html.contains("filter-bar-mount"));
        assert!(html.contains("inline-status-global"));
    }

    #[test]
    fn test_render_dashboard_has_xss_protection() {
        let html = assets::render_dashboard();
        // XSS safety is now provided by Preact's JSX rendering pipeline,
        // which funnels all text children through document.createTextNode
        // (never innerHTML). Verify the bundle retains that path and that
        // the asset placeholders have actually been substituted.
        assert!(html.contains("createTextNode"));
        assert!(!html.contains("__STYLE_CSS__"));
        assert!(!html.contains("__APP_JS__"));
    }

    // -----------------------------------------------------------------------
    // Phase 14 — Client-sent timezone: daily_by_model bucketing tests
    // -----------------------------------------------------------------------

    /// Seed a DB with two turns that straddle a UTC midnight and return the
    /// app router so integration tests can call `/api/data?tz_offset_min=N`.
    ///
    /// Turn A: 2026-04-17T23:30:00Z  (Apr 17 UTC)
    /// Turn B: 2026-04-18T00:30:00Z  (Apr 18 UTC)
    fn setup_tz_test_db(tmp: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
        let projects = tmp.path().join("projects").join("user").join("tzproj");
        std::fs::create_dir_all(&projects).unwrap();

        let filepath = projects.join("tz_sess.jsonl");
        let mut f = std::fs::File::create(&filepath).unwrap();

        // User message (required to anchor the session)
        writeln!(
            f,
            "{}",
            serde_json::json!({
                "type": "user",
                "sessionId": "tz-session",
                "timestamp": "2026-04-17T23:00:00Z",
                "cwd": "/home/user/tzproj"
            })
        )
        .unwrap();

        // Turn A — 2026-04-17T23:30:00Z
        writeln!(
            f,
            "{}",
            serde_json::json!({
                "type": "assistant",
                "sessionId": "tz-session",
                "timestamp": "2026-04-17T23:30:00Z",
                "cwd": "/home/user/tzproj",
                "message": {
                    "id": "tz-msg-1",
                    "model": "claude-sonnet-4-6",
                    "usage": {
                        "input_tokens": 100,
                        "output_tokens": 50,
                        "cache_read_input_tokens": 0,
                        "cache_creation_input_tokens": 0
                    },
                    "content": []
                }
            })
        )
        .unwrap();

        // Turn B — 2026-04-18T00:30:00Z
        writeln!(
            f,
            "{}",
            serde_json::json!({
                "type": "assistant",
                "sessionId": "tz-session",
                "timestamp": "2026-04-18T00:30:00Z",
                "cwd": "/home/user/tzproj",
                "message": {
                    "id": "tz-msg-2",
                    "model": "claude-sonnet-4-6",
                    "usage": {
                        "input_tokens": 200,
                        "output_tokens": 100,
                        "cache_read_input_tokens": 0,
                        "cache_creation_input_tokens": 0
                    },
                    "content": []
                }
            })
        )
        .unwrap();

        let db_path = tmp.path().join("tz_usage.db");
        let parent = tmp.path().join("projects");
        scanner::scan(Some(vec![parent.clone()]), &db_path, false).unwrap();

        (db_path, parent)
    }

    /// Fetch `/api/data` with an optional `tz_offset_min` query param and
    /// return the `daily_by_model` array from the JSON response.
    async fn fetch_daily_by_model(
        app: Router,
        tz_offset_min: Option<i32>,
    ) -> Vec<serde_json::Value> {
        let uri = match tz_offset_min {
            Some(offset) => format!("/api/data?tz_offset_min={offset}"),
            None => "/api/data".to_string(),
        };
        let resp = app
            .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        data["daily_by_model"].as_array().unwrap().clone()
    }

    /// UTC (offset=0): two turns on different UTC days → two buckets.
    #[tokio::test]
    async fn test_tz_utc_produces_two_day_buckets() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_tz_test_db(&tmp);
        let app = test_app(db_path, projects);

        let rows = fetch_daily_by_model(app, Some(0)).await;

        let days: Vec<&str> = rows.iter().map(|r| r["day"].as_str().unwrap()).collect();
        assert!(
            days.contains(&"2026-04-17"),
            "expected 2026-04-17 bucket, got: {days:?}"
        );
        assert!(
            days.contains(&"2026-04-18"),
            "expected 2026-04-18 bucket, got: {days:?}"
        );
        assert_eq!(days.len(), 2, "expected exactly 2 buckets, got: {days:?}");
    }

    /// UTC+2 (offset=+120): both turns shift to Apr 18 local → one bucket.
    ///
    /// Turn A: 2026-04-17T23:30Z + 120 min = 2026-04-18T01:30 local
    /// Turn B: 2026-04-18T00:30Z + 120 min = 2026-04-18T02:30 local
    #[tokio::test]
    async fn test_tz_plus120_collapses_to_one_bucket() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_tz_test_db(&tmp);
        let app = test_app(db_path, projects);

        let rows = fetch_daily_by_model(app, Some(120)).await;

        let days: Vec<&str> = rows.iter().map(|r| r["day"].as_str().unwrap()).collect();
        assert_eq!(
            days.len(),
            1,
            "UTC+2: expected 1 bucket (both turns on Apr 18 local), got: {days:?}"
        );
        assert_eq!(
            days[0], "2026-04-18",
            "UTC+2: bucket should be Apr 18, got: {}",
            days[0]
        );
    }

    /// UTC-8 (offset=-480): both turns shift to Apr 17 local → one bucket.
    ///
    /// Turn A: 2026-04-17T23:30Z − 480 min = 2026-04-17T15:30 local
    /// Turn B: 2026-04-18T00:30Z − 480 min = 2026-04-17T16:30 local
    #[tokio::test]
    async fn test_tz_minus480_collapses_to_one_bucket() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_tz_test_db(&tmp);
        let app = test_app(db_path, projects);

        let rows = fetch_daily_by_model(app, Some(-480)).await;

        let days: Vec<&str> = rows.iter().map(|r| r["day"].as_str().unwrap()).collect();
        assert_eq!(
            days.len(),
            1,
            "UTC-8: expected 1 bucket (both turns on Apr 17 local), got: {days:?}"
        );
        assert_eq!(
            days[0], "2026-04-17",
            "UTC-8: bucket should be Apr 17, got: {}",
            days[0]
        );
    }

    /// No tz param (omitted) behaves identically to UTC: two buckets.
    #[tokio::test]
    async fn test_tz_absent_defaults_to_utc() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_tz_test_db(&tmp);
        let app = test_app(db_path, projects);

        let rows = fetch_daily_by_model(app, None).await;

        let days: Vec<&str> = rows.iter().map(|r| r["day"].as_str().unwrap()).collect();
        assert_eq!(
            days.len(),
            2,
            "No offset: expected 2 UTC buckets, got: {days:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Phase 13 — 7×24 Activity Heatmap tests
    // -----------------------------------------------------------------------

    /// Helper: build a DB with no turns and return the app router.
    fn empty_db_app(tmp: &TempDir) -> Router {
        let db_path = tmp.path().join("empty.db");
        let conn = crate::scanner::db::open_db(&db_path).unwrap();
        crate::scanner::db::init_db(&conn).unwrap();
        drop(conn);
        let projects = tmp.path().join("projects");
        std::fs::create_dir_all(&projects).unwrap();
        test_app(db_path, projects)
    }

    /// Helper: build a DB seeded with specific turns and return the app router.
    fn seeded_heatmap_db(tmp: &TempDir, entries: &[(&str, &str, i64, i64)]) -> Router {
        // entries: (session_id, timestamp, input_tokens, output_tokens)
        let projects = tmp.path().join("projects").join("user").join("hm");
        std::fs::create_dir_all(&projects).unwrap();
        let filepath = projects.join("hm.jsonl");
        let mut f = std::fs::File::create(&filepath).unwrap();

        for (i, (session_id, ts, _input, _output)) in entries.iter().enumerate() {
            // user message to anchor session
            writeln!(
                f,
                "{}",
                serde_json::json!({
                    "type": "user",
                    "sessionId": session_id,
                    "timestamp": ts,
                    "cwd": "/tmp/hm"
                })
            )
            .unwrap();
            writeln!(
                f,
                "{}",
                serde_json::json!({
                    "type": "assistant",
                    "sessionId": session_id,
                    "timestamp": ts,
                    "cwd": "/tmp/hm",
                    "message": {
                        "id": format!("msg-hm-{i}"),
                        "model": "claude-sonnet-4-6",
                        "usage": {
                            "input_tokens": entries[i].2,
                            "output_tokens": entries[i].3,
                            "cache_read_input_tokens": 0,
                            "cache_creation_input_tokens": 0
                        },
                        "content": []
                    }
                })
            )
            .unwrap();
        }

        let db_path = tmp.path().join("hm.db");
        let parent = tmp.path().join("projects");
        crate::scanner::scan(Some(vec![parent.clone()]), &db_path, false).unwrap();
        test_app(db_path, parent)
    }

    /// Empty DB → /api/heatmap returns 168 cells, all zero.
    #[tokio::test]
    async fn test_heatmap_empty_db_returns_168_zero_cells() {
        let tmp = TempDir::new().unwrap();
        let app = empty_db_app(&tmp);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/heatmap?period=all")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let cells = data["cells"].as_array().unwrap();
        assert_eq!(cells.len(), 168, "expected exactly 168 cells");
        for cell in cells {
            assert_eq!(cell["cost_nanos"].as_i64().unwrap(), 0);
            assert_eq!(cell["call_count"].as_i64().unwrap(), 0);
        }
        assert_eq!(data["max_cost_nanos"].as_i64().unwrap(), 0);
        assert_eq!(data["active_days"].as_i64().unwrap(), 0);
    }

    /// Seeded turns on a specific known timestamp → correct cell non-zero.
    /// 2026-04-17T10:30:00Z → Friday (dow=5) hour=10.
    #[tokio::test]
    async fn test_heatmap_seeded_turns_correct_cell() {
        let tmp = TempDir::new().unwrap();
        // 2026-04-17T10:30:00Z: strftime('%w')=5 (Fri), strftime('%H')=10
        let entries = [
            ("s-hm1", "2026-04-17T10:30:00Z", 1000i64, 500i64),
            ("s-hm2", "2026-04-17T10:45:00Z", 800i64, 400i64),
            ("s-hm3", "2026-04-17T14:00:00Z", 200i64, 100i64),
        ];
        let app = seeded_heatmap_db(&tmp, &entries);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/heatmap?period=all")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let cells = data["cells"].as_array().unwrap();
        assert_eq!(cells.len(), 168, "must still be 168 cells");

        // Find cell dow=5, hour=10 (Friday 10:00)
        let cell_fri_10 = cells
            .iter()
            .find(|c| c["dow"].as_i64() == Some(5) && c["hour"].as_i64() == Some(10))
            .expect("cell dow=5 hour=10 not found");
        assert!(
            cell_fri_10["call_count"].as_i64().unwrap() >= 2,
            "two turns at hour 10 → call_count >= 2"
        );
        assert!(
            cell_fri_10["cost_nanos"].as_i64().unwrap() > 0,
            "cost_nanos must be positive for non-zero turns"
        );

        // Find cell dow=5, hour=14
        let cell_fri_14 = cells
            .iter()
            .find(|c| c["dow"].as_i64() == Some(5) && c["hour"].as_i64() == Some(14))
            .expect("cell dow=5 hour=14 not found");
        assert!(
            cell_fri_14["call_count"].as_i64().unwrap() >= 1,
            "one turn at hour 14"
        );

        // active_days should be 1 (all turns on 2026-04-17)
        assert_eq!(
            data["active_days"].as_i64().unwrap(),
            1,
            "all turns same day → 1 active day"
        );
    }

    /// TZ shift: turn at 2026-04-17T23:30:00Z with +120 min offset → hour=1 on Apr 18.
    #[tokio::test]
    async fn test_heatmap_tz_shift_moves_hour() {
        let tmp = TempDir::new().unwrap();
        // 2026-04-17T23:30:00Z + 120min = 2026-04-18T01:30 local → hour=1, dow=6 (Sat)
        let entries = [("s-tz", "2026-04-17T23:30:00Z", 1000i64, 500i64)];
        let app = seeded_heatmap_db(&tmp, &entries);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/heatmap?period=all&tz_offset_min=120")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let cells = data["cells"].as_array().unwrap();

        // UTC: dow=5 (Fri), hour=23. Shifted: dow=6 (Sat), hour=1.
        let cell_shifted = cells
            .iter()
            .find(|c| c["dow"].as_i64() == Some(6) && c["hour"].as_i64() == Some(1))
            .expect("shifted cell dow=6 hour=1 not found");
        assert!(
            cell_shifted["call_count"].as_i64().unwrap() >= 1,
            "turn should appear at shifted hour=1"
        );

        // Original UTC slot should be empty.
        let cell_utc = cells
            .iter()
            .find(|c| c["dow"].as_i64() == Some(5) && c["hour"].as_i64() == Some(23))
            .unwrap();
        assert_eq!(
            cell_utc["call_count"].as_i64().unwrap(),
            0,
            "UTC slot must be empty after shift"
        );
    }

    /// active_days counts distinct spend days correctly.
    #[tokio::test]
    async fn test_heatmap_active_days_counting() {
        let tmp = TempDir::new().unwrap();
        // Three turns across two days.
        let entries = [
            ("s-ad1", "2026-04-15T09:00:00Z", 1000i64, 500i64),
            ("s-ad2", "2026-04-15T15:00:00Z", 800i64, 400i64),
            ("s-ad3", "2026-04-16T11:00:00Z", 200i64, 100i64),
        ];
        let app = seeded_heatmap_db(&tmp, &entries);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/heatmap?period=all")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            data["active_days"].as_i64().unwrap(),
            2,
            "turns on 2 distinct days → active_days=2"
        );
    }
}
