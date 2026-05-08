use std::collections::HashMap;

use anyhow::Result;
use rusqlite::Connection;

use super::McpUsageStats;

/// Fetch per-server usage statistics from the `tool_invocations` table.
pub fn fetch_usage_stats(conn: &Connection) -> Result<HashMap<String, McpUsageStats>> {
    let mut stmt = conn.prepare(
        "SELECT lower(mcp_server) AS server,
                COUNT(*) AS total_calls,
                MAX(timestamp) AS last_used,
                COUNT(DISTINCT session_id) AS distinct_sessions,
                COUNT(DISTINCT mcp_tool) AS distinct_tools
         FROM tool_invocations
         WHERE mcp_server IS NOT NULL
         GROUP BY lower(mcp_server)",
    )?;

    let rows = stmt.query_map([], |row| {
        let server: String = row.get(0)?;
        let total_calls: u64 = row.get::<_, i64>(1)? as u64;
        let last_used: Option<String> = row.get(2)?;
        let distinct_sessions: u64 = row.get::<_, i64>(3)? as u64;
        let distinct_tools: u64 = row.get::<_, i64>(4)? as u64;
        Ok((
            server,
            McpUsageStats {
                total_calls,
                last_used,
                distinct_sessions,
                distinct_tools,
            },
        ))
    })?;

    let mut map = HashMap::new();
    for row in rows {
        match row {
            Ok((server, stats)) => {
                map.insert(server, stats);
            }
            Err(e) => {
                tracing::warn!("mcp_servers: usage stats row error: {e}");
            }
        }
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE tool_invocations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                provider TEXT NOT NULL DEFAULT 'claude',
                message_id TEXT,
                tool_name TEXT NOT NULL,
                mcp_server TEXT,
                mcp_tool TEXT,
                tool_category TEXT NOT NULL DEFAULT 'builtin',
                source_path TEXT NOT NULL DEFAULT '',
                timestamp TEXT NOT NULL DEFAULT '',
                tool_use_id TEXT,
                is_error INTEGER DEFAULT 0,
                error_text TEXT
            );",
        )
        .unwrap();
        conn
    }

    fn insert_row(
        conn: &Connection,
        session_id: &str,
        mcp_server: Option<&str>,
        mcp_tool: Option<&str>,
        timestamp: &str,
    ) {
        conn.execute(
            "INSERT INTO tool_invocations (session_id, tool_name, mcp_server, mcp_tool, timestamp)
             VALUES (?1, 'tool', ?2, ?3, ?4)",
            rusqlite::params![session_id, mcp_server, mcp_tool, timestamp],
        )
        .unwrap();
    }

    #[test]
    fn fetch_aggregates_correctly() {
        let conn = setup_db();

        // 3 calls for "my-server" across 2 sessions, 2 distinct tools
        insert_row(
            &conn,
            "sess1",
            Some("my-server"),
            Some("tool-a"),
            "2026-01-01T00:00:00Z",
        );
        insert_row(
            &conn,
            "sess1",
            Some("my-server"),
            Some("tool-b"),
            "2026-01-02T00:00:00Z",
        );
        insert_row(
            &conn,
            "sess2",
            Some("my-server"),
            Some("tool-a"),
            "2026-01-03T00:00:00Z",
        );

        // 1 call for "other-server"
        insert_row(
            &conn,
            "sess1",
            Some("other-server"),
            Some("tool-x"),
            "2026-01-01T00:00:00Z",
        );

        // row with no mcp_server (should be excluded)
        insert_row(&conn, "sess1", None, None, "2026-01-01T00:00:00Z");

        let stats = fetch_usage_stats(&conn).unwrap();

        assert_eq!(stats.len(), 2);

        let ms = stats.get("my-server").unwrap();
        assert_eq!(ms.total_calls, 3);
        assert_eq!(ms.distinct_sessions, 2);
        assert_eq!(ms.distinct_tools, 2);
        assert_eq!(ms.last_used.as_deref(), Some("2026-01-03T00:00:00Z"));

        let os = stats.get("other-server").unwrap();
        assert_eq!(os.total_calls, 1);
        assert_eq!(os.distinct_sessions, 1);
        assert_eq!(os.distinct_tools, 1);
    }

    #[test]
    fn keys_are_lowercased() {
        let conn = setup_db();
        insert_row(
            &conn,
            "s1",
            Some("My-Server"),
            Some("t"),
            "2026-01-01T00:00:00Z",
        );
        let stats = fetch_usage_stats(&conn).unwrap();
        // Key should be lowercased
        assert!(stats.contains_key("my-server"));
        assert!(!stats.contains_key("My-Server"));
    }

    #[test]
    fn empty_table_returns_empty_map() {
        let conn = setup_db();
        let stats = fetch_usage_stats(&conn).unwrap();
        assert!(stats.is_empty());
    }
}
