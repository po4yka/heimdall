#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::net::SocketAddr;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tempfile::TempDir;

    /// Tests that mutate `HOME` to redirect `dirs::home_dir()` into a tempdir
    /// must serialize against each other — `HOME` is process-global and
    /// `cargo test` runs tests in parallel by default.
    static HOME_LOCK: Mutex<()> = Mutex::new(());

    use axum::Router;
    use axum::body::Body;
    use axum::extract::State;
    use axum::http::{Request, StatusCode};
    use axum::response::Html;
    use axum::routing::get;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::config::{AgentStatusConfig, AggregatorConfig, WebhookConfig};
    use crate::models::OpenAiReconciliation;
    use crate::oauth::models::{BudgetInfo, Identity, Plan, UsageWindowsResponse, WindowInfo};
    use crate::scanner;
    use crate::server::api::{
        AppState, CostReconciliationParams, HeatmapParams, api_agent_status, api_billing_blocks,
        api_claude_usage, api_community_signal, api_context_window, api_cost_reconciliation,
        api_data, api_heatmap, api_live_monitor, api_live_provider_history,
        api_live_provider_refresh, api_live_providers, api_mobile_snapshot, api_rescan, api_stream,
        api_usage_windows,
    };
    use crate::server::assets;
    use crate::server::{ServeOptions, build_router, build_state, start_background_pollers_with};
    use crate::tz::TzParams;
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

    fn test_options(db_path: std::path::PathBuf, projects_dir: std::path::PathBuf) -> ServeOptions {
        test_options_with_agent_status(db_path, projects_dir, AgentStatusConfig::default())
    }

    fn test_options_with_agent_status(
        db_path: std::path::PathBuf,
        projects_dir: std::path::PathBuf,
        agent_status_config: AgentStatusConfig,
    ) -> ServeOptions {
        ServeOptions {
            host: "127.0.0.1".into(),
            port: 0,
            db_path,
            projects_dirs: Some(vec![projects_dir]),
            oauth_enabled: false,
            oauth_refresh_interval: 60,
            claude_admin_enabled: false,
            claude_admin_key_env: "ANTHROPIC_ADMIN_KEY".into(),
            claude_admin_refresh_interval: 300,
            claude_admin_lookback_days: 30,
            openai_enabled: false,
            openai_admin_key_env: "OPENAI_ADMIN_KEY".into(),
            openai_refresh_interval: 300,
            openai_lookback_days: 30,
            webhook_config: WebhookConfig::default(),
            watch: false,
            background_poll: false,
            agent_status_config,
            aggregator_config: AggregatorConfig::default(),
            blocks_token_limit: None,
            session_length_hours: 5.0,
            project_aliases: std::collections::HashMap::new(),
        }
    }

    fn test_app(db_path: std::path::PathBuf, projects_dir: std::path::PathBuf) -> Router {
        let options = test_options(db_path, projects_dir);
        build_router(build_state(
            &options,
            tokio::sync::broadcast::channel::<String>(16).0,
        ))
    }

    fn test_app_with_agent_status(
        db_path: std::path::PathBuf,
        projects_dir: std::path::PathBuf,
        agent_status_config: AgentStatusConfig,
    ) -> Router {
        let options = test_options_with_agent_status(db_path, projects_dir, agent_status_config);
        build_router(build_state(
            &options,
            tokio::sync::broadcast::channel::<String>(16).0,
        ))
    }

    fn base_state(db_path: std::path::PathBuf, projects_dir: std::path::PathBuf) -> AppState {
        AppState {
            db_path,
            projects_dirs: Some(vec![projects_dir]),
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
            openai_refresh_interval: 300,
            openai_lookback_days: 30,
            openai_cache: tokio::sync::RwLock::new(None),
            openai_refresh_lock: tokio::sync::Mutex::new(()),
            db_lock: tokio::sync::Mutex::new(()),
            webhook_state: tokio::sync::Mutex::new(WebhookState::default()),
            webhook_config: WebhookConfig::default(),
            scan_event_tx: tokio::sync::broadcast::channel::<String>(16).0,
            agent_status_config: AgentStatusConfig::default(),
            agent_status_cache: tokio::sync::RwLock::new(None),
            agent_status_refresh_lock: tokio::sync::Mutex::new(()),
            aggregator_config: AggregatorConfig::default(),
            aggregator_cache: tokio::sync::RwLock::new(None),
            aggregator_refresh_lock: tokio::sync::Mutex::new(()),
            blocks_token_limit: None,
            session_length_hours: 5.0,
            project_aliases: std::collections::HashMap::new(),
            live_provider_cache: tokio::sync::RwLock::new(None),
            live_provider_refresh_lock: tokio::sync::Mutex::new(()),
        }
    }

    fn cached_live_provider_response(stale_codex: bool) -> crate::models::LiveProvidersResponse {
        crate::models::LiveProvidersResponse {
            contract_version: crate::models::LIVE_PROVIDERS_CONTRACT_VERSION,
            providers: vec![
                crate::models::LiveProviderSnapshot {
                    provider: "claude".into(),
                    available: true,
                    source_used: "oauth".into(),
                    last_attempted_source: Some("oauth".into()),
                    resolved_via_fallback: false,
                    refresh_duration_ms: 1,
                    source_attempts: vec![],
                    identity: None,
                    primary: Some(crate::models::LiveRateWindow {
                        used_percent: 42.0,
                        resets_at: Some("2026-04-21T18:00:00Z".into()),
                        resets_in_minutes: Some(120),
                        window_minutes: Some(300),
                        reset_label: Some("2h".into()),
                    }),
                    secondary: None,
                    tertiary: None,
                    credits: None,
                    status: None,
                    auth: crate::models::LiveProviderAuth::default(),
                    cost_summary: crate::models::ProviderCostSummary::default(),
                    claude_usage: None,
                    claude_admin: None,
                    quota_suggestions: Some(crate::models::LiveQuotaSuggestions {
                        sample_count: 3,
                        population_count: 3,
                        recommended_key: "p90".into(),
                        sample_strategy: "completed_blocks".into(),
                        sample_label: "3 completed blocks".into(),
                        levels: vec![
                            crate::models::LiveQuotaSuggestionLevel {
                                key: "p90".into(),
                                label: "P90".into(),
                                limit_tokens: 800_000,
                            },
                            crate::models::LiveQuotaSuggestionLevel {
                                key: "p95".into(),
                                label: "P95".into(),
                                limit_tokens: 900_000,
                            },
                            crate::models::LiveQuotaSuggestionLevel {
                                key: "max".into(),
                                label: "Max".into(),
                                limit_tokens: 950_000,
                            },
                        ],
                        note: Some("Based on fewer than 10 completed blocks.".into()),
                    }),
                    depletion_forecast: Some(crate::models::DepletionForecast {
                        primary_signal: crate::models::DepletionForecastSignal {
                            kind: "primary_window".into(),
                            title: "Primary window".into(),
                            used_percent: 42.0,
                            projected_percent: None,
                            remaining_tokens: None,
                            remaining_percent: Some(58.0),
                            resets_in_minutes: Some(120),
                            pace_label: Some("Steady".into()),
                            end_time: Some("2026-04-21T18:00:00Z".into()),
                        },
                        secondary_signals: vec![],
                        summary_label: "Primary window currently at 42% used".into(),
                        severity: "ok".into(),
                        note: None,
                    }),
                    predictive_insights: Some(crate::models::LivePredictiveInsights {
                        rolling_hour_burn: Some(crate::models::LivePredictiveBurnRate {
                            tokens_per_min: 2800.0,
                            cost_per_hour_nanos: 1_200_000_000,
                            coverage_minutes: 40,
                            tier: "moderate".into(),
                        }),
                        historical_envelope: Some(crate::models::LiveHistoricalEnvelope {
                            sample_count: 7,
                            tokens: crate::models::LiveIntegerPercentiles {
                                average: 640_000,
                                p50: 600_000,
                                p75: 700_000,
                                p90: 900_000,
                                p95: 940_000,
                            },
                            cost_usd: crate::models::LiveFloatPercentiles {
                                average: 3.8,
                                p50: 3.1,
                                p75: 4.6,
                                p90: 5.4,
                                p95: 5.8,
                            },
                            turns: crate::models::LiveIntegerPercentiles {
                                average: 14,
                                p50: 12,
                                p75: 16,
                                p90: 20,
                                p95: 22,
                            },
                        }),
                        limit_hit_analysis: Some(crate::models::LiveLimitHitAnalysis {
                            sample_count: 7,
                            hit_count: 2,
                            hit_rate: 2.0 / 7.0,
                            threshold_tokens: 900_000,
                            threshold_percent: 90.0,
                            active_current_hit: Some(false),
                            active_projected_hit: Some(true),
                            risk_level: "high".into(),
                            summary_label: "2 of 7 completed blocks reached 90% of the configured limit · active block is on pace to join them".into(),
                        }),
                    }),
                    last_refresh: "2026-04-21T09:00:00Z".into(),
                    stale: false,
                    error: None,
                },
                crate::models::LiveProviderSnapshot {
                    provider: "codex".into(),
                    available: true,
                    source_used: "cli-rpc".into(),
                    last_attempted_source: Some("cli-rpc".into()),
                    resolved_via_fallback: false,
                    refresh_duration_ms: 2,
                    source_attempts: vec![],
                    identity: None,
                    primary: Some(crate::models::LiveRateWindow {
                        used_percent: 18.0,
                        resets_at: Some("2026-04-21T17:30:00Z".into()),
                        resets_in_minutes: Some(90),
                        window_minutes: Some(60),
                        reset_label: Some("90m".into()),
                    }),
                    secondary: None,
                    tertiary: None,
                    credits: Some(12.5),
                    status: None,
                    auth: crate::models::LiveProviderAuth::default(),
                    cost_summary: crate::models::ProviderCostSummary::default(),
                    claude_usage: None,
                    claude_admin: None,
                    quota_suggestions: None,
                    depletion_forecast: Some(crate::models::DepletionForecast {
                        primary_signal: crate::models::DepletionForecastSignal {
                            kind: "primary_window".into(),
                            title: "Primary window".into(),
                            used_percent: 18.0,
                            projected_percent: None,
                            remaining_tokens: None,
                            remaining_percent: Some(82.0),
                            resets_in_minutes: Some(90),
                            pace_label: Some("Comfortable".into()),
                            end_time: Some("2026-04-21T17:30:00Z".into()),
                        },
                        secondary_signals: vec![],
                        summary_label: "Primary window currently at 18% used".into(),
                        severity: "ok".into(),
                        note: None,
                    }),
                    predictive_insights: None,
                    last_refresh: "2026-04-21T08:30:00Z".into(),
                    stale: stale_codex,
                    error: None,
                },
            ],
            fetched_at: "2026-04-21T09:00:00Z".into(),
            requested_provider: None,
            response_scope: "all".into(),
            cache_hit: false,
            refreshed_providers: vec!["claude".into(), "codex".into()],
            local_notification_state: Some(crate::models::LocalNotificationState {
                generated_at: "2026-04-21T09:00:00Z".into(),
                cost_threshold_usd: Some(25.0),
                conditions: vec![crate::models::LocalNotificationCondition {
                    id: "claude-session-depleted".into(),
                    kind: "session_depleted".into(),
                    provider: Some("claude".into()),
                    service_label: "Claude".into(),
                    is_active: false,
                    activation_title: "Claude session depleted".into(),
                    activation_body: "Claude session is depleted.".into(),
                    recovery_title: Some("Claude session restored".into()),
                    recovery_body: Some("Claude session capacity is available again.".into()),
                    day_key: None,
                }],
            }),
        }
    }

    fn available_usage_response(
        used_percent: f64,
        resets_in_minutes: i64,
        tier: &str,
    ) -> UsageWindowsResponse {
        UsageWindowsResponse {
            available: true,
            source: "oauth".into(),
            session: Some(WindowInfo {
                used_percent,
                resets_at: Some("2099-01-01T00:00:00Z".into()),
                resets_in_minutes: Some(resets_in_minutes),
            }),
            weekly: None,
            weekly_opus: None,
            weekly_sonnet: None,
            budget: Some(BudgetInfo {
                used: 12.5,
                limit: 50.0,
                currency: "USD".into(),
                utilization: 25.0,
            }),
            identity: Some(Identity {
                plan: Some(Plan::Pro),
                rate_limit_tier: Some(tier.into()),
            }),
            admin_fallback: None,
            error: None,
        }
    }

    fn available_openai_reconciliation(
        estimated_local_cost: f64,
        api_usage_cost: f64,
        api_requests: i64,
    ) -> OpenAiReconciliation {
        OpenAiReconciliation {
            available: true,
            lookback_days: 7,
            start_date: "2026-04-13".into(),
            end_date: "2026-04-19".into(),
            estimated_local_cost,
            api_usage_cost,
            api_input_tokens: 1_000,
            api_output_tokens: 500,
            api_cached_input_tokens: 250,
            api_requests,
            delta_cost: api_usage_cost - estimated_local_cost,
            error: None,
        }
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
    async fn test_real_router_wires_dashboard_aliases_and_favicon() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app(db_path, projects);

        let root = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let root_body = root.into_body().collect().await.unwrap().to_bytes();

        let index = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/index.html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(index.status(), StatusCode::OK);
        let index_body = index.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            root_body, index_body,
            "/ and /index.html should share the same asset"
        );

        let favicon = app
            .oneshot(
                Request::builder()
                    .uri("/favicon.ico")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(favicon.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_real_router_registers_all_bootstrap_routes() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app(db_path, projects);

        let cases = [
            ("GET", "/", StatusCode::OK),
            ("GET", "/index.html", StatusCode::OK),
            ("GET", "/monitor", StatusCode::OK),
            ("GET", "/favicon.ico", StatusCode::NO_CONTENT),
            ("GET", "/api/data", StatusCode::OK),
            ("POST", "/api/rescan", StatusCode::OK),
            ("GET", "/api/usage-windows", StatusCode::OK),
            ("GET", "/api/claude-usage", StatusCode::OK),
            ("GET", "/api/health", StatusCode::OK),
            ("GET", "/api/heatmap", StatusCode::OK),
            ("GET", "/api/stream", StatusCode::OK),
            ("GET", "/api/agent-status", StatusCode::OK),
            ("GET", "/api/community-signal", StatusCode::OK),
            ("GET", "/api/billing-blocks", StatusCode::OK),
            ("GET", "/api/context-window", StatusCode::OK),
            ("GET", "/api/live-monitor", StatusCode::OK),
            ("GET", "/api/cost-reconciliation", StatusCode::OK),
        ];

        for (method, path, expected) in cases {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(path)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                expected,
                "route {method} {path} should be wired"
            );
        }

        let wrong_method = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/rescan")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(wrong_method.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_api_rescan_preserves_runtime_history_tables() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        {
            let conn = crate::scanner::db::open_db(&db_path).unwrap();
            crate::scanner::db::init_db(&conn).unwrap();
            conn.execute(
                "INSERT OR IGNORE INTO live_events
                    (dedup_key, received_at, session_id, tool_name, cost_usd_nanos,
                     input_tokens, output_tokens, raw_json, context_input_tokens,
                     context_window_size, hook_reported_cost_nanos)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    "sess-1:tool-1",
                    "2026-04-08T10:05:00Z",
                    "sess-1",
                    "Read",
                    1234_i64,
                    10_i64,
                    5_i64,
                    "{}",
                    77_i64,
                    200_000_i64,
                    1234_i64,
                ],
            )
            .unwrap();
            conn.execute(
                "INSERT OR IGNORE INTO agent_status_history
                    (ts_epoch, provider, component_id, component_name, status)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![1_712_570_400_i64, "claude", "cid-1", "API", "major_outage"],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO rate_window_history
                    (timestamp, window_type, used_percent, resets_at, source_kind, source_path)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    "2026-04-08T10:05:00Z",
                    "session",
                    91.5_f64,
                    "2026-04-08T11:00:00Z",
                    "oauth",
                    "",
                ],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO claude_usage_runs
                    (captured_at, status, exit_code, stdout_raw, stderr_raw, invocation_mode, period, parser_version, error_summary)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    "2026-04-08T10:05:00Z",
                    "success",
                    0_i64,
                    "stdout",
                    "",
                    "print_slash_command",
                    "today",
                    "v1",
                    "",
                ],
            )
            .unwrap();
            let run_id = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO claude_usage_factors
                    (run_id, factor_key, display_label, percent, description, advice_text, display_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    run_id,
                    "parallel_sessions",
                    "was while 4+ sessions ran in parallel",
                    98.0_f64,
                    "All sessions share one limit.",
                    "All sessions share one limit.",
                    0_i64,
                ],
            )
            .unwrap();
        }

        let app = test_app(db_path.clone(), projects);
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

        let conn = crate::scanner::db::open_db(&db_path).unwrap();
        let live_events: i64 = conn
            .query_row("SELECT COUNT(*) FROM live_events", [], |r| r.get(0))
            .unwrap();
        let agent_history: i64 = conn
            .query_row("SELECT COUNT(*) FROM agent_status_history", [], |r| {
                r.get(0)
            })
            .unwrap();
        let oauth_windows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM rate_window_history WHERE source_kind = 'oauth'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let usage_runs: i64 = conn
            .query_row("SELECT COUNT(*) FROM claude_usage_runs", [], |r| r.get(0))
            .unwrap();
        let usage_factors: i64 = conn
            .query_row("SELECT COUNT(*) FROM claude_usage_factors", [], |r| {
                r.get(0)
            })
            .unwrap();

        assert_eq!(live_events, 1);
        assert_eq!(agent_history, 1);
        assert_eq!(oauth_windows, 1);
        assert_eq!(usage_runs, 1);
        assert_eq!(usage_factors, 1);
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
    async fn test_api_data_includes_subscription_quota_section() {
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

        // The subscription_quota section is attached even when providers are
        // unavailable in tests — its `changelog` is statically loaded from
        // the bundled JSON, so we always have something to assert against.
        let section = data
            .get("subscription_quota")
            .expect("subscription_quota section");
        assert!(
            section["providers"].is_array(),
            "providers should be array, got {section}"
        );
        assert!(section["history"].is_array());
        let changelog = section["changelog"]
            .as_array()
            .expect("changelog should be array");
        assert!(
            changelog.len() >= 6,
            "expected ≥6 curated changelog entries, got {}",
            changelog.len()
        );
        assert!(section["generated_at"].is_string());
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
    async fn test_api_rescan_rejects_non_loopback_client() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let state = Arc::new(base_state(db_path, projects));
        let remote: SocketAddr = "192.168.1.20:43120".parse().unwrap();
        let mut req = Request::builder()
            .method("POST")
            .uri("/api/rescan")
            .body(Body::empty())
            .unwrap();
        req.extensions_mut()
            .insert(axum::extract::ConnectInfo(remote));

        let result = api_rescan(State(state), req).await;

        assert_eq!(result.unwrap_err(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_sensitive_helper_endpoints_reject_non_loopback_client() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let state = Arc::new(base_state(db_path, projects));
        let remote: SocketAddr = "192.168.1.20:43120".parse().unwrap();

        let data_req = {
            let mut req = Request::builder()
                .uri("/api/data")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(remote));
            req
        };
        let usage_req = {
            let mut req = Request::builder()
                .uri("/api/usage-windows")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(remote));
            req
        };
        let claude_req = {
            let mut req = Request::builder()
                .uri("/api/claude-usage")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(remote));
            req
        };
        let live_req = {
            let mut req = Request::builder()
                .uri("/api/live-providers")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(remote));
            req
        };
        let refresh_req = {
            let mut req = Request::builder()
                .uri("/api/live-providers/refresh")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(remote));
            req
        };
        let history_req = {
            let mut req = Request::builder()
                .uri("/api/live-providers/history")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(remote));
            req
        };
        let mobile_snapshot_req = {
            let mut req = Request::builder()
                .uri("/api/mobile-snapshot")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(remote));
            req
        };
        let live_monitor_req = {
            let mut req = Request::builder()
                .uri("/api/live-monitor")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(remote));
            req
        };
        let heatmap_req = {
            let mut req = Request::builder()
                .uri("/api/heatmap")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(remote));
            req
        };
        let stream_req = {
            let mut req = Request::builder()
                .uri("/api/stream")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(remote));
            req
        };
        let billing_blocks_req = {
            let mut req = Request::builder()
                .uri("/api/billing-blocks")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(remote));
            req
        };
        let context_window_req = {
            let mut req = Request::builder()
                .uri("/api/context-window")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(remote));
            req
        };
        let cost_reconciliation_req = {
            let mut req = Request::builder()
                .uri("/api/cost-reconciliation")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(remote));
            req
        };
        let agent_status_req = {
            let mut req = Request::builder()
                .uri("/api/agent-status")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(remote));
            req
        };
        let community_signal_req = {
            let mut req = Request::builder()
                .uri("/api/community-signal")
                .body(Body::empty())
                .unwrap();
            req.extensions_mut()
                .insert(axum::extract::ConnectInfo(remote));
            req
        };

        let data = api_data(
            State(state.clone()),
            axum::extract::Query(TzParams::default()),
            data_req,
        )
        .await;
        let usage = api_usage_windows(State(state.clone()), usage_req).await;
        let claude_usage = api_claude_usage(State(state.clone()), claude_req).await;
        let live = api_live_providers(
            State(state.clone()),
            axum::extract::Query(crate::server::api::LiveProviderQuery {
                provider: None,
                scope: None,
                startup: None,
            }),
            live_req,
        )
        .await;
        let refresh = api_live_provider_refresh(
            State(state.clone()),
            axum::extract::Query(crate::server::api::LiveProviderQuery {
                provider: None,
                scope: None,
                startup: None,
            }),
            refresh_req,
        )
        .await;
        let history = api_live_provider_history(
            State(state.clone()),
            axum::extract::Query(crate::server::api::LiveProviderQuery {
                provider: Some("claude".into()),
                scope: None,
                startup: None,
            }),
            history_req,
        )
        .await;
        let mobile_snapshot = api_mobile_snapshot(State(state.clone()), mobile_snapshot_req).await;
        let live_monitor = api_live_monitor(
            State(state.clone()),
            axum::extract::Query(TzParams::default()),
            live_monitor_req,
        )
        .await;
        let heatmap = api_heatmap(
            State(state.clone()),
            axum::extract::Query(HeatmapParams::default()),
            heatmap_req,
        )
        .await;
        let stream = api_stream(State(state.clone()), stream_req).await;
        let billing_blocks = api_billing_blocks(State(state.clone()), billing_blocks_req).await;
        let context_window = api_context_window(State(state.clone()), context_window_req).await;
        let cost_reconciliation = api_cost_reconciliation(
            State(state.clone()),
            axum::extract::Query(CostReconciliationParams::default()),
            cost_reconciliation_req,
        )
        .await;
        let agent_status = api_agent_status(State(state.clone()), agent_status_req).await;
        let community_signal =
            api_community_signal(State(state.clone()), community_signal_req).await;

        assert_eq!(data.unwrap_err(), StatusCode::FORBIDDEN);
        assert_eq!(usage.unwrap_err(), StatusCode::FORBIDDEN);
        assert_eq!(claude_usage.unwrap_err(), StatusCode::FORBIDDEN);
        assert_eq!(live.unwrap_err(), StatusCode::FORBIDDEN);
        assert_eq!(refresh.unwrap_err(), StatusCode::FORBIDDEN);
        assert_eq!(history.unwrap_err(), StatusCode::FORBIDDEN);
        assert_eq!(mobile_snapshot.unwrap_err(), StatusCode::FORBIDDEN);
        assert_eq!(live_monitor.unwrap_err(), StatusCode::FORBIDDEN);
        assert_eq!(heatmap.unwrap_err(), StatusCode::FORBIDDEN);
        assert!(matches!(stream, Err(StatusCode::FORBIDDEN)));
        assert_eq!(billing_blocks.unwrap_err(), StatusCode::FORBIDDEN);
        assert_eq!(context_window.unwrap_err(), StatusCode::FORBIDDEN);
        assert_eq!(cost_reconciliation.unwrap_err(), StatusCode::FORBIDDEN);
        assert_eq!(agent_status.unwrap_err(), StatusCode::FORBIDDEN);
        assert_eq!(community_signal.unwrap_err(), StatusCode::FORBIDDEN);
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
    async fn test_api_live_provider_history_returns_cost_summary() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app(db_path, projects);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/live-providers/history?provider=claude")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(data["provider"], "claude");
        assert!(data["summary"]["last_30_days_tokens"].is_number());
        assert!(data["summary"]["last_30_days_cost_usd"].is_number());
        assert!(data["summary"]["daily"].is_array());
    }

    #[tokio::test]
    async fn test_api_mobile_snapshot_returns_aggregate_payload() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let state = Arc::new(base_state(db_path, projects));
        {
            let mut cache = state.live_provider_cache.write().await;
            *cache = Some((
                std::time::Instant::now(),
                cached_live_provider_response(true),
            ));
        }
        let app = build_router(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/mobile-snapshot")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            data["contract_version"],
            crate::models::MOBILE_SNAPSHOT_CONTRACT_VERSION
        );
        assert_eq!(data["providers"].as_array().unwrap().len(), 2);
        assert_eq!(data["history_90d"].as_array().unwrap().len(), 2);
        assert!(data["totals"]["today_tokens"].is_number());
        assert!(data["totals"]["last_90_days_tokens"].is_number());
        assert_eq!(data["freshness"]["has_stale_providers"], true);
        assert_eq!(data["freshness"]["stale_providers"][0], "codex");
    }

    #[tokio::test]
    async fn test_api_live_providers_provider_scope_filters_cached_response() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let state = Arc::new(base_state(db_path, projects));
        {
            let mut cache = state.live_provider_cache.write().await;
            *cache = Some((
                std::time::Instant::now(),
                crate::models::LiveProvidersResponse {
                    contract_version: crate::models::LIVE_PROVIDERS_CONTRACT_VERSION,
                    providers: vec![
                        crate::models::LiveProviderSnapshot {
                            provider: "claude".into(),
                            available: true,
                            source_used: "oauth".into(),
                            last_attempted_source: Some("oauth".into()),
                            resolved_via_fallback: false,
                            refresh_duration_ms: 1,
                            source_attempts: vec![],
                            identity: None,
                            primary: None,
                            secondary: None,
                            tertiary: None,
                            credits: None,
                            status: None,
                            auth: crate::models::LiveProviderAuth::default(),
                            cost_summary: crate::models::ProviderCostSummary::default(),
                            claude_usage: None,
                            claude_admin: None,
                            quota_suggestions: None,
                            depletion_forecast: None,
                            predictive_insights: None,
                            last_refresh: "2026-01-01T00:00:00Z".into(),
                            stale: false,
                            error: None,
                        },
                        crate::models::LiveProviderSnapshot {
                            provider: "codex".into(),
                            available: true,
                            source_used: "cli-rpc".into(),
                            last_attempted_source: Some("cli-rpc".into()),
                            resolved_via_fallback: true,
                            refresh_duration_ms: 2,
                            source_attempts: vec![],
                            identity: None,
                            primary: None,
                            secondary: None,
                            tertiary: None,
                            credits: None,
                            status: None,
                            auth: crate::models::LiveProviderAuth::default(),
                            cost_summary: crate::models::ProviderCostSummary::default(),
                            claude_usage: None,
                            claude_admin: None,
                            quota_suggestions: None,
                            depletion_forecast: None,
                            predictive_insights: None,
                            last_refresh: "2026-01-01T00:00:00Z".into(),
                            stale: false,
                            error: None,
                        },
                    ],
                    fetched_at: "2026-01-01T00:00:00Z".into(),
                    requested_provider: None,
                    response_scope: "all".into(),
                    cache_hit: false,
                    refreshed_providers: vec![],
                    local_notification_state: None,
                },
            ));
        }

        let app = crate::server::build_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/live-providers?provider=codex&scope=provider")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(data["requested_provider"], "codex");
        assert_eq!(
            data["contract_version"],
            crate::models::LIVE_PROVIDERS_CONTRACT_VERSION
        );
        assert_eq!(data["response_scope"], "provider");
        assert_eq!(data["cache_hit"], true);
        assert_eq!(data["providers"].as_array().unwrap().len(), 1);
        assert_eq!(data["providers"][0]["provider"], "codex");
    }

    #[tokio::test]
    async fn test_api_live_providers_exposes_quota_suggestions_only_for_claude() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let state = Arc::new(base_state(db_path, projects));
        {
            let mut cache = state.live_provider_cache.write().await;
            *cache = Some((
                std::time::Instant::now(),
                cached_live_provider_response(false),
            ));
        }

        let app = crate::server::build_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/live-providers")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let claude = data["providers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|provider| provider["provider"] == "claude")
            .unwrap();
        assert!(claude.get("quota_suggestions").is_some());
        assert!(claude.get("depletion_forecast").is_some());
        assert!(claude.get("predictive_insights").is_some());

        let codex = data["providers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|provider| provider["provider"] == "codex")
            .unwrap();
        assert!(codex.get("quota_suggestions").is_none());
        assert!(codex.get("depletion_forecast").is_some());
        assert!(codex.get("predictive_insights").is_none());
    }

    #[tokio::test]
    async fn test_api_live_providers_exposes_local_notification_state() {
        use crate::agent_status::models::{AgentStatusSnapshot, ProviderStatus, StatusIndicator};
        use crate::status_aggregator::models::{CommunitySignal, ServiceSignal, SignalLevel};

        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let mut raw_state = base_state(db_path, projects);
        // Enable oauth so the non-startup fetcher path consults oauth_cache via
        // its TTL fast-path; with oauth_enabled=false `refresh_usage_windows`
        // short-circuits to `unavailable()` and skips the test fixture entirely.
        raw_state.oauth_enabled = true;
        raw_state.aggregator_config.enabled = true;
        raw_state.webhook_config.cost_threshold = Some(50.0);
        let state = Arc::new(raw_state);

        {
            let mut oauth_cache = state.oauth_cache.write().await;
            *oauth_cache = Some((
                std::time::Instant::now(),
                available_usage_response(100.0, 30, "pro"),
            ));
        }
        {
            let mut agent_status_cache = state.agent_status_cache.write().await;
            *agent_status_cache = Some((
                std::time::Instant::now(),
                AgentStatusSnapshot {
                    claude: Some(ProviderStatus {
                        indicator: StatusIndicator::None,
                        description: "All systems operational".into(),
                        components: vec![],
                        active_incidents: vec![],
                        page_url: "https://status.claude.com".into(),
                    }),
                    openai: Some(ProviderStatus {
                        indicator: StatusIndicator::Major,
                        description: "Major outage".into(),
                        components: vec![],
                        active_incidents: vec![],
                        page_url: "https://status.openai.com".into(),
                    }),
                    fetched_at: chrono::Utc::now().to_rfc3339(),
                },
                None,
            ));
        }
        {
            state.aggregator_cache.write().await.replace((
                std::time::Instant::now(),
                CommunitySignal {
                    fetched_at: chrono::Utc::now().to_rfc3339(),
                    enabled: true,
                    claude: vec![ServiceSignal {
                        slug: "claude-ai".into(),
                        name: "Claude AI".into(),
                        level: SignalLevel::Spike,
                        report_count_last_hour: Some(31),
                        report_baseline: Some(10),
                        detail: "Spike".into(),
                        source_url: "https://statusgator.com/services/claude-ai".into(),
                    }],
                    openai: vec![ServiceSignal {
                        slug: "openai".into(),
                        name: "OpenAI".into(),
                        level: SignalLevel::Normal,
                        report_count_last_hour: Some(1),
                        report_baseline: Some(10),
                        detail: "Normal".into(),
                        source_url: "https://statusgator.com/services/openai".into(),
                    }],
                },
            ));
        }

        // NOTE: this asserts non-startup synthesis — `startup=true` is a
        // readiness-gate fast path that intentionally skips the fetcher, so
        // it never populates `local_notification_state`.
        let app = crate::server::build_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/live-providers")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let notification_state = data
            .get("local_notification_state")
            .expect("local_notification_state should be present");
        assert_eq!(notification_state["cost_threshold_usd"], 50.0);

        let conditions = notification_state["conditions"].as_array().unwrap();
        let session = conditions
            .iter()
            .find(|condition| condition["id"] == "claude-session-depleted")
            .unwrap();
        assert_eq!(session["is_active"], true);

        let degraded = conditions
            .iter()
            .find(|condition| condition["id"] == "codex-service-degraded")
            .unwrap();
        assert_eq!(degraded["is_active"], true);
        assert_eq!(degraded["service_label"], "OpenAI");

        let spike = conditions
            .iter()
            .find(|condition| condition["id"] == "claude-community-spike")
            .unwrap();
        assert_eq!(spike["is_active"], true);

        let daily = conditions
            .iter()
            .find(|condition| condition["id"] == "daily-cost-threshold")
            .unwrap();
        assert!(daily.get("day_key").is_some());
    }

    #[tokio::test]
    async fn test_api_live_monitor_returns_provider_details_with_optional_capabilities() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let state = Arc::new(base_state(db_path.clone(), projects.clone()));

        {
            let mut cache = state.live_provider_cache.write().await;
            let mut response = cached_live_provider_response(false);
            response.providers[0].cost_summary.recent_sessions =
                vec![crate::models::ProviderSession {
                    session_id: "claude-session".into(),
                    display_name: "Claude session".into(),
                    started_at: "2026-04-21T09:00:00Z".into(),
                    duration_minutes: 42,
                    turns: 9,
                    cost_usd: 2.5,
                    model: Some("claude-sonnet-4-6".into()),
                }];
            *cache = Some((std::time::Instant::now(), response));
        }

        {
            let conn = crate::scanner::db::open_db(&db_path).unwrap();
            crate::scanner::db::init_db(&conn).unwrap();
            conn.execute(
                "INSERT OR IGNORE INTO live_events
                    (dedup_key, received_at, session_id, tool_name, cost_usd_nanos,
                     input_tokens, output_tokens, raw_json, context_input_tokens,
                     context_window_size, hook_reported_cost_nanos)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                rusqlite::params![
                    "ctx-1",
                    chrono::Utc::now().to_rfc3339(),
                    "claude-session",
                    "Read",
                    500_i64,
                    10_i64,
                    5_i64,
                    "{}",
                    75_000_i64,
                    200_000_i64,
                    500_i64,
                ],
            )
            .unwrap();
        }

        let app = crate::server::build_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/live-monitor")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            data["contract_version"],
            crate::models::LIVE_MONITOR_CONTRACT_VERSION
        );
        assert_eq!(data["default_focus"], "all");
        assert_eq!(data["providers"].as_array().unwrap().len(), 2);

        let claude = data["providers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|provider| provider["provider"] == "claude")
            .unwrap();
        assert!(claude.get("context_window").is_some());
        assert!(claude.get("recent_session").is_some());
        assert!(claude.get("quota_suggestions").is_some());
        assert!(claude.get("depletion_forecast").is_some());
        assert!(claude.get("predictive_insights").is_some());

        let codex = data["providers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|provider| provider["provider"] == "codex")
            .unwrap();
        assert!(codex.get("context_window").is_none());
        assert!(codex.get("active_block").is_none());
        assert!(codex.get("quota_suggestions").is_none());
        assert!(codex.get("depletion_forecast").is_some());
        assert!(codex.get("predictive_insights").is_none());
    }

    #[tokio::test]
    async fn test_api_live_monitor_uses_client_timezone_for_codex_today_cost() {
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("usage.db");
        let projects = tmp.path().join("projects");
        std::fs::create_dir_all(&projects).unwrap();
        let state = Arc::new(base_state(db_path.clone(), projects));

        {
            let mut cache = state.live_provider_cache.write().await;
            *cache = Some((
                std::time::Instant::now(),
                cached_live_provider_response(false),
            ));
        }

        {
            let conn = crate::scanner::db::open_db(&db_path).unwrap();
            crate::scanner::db::init_db(&conn).unwrap();
            let local_today = (chrono::Utc::now() + chrono::Duration::minutes(120)).date_naive();
            let local_time = chrono::NaiveTime::from_hms_opt(0, 30, 0).unwrap();
            let utc_time = local_today.and_time(local_time) - chrono::Duration::minutes(120);
            let timestamp = format!("{}Z", utc_time.format("%Y-%m-%dT%H:%M:%S"));
            conn.execute(
                "INSERT INTO turns
                    (session_id, provider, timestamp, model, input_tokens, output_tokens,
                     estimated_cost_nanos, source_path, pricing_version, pricing_model,
                     billing_mode, cost_confidence, category)
                 VALUES (?1, 'codex', ?2, 'gpt-5.4', 100, 50, ?3, '', 'builtin',
                         'gpt-5.4', 'estimated_local', 'high', '')",
                rusqlite::params!["codex:tz-session", timestamp, 1_250_000_000_i64],
            )
            .unwrap();
        }

        let app = crate::server::build_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/live-monitor?tz_offset_min=120")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let codex = data["providers"]
            .as_array()
            .unwrap()
            .iter()
            .find(|provider| provider["provider"] == "codex")
            .unwrap();
        assert_eq!(codex["today_cost_usd"].as_f64().unwrap(), 1.25);
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
    async fn test_refresh_usage_windows_uses_fresh_cache_without_fetch() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let cached = available_usage_response(42.0, 75, "cached-tier");
        let mut state = base_state(db_path, projects);
        state.oauth_enabled = true;
        state.oauth_refresh_interval = 3600;
        state.oauth_cache =
            tokio::sync::RwLock::new(Some((std::time::Instant::now(), cached.clone())));
        let state = Arc::new(state);

        let fetch_count = Arc::new(AtomicUsize::new(0));
        let returned = crate::server::api::refresh_usage_windows_with(
            &state,
            {
                let fetch_count = fetch_count.clone();
                move || async move {
                    fetch_count.fetch_add(1, Ordering::SeqCst);
                    UsageWindowsResponse::with_error("should not fetch".into())
                }
            },
            || async {
                crate::models::ClaudeAdminSummary {
                    error: Some("should not fetch admin".into()),
                    ..crate::models::ClaudeAdminSummary::default()
                }
            },
        )
        .await;

        assert_eq!(fetch_count.load(Ordering::SeqCst), 0);
        assert_eq!(
            serde_json::to_value(&returned).unwrap(),
            serde_json::to_value(&cached).unwrap()
        );
    }

    #[tokio::test]
    async fn test_refresh_usage_windows_replaces_stale_cache_with_fresh_response() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let cached = available_usage_response(42.0, 75, "cached-tier");
        let fresh = available_usage_response(88.0, 12, "fresh-tier");

        let mut state = base_state(db_path, projects);
        state.oauth_enabled = true;
        state.oauth_refresh_interval = 1;
        state.oauth_cache = tokio::sync::RwLock::new(Some((
            std::time::Instant::now() - Duration::from_secs(5),
            cached,
        )));
        let state = Arc::new(state);

        let fetch_count = Arc::new(AtomicUsize::new(0));
        let returned = crate::server::api::refresh_usage_windows_with(
            &state,
            {
                let fetch_count = fetch_count.clone();
                let fresh = fresh.clone();
                move || async move {
                    fetch_count.fetch_add(1, Ordering::SeqCst);
                    fresh
                }
            },
            || async {
                crate::models::ClaudeAdminSummary {
                    error: Some("unused admin fetch".into()),
                    ..crate::models::ClaudeAdminSummary::default()
                }
            },
        )
        .await;

        assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
        assert_eq!(
            serde_json::to_value(&returned).unwrap(),
            serde_json::to_value(&fresh).unwrap()
        );

        let stored = state.oauth_cache.read().await;
        let (_, stored_resp) = stored.as_ref().expect("cache should be populated");
        assert_eq!(
            serde_json::to_value(stored_resp).unwrap(),
            serde_json::to_value(&fresh).unwrap()
        );
    }

    #[tokio::test]
    async fn test_refresh_usage_windows_falls_back_to_stale_cache_when_fetch_unavailable() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let cached = available_usage_response(64.0, 33, "cached-tier");

        let mut state = base_state(db_path, projects);
        state.oauth_enabled = true;
        state.oauth_refresh_interval = 1;
        state.oauth_cache = tokio::sync::RwLock::new(Some((
            std::time::Instant::now() - Duration::from_secs(5),
            cached.clone(),
        )));
        let state = Arc::new(state);

        let fetch_count = Arc::new(AtomicUsize::new(0));
        let returned = crate::server::api::refresh_usage_windows_with(
            &state,
            {
                let fetch_count = fetch_count.clone();
                move || async move {
                    fetch_count.fetch_add(1, Ordering::SeqCst);
                    UsageWindowsResponse::with_error("upstream unavailable".into())
                }
            },
            || async {
                crate::models::ClaudeAdminSummary {
                    error: Some("admin unavailable".into()),
                    ..crate::models::ClaudeAdminSummary::default()
                }
            },
        )
        .await;

        assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
        assert_eq!(
            serde_json::to_value(&returned).unwrap(),
            serde_json::to_value(&cached).unwrap()
        );

        let stored = state.oauth_cache.read().await;
        let (_, stored_resp) = stored.as_ref().expect("cache should retain stale value");
        assert_eq!(
            serde_json::to_value(stored_resp).unwrap(),
            serde_json::to_value(&cached).unwrap()
        );
    }

    #[tokio::test]
    async fn test_refresh_usage_windows_stores_unavailable_response_without_cache() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let mut state = base_state(db_path, projects);
        state.oauth_enabled = true;
        let state = Arc::new(state);

        let returned = crate::server::api::refresh_usage_windows_with(
            &state,
            || async { UsageWindowsResponse::with_error("token expired".into()) },
            || async {
                crate::models::ClaudeAdminSummary {
                    error: Some("admin unavailable".into()),
                    ..crate::models::ClaudeAdminSummary::default()
                }
            },
        )
        .await;

        assert!(!returned.available);
        assert_eq!(returned.error.as_deref(), Some("token expired"));

        let stored = state.oauth_cache.read().await;
        let (_, stored_resp) = stored
            .as_ref()
            .expect("unavailable response should be cached");
        assert_eq!(stored_resp.error.as_deref(), Some("token expired"));
    }

    #[tokio::test]
    async fn test_api_usage_windows_returns_cached_payload_when_enabled() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let cached = available_usage_response(73.0, 18, "cached-tier");
        let mut state = base_state(db_path, projects);
        state.oauth_enabled = true;
        state.oauth_refresh_interval = 3600;
        state.oauth_cache = tokio::sync::RwLock::new(Some((std::time::Instant::now(), cached)));
        let app = Router::new()
            .route(
                "/api/usage-windows",
                get(crate::server::api::api_usage_windows),
            )
            .with_state(Arc::new(state));

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
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["available"], true);
        assert_eq!(json["session"]["used_percent"], serde_json::json!(73.0));
        assert_eq!(json["identity"]["rate_limit_tier"], "cached-tier");
    }

    #[tokio::test]
    async fn test_api_claude_usage_returns_latest_snapshot() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        {
            let conn = crate::scanner::db::open_db(&db_path).unwrap();
            crate::scanner::db::init_db(&conn).unwrap();
            let run_id = crate::scanner::db::insert_claude_usage_run(
                &conn,
                &crate::scanner::db::ClaudeUsageRunInsert {
                    status: "success",
                    exit_code: Some(0),
                    stdout_raw: "stdout",
                    stderr_raw: "",
                    invocation_mode: "print_slash_command",
                    period: "today",
                    parser_version: "v1",
                    error_summary: None,
                },
            )
            .unwrap();
            crate::scanner::db::insert_claude_usage_factors(
                &conn,
                run_id,
                &[crate::models::ClaudeUsageFactor {
                    factor_key: "parallel_sessions".into(),
                    display_label: "was while 4+ sessions ran in parallel".into(),
                    percent: 98.0,
                    description: "All sessions share one limit.".into(),
                    advice_text: "All sessions share one limit.".into(),
                    display_order: 0,
                }],
            )
            .unwrap();
        }

        let app = test_app(db_path, projects);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/claude-usage")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(data["available"], true);
        assert_eq!(
            data["latest_snapshot"]["factors"][0]["factor_key"],
            "parallel_sessions"
        );
        assert_eq!(data["last_run"]["status"], "success");
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

    // -----------------------------------------------------------------------
    // Phase 16 — CC Version distribution: version_summary aggregation tests
    // -----------------------------------------------------------------------

    /// Build a DB with three turns:
    ///   - two with `version = "1.0.0"` (known version)
    ///   - one with no version field at all (NULL → should appear as "unknown")
    ///
    /// The test asserts that the `/api/data` response `version_summary` array:
    ///   1. Contains exactly two entries (one for "1.0.0", one for "unknown").
    ///   2. The cost + tokens fields are non-negative numbers.
    ///   3. The NULL-version bucket uses the sentinel key "unknown".
    fn setup_version_test_db(tmp: &TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
        let projects = tmp.path().join("projects").join("user").join("verproj");
        std::fs::create_dir_all(&projects).unwrap();

        let filepath = projects.join("ver_sess.jsonl");
        let mut f = std::fs::File::create(&filepath).unwrap();

        // Anchor user message
        writeln!(
            f,
            "{}",
            serde_json::json!({
                "type": "user",
                "sessionId": "ver-session",
                "timestamp": "2026-04-10T10:00:00Z",
                "cwd": "/home/user/verproj"
            })
        )
        .unwrap();

        // Turn 1 — version = "1.0.0"
        writeln!(
            f,
            "{}",
            serde_json::json!({
                "type": "assistant",
                "sessionId": "ver-session",
                "timestamp": "2026-04-10T10:01:00Z",
                "cwd": "/home/user/verproj",
                "version": "1.0.0",
                "message": {
                    "id": "ver-msg-1",
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

        // Turn 2 — version = "1.0.0" (same bucket)
        writeln!(
            f,
            "{}",
            serde_json::json!({
                "type": "assistant",
                "sessionId": "ver-session",
                "timestamp": "2026-04-10T10:02:00Z",
                "cwd": "/home/user/verproj",
                "version": "1.0.0",
                "message": {
                    "id": "ver-msg-2",
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

        // Turn 3 — no version field → NULL → "unknown"
        writeln!(
            f,
            "{}",
            serde_json::json!({
                "type": "assistant",
                "sessionId": "ver-session",
                "timestamp": "2026-04-10T10:03:00Z",
                "cwd": "/home/user/verproj",
                "message": {
                    "id": "ver-msg-3",
                    "model": "claude-sonnet-4-6",
                    "usage": {
                        "input_tokens": 50,
                        "output_tokens": 25,
                        "cache_read_input_tokens": 0,
                        "cache_creation_input_tokens": 0
                    },
                    "content": []
                }
            })
        )
        .unwrap();

        let db_path = tmp.path().join("ver_usage.db");
        let parent = tmp.path().join("projects");
        scanner::scan(Some(vec![parent.clone()]), &db_path, false).unwrap();
        (db_path, parent)
    }

    #[tokio::test]
    async fn test_version_summary_includes_unknown_bucket() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_version_test_db(&tmp);
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

        let version_summary = data["version_summary"]
            .as_array()
            .expect("version_summary is array");

        // Must have exactly two buckets: "1.0.0" and "unknown"
        assert_eq!(version_summary.len(), 2, "expected 2 version buckets");

        let known = version_summary
            .iter()
            .find(|v| v["version"].as_str() == Some("1.0.0"))
            .expect("bucket for version 1.0.0 must exist");
        let unknown = version_summary
            .iter()
            .find(|v| v["version"].as_str() == Some("unknown"))
            .expect("bucket for unknown version must exist");

        // known bucket: 2 turns, tokens = 100+50 + 200+100 = 450
        assert_eq!(known["turns"].as_i64().unwrap(), 2);
        assert_eq!(known["tokens"].as_i64().unwrap(), 450);
        assert!(known["cost"].as_f64().unwrap() >= 0.0);

        // unknown bucket: 1 turn, tokens = 50+25 = 75
        assert_eq!(unknown["turns"].as_i64().unwrap(), 1);
        assert_eq!(unknown["tokens"].as_i64().unwrap(), 75);
        assert!(unknown["cost"].as_f64().unwrap() >= 0.0);
    }

    #[tokio::test]
    async fn test_version_summary_cost_and_tokens_fields_present() {
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

        // version_summary is present in the payload (may be empty when no version field present)
        assert!(
            data.get("version_summary").is_some(),
            "version_summary field must exist"
        );

        // If entries exist, each must carry cost and tokens
        if let Some(entries) = data["version_summary"].as_array() {
            for entry in entries {
                assert!(
                    entry.get("cost").is_some(),
                    "each version entry must have cost"
                );
                assert!(
                    entry.get("tokens").is_some(),
                    "each version entry must have tokens"
                );
                assert!(entry["cost"].as_f64().is_some(), "cost must be a number");
                assert!(
                    entry["tokens"].as_i64().is_some(),
                    "tokens must be an integer"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Agent status endpoint tests
    // -----------------------------------------------------------------------

    /// Build a minimal app with a pre-populated cache snapshot carrying non-null
    /// uptime fields — used to verify that `/api/agent-status` serialises them.
    fn test_app_with_seeded_history(tmp: &TempDir) -> Router {
        use crate::agent_status::models::{
            AgentStatusSnapshot, ComponentStatus, ProviderStatus, StatusIndicator,
        };
        use crate::scanner::db;

        let db_path = tmp.path().join("history_test.db");
        let conn = db::open_db(&db_path).unwrap();
        db::init_db(&conn).unwrap();

        // Seed 15 samples (all operational) for the Claude Code component so
        // uptime_pct() returns a non-None value when called live.
        let now = chrono::Utc::now().timestamp();
        for i in 0..15i64 {
            db::insert_agent_status_samples(
                &conn,
                "claude",
                &[(
                    "yyzkbfz2thpt".to_string(),
                    "Claude Code".to_string(),
                    "operational".to_string(),
                )],
                now - i,
            )
            .unwrap();
        }
        drop(conn);

        // Pre-populate the cache with a snapshot that already has uptime values.
        // The handler returns the cached snapshot immediately (refresh_interval is
        // large), so the test asserts the serialised fields are present.
        let snapshot = AgentStatusSnapshot {
            claude: Some(ProviderStatus {
                indicator: StatusIndicator::None,
                description: "All Systems Operational".to_string(),
                components: vec![ComponentStatus {
                    id: "yyzkbfz2thpt".to_string(),
                    name: "Claude Code".to_string(),
                    status: "operational".to_string(),
                    uptime_30d: Some(1.0),
                    uptime_7d: Some(1.0),
                }],
                active_incidents: vec![],
                page_url: "https://status.claude.com".to_string(),
            }),
            openai: None,
            fetched_at: chrono::Utc::now().to_rfc3339(),
        };

        let projects = tmp.path().join("projects");
        std::fs::create_dir_all(&projects).unwrap();

        let mut cfg = AgentStatusConfig::default();
        cfg.enabled = true;
        cfg.claude_enabled = false;
        cfg.openai_enabled = false;
        cfg.refresh_interval = 3600; // long TTL — cache never expires during test

        let state = Arc::new(AppState {
            db_path,
            projects_dirs: Some(vec![projects]),
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
            openai_refresh_interval: 300,
            openai_lookback_days: 30,
            openai_cache: tokio::sync::RwLock::new(None),
            openai_refresh_lock: tokio::sync::Mutex::new(()),
            db_lock: tokio::sync::Mutex::new(()),
            webhook_state: tokio::sync::Mutex::new(WebhookState::default()),
            webhook_config: WebhookConfig::default(),
            scan_event_tx: tokio::sync::broadcast::channel::<String>(16).0,
            agent_status_config: cfg,
            agent_status_cache: tokio::sync::RwLock::new(Some((
                std::time::Instant::now(),
                snapshot,
                None,
            ))),
            agent_status_refresh_lock: tokio::sync::Mutex::new(()),
            aggregator_config: AggregatorConfig::default(),
            aggregator_cache: tokio::sync::RwLock::new(None),
            aggregator_refresh_lock: tokio::sync::Mutex::new(()),
            blocks_token_limit: None,
            session_length_hours: 5.0,
            project_aliases: std::collections::HashMap::new(),
            live_provider_cache: tokio::sync::RwLock::new(None),
            live_provider_refresh_lock: tokio::sync::Mutex::new(()),
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
            .route("/api/agent-status", get(api_agent_status))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_agent_status_with_seeded_history_returns_uptime_fields() {
        let tmp = TempDir::new().unwrap();
        let app = test_app_with_seeded_history(&tmp);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/agent-status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let components = json["claude"]["components"].as_array().unwrap();
        assert!(!components.is_empty(), "should have at least one component");

        let comp = &components[0];
        assert!(
            comp.get("uptime_30d").is_some(),
            "uptime_30d should be present"
        );
        assert!(
            comp.get("uptime_7d").is_some(),
            "uptime_7d should be present"
        );
        // The pre-seeded cache has non-null values.
        assert!(
            !comp["uptime_30d"].is_null(),
            "uptime_30d should be non-null when samples exist"
        );
        assert!(
            !comp["uptime_7d"].is_null(),
            "uptime_7d should be non-null when samples exist"
        );
    }

    #[tokio::test]
    async fn test_agent_status_disabled_returns_empty_snapshot() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let mut cfg = AgentStatusConfig::default();
        cfg.enabled = false;

        let app = test_app_with_agent_status(db_path, projects, cfg);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/agent-status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // When disabled, both providers are null and fetched_at is present.
        assert!(
            json["claude"].is_null(),
            "claude should be null when disabled"
        );
        assert!(
            json["openai"].is_null(),
            "openai should be null when disabled"
        );
        assert!(
            json["fetched_at"].is_string(),
            "fetched_at should be present"
        );
    }

    #[tokio::test]
    async fn test_agent_status_enabled_with_both_providers_disabled_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let mut cfg = AgentStatusConfig::default();
        cfg.enabled = true;
        cfg.claude_enabled = false;
        cfg.openai_enabled = false;

        let app = test_app_with_agent_status(db_path, projects, cfg);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/agent-status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["claude"].is_null());
        assert!(json["openai"].is_null());
    }

    #[tokio::test]
    async fn test_refresh_agent_status_primes_cache_and_persists_history() {
        use crate::agent_status::models::{
            AgentStatusSnapshot, ComponentStatus, ProviderStatus, StatusIndicator,
        };

        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let mut cfg = AgentStatusConfig::default();
        cfg.enabled = true;
        cfg.refresh_interval = 3600;

        let state = Arc::new(AppState {
            db_path: db_path.clone(),
            projects_dirs: Some(vec![projects]),
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
            openai_refresh_interval: 300,
            openai_lookback_days: 30,
            openai_cache: tokio::sync::RwLock::new(None),
            openai_refresh_lock: tokio::sync::Mutex::new(()),
            db_lock: tokio::sync::Mutex::new(()),
            webhook_state: tokio::sync::Mutex::new(WebhookState::default()),
            webhook_config: WebhookConfig::default(),
            scan_event_tx: tokio::sync::broadcast::channel::<String>(16).0,
            agent_status_config: cfg,
            agent_status_cache: tokio::sync::RwLock::new(None),
            agent_status_refresh_lock: tokio::sync::Mutex::new(()),
            aggregator_config: AggregatorConfig::default(),
            aggregator_cache: tokio::sync::RwLock::new(None),
            aggregator_refresh_lock: tokio::sync::Mutex::new(()),
            blocks_token_limit: None,
            session_length_hours: 5.0,
            project_aliases: std::collections::HashMap::new(),
            live_provider_cache: tokio::sync::RwLock::new(None),
            live_provider_refresh_lock: tokio::sync::Mutex::new(()),
        });

        let snapshot = AgentStatusSnapshot {
            claude: Some(ProviderStatus {
                indicator: StatusIndicator::None,
                description: "All Systems Operational".into(),
                components: vec![ComponentStatus {
                    id: "component-1".into(),
                    name: "Claude API".into(),
                    status: "operational".into(),
                    uptime_30d: None,
                    uptime_7d: None,
                }],
                active_incidents: vec![],
                page_url: "https://status.claude.com".into(),
            }),
            openai: None,
            fetched_at: chrono::Utc::now().to_rfc3339(),
        };

        let fetch_count = Arc::new(AtomicUsize::new(0));
        let returned = crate::server::api::refresh_agent_status_with(&state, {
            let snapshot = snapshot.clone();
            let fetch_count = fetch_count.clone();
            move |_, _| async move {
                fetch_count.fetch_add(1, Ordering::SeqCst);
                Ok((snapshot, Some("etag-1".into())))
            }
        })
        .await
        .unwrap();

        assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
        assert!(returned.claude.is_some());

        let cached = state.agent_status_cache.read().await;
        assert!(cached.is_some(), "refresh should populate the cache");
        drop(cached);

        let conn = crate::scanner::db::open_db(&db_path).unwrap();
        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM agent_status_history", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(rows, 1, "one component sample should be persisted");
    }

    #[tokio::test]
    async fn test_refresh_agent_status_reuses_cache_within_ttl() {
        use crate::agent_status::models::AgentStatusSnapshot;

        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let mut cfg = AgentStatusConfig::default();
        cfg.enabled = true;
        cfg.refresh_interval = 3600;

        let state = Arc::new(AppState {
            db_path,
            projects_dirs: Some(vec![projects]),
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
            openai_refresh_interval: 300,
            openai_lookback_days: 30,
            openai_cache: tokio::sync::RwLock::new(None),
            openai_refresh_lock: tokio::sync::Mutex::new(()),
            db_lock: tokio::sync::Mutex::new(()),
            webhook_state: tokio::sync::Mutex::new(WebhookState::default()),
            webhook_config: WebhookConfig::default(),
            scan_event_tx: tokio::sync::broadcast::channel::<String>(16).0,
            agent_status_config: cfg,
            agent_status_cache: tokio::sync::RwLock::new(None),
            agent_status_refresh_lock: tokio::sync::Mutex::new(()),
            aggregator_config: AggregatorConfig::default(),
            aggregator_cache: tokio::sync::RwLock::new(None),
            aggregator_refresh_lock: tokio::sync::Mutex::new(()),
            blocks_token_limit: None,
            session_length_hours: 5.0,
            project_aliases: std::collections::HashMap::new(),
            live_provider_cache: tokio::sync::RwLock::new(None),
            live_provider_refresh_lock: tokio::sync::Mutex::new(()),
        });

        let fetch_count = Arc::new(AtomicUsize::new(0));
        let first = crate::server::api::refresh_agent_status_with(&state, {
            let fetch_count = fetch_count.clone();
            move |_, _| async move {
                fetch_count.fetch_add(1, Ordering::SeqCst);
                Ok((AgentStatusSnapshot::default(), Some("etag-1".into())))
            }
        })
        .await
        .unwrap();
        assert!(first.claude.is_none());

        let second = crate::server::api::refresh_agent_status_with(&state, {
            let fetch_count = fetch_count.clone();
            move |_, _| async move {
                fetch_count.fetch_add(1, Ordering::SeqCst);
                Ok((AgentStatusSnapshot::default(), Some("etag-2".into())))
            }
        })
        .await
        .unwrap();

        assert!(second.claude.is_none());
        assert_eq!(
            fetch_count.load(Ordering::SeqCst),
            1,
            "cached snapshot should suppress the second fetch within the TTL"
        );
    }

    #[tokio::test]
    async fn test_background_loop_delays_first_run_and_survives_errors() {
        let runs = Arc::new(AtomicUsize::new(0));
        let fail_once = Arc::new(AtomicBool::new(true));

        let handle = crate::server::spawn_background_loop("test", 1, {
            let runs = runs.clone();
            let fail_once = fail_once.clone();
            move || {
                let runs = runs.clone();
                let fail_once = fail_once.clone();
                async move {
                    runs.fetch_add(1, Ordering::SeqCst);
                    if fail_once.swap(false, Ordering::SeqCst) {
                        Err(StatusCode::SERVICE_UNAVAILABLE)
                    } else {
                        Ok(())
                    }
                }
            }
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(
            runs.load(Ordering::SeqCst),
            0,
            "background loop should wait before the first refresh"
        );

        tokio::time::sleep(Duration::from_millis(1100)).await;
        assert!(
            runs.load(Ordering::SeqCst) >= 1,
            "background loop should run after the startup delay"
        );

        tokio::time::sleep(Duration::from_millis(1100)).await;
        assert!(
            runs.load(Ordering::SeqCst) >= 2,
            "background loop should keep running after an error"
        );

        handle.abort();
    }

    #[test]
    fn test_background_poll_startup_delay_is_staggered_and_bounded() {
        let oauth_delay = crate::server::background_poll_startup_delay("oauth usage", 60);
        let agent_delay = crate::server::background_poll_startup_delay("agent status", 60);
        let short_delay = crate::server::background_poll_startup_delay("short", 3);

        assert!(oauth_delay >= Duration::from_secs(1));
        assert!(agent_delay >= Duration::from_secs(1));
        assert!(oauth_delay <= Duration::from_secs(60));
        assert!(agent_delay <= Duration::from_secs(60));
        assert_ne!(
            oauth_delay, agent_delay,
            "different pollers should not all wake at the same instant"
        );
        assert_eq!(
            short_delay,
            Duration::from_secs(3),
            "startup delay should never exceed the poll interval"
        );
    }

    #[test]
    fn test_start_background_pollers_wires_enabled_features() {
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("pollers.db");
        let projects = tmp.path().join("projects");
        std::fs::create_dir_all(&projects).unwrap();

        let mut options = test_options(db_path, projects);
        options.oauth_enabled = true;
        options.oauth_refresh_interval = 61;
        options.openai_enabled = true;
        options.openai_refresh_interval = 305;
        options.agent_status_config.enabled = true;
        options.agent_status_config.refresh_interval = 901;
        options.aggregator_config.enabled = true;
        options.aggregator_config.refresh_interval = 1201;

        let state = build_state(&options, tokio::sync::broadcast::channel::<String>(16).0);
        let mut spawned = Vec::new();
        start_background_pollers_with(state, |name, interval_secs, _refresh| {
            spawned.push((name, interval_secs));
        });

        assert_eq!(
            spawned,
            vec![
                ("oauth usage", 61),
                ("agent status", 901),
                ("community signal", 1201),
                ("OpenAI reconciliation", 305),
            ]
        );
    }

    #[test]
    fn test_start_background_pollers_skips_disabled_features() {
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("pollers-disabled.db");
        let projects = tmp.path().join("projects");
        std::fs::create_dir_all(&projects).unwrap();

        let mut options = test_options(db_path, projects);
        options.agent_status_config.enabled = false;
        let state = build_state(&options, tokio::sync::broadcast::channel::<String>(16).0);
        let mut spawned = Vec::new();
        start_background_pollers_with(state, |name, interval_secs, _refresh| {
            spawned.push((name, interval_secs));
        });

        assert!(
            spawned.is_empty(),
            "disabled features should not register background pollers"
        );
    }

    // -----------------------------------------------------------------------
    // Community signal endpoint tests
    // -----------------------------------------------------------------------

    /// When aggregator is disabled (default), `/api/community-signal` returns
    /// `{"enabled": false}` with status 200 and makes no network calls.
    #[tokio::test]
    async fn test_community_signal_disabled_returns_enabled_false() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app(db_path, projects); // AggregatorConfig::default() has enabled=false

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/community-signal")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json["enabled"], false,
            "disabled aggregator must return {{\"enabled\": false}}"
        );
    }

    /// When aggregator is enabled but no API key is set, the endpoint still
    /// returns 200 with a signal that has `enabled: true` (the backend returns
    /// Unknown-level signals when the key is absent).
    #[tokio::test]
    async fn test_community_signal_enabled_no_key_returns_200() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let mut agg_cfg = AggregatorConfig::default();
        agg_cfg.enabled = true;
        // key_env_var points at a var that is definitely not set in CI.
        agg_cfg.key_env_var = "HEIMDALL_TEST_NONEXISTENT_SG_KEY".into();

        let state = Arc::new(crate::server::api::AppState {
            db_path,
            projects_dirs: Some(vec![projects]),
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
            openai_refresh_interval: 300,
            openai_lookback_days: 30,
            openai_cache: tokio::sync::RwLock::new(None),
            openai_refresh_lock: tokio::sync::Mutex::new(()),
            db_lock: tokio::sync::Mutex::new(()),
            webhook_state: tokio::sync::Mutex::new(WebhookState::default()),
            webhook_config: WebhookConfig::default(),
            scan_event_tx: tokio::sync::broadcast::channel::<String>(16).0,
            agent_status_config: AgentStatusConfig::default(),
            agent_status_cache: tokio::sync::RwLock::new(None),
            agent_status_refresh_lock: tokio::sync::Mutex::new(()),
            aggregator_config: agg_cfg,
            aggregator_cache: tokio::sync::RwLock::new(None),
            aggregator_refresh_lock: tokio::sync::Mutex::new(()),
            blocks_token_limit: None,
            session_length_hours: 5.0,
            project_aliases: std::collections::HashMap::new(),
            live_provider_cache: tokio::sync::RwLock::new(None),
            live_provider_refresh_lock: tokio::sync::Mutex::new(()),
        });

        let html = crate::server::assets::render_dashboard();
        let app = Router::new()
            .route(
                "/",
                get({
                    let h = html.clone();
                    move || async { Html(h) }
                }),
            )
            .route("/api/community-signal", get(api_community_signal))
            .with_state(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/community-signal")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        // The response must carry `enabled: true` when the feature is on.
        assert_eq!(
            json["enabled"], true,
            "enabled aggregator must return enabled=true, got: {json}"
        );
        // Must have claude and openai arrays.
        assert!(
            json["claude"].is_array(),
            "response must contain claude array"
        );
        assert!(
            json["openai"].is_array(),
            "response must contain openai array"
        );
    }

    /// Cache hit: a pre-populated aggregator cache is returned without a
    /// network call and the TTL is honoured.
    #[tokio::test]
    async fn test_community_signal_cache_hit_returns_cached_data() {
        use crate::status_aggregator::models::{CommunitySignal, ServiceSignal, SignalLevel};

        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let mut agg_cfg = AggregatorConfig::default();
        agg_cfg.enabled = true;
        agg_cfg.refresh_interval = 3600; // long TTL — cache never expires

        let cached_signal = CommunitySignal {
            fetched_at: chrono::Utc::now().to_rfc3339(),
            enabled: true,
            claude: vec![ServiceSignal {
                slug: "claude-ai".into(),
                name: "Claude AI".into(),
                level: SignalLevel::Normal,
                report_count_last_hour: Some(0),
                report_baseline: Some(0),
                detail: "All good".into(),
                source_url: "https://statusgator.com/services/claude-ai".into(),
            }],
            openai: vec![],
        };

        let state = Arc::new(crate::server::api::AppState {
            db_path,
            projects_dirs: Some(vec![projects]),
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
            openai_refresh_interval: 300,
            openai_lookback_days: 30,
            openai_cache: tokio::sync::RwLock::new(None),
            openai_refresh_lock: tokio::sync::Mutex::new(()),
            db_lock: tokio::sync::Mutex::new(()),
            webhook_state: tokio::sync::Mutex::new(WebhookState::default()),
            webhook_config: WebhookConfig::default(),
            scan_event_tx: tokio::sync::broadcast::channel::<String>(16).0,
            agent_status_config: AgentStatusConfig::default(),
            agent_status_cache: tokio::sync::RwLock::new(None),
            agent_status_refresh_lock: tokio::sync::Mutex::new(()),
            aggregator_config: agg_cfg,
            aggregator_cache: tokio::sync::RwLock::new(Some((
                std::time::Instant::now(),
                cached_signal,
            ))),
            aggregator_refresh_lock: tokio::sync::Mutex::new(()),
            blocks_token_limit: None,
            session_length_hours: 5.0,
            project_aliases: std::collections::HashMap::new(),
            live_provider_cache: tokio::sync::RwLock::new(None),
            live_provider_refresh_lock: tokio::sync::Mutex::new(()),
        });

        let html = crate::server::assets::render_dashboard();
        let app = Router::new()
            .route(
                "/",
                get({
                    let h = html.clone();
                    move || async { Html(h) }
                }),
            )
            .route("/api/community-signal", get(api_community_signal))
            .with_state(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/community-signal")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["enabled"], true);
        // The cached claude slug must be present.
        let claude_arr = json["claude"].as_array().unwrap();
        assert_eq!(claude_arr.len(), 1);
        assert_eq!(claude_arr[0]["slug"], "claude-ai");
        assert_eq!(claude_arr[0]["level"], "normal");
    }

    fn test_app_with_token_limit(
        db_path: std::path::PathBuf,
        projects_dir: std::path::PathBuf,
        token_limit: Option<i64>,
    ) -> Router {
        let state = Arc::new(AppState {
            db_path,
            projects_dirs: Some(vec![projects_dir]),
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
            openai_refresh_interval: 300,
            openai_lookback_days: 30,
            openai_cache: tokio::sync::RwLock::new(None),
            openai_refresh_lock: tokio::sync::Mutex::new(()),
            db_lock: tokio::sync::Mutex::new(()),
            webhook_state: tokio::sync::Mutex::new(WebhookState::default()),
            webhook_config: WebhookConfig::default(),
            scan_event_tx: tokio::sync::broadcast::channel::<String>(16).0,
            agent_status_config: AgentStatusConfig::default(),
            agent_status_cache: tokio::sync::RwLock::new(None),
            agent_status_refresh_lock: tokio::sync::Mutex::new(()),
            aggregator_config: AggregatorConfig::default(),
            aggregator_cache: tokio::sync::RwLock::new(None),
            aggregator_refresh_lock: tokio::sync::Mutex::new(()),
            blocks_token_limit: token_limit,
            session_length_hours: 5.0,
            project_aliases: std::collections::HashMap::new(),
            live_provider_cache: tokio::sync::RwLock::new(None),
            live_provider_refresh_lock: tokio::sync::Mutex::new(()),
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
            .route(
                "/api/billing-blocks",
                get(crate::server::api::api_billing_blocks),
            )
            .with_state(state)
    }

    #[tokio::test]
    async fn test_api_billing_blocks_quota_present_when_limit_set() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app_with_token_limit(db_path, projects, Some(1_000_000));

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/billing-blocks")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // token_limit must be echoed back.
        assert_eq!(json["token_limit"].as_i64().unwrap(), 1_000_000);

        let blocks = json["blocks"].as_array().unwrap();
        assert!(
            !blocks.is_empty(),
            "seeded DB must produce at least one block"
        );

        for b in blocks {
            if b["is_active"].as_bool().unwrap_or(false) {
                // Active block must carry a quota object.
                let quota = b.get("quota").expect("active block must have quota");
                assert_eq!(quota["limit_tokens"].as_i64().unwrap(), 1_000_000);
                let sev = quota["current_severity"].as_str().unwrap();
                assert!(
                    ["ok", "warn", "danger"].contains(&sev),
                    "current_severity must be ok/warn/danger, got: {sev}"
                );
            } else {
                // Inactive (historical) blocks must NOT have quota.
                assert!(
                    b.get("quota").is_none(),
                    "inactive block must not have quota field"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_api_billing_blocks_exposes_depletion_forecast_for_active_block() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join("projects").join("user").join("proj");
        std::fs::create_dir_all(&projects).unwrap();

        let filepath = projects.join("active.jsonl");
        let mut f = std::fs::File::create(&filepath).unwrap();
        let user_ts = (chrono::Utc::now() - chrono::Duration::minutes(30)).to_rfc3339();
        let assistant_ts = (chrono::Utc::now() - chrono::Duration::minutes(5)).to_rfc3339();
        writeln!(
            f,
            "{}",
            serde_json::json!({
                "type": "user",
                "sessionId": "active-s1",
                "timestamp": user_ts,
                "cwd": "/home/user/project"
            })
        )
        .unwrap();
        writeln!(
            f,
            "{}",
            serde_json::json!({
                "type": "assistant",
                "sessionId": "active-s1",
                "timestamp": assistant_ts,
                "cwd": "/home/user/project",
                "message": {
                    "id": "active-msg-1",
                    "model": "claude-sonnet-4-6",
                    "usage": {
                        "input_tokens": 200000,
                        "output_tokens": 100000,
                        "cache_read_input_tokens": 0,
                        "cache_creation_input_tokens": 0
                    },
                    "content": []
                }
            })
        )
        .unwrap();

        let db_path = tmp.path().join("usage.db");
        scanner::scan(Some(vec![tmp.path().join("projects")]), &db_path, false).unwrap();
        let app = test_app_with_token_limit(db_path, tmp.path().join("projects"), Some(1_000_000));

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/billing-blocks")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json.get("depletion_forecast").is_some());
        assert_eq!(
            json["depletion_forecast"]["primary_signal"]["kind"],
            "billing_block"
        );
    }

    #[tokio::test]
    async fn test_api_billing_blocks_shape() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app(db_path, projects);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/billing-blocks")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Required top-level fields.
        assert!(json.get("session_length_hours").is_some());
        assert!(json.get("historical_max_tokens").is_some());
        assert!(json.get("blocks").is_some());
        // token_limit should be null when not configured.
        assert!(json["token_limit"].is_null());
        assert!(json.get("quota_suggestions").is_some());
        assert!(json.get("depletion_forecast").is_none());
        assert!(json.get("predictive_insights").is_some());
        // blocks must be an array.
        assert!(json["blocks"].is_array());
        // At least one block from the seeded turn.
        let blocks = json["blocks"].as_array().unwrap();
        assert!(
            !blocks.is_empty(),
            "seeded DB should produce at least one block"
        );
        // Each block must have required fields.
        let b = &blocks[0];
        assert!(b.get("start").is_some());
        assert!(b.get("end").is_some());
        assert!(b.get("tokens").is_some());
        assert!(b.get("cost_nanos").is_some());
        assert!(b.get("is_active").is_some());
        assert!(b.get("entry_count").is_some());
        // quota field must be absent when token_limit is not set.
        assert!(b.get("quota").is_none());
    }

    #[tokio::test]
    async fn test_api_billing_blocks_returns_configured_session_length() {
        // Verify that AppState.session_length_hours is echoed back in the response.
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        // Build a custom AppState with session_length_hours = 2.5
        let state = Arc::new(AppState {
            db_path,
            projects_dirs: Some(vec![projects]),
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
            openai_refresh_interval: 300,
            openai_lookback_days: 30,
            openai_cache: tokio::sync::RwLock::new(None),
            openai_refresh_lock: tokio::sync::Mutex::new(()),
            db_lock: tokio::sync::Mutex::new(()),
            webhook_state: tokio::sync::Mutex::new(WebhookState::default()),
            webhook_config: WebhookConfig::default(),
            scan_event_tx: tokio::sync::broadcast::channel::<String>(16).0,
            agent_status_config: AgentStatusConfig::default(),
            agent_status_cache: tokio::sync::RwLock::new(None),
            agent_status_refresh_lock: tokio::sync::Mutex::new(()),
            aggregator_config: AggregatorConfig::default(),
            aggregator_cache: tokio::sync::RwLock::new(None),
            aggregator_refresh_lock: tokio::sync::Mutex::new(()),
            blocks_token_limit: None,
            session_length_hours: 2.5,
            project_aliases: std::collections::HashMap::new(),
            live_provider_cache: tokio::sync::RwLock::new(None),
            live_provider_refresh_lock: tokio::sync::Mutex::new(()),
        });
        let app = Router::new()
            .route(
                "/api/billing-blocks",
                get(crate::server::api::api_billing_blocks),
            )
            .with_state(state);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/billing-blocks")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let returned = json["session_length_hours"].as_f64().unwrap();
        assert!(
            (returned - 2.5).abs() < 1e-9,
            "expected session_length_hours=2.5, got {returned}"
        );
    }

    // ── Phase 3: weekly_by_model in /api/data ─────────────────────────────────

    fn make_assistant_ts(
        session_id: &str,
        ts: &str,
        input: i64,
        output: i64,
        msg_id: &str,
    ) -> String {
        let msg = serde_json::json!({
            "model": "claude-sonnet-4-6",
            "usage": {
                "input_tokens": input,
                "output_tokens": output,
                "cache_read_input_tokens": 0,
                "cache_creation_input_tokens": 0,
            },
            "content": [],
            "id": msg_id,
        });
        serde_json::json!({
            "type": "assistant",
            "sessionId": session_id,
            "timestamp": ts,
            "cwd": "/home/user/project",
            "message": msg,
        })
        .to_string()
    }

    #[tokio::test]
    async fn api_data_weekly_by_model_present_and_ordered() {
        let tmp = TempDir::new().unwrap();
        let projects = tmp.path().join("projects").join("user").join("proj");
        std::fs::create_dir_all(&projects).unwrap();

        // Seed turns in two different weeks.
        let filepath = projects.join("weekly_sess.jsonl");
        let mut f = std::fs::File::create(&filepath).unwrap();
        writeln!(
            f,
            "{}",
            serde_json::json!({"type":"user","sessionId":"s_weekly","timestamp":"2026-04-06T09:00:00Z","cwd":"/home"})
        )
        .unwrap();
        // Week 1: Apr 6
        writeln!(
            f,
            "{}",
            make_assistant_ts("s_weekly", "2026-04-06T10:00:00Z", 1000, 500, "wm-1")
        )
        .unwrap();
        // Week 2: Apr 13 (next Mon-week)
        writeln!(
            f,
            "{}",
            make_assistant_ts("s_weekly", "2026-04-13T10:00:00Z", 2000, 800, "wm-2")
        )
        .unwrap();

        let db_path = tmp.path().join("weekly.db");
        let parent = tmp.path().join("projects");
        scanner::scan(Some(vec![parent.clone()]), &db_path, false).unwrap();

        let app = test_app(db_path, parent);
        let req = Request::builder()
            .uri("/api/data")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // weekly_by_model must be present.
        let wbm = json
            .get("weekly_by_model")
            .expect("weekly_by_model field missing");
        assert!(wbm.is_array(), "weekly_by_model must be an array");
        let arr = wbm.as_array().unwrap();

        // Must have at least one entry (from the seeded turns).
        assert!(
            !arr.is_empty(),
            "weekly_by_model should not be empty after seeding turns"
        );

        // Entries must have expected fields.
        let entry = &arr[0];
        assert!(entry.get("week").is_some(), "entry.week missing");
        assert!(entry.get("model").is_some(), "entry.model missing");
        assert!(
            entry.get("input_tokens").is_some(),
            "entry.input_tokens missing"
        );
        assert!(
            entry.get("output_tokens").is_some(),
            "entry.output_tokens missing"
        );
        assert!(
            entry.get("cost_nanos").is_some(),
            "entry.cost_nanos missing"
        );

        // Entries must be ordered week ASC.
        let weeks: Vec<&str> = arr
            .iter()
            .filter_map(|e| e.get("week").and_then(|w| w.as_str()))
            .collect();
        let mut sorted_weeks = weeks.clone();
        sorted_weeks.sort();
        assert_eq!(
            weeks, sorted_weeks,
            "weekly_by_model must be sorted by week ASC"
        );
    }

    // ── Phase 5: /api/context-window ──────────────────────────────────────────

    /// Empty live_events (no context rows) → `{"enabled": false}`.
    #[tokio::test]
    async fn test_api_context_window_empty_returns_disabled() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app(db_path, projects);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/context-window")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            data["enabled"], false,
            "empty live_events must return enabled:false, got: {}",
            data
        );
    }

    /// live_events with context columns → endpoint returns expected shape.
    #[tokio::test]
    async fn test_api_context_window_populated_returns_shape() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        // Seed a live_events row with context data directly.
        {
            use crate::scanner::db::{init_db, open_db};
            let conn = open_db(&db_path).unwrap();
            init_db(&conn).unwrap();
            conn.execute(
                "INSERT OR IGNORE INTO live_events
                    (dedup_key, received_at, session_id, tool_name,
                     cost_usd_nanos, input_tokens, output_tokens, raw_json,
                     context_input_tokens, context_window_size)
                 VALUES ('ctx-key-1', '2026-04-18T10:00:00Z', 'claude:ses1', 'Edit',
                         1000000, 500, 100, '{}', 110000, 200000)",
                [],
            )
            .unwrap();
        }

        let app = test_app(db_path, projects);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/context-window")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(data["total_input_tokens"].as_i64(), Some(110_000));
        assert_eq!(data["context_window_size"].as_i64(), Some(200_000));
        let pct = data["pct"].as_f64().expect("pct must be f64");
        assert!((pct - 0.55).abs() < 1e-6, "expected pct≈0.55, got {}", pct);
        assert_eq!(
            data["severity"].as_str(),
            Some("warn"),
            "55% fill should be warn"
        );
        assert!(data.get("captured_at").is_some(), "captured_at missing");
    }

    // ── Phase 8: /api/cost-reconciliation tests ───────────────────────────────

    /// Empty DB (no hook events) → { "enabled": false }.
    #[tokio::test]
    async fn test_cost_reconciliation_empty_db() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);
        let app = test_app(db_path, projects);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/cost-reconciliation")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(data["enabled"], serde_json::json!(false));
    }

    /// Seeded DB with hook events and turns → full reconciliation shape.
    #[tokio::test]
    async fn test_cost_reconciliation_seeded_db() {
        use crate::scanner::db::{init_db, open_db};

        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        // Seed a live_event with hook_reported_cost_nanos.
        {
            let conn = open_db(&db_path).unwrap();
            init_db(&conn).unwrap();
            conn.execute(
                "INSERT OR IGNORE INTO live_events
                    (dedup_key, received_at, session_id, tool_name,
                     cost_usd_nanos, input_tokens, output_tokens, raw_json,
                     hook_reported_cost_nanos)
                 VALUES ('k1', datetime('now'), 'ses1', 'Bash',
                         140000000, 100, 50, '{}', 140000000)",
                [],
            )
            .unwrap();
        }

        let app = test_app(db_path, projects);

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/cost-reconciliation?period=month")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let data: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(data["enabled"], serde_json::json!(true));
        assert_eq!(data["period"].as_str(), Some("month"));
        assert!(
            data["hook_total_nanos"].as_i64().unwrap_or(0) > 0,
            "hook_total_nanos should be > 0"
        );
        assert!(
            data["local_total_nanos"].is_number(),
            "local_total_nanos missing"
        );
        assert!(data["divergence_pct"].is_number(), "divergence_pct missing");
        let breakdown = data["breakdown"]
            .as_array()
            .expect("breakdown must be array");
        assert!(!breakdown.is_empty(), "breakdown should be non-empty");
        // Each entry must have day, hook_nanos, local_nanos.
        let first = &breakdown[0];
        assert!(first["day"].is_string(), "day missing");
        assert!(first["hook_nanos"].is_number(), "hook_nanos missing");
        assert!(first["local_nanos"].is_number(), "local_nanos missing");
    }

    #[tokio::test]
    async fn test_refresh_openai_reconciliation_uses_fresh_cache_without_fetch() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let cached = available_openai_reconciliation(0.12, 0.25, 7);
        let mut state = base_state(db_path, projects);
        state.openai_refresh_interval = 3600;
        state.openai_cache =
            tokio::sync::RwLock::new(Some((std::time::Instant::now(), cached.clone())));
        let state = Arc::new(state);

        let fetch_count = Arc::new(AtomicUsize::new(0));
        let returned = crate::server::api::refresh_openai_reconciliation_with(
            &state,
            Some(120_000_000),
            Some("test-key".into()),
            {
                let fetch_count = fetch_count.clone();
                move |_, _, _| async move {
                    fetch_count.fetch_add(1, Ordering::SeqCst);
                    OpenAiReconciliation::default()
                }
            },
        )
        .await;

        assert_eq!(fetch_count.load(Ordering::SeqCst), 0);
        assert_eq!(
            serde_json::to_value(&returned).unwrap(),
            serde_json::to_value(&cached).unwrap()
        );
    }

    #[tokio::test]
    async fn test_refresh_openai_reconciliation_replaces_stale_cache_with_fresh_response() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let cached = available_openai_reconciliation(0.12, 0.25, 7);
        let fresh = available_openai_reconciliation(0.45, 0.99, 19);

        let mut state = base_state(db_path, projects);
        state.openai_refresh_interval = 1;
        state.openai_lookback_days = 14;
        state.openai_cache = tokio::sync::RwLock::new(Some((
            std::time::Instant::now() - Duration::from_secs(5),
            cached,
        )));
        let state = Arc::new(state);

        let fetch_count = Arc::new(AtomicUsize::new(0));
        let returned = crate::server::api::refresh_openai_reconciliation_with(
            &state,
            Some(450_000_000),
            Some("test-key".into()),
            {
                let fetch_count = fetch_count.clone();
                let fresh = fresh.clone();
                move |key, days, estimated_local_cost| async move {
                    fetch_count.fetch_add(1, Ordering::SeqCst);
                    assert_eq!(key, "test-key");
                    assert_eq!(days, 14);
                    assert!((estimated_local_cost - 0.45).abs() < f64::EPSILON);
                    fresh
                }
            },
        )
        .await;

        assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
        assert_eq!(
            serde_json::to_value(&returned).unwrap(),
            serde_json::to_value(&fresh).unwrap()
        );

        let stored = state.openai_cache.read().await;
        let (_, stored_resp) = stored.as_ref().expect("cache should be updated");
        assert_eq!(
            serde_json::to_value(stored_resp).unwrap(),
            serde_json::to_value(&fresh).unwrap()
        );
    }

    #[tokio::test]
    async fn test_refresh_openai_reconciliation_falls_back_to_stale_cache_when_fetch_unavailable() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let cached = available_openai_reconciliation(0.12, 0.25, 7);

        let mut state = base_state(db_path, projects);
        state.openai_refresh_interval = 1;
        state.openai_cache = tokio::sync::RwLock::new(Some((
            std::time::Instant::now() - Duration::from_secs(5),
            cached.clone(),
        )));
        let state = Arc::new(state);

        let fetch_count = Arc::new(AtomicUsize::new(0));
        let returned = crate::server::api::refresh_openai_reconciliation_with(
            &state,
            Some(120_000_000),
            Some("test-key".into()),
            {
                let fetch_count = fetch_count.clone();
                move |_, _, _| async move {
                    fetch_count.fetch_add(1, Ordering::SeqCst);
                    OpenAiReconciliation {
                        available: false,
                        lookback_days: 7,
                        start_date: "2026-04-13".into(),
                        end_date: "2026-04-19".into(),
                        estimated_local_cost: 0.12,
                        api_usage_cost: 0.0,
                        api_input_tokens: 0,
                        api_output_tokens: 0,
                        api_cached_input_tokens: 0,
                        api_requests: 0,
                        delta_cost: 0.0,
                        error: Some("upstream unavailable".into()),
                    }
                }
            },
        )
        .await;

        assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
        assert_eq!(
            serde_json::to_value(&returned).unwrap(),
            serde_json::to_value(&cached).unwrap()
        );

        let stored = state.openai_cache.read().await;
        let (_, stored_resp) = stored.as_ref().expect("cache should retain stale value");
        assert_eq!(
            serde_json::to_value(stored_resp).unwrap(),
            serde_json::to_value(&cached).unwrap()
        );
    }

    #[tokio::test]
    async fn test_refresh_openai_reconciliation_without_admin_key_returns_setup_error() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let mut state = base_state(db_path, projects);
        state.openai_admin_key_env = "MISSING_OPENAI_ADMIN_KEY".into();
        state.openai_lookback_days = 21;
        let state = Arc::new(state);

        let fetch_count = Arc::new(AtomicUsize::new(0));
        let returned = crate::server::api::refresh_openai_reconciliation_with(
            &state,
            Some(330_000_000),
            None,
            {
                let fetch_count = fetch_count.clone();
                move |_, _, _| async move {
                    fetch_count.fetch_add(1, Ordering::SeqCst);
                    OpenAiReconciliation::default()
                }
            },
        )
        .await;

        assert_eq!(fetch_count.load(Ordering::SeqCst), 0);
        assert!(!returned.available);
        assert!((returned.estimated_local_cost - 0.33).abs() < f64::EPSILON);
        assert_eq!(returned.lookback_days, 21);
        assert_eq!(
            returned.error.as_deref(),
            Some(
                "Set MISSING_OPENAI_ADMIN_KEY to enable OpenAI organization usage reconciliation."
            )
        );
    }

    #[tokio::test]
    async fn test_api_data_includes_cached_openai_reconciliation_when_enabled() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let cached = available_openai_reconciliation(0.12, 0.25, 7);
        let mut state = base_state(db_path, projects);
        state.openai_enabled = true;
        state.openai_refresh_interval = 3600;
        state.openai_cache = tokio::sync::RwLock::new(Some((std::time::Instant::now(), cached)));
        let app = Router::new()
            .route("/api/data", get(crate::server::api::api_data))
            .with_state(Arc::new(state));

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
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["openai_reconciliation"]["available"], true);
        assert_eq!(json["openai_reconciliation"]["api_requests"], 7);
        assert_eq!(
            json["openai_reconciliation"]["api_usage_cost"],
            serde_json::json!(0.25)
        );
    }

    #[tokio::test]
    async fn api_archive_imports_returns_empty_array_when_no_imports() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        // Point HOME at the tempdir so default_root() resolves to a fresh
        // directory with no imports. list_imports() returns [] when the
        // exports sub-directory does not exist.
        let _home_guard = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev_home = std::env::var_os("HOME");
        unsafe { std::env::set_var("HOME", tmp.path()) };

        let app = test_app(db_path, projects);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/archive/imports")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Restore HOME before any assertion that could panic.
        match prev_home {
            Some(prev) => unsafe { std::env::set_var("HOME", prev) },
            None => unsafe { std::env::remove_var("HOME") },
        }

        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&*bytes, b"[]");
    }

    #[tokio::test]
    async fn api_archive_list_returns_empty_array_when_no_snapshots() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        // Point HOME at the tempdir so default_root() resolves to a fresh
        // directory with no snapshots. The archive list() call returns [] when
        // the snapshots sub-directory does not exist.
        let _home_guard = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev_home = std::env::var_os("HOME");
        unsafe { std::env::set_var("HOME", tmp.path()) };

        let app = test_app(db_path, projects);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/archive")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Restore HOME before any assertion that could panic.
        match prev_home {
            Some(prev) => unsafe { std::env::set_var("HOME", prev) },
            None => unsafe { std::env::remove_var("HOME") },
        }

        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&*bytes, b"[]");
    }

    #[tokio::test]
    async fn web_conversation_returns_401_without_bearer() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let _home_guard = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev_home = std::env::var_os("HOME");
        unsafe { std::env::set_var("HOME", tmp.path()) };

        let app = test_app(db_path, projects);
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/archive/web-conversation")
                    .header("content-type", "application/json")
                    .body(Body::from(b"{}".as_ref()))
                    .unwrap(),
            )
            .await
            .unwrap();

        match prev_home {
            Some(prev) => unsafe { std::env::set_var("HOME", prev) },
            None => unsafe { std::env::remove_var("HOME") },
        }

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn web_conversation_saves_with_valid_bearer() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let _home_guard = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev_home = std::env::var_os("HOME");
        unsafe { std::env::set_var("HOME", tmp.path()) };

        // Seed the companion token into the tempdir.
        let token_path = tmp.path().join(".heimdall").join("companion-token");
        let token = crate::archive::companion_token::read_or_init(&token_path).unwrap();
        let token_hex = token.as_hex().to_owned();

        let conv = crate::archive::web::WebConversation {
            vendor: "claude.ai".into(),
            conversation_id: "test-conv-1".into(),
            captured_at: "2026-04-28T12:00:00.000000Z".into(),
            schema_fingerprint: "test/v1".into(),
            payload: serde_json::json!({"hello": "world"}),
        };
        let body = serde_json::to_vec(&conv).unwrap();

        let app = test_app(db_path, projects);
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/archive/web-conversation")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token_hex}"))
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        match prev_home {
            Some(prev) => unsafe { std::env::set_var("HOME", prev) },
            None => unsafe { std::env::remove_var("HOME") },
        }

        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["saved"], true);

        // File should exist under <HOME>/.heimdall/archive/web/claude.ai/test-conv-1.json
        let expected = tmp
            .path()
            .join(".heimdall")
            .join("archive")
            .join("web")
            .join("claude.ai")
            .join("test-conv-1.json");
        assert!(
            expected.is_file(),
            "saved file should exist at {expected:?}"
        );
    }

    #[tokio::test]
    async fn web_conversation_returns_unchanged_for_byte_identical_payload() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let _home_guard = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev_home = std::env::var_os("HOME");
        unsafe { std::env::set_var("HOME", tmp.path()) };

        // Seed the companion token into the tempdir.
        let token_path = tmp.path().join(".heimdall").join("companion-token");
        let token = crate::archive::companion_token::read_or_init(&token_path).unwrap();
        let token_hex = token.as_hex().to_owned();

        let conv = crate::archive::web::WebConversation {
            vendor: "claude.ai".into(),
            conversation_id: "test-conv-2".into(),
            captured_at: "2026-04-28T12:00:00.000000Z".into(),
            schema_fingerprint: "test/v1".into(),
            payload: serde_json::json!({"hello": "world"}),
        };
        let body = serde_json::to_vec(&conv).unwrap();

        // First POST — should save.
        let app1 = test_app(db_path.clone(), projects.clone());
        let _ = app1
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/archive/web-conversation")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token_hex}"))
                    .body(Body::from(body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Second POST with identical payload — should be unchanged.
        let app2 = test_app(db_path, projects);
        let resp = app2
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/archive/web-conversation")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token_hex}"))
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        match prev_home {
            Some(prev) => unsafe { std::env::set_var("HOME", prev) },
            None => unsafe { std::env::remove_var("HOME") },
        }

        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["unchanged"], true);
    }

    #[tokio::test]
    async fn web_conversations_returns_empty_listing_when_no_captures() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let _home_guard = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev_home = std::env::var_os("HOME");
        unsafe { std::env::set_var("HOME", tmp.path()) };

        let app = test_app(db_path, projects);
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/archive/web-conversations")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        match prev_home {
            Some(prev) => unsafe { std::env::set_var("HOME", prev) },
            None => unsafe { std::env::remove_var("HOME") },
        }

        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["conversations"], serde_json::json!([]));
        assert_eq!(json["heartbeat"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn companion_heartbeat_requires_bearer() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let _home_guard = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev_home = std::env::var_os("HOME");
        unsafe { std::env::set_var("HOME", tmp.path()) };

        let app = test_app(db_path, projects);
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/archive/companion-heartbeat")
                    .header("content-type", "application/json")
                    .body(Body::from(b"{}".as_ref()))
                    .unwrap(),
            )
            .await
            .unwrap();

        match prev_home {
            Some(prev) => unsafe { std::env::set_var("HOME", prev) },
            None => unsafe { std::env::remove_var("HOME") },
        }

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn companion_heartbeat_persists_to_disk() {
        let tmp = TempDir::new().unwrap();
        let (db_path, projects) = setup_test_db(&tmp);

        let _home_guard = HOME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev_home = std::env::var_os("HOME");
        unsafe { std::env::set_var("HOME", tmp.path()) };

        // Seed the companion token into the tempdir so the handler finds it.
        let token_path = tmp.path().join(".heimdall").join("companion-token");
        let token = crate::archive::companion_token::read_or_init(&token_path).unwrap();
        let token_hex = token.as_hex().to_owned();

        let body = serde_json::json!({
            "extension_version": "0.1.0",
            "user_agent": "UA",
            "vendor": "claude.ai",
        });

        let app = test_app(db_path, projects);
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/archive/companion-heartbeat")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token_hex}"))
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        match prev_home {
            Some(prev) => unsafe { std::env::set_var("HOME", prev) },
            None => unsafe { std::env::remove_var("HOME") },
        }

        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["ok"], true);

        // Heartbeat file should exist on disk.
        let expected = tmp
            .path()
            .join(".heimdall")
            .join("archive")
            .join("web")
            .join("companion-heartbeat.json");
        assert!(
            expected.is_file(),
            "heartbeat file should exist at {expected:?}"
        );
    }
}
