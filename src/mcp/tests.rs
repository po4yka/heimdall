/// In-process MCP client/server round-trip tests.
///
/// Each test seeds a tempdir DB, spins up a `HeimdallMcpServer` with an
/// in-memory `tokio::io::duplex` transport (same approach as `mcp_http_handler`),
/// and asserts tool response shapes by parsing newline-delimited JSON.
#[cfg(test)]
mod mcp_tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use http_body_util::BodyExt;
    use rmcp::ServiceExt;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::super::{McpTransport, is_loopback_bind_host, mcp_http_handler};
    use crate::mcp::tools::HeimdallMcpServer;
    use crate::scanner::db::{init_db, open_db};

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Create an empty but schema-initialised DB in a tempdir.
    fn seed_db() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_db(&db_path).unwrap();
        init_db(&conn).unwrap();
        (dir, db_path)
    }

    /// Seed a DB with minimal turns so today / stats return non-empty data.
    fn seed_db_with_data() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = open_db(&db_path).unwrap();
        init_db(&conn).unwrap();

        let today = chrono::Local::now()
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();
        conn.execute(
            "INSERT OR IGNORE INTO sessions
             (session_id, provider, first_timestamp, last_timestamp)
             VALUES ('claude:test-session', 'claude', ?1, ?1)",
            rusqlite::params![today],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO turns
             (session_id, provider, timestamp, model,
              input_tokens, output_tokens, estimated_cost_nanos, billing_mode, cost_confidence)
             VALUES ('claude:test-session', 'claude', ?1, 'claude-sonnet-4-5',
                     1000, 200, 5000000, 'estimated_local', 'high')",
            rusqlite::params![today],
        )
        .unwrap();

        (dir, db_path)
    }

    fn insert_session(
        conn: &rusqlite::Connection,
        session_id: &str,
        project_name: Option<&str>,
        first_timestamp: &str,
        last_timestamp: &str,
        total_input_tokens: i64,
        total_output_tokens: i64,
        total_estimated_cost_nanos: i64,
        turn_count: i64,
    ) {
        conn.execute(
            "INSERT INTO sessions
             (session_id, provider, project_name, first_timestamp, last_timestamp,
              total_input_tokens, total_output_tokens, total_estimated_cost_nanos, turn_count)
             VALUES (?1, 'claude', ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                session_id,
                project_name,
                first_timestamp,
                last_timestamp,
                total_input_tokens,
                total_output_tokens,
                total_estimated_cost_nanos,
                turn_count
            ],
        )
        .unwrap();
    }

    fn insert_turn(
        conn: &rusqlite::Connection,
        session_id: &str,
        timestamp: &str,
        model: &str,
        input_tokens: i64,
        output_tokens: i64,
        estimated_cost_nanos: i64,
    ) {
        conn.execute(
            "INSERT INTO turns
             (session_id, provider, timestamp, model, input_tokens, output_tokens,
              estimated_cost_nanos, billing_mode, cost_confidence, message_id, source_path)
             VALUES (?1, 'claude', ?2, ?3, ?4, ?5, ?6, 'estimated_local', 'high', ?7, '')",
            rusqlite::params![
                session_id,
                timestamp,
                model,
                input_tokens,
                output_tokens,
                estimated_cost_nanos,
                format!("{session_id}-{timestamp}-{model}")
            ],
        )
        .unwrap();
    }

    fn insert_live_event_with_hook_cost(
        conn: &rusqlite::Connection,
        dedup_key: &str,
        received_at: &str,
        session_id: &str,
        hook_reported_cost_nanos: i64,
    ) {
        conn.execute(
            "INSERT INTO live_events
             (dedup_key, received_at, session_id, tool_name, raw_json, hook_reported_cost_nanos)
             VALUES (?1, ?2, ?3, 'Bash', '{}', ?4)",
            rusqlite::params![dedup_key, received_at, session_id, hook_reported_cost_nanos],
        )
        .unwrap();
    }

    /// Spin up a server on an in-memory duplex transport.
    ///
    /// Returns a write handle to send JSON-RPC messages and a read handle to
    /// receive responses. Dropping the write handle signals EOF to the server.
    async fn connect_server(
        db_path: PathBuf,
    ) -> (
        tokio::io::DuplexStream, // client writes here (server reads)
        tokio::io::DuplexStream, // client reads here  (server writes)
    ) {
        let (client_write, server_read) = tokio::io::duplex(128 * 1024);
        let (server_write, client_read) = tokio::io::duplex(128 * 1024);

        let server = HeimdallMcpServer {
            db_path,
            default_session_length_hours: 5.0,
        };
        // Spawn the server in the background; it runs until EOF on server_read.
        tokio::spawn(async move {
            if let Ok(svc) = server.serve((server_read, server_write)).await {
                let _ = svc.waiting().await;
            }
        });

        (client_write, client_read)
    }

    /// Send a raw JSON-RPC line to the server.
    async fn send_msg(w: &mut tokio::io::DuplexStream, msg: serde_json::Value) {
        let mut line = serde_json::to_vec(&msg).unwrap();
        line.push(b'\n');
        w.write_all(&line).await.unwrap();
    }

    /// Read the next newline-terminated JSON-RPC line from the server.
    ///
    /// We read byte-by-byte to avoid pulling in `tokio_util`/`futures` codecs.
    async fn read_line(r: &mut tokio::io::DuplexStream) -> serde_json::Value {
        let mut buf = Vec::new();
        loop {
            let mut byte = [0u8; 1];
            match r.read(&mut byte).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if byte[0] == b'\n' {
                        break;
                    }
                    buf.push(byte[0]);
                }
                Err(_) => break,
            }
        }
        serde_json::from_slice(&buf).unwrap_or(serde_json::Value::Null)
    }

    /// Perform the MCP initialize handshake and discard the response.
    async fn do_initialize(w: &mut tokio::io::DuplexStream, r: &mut tokio::io::DuplexStream) {
        send_msg(
            w,
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 0,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": { "name": "test", "version": "1" }
                }
            }),
        )
        .await;
        let _ = read_line(r).await; // consume the initialize response

        // Send the initialized notification (no response expected).
        send_msg(
            w,
            serde_json::json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            }),
        )
        .await;
    }

    /// Call a tool and return the parsed response.
    async fn call_tool(
        w: &mut tokio::io::DuplexStream,
        r: &mut tokio::io::DuplexStream,
        id: u64,
        tool_name: &str,
        args: serde_json::Value,
    ) -> serde_json::Value {
        send_msg(
            w,
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "tools/call",
                "params": { "name": tool_name, "arguments": args }
            }),
        )
        .await;
        read_line(r).await
    }

    fn build_http_request(body: impl Into<Body>) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/mcp")
            .header(header::CONTENT_TYPE, "application/json")
            .body(body.into())
            .unwrap()
    }

    async fn response_body_text(resp: axum::response::Response) -> String {
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    fn parse_sse_message(body: &str) -> serde_json::Value {
        let data_line = body
            .lines()
            .find(|line| line.starts_with("data: "))
            .expect("SSE data line");
        serde_json::from_str(data_line.trim_start_matches("data: ")).expect("valid JSON-RPC body")
    }

    // ── Transport parsing ─────────────────────────────────────────────────────

    #[test]
    fn mcp_transport_from_str_is_case_insensitive_and_rejects_unknown_values() {
        assert_eq!(
            "stdio".parse::<McpTransport>().unwrap(),
            McpTransport::Stdio
        );
        assert_eq!("HTTP".parse::<McpTransport>().unwrap(), McpTransport::Http);

        let err = "socket".parse::<McpTransport>().unwrap_err();
        assert_eq!(err, "unknown transport 'socket': expected stdio | http");
    }

    #[test]
    fn loopback_bind_host_accepts_only_local_targets() {
        assert!(is_loopback_bind_host("localhost"));
        assert!(is_loopback_bind_host("127.0.0.1"));
        assert!(is_loopback_bind_host("::1"));
        assert!(is_loopback_bind_host("[::1]"));
        assert!(!is_loopback_bind_host("0.0.0.0"));
        assert!(!is_loopback_bind_host("192.168.1.10"));
    }

    // ── HTTP transport ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn mcp_http_handler_invalid_json_returns_parse_error_document() {
        let (_dir, db_path) = seed_db();
        let req = build_http_request("{ invalid json");

        let resp = mcp_http_handler(req, Arc::new(db_path)).await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let body = response_body_text(resp).await;
        let payload: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(payload["jsonrpc"], "2.0");
        assert!(payload["id"].is_null());
        assert_eq!(payload["error"]["code"], -32700);
        assert_eq!(payload["error"]["message"], "Parse error: invalid JSON");
    }

    #[tokio::test]
    async fn mcp_http_handler_rejects_bodies_over_limit() {
        let (_dir, db_path) = seed_db();
        let oversized = vec![b'x'; 4 * 1024 * 1024 + 1];
        let req = build_http_request(oversized);

        let resp = mcp_http_handler(req, Arc::new(db_path)).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = response_body_text(resp).await;
        assert_eq!(body, "body read error");
    }

    #[tokio::test]
    async fn mcp_http_handler_initialize_returns_sse_message_frame() {
        let (_dir, db_path) = seed_db();
        let req = build_http_request(
            serde_json::to_vec(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 41,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": { "name": "http-test", "version": "1" }
                }
            }))
            .unwrap(),
        );

        let resp = mcp_http_handler(req, Arc::new(db_path)).await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/event-stream"
        );

        let body = response_body_text(resp).await;
        assert!(body.starts_with("event: message\n"));
        assert!(body.ends_with("\n\n"));

        let payload = parse_sse_message(&body);
        assert_eq!(payload["jsonrpc"], "2.0");
        assert_eq!(payload["id"], 41);
        assert!(
            payload["result"].is_object(),
            "initialize should return a result"
        );
    }

    #[tokio::test]
    async fn mcp_http_handler_notification_without_initialize_returns_internal_error() {
        let (_dir, db_path) = seed_db();
        let req = build_http_request(
            serde_json::to_vec(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            }))
            .unwrap(),
        );

        let resp = mcp_http_handler(req, Arc::new(db_path)).await;

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let body = response_body_text(resp).await;
        let payload: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(payload["jsonrpc"], "2.0");
        assert!(payload["id"].is_null());
        assert_eq!(payload["error"]["code"], -32000);
        assert!(
            payload["error"]["message"]
                .as_str()
                .is_some_and(|msg| !msg.is_empty()),
            "internal error should include a message"
        );
    }

    #[tokio::test]
    async fn mcp_http_handler_non_initialize_request_returns_internal_error() {
        let (_dir, db_path) = seed_db();
        let req = build_http_request(
            serde_json::to_vec(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 77,
                "method": "heimdall/does-not-exist"
            }))
            .unwrap(),
        );

        let resp = mcp_http_handler(req, Arc::new(db_path)).await;

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let body = response_body_text(resp).await;
        let payload: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(payload["jsonrpc"], "2.0");
        assert!(payload["id"].is_null());
        assert!(
            payload["error"].is_object(),
            "non-initialize requests should surface a JSON-RPC transport error"
        );
    }

    // ── Tool list test ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn tools_list_returns_ten_tools() {
        let (_dir, db_path) = seed_db();
        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        send_msg(
            &mut w,
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/list",
                "params": {}
            }),
        )
        .await;
        let v = read_line(&mut r).await;

        let tools = v["result"]["tools"].as_array().expect("tools array");
        assert_eq!(tools.len(), 10, "expected 10 tools, got: {}", tools.len());

        let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"heimdall_today"), "missing heimdall_today");
        assert!(names.contains(&"heimdall_stats"), "missing heimdall_stats");
        assert!(
            names.contains(&"heimdall_cost_reconciliation"),
            "missing heimdall_cost_reconciliation"
        );
        assert!(
            names.contains(&"heimdall_weekly"),
            "missing heimdall_weekly"
        );
        assert!(
            names.contains(&"heimdall_sessions"),
            "missing heimdall_sessions"
        );
        assert!(
            names.contains(&"heimdall_blocks_active"),
            "missing heimdall_blocks_active"
        );
        assert!(
            names.contains(&"heimdall_optimize_grade"),
            "missing heimdall_optimize_grade"
        );
        assert!(
            names.contains(&"heimdall_rate_windows"),
            "missing heimdall_rate_windows"
        );
        assert!(
            names.contains(&"heimdall_context_window"),
            "missing heimdall_context_window"
        );
        assert!(names.contains(&"heimdall_quota"), "missing heimdall_quota");
    }

    // ── heimdall_today ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn heimdall_today_returns_correct_shape() {
        let (_dir, db_path) = seed_db_with_data();
        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let resp = call_tool(&mut w, &mut r, 2, "heimdall_today", serde_json::json!({})).await;
        assert!(resp["error"].is_null(), "tool error: {:?}", resp["error"]);

        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        let data: serde_json::Value = serde_json::from_str(text).unwrap();

        assert!(data["date"].is_string(), "missing date");
        assert!(data["total_cost_usd"].is_number(), "missing total_cost_usd");
        assert!(data["by_model"].is_array(), "missing by_model");
        assert!(data["by_provider"].is_array(), "missing by_provider");
    }

    // ── heimdall_blocks_active ────────────────────────────────────────────────

    #[tokio::test]
    async fn heimdall_blocks_active_empty_db_returns_null_block() {
        let (_dir, db_path) = seed_db();
        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let resp = call_tool(
            &mut w,
            &mut r,
            3,
            "heimdall_blocks_active",
            serde_json::json!({}),
        )
        .await;
        assert!(resp["error"].is_null(), "tool error: {:?}", resp["error"]);

        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(data["block"].is_null(), "expected null block on empty DB");
    }

    // ── heimdall_optimize_grade ───────────────────────────────────────────────

    #[tokio::test]
    async fn heimdall_optimize_grade_returns_grade() {
        let (_dir, db_path) = seed_db();
        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let resp = call_tool(
            &mut w,
            &mut r,
            4,
            "heimdall_optimize_grade",
            serde_json::json!({}),
        )
        .await;
        assert!(resp["error"].is_null(), "tool error: {:?}", resp["error"]);

        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        let data: serde_json::Value = serde_json::from_str(text).unwrap();

        assert!(data["grade"].is_string(), "missing grade");
        assert!(data["findings"].is_array(), "missing findings");
        assert!(
            data["total_estimated_waste_usd"].is_number(),
            "missing total_estimated_waste_usd"
        );

        let grade = data["grade"].as_str().unwrap();
        assert!(
            matches!(grade, "A" | "B" | "C" | "D" | "F"),
            "grade must be A-F, got: {grade}"
        );
    }

    // ── heimdall_stats ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn heimdall_stats_returns_correct_shape() {
        let (_dir, db_path) = seed_db_with_data();
        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let resp = call_tool(&mut w, &mut r, 5, "heimdall_stats", serde_json::json!({})).await;
        assert!(resp["error"].is_null(), "tool error: {:?}", resp["error"]);

        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        let data: serde_json::Value = serde_json::from_str(text).unwrap();

        assert!(data["total_sessions"].is_number(), "missing total_sessions");
        assert!(data["total_turns"].is_number(), "missing total_turns");
        assert!(
            data["total_estimated_cost_usd"].is_number(),
            "missing total_estimated_cost_usd"
        );
        assert!(data["by_model"].is_array(), "missing by_model");
    }

    // ── heimdall_sessions ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn heimdall_sessions_paginates() {
        let (_dir, db_path) = seed_db_with_data();
        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let resp = call_tool(
            &mut w,
            &mut r,
            6,
            "heimdall_sessions",
            serde_json::json!({ "limit": 10, "offset": 0 }),
        )
        .await;
        assert!(resp["error"].is_null(), "tool error: {:?}", resp["error"]);

        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        let data: serde_json::Value = serde_json::from_str(text).unwrap();

        assert!(data["total"].is_number(), "missing total");
        assert!(data["sessions"].is_array(), "missing sessions");
    }

    #[tokio::test]
    async fn heimdall_sessions_defaults_to_limit_50_and_clamps_large_limits() {
        let (_dir, db_path) = seed_db_with_data();
        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let default_resp = call_tool(
            &mut w,
            &mut r,
            61,
            "heimdall_sessions",
            serde_json::json!({}),
        )
        .await;
        assert!(
            default_resp["error"].is_null(),
            "tool error: {:?}",
            default_resp["error"]
        );
        let default_text = default_resp["result"]["content"][0]["text"]
            .as_str()
            .unwrap();
        let default_data: serde_json::Value = serde_json::from_str(default_text).unwrap();
        assert_eq!(default_data["limit"], 50);
        assert_eq!(default_data["offset"], 0);

        let clamped_resp = call_tool(
            &mut w,
            &mut r,
            62,
            "heimdall_sessions",
            serde_json::json!({ "limit": 999, "offset": 7 }),
        )
        .await;
        assert!(
            clamped_resp["error"].is_null(),
            "tool error: {:?}",
            clamped_resp["error"]
        );
        let clamped_text = clamped_resp["result"]["content"][0]["text"]
            .as_str()
            .unwrap();
        let clamped_data: serde_json::Value = serde_json::from_str(clamped_text).unwrap();
        assert_eq!(clamped_data["limit"], 500);
        assert_eq!(clamped_data["offset"], 7);
    }

    #[tokio::test]
    async fn heimdall_sessions_project_filter_matches_substrings_and_handles_empty_results() {
        let (dir, db_path) = seed_db();
        {
            let conn = open_db(&db_path).unwrap();
            insert_session(
                &conn,
                "claude:alpha-1",
                Some("alpha-app"),
                "2026-04-17T09:00:00Z",
                "2026-04-17T10:00:00Z",
                1200,
                300,
                12_500_000,
                3,
            );
            insert_session(
                &conn,
                "claude:beta-1",
                Some("beta-service"),
                "2026-04-18T09:00:00Z",
                "2026-04-18T10:00:00Z",
                2200,
                500,
                18_500_000,
                4,
            );
        }
        let _ = dir;

        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let filtered_resp = call_tool(
            &mut w,
            &mut r,
            63,
            "heimdall_sessions",
            serde_json::json!({ "project": "beta" }),
        )
        .await;
        assert!(
            filtered_resp["error"].is_null(),
            "tool error: {:?}",
            filtered_resp["error"]
        );
        let filtered_text = filtered_resp["result"]["content"][0]["text"]
            .as_str()
            .unwrap();
        let filtered_data: serde_json::Value = serde_json::from_str(filtered_text).unwrap();
        assert_eq!(filtered_data["total"], 1);
        let sessions = filtered_data["sessions"].as_array().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0]["project_name"], "beta-service");
        assert_eq!(sessions[0]["session_id"], "claude:beta-1");

        let missing_resp = call_tool(
            &mut w,
            &mut r,
            64,
            "heimdall_sessions",
            serde_json::json!({ "project": "missing" }),
        )
        .await;
        assert!(
            missing_resp["error"].is_null(),
            "tool error: {:?}",
            missing_resp["error"]
        );
        let missing_text = missing_resp["result"]["content"][0]["text"]
            .as_str()
            .unwrap();
        let missing_data: serde_json::Value = serde_json::from_str(missing_text).unwrap();
        assert_eq!(missing_data["total"], 0);
        assert_eq!(missing_data["sessions"], serde_json::json!([]));
    }

    // ── heimdall_rate_windows ─────────────────────────────────────────────────

    #[tokio::test]
    async fn heimdall_rate_windows_returns_array() {
        let (_dir, db_path) = seed_db();
        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let resp = call_tool(
            &mut w,
            &mut r,
            7,
            "heimdall_rate_windows",
            serde_json::json!({}),
        )
        .await;
        assert!(resp["error"].is_null(), "tool error: {:?}", resp["error"]);

        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(data["rate_windows"].is_array(), "missing rate_windows");
    }

    // ── heimdall_context_window ───────────────────────────────────────────────

    #[tokio::test]
    async fn heimdall_context_window_empty_returns_disabled() {
        let (_dir, db_path) = seed_db();
        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let resp = call_tool(
            &mut w,
            &mut r,
            8,
            "heimdall_context_window",
            serde_json::json!({}),
        )
        .await;
        assert!(resp["error"].is_null(), "tool error: {:?}", resp["error"]);

        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(
            data["enabled"], false,
            "empty DB should return enabled:false"
        );
    }

    // ── heimdall_weekly ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn heimdall_weekly_returns_weeks_array() {
        let (_dir, db_path) = seed_db_with_data();
        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let resp = call_tool(
            &mut w,
            &mut r,
            9,
            "heimdall_weekly",
            serde_json::json!({ "start_of_week": "monday" }),
        )
        .await;
        assert!(resp["error"].is_null(), "tool error: {:?}", resp["error"]);

        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(data["weeks"].is_array(), "missing weeks");
        assert_eq!(data["start_of_week"], "monday");
    }

    #[tokio::test]
    async fn heimdall_weekly_unknown_weekday_uses_monday_grouping() {
        let (dir, db_path) = seed_db();
        {
            let conn = open_db(&db_path).unwrap();
            insert_turn(
                &conn,
                "claude:weekly-boundary",
                "2027-01-10T10:00:00Z",
                "claude-sonnet-4-5",
                100,
                25,
                1_000,
            );
            insert_turn(
                &conn,
                "claude:weekly-boundary",
                "2027-01-11T10:00:00Z",
                "claude-sonnet-4-5",
                200,
                50,
                2_000,
            );
        }
        let _ = dir;

        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let monday_resp = call_tool(
            &mut w,
            &mut r,
            91,
            "heimdall_weekly",
            serde_json::json!({ "start_of_week": "monday" }),
        )
        .await;
        let monday_text = monday_resp["result"]["content"][0]["text"]
            .as_str()
            .unwrap();
        let monday_data: serde_json::Value = serde_json::from_str(monday_text).unwrap();

        let invalid_resp = call_tool(
            &mut w,
            &mut r,
            92,
            "heimdall_weekly",
            serde_json::json!({ "start_of_week": "noday" }),
        )
        .await;
        assert!(
            invalid_resp["error"].is_null(),
            "tool error: {:?}",
            invalid_resp["error"]
        );
        let invalid_text = invalid_resp["result"]["content"][0]["text"]
            .as_str()
            .unwrap();
        let invalid_data: serde_json::Value = serde_json::from_str(invalid_text).unwrap();

        assert_eq!(invalid_data["start_of_week"], "noday");
        assert_eq!(invalid_data["weeks"], monday_data["weeks"]);
        assert_eq!(invalid_data["weeks"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn heimdall_weekly_accepts_weekday_abbreviations() {
        let (dir, db_path) = seed_db();
        {
            let conn = open_db(&db_path).unwrap();
            insert_turn(
                &conn,
                "claude:weekly-sun",
                "2027-01-10T10:00:00Z",
                "claude-sonnet-4-5",
                100,
                25,
                1_000,
            );
            insert_turn(
                &conn,
                "claude:weekly-sun",
                "2027-01-11T10:00:00Z",
                "claude-sonnet-4-5",
                200,
                50,
                2_000,
            );
        }
        let _ = dir;

        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let resp = call_tool(
            &mut w,
            &mut r,
            93,
            "heimdall_weekly",
            serde_json::json!({ "start_of_week": "sun" }),
        )
        .await;
        assert!(resp["error"].is_null(), "tool error: {:?}", resp["error"]);

        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        let weeks = data["weeks"].as_array().unwrap();
        assert_eq!(data["start_of_week"], "sun");
        assert_eq!(weeks.len(), 1);
        assert_eq!(weeks[0]["total_input_tokens"], 300);
        assert_eq!(weeks[0]["total_output_tokens"], 75);
    }

    // ── heimdall_quota ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn heimdall_quota_without_config_returns_enabled_field() {
        let (_dir, db_path) = seed_db();
        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let resp = call_tool(&mut w, &mut r, 10, "heimdall_quota", serde_json::json!({})).await;
        assert!(resp["error"].is_null(), "tool error: {:?}", resp["error"]);

        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        // Either enabled:false (no token_limit configured) or enabled:true with block data.
        assert!(
            data.get("enabled").is_some(),
            "response must contain 'enabled' field"
        );
    }

    // ── heimdall_cost_reconciliation ──────────────────────────────────────────

    /// Empty DB → { "enabled": false }.
    #[tokio::test]
    async fn heimdall_cost_reconciliation_empty_db_returns_disabled() {
        let (_dir, db_path) = seed_db();
        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let resp = call_tool(
            &mut w,
            &mut r,
            11,
            "heimdall_cost_reconciliation",
            serde_json::json!({}),
        )
        .await;
        assert!(resp["error"].is_null(), "tool error: {:?}", resp["error"]);

        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(
            data["enabled"],
            serde_json::json!(false),
            "empty DB should return enabled:false"
        );
    }

    /// Seeded DB with a live event → full shape with enabled:true.
    #[tokio::test]
    async fn heimdall_cost_reconciliation_seeded_db_returns_full_shape() {
        use crate::scanner::db::{init_db, open_db};

        let (dir, db_path) = seed_db_with_data();
        // Seed a live_event with hook_reported_cost_nanos.
        {
            let conn = open_db(&db_path).unwrap();
            init_db(&conn).unwrap();
            conn.execute(
                "INSERT OR IGNORE INTO live_events
                    (dedup_key, received_at, session_id, tool_name,
                     cost_usd_nanos, input_tokens, output_tokens, raw_json,
                     hook_reported_cost_nanos)
                 VALUES ('mcp_k1', datetime('now'), 'claude:test-session', 'Bash',
                         140000000, 100, 50, '{}', 140000000)",
                [],
            )
            .unwrap();
        }
        let _ = dir; // keep tempdir alive

        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let resp = call_tool(
            &mut w,
            &mut r,
            12,
            "heimdall_cost_reconciliation",
            serde_json::json!({ "period": "month" }),
        )
        .await;
        assert!(resp["error"].is_null(), "tool error: {:?}", resp["error"]);

        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(data["enabled"], serde_json::json!(true));
        assert_eq!(data["period"].as_str(), Some("month"));
        assert!(
            data["hook_total_nanos"].is_number(),
            "hook_total_nanos missing"
        );
        assert!(
            data["local_total_nanos"].is_number(),
            "local_total_nanos missing"
        );
        assert!(data["divergence_pct"].is_number(), "divergence_pct missing");
        assert!(data["breakdown"].is_array(), "breakdown must be array");
    }

    #[tokio::test]
    async fn heimdall_cost_reconciliation_invalid_period_uses_month_window() {
        let (dir, db_path) = seed_db();
        let recent = (chrono::Utc::now() - chrono::Duration::days(3)).to_rfc3339();
        let month_only = (chrono::Utc::now() - chrono::Duration::days(20)).to_rfc3339();
        let too_old = (chrono::Utc::now() - chrono::Duration::days(40)).to_rfc3339();
        {
            let conn = open_db(&db_path).unwrap();

            insert_turn(
                &conn,
                "claude:recon-recent",
                &recent,
                "claude-sonnet-4-5",
                100,
                50,
                90,
            );
            insert_turn(
                &conn,
                "claude:recon-month",
                &month_only,
                "claude-sonnet-4-5",
                100,
                50,
                120,
            );
            insert_turn(
                &conn,
                "claude:recon-old",
                &too_old,
                "claude-sonnet-4-5",
                100,
                50,
                500,
            );

            insert_live_event_with_hook_cost(&conn, "recent", &recent, "claude:recon-recent", 100);
            insert_live_event_with_hook_cost(
                &conn,
                "month-only",
                &month_only,
                "claude:recon-month",
                150,
            );
            insert_live_event_with_hook_cost(&conn, "too-old", &too_old, "claude:recon-old", 700);
        }
        let _ = dir;

        let (mut w, mut r) = connect_server(db_path).await;
        do_initialize(&mut w, &mut r).await;

        let resp = call_tool(
            &mut w,
            &mut r,
            121,
            "heimdall_cost_reconciliation",
            serde_json::json!({ "period": "quarter" }),
        )
        .await;
        assert!(resp["error"].is_null(), "tool error: {:?}", resp["error"]);

        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        let data: serde_json::Value = serde_json::from_str(text).unwrap();
        let breakdown = data["breakdown"].as_array().unwrap();

        assert_eq!(data["period"], "quarter");
        assert_eq!(data["hook_total_nanos"], 250);
        assert_eq!(data["local_total_nanos"], 210);
        assert_eq!(breakdown.len(), 2);
        assert!(
            breakdown.iter().all(|row| row["day"] != too_old[..10]),
            "rows outside the 30-day fallback window should be excluded"
        );
    }
}
