use std::collections::HashMap;

use anyhow::Result;
use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SkillInvocationStats {
    pub total_calls: u64,
    pub last_used: Option<String>,
    pub distinct_sessions: u64,
}

/// Fetch per-skill invocation statistics from the `tool_invocations` table.
/// Claude Code records Skill tool calls with `tool_name = 'Skill'` and the
/// skill name in `tool_input_json` as `{"skill": "<name>", ...}`.
pub fn fetch_skill_invocation_stats(
    conn: &Connection,
) -> Result<HashMap<String, SkillInvocationStats>> {
    let mut stmt = conn.prepare(
        "SELECT lower(json_extract(tool_input_json, '$.skill')) AS skill_name,
                COUNT(*) AS total_calls,
                MAX(timestamp) AS last_used,
                COUNT(DISTINCT session_id) AS distinct_sessions
         FROM tool_invocations
         WHERE tool_name = 'Skill'
           AND tool_input_json IS NOT NULL
           AND json_extract(tool_input_json, '$.skill') IS NOT NULL
         GROUP BY lower(json_extract(tool_input_json, '$.skill'))",
    )?;

    let rows = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let total_calls: u64 = row.get::<_, i64>(1)? as u64;
        let last_used: Option<String> = row.get(2)?;
        let distinct_sessions: u64 = row.get::<_, i64>(3)? as u64;
        Ok((
            name,
            SkillInvocationStats {
                total_calls,
                last_used,
                distinct_sessions,
            },
        ))
    })?;

    let mut map = HashMap::new();
    for row in rows {
        match row {
            Ok((name, stats)) => {
                map.insert(name, stats);
            }
            Err(e) => {
                tracing::warn!("skills: usage stats row error: {e}");
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
                tool_name TEXT NOT NULL,
                tool_input_json TEXT,
                timestamp TEXT NOT NULL DEFAULT '',
                mcp_server TEXT,
                mcp_tool TEXT,
                tool_category TEXT NOT NULL DEFAULT 'builtin',
                source_path TEXT NOT NULL DEFAULT '',
                tool_use_id TEXT,
                is_error INTEGER DEFAULT 0,
                error_text TEXT
            );",
        )
        .unwrap();
        conn
    }

    fn insert_skill(conn: &Connection, session_id: &str, skill: &str, ts: &str) {
        let json = format!(r#"{{"skill": "{skill}", "args": ""}}"#);
        conn.execute(
            "INSERT INTO tool_invocations (session_id, tool_name, tool_input_json, timestamp)
             VALUES (?1, 'Skill', ?2, ?3)",
            rusqlite::params![session_id, json, ts],
        )
        .unwrap();
    }

    #[test]
    fn aggregates_skill_calls() {
        let conn = setup_db();
        insert_skill(&conn, "s1", "my-skill", "2026-01-01T00:00:00Z");
        insert_skill(&conn, "s1", "my-skill", "2026-01-02T00:00:00Z");
        insert_skill(&conn, "s2", "my-skill", "2026-01-03T00:00:00Z");
        insert_skill(&conn, "s1", "other-skill", "2026-01-01T00:00:00Z");

        let stats = fetch_skill_invocation_stats(&conn).unwrap();
        assert_eq!(stats.len(), 2);

        let ms = stats.get("my-skill").unwrap();
        assert_eq!(ms.total_calls, 3);
        assert_eq!(ms.distinct_sessions, 2);
        assert_eq!(ms.last_used.as_deref(), Some("2026-01-03T00:00:00Z"));
    }

    #[test]
    fn keys_are_lowercased() {
        let conn = setup_db();
        insert_skill(&conn, "s1", "My-Skill", "2026-01-01T00:00:00Z");
        let stats = fetch_skill_invocation_stats(&conn).unwrap();
        assert!(stats.contains_key("my-skill"));
    }

    #[test]
    fn non_skill_tools_excluded() {
        let conn = setup_db();
        // Insert a non-Skill tool call
        conn.execute(
            "INSERT INTO tool_invocations (session_id, tool_name, tool_input_json, timestamp)
             VALUES ('s1', 'Bash', '{\"command\":\"ls\"}', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        let stats = fetch_skill_invocation_stats(&conn).unwrap();
        assert!(stats.is_empty());
    }

    #[test]
    fn empty_table_returns_empty_map() {
        let conn = setup_db();
        let stats = fetch_skill_invocation_stats(&conn).unwrap();
        assert!(stats.is_empty());
    }
}
