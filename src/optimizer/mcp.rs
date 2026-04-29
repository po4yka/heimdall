//! Detector: unused MCP servers.
//!
//! Reads `~/.claude/settings.json` (or an injected override for tests),
//! extracts configured MCP server names, then queries the database to find
//! servers that were never actually invoked.
//!
//! # MCP server detection in the database
//! Phase 12 stores tool events with `kind = 'mcp'` and the tool name as
//! `value`. The tool name format from Claude Code is `mcp__<server>__<tool>`,
//! but we only stored the part after the last double-underscore split.
//! As a pragmatic substitute we also query `tool_invocations.mcp_server` which
//! records the server name directly for any row where the tool was MCP-dispatched.
//!
//! If a configured server name appears in neither source, it is considered unused.
//!
//! # Severity and waste
//! MCP overhead is token-costly only when a server is invoked; a configured-but-
//! never-used server is configuration clutter rather than a direct spend driver.
//! Severity is therefore always `Low` and `estimated_monthly_waste_nanos` is 0.

use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::Result;
use rusqlite::Connection;
use serde_json::Value;

use super::{Detector, Finding, Severity};

pub struct UnusedMcpDetector {
    /// Override path for tests. If `None`, defaults to `~/.claude/settings.json`.
    path_override: Option<PathBuf>,
}

impl UnusedMcpDetector {
    pub fn new(path_override: Option<PathBuf>) -> Self {
        Self { path_override }
    }

    fn settings_path(&self) -> Option<PathBuf> {
        if let Some(p) = &self.path_override {
            return Some(p.clone());
        }
        dirs::home_dir().map(|h| h.join(".claude").join("settings.json"))
    }
}

/// Parse the set of configured MCP server names from a `settings.json` value.
///
/// Supports both top-level `mcpServers` and nested `permissions.mcp` object keys.
fn configured_servers(root: &Value) -> HashSet<String> {
    let mut servers = HashSet::new();

    // Try top-level `mcpServers` object.
    if let Some(obj) = root.get("mcpServers").and_then(Value::as_object) {
        for key in obj.keys() {
            servers.insert(key.clone());
        }
    }

    // Try `permissions.mcp` object (alternate schema).
    if let Some(obj) = root
        .get("permissions")
        .and_then(|p| p.get("mcp"))
        .and_then(Value::as_object)
    {
        for key in obj.keys() {
            servers.insert(key.clone());
        }
    }

    servers
}

/// Query the DB for the set of MCP server names that were actually invoked.
///
/// Two sources are checked:
///   1. `tool_events` where `kind = 'mcp'` — the `value` column holds the tool
///      name. We cannot recover the server name from just the tool name without
///      the original `mcp__<server>__<tool>` format, so this source is currently
///      informational only and returns an empty set (see inline comment).
///   2. `tool_invocations.mcp_server` — records the server name directly.
fn invoked_servers(conn: &Connection) -> Result<HashSet<String>> {
    let mut servers = HashSet::new();

    // Source 1: tool_invocations.mcp_server (most reliable).
    // Table may not exist in very old DBs; silently skip if absent.
    // SQL kept here for detector-local cohesion; see scanner/db.rs for shared queries.
    let ti_result = conn.prepare(
        "SELECT DISTINCT mcp_server FROM tool_invocations \
         WHERE mcp_server IS NOT NULL AND mcp_server != ''",
    );
    if let Ok(mut stmt) = ti_result {
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        for row in rows.flatten() {
            servers.insert(row);
        }
    }

    // Source 2: tool_events kind='mcp'.
    // The `value` column stores the tool name (not server name) in the current
    // Phase 12 schema. Without the full `mcp__<server>__<tool>` string we
    // cannot reliably extract the server. This path is a no-op today but is
    // left as an extension point for when `value` contains the full MCP path.
    let te_result = conn.prepare("SELECT DISTINCT value FROM tool_events WHERE kind = 'mcp'");
    if let Ok(mut stmt) = te_result {
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        for row in rows.flatten() {
            // If the value has the format `mcp__<server>__<tool>`, extract the
            // server name. Otherwise ignore — we can't map it to a server name.
            let parts: Vec<&str> = row.splitn(3, "__").collect();
            if parts.len() == 3 && parts[0] == "mcp" {
                servers.insert(parts[1].to_string());
            }
        }
    }

    Ok(servers)
}

