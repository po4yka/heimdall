/// In-process MCP client/server round-trip tests.
///
/// Each test seeds a tempdir DB, spins up a `HeimdallMcpServer` with an
/// in-memory `tokio::io::duplex` transport (same approach as `mcp_http_handler`),
/// and asserts tool response shapes by parsing newline-delimited JSON.
#[cfg(test)]
mod mcp_tests {
    use std::path::PathBuf;

    use rmcp::ServiceExt;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

        let server = HeimdallMcpServer { db_path };
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
        assert!(data["hook_total_nanos"].is_number(), "hook_total_nanos missing");
        assert!(data["local_total_nanos"].is_number(), "local_total_nanos missing");
        assert!(data["divergence_pct"].is_number(), "divergence_pct missing");
        assert!(data["breakdown"].is_array(), "breakdown must be array");
    }
}