impl Detector for UnusedMcpDetector {
    fn name(&self) -> &'static str {
        "unused_mcp"
    }

    fn run(&self, conn: &Connection) -> Result<Vec<Finding>> {
        let settings_path = match self.settings_path() {
            Some(p) => p,
            None => return Ok(vec![]),
        };

        let contents = match std::fs::read_to_string(&settings_path) {
            Ok(s) => s,
            Err(_) => return Ok(vec![]),
        };

        let root: Value = match serde_json::from_str(&contents) {
            Ok(v) => v,
            Err(_) => return Ok(vec![]),
        };

        let configured = configured_servers(&root);
        if configured.is_empty() {
            return Ok(vec![]);
        }

        let invoked = invoked_servers(conn)?;

        let mut findings = Vec::new();
        let mut unused: Vec<&String> = configured
            .iter()
            .filter(|name| !invoked.contains(*name))
            .collect();
        // Sort for deterministic output.
        unused.sort();

        for name in unused {
            findings.push(Finding {
                detector: self.name().into(),
                severity: Severity::Low,
                title: format!("MCP server '{name}' is configured but never invoked"),
                detail: format!(
                    "The MCP server '{name}' is listed in settings.json but has no recorded \
                     invocations in the database. If you no longer use this server, removing \
                     it from the configuration reduces cognitive overhead and avoids accidental \
                     invocations."
                ),
                estimated_monthly_waste_nanos: 0,
            });
        }

        Ok(findings)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::optimizer::mod_tests::{empty_db, write_temp_file};

    fn det(path: &Path) -> UnusedMcpDetector {
        UnusedMcpDetector::new(Some(path.to_path_buf()))
    }

    // -----------------------------------------------------------------------
    // No finding cases
    // -----------------------------------------------------------------------

    #[test]
    fn missing_settings_produces_no_finding() {
        let (dir, conn) = empty_db();
        let det = UnusedMcpDetector::new(Some(dir.path().join("nonexistent.json")));
        let findings = det.run(&conn).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn empty_mcp_servers_produces_no_finding() {
        let (dir, conn) = empty_db();
        let path = write_temp_file(&dir, "settings.json", r#"{"mcpServers": {}}"#);
        let findings = det(&path).run(&conn).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn invalid_json_produces_no_finding() {
        let (dir, conn) = empty_db();
        let path = write_temp_file(&dir, "settings.json", "not json {{");
        let findings = det(&path).run(&conn).unwrap();
        assert!(findings.is_empty());
    }

    // -----------------------------------------------------------------------
    // Detection: unused servers
    // -----------------------------------------------------------------------

    #[test]
    fn configured_but_not_invoked_is_reported() {
        let (dir, conn) = empty_db();
        let json = r#"{"mcpServers": {"github": {}, "filesystem": {}}}"#;
        let path = write_temp_file(&dir, "settings.json", json);
        let findings = det(&path).run(&conn).unwrap();
        assert_eq!(findings.len(), 2);
        let titles: Vec<&str> = findings.iter().map(|f| f.title.as_str()).collect();
        assert!(titles.iter().any(|t| t.contains("filesystem")));
        assert!(titles.iter().any(|t| t.contains("github")));
        assert!(findings.iter().all(|f| f.severity == Severity::Low));
        assert!(
            findings
                .iter()
                .all(|f| f.estimated_monthly_waste_nanos == 0)
        );
    }

    #[test]
    fn invoked_server_not_reported() {
        let (dir, conn) = empty_db();
        // Seed an invocation for "github" via tool_invocations.
        conn.execute(
            "INSERT INTO tool_invocations \
             (session_id, provider, tool_name, mcp_server, mcp_tool, tool_category) \
             VALUES ('claude:s1', 'claude', 'mcp__github__search', 'github', 'search', 'mcp')",
            [],
        )
        .unwrap();
        let json = r#"{"mcpServers": {"github": {}, "filesystem": {}}}"#;
        let path = write_temp_file(&dir, "settings.json", json);
        let findings = det(&path).run(&conn).unwrap();
        // Only filesystem should be reported as unused.
        assert_eq!(findings.len(), 1);
        assert!(findings[0].title.contains("filesystem"));
    }

    #[test]
    fn all_servers_invoked_no_findings() {
        let (dir, conn) = empty_db();
        for server in &["github", "filesystem"] {
            conn.execute(
                "INSERT INTO tool_invocations \
                 (session_id, provider, tool_name, mcp_server, mcp_tool, tool_category) \
                 VALUES (?1, 'claude', ?2, ?3, 'op', 'mcp')",
                rusqlite::params![
                    format!("claude:s_{server}"),
                    format!("mcp__{server}__op"),
                    server
                ],
            )
            .unwrap();
        }
        let json = r#"{"mcpServers": {"github": {}, "filesystem": {}}}"#;
        let path = write_temp_file(&dir, "settings.json", json);
        let findings = det(&path).run(&conn).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn permissions_mcp_schema_also_parsed() {
        let (dir, conn) = empty_db();
        let json = r#"{"permissions": {"mcp": {"myserver": {}}}}"#;
        let path = write_temp_file(&dir, "settings.json", json);
        let findings = det(&path).run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].title.contains("myserver"));
    }

    #[test]
    fn tool_events_mcp_kind_full_format_detected() {
        let (dir, conn) = empty_db();
        // Insert a tool_events row with the full mcp__server__tool format.
        conn.execute(
            "INSERT INTO tool_events \
             (dedup_key, ts_epoch, session_id, provider, kind, value, cost_nanos) \
             VALUES ('k1', 1700000000, 'claude:s1', 'claude', 'mcp', 'mcp__mysvr__do_thing', 0)",
            [],
        )
        .unwrap();
        let json = r#"{"mcpServers": {"mysvr": {}, "unused_svr": {}}}"#;
        let path = write_temp_file(&dir, "settings.json", json);
        let findings = det(&path).run(&conn).unwrap();
        // mysvr was invoked via tool_events; only unused_svr should be reported.
        assert_eq!(findings.len(), 1);
        assert!(findings[0].title.contains("unused_svr"));
    }
}
