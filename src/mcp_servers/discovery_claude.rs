use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;

use super::{McpServerEntry, RedactedValue, RuntimeState, ScopeKind, Transport};
use crate::mcp_servers::redact;

/// Parse `~/.claude.json` — returns (global_entries, project_entries).
pub fn parse_claude_dotjson(path: &Path) -> (Vec<McpServerEntry>, Vec<McpServerEntry>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!("mcp_servers: cannot read {}: {e}", path.display());
            return (vec![], vec![]);
        }
    };

    let root: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("mcp_servers: failed to parse {}: {e}", path.display());
            return (vec![], vec![]);
        }
    };

    let mut global = Vec::new();
    let mut project_entries = Vec::new();

    // Top-level mcpServers
    if let Some(servers) = root.get("mcpServers").and_then(|v| v.as_object()) {
        for (name, val) in servers {
            if let Some(entry) = parse_server_entry(
                name,
                val,
                ScopeKind::ClaudeUserGlobal,
                path,
                None,
                None,
            ) {
                global.push(entry);
            }
        }
    }

    // projects.<key>.mcpServers
    if let Some(projects) = root.get("projects").and_then(|v| v.as_object()) {
        for (project_key, project_val) in projects {
            let label = std::path::Path::new(project_key)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .or_else(|| Some(project_key.clone()));

            if let Some(servers) = project_val.get("mcpServers").and_then(|v| v.as_object()) {
                for (name, val) in servers {
                    if let Some(entry) = parse_server_entry(
                        name,
                        val,
                        ScopeKind::ClaudeProject,
                        path,
                        label.clone(),
                        None,
                    ) {
                        project_entries.push(entry);
                    }
                }
            }
        }
    }

    (global, project_entries)
}

/// Parse `~/.claude/.mcp.json` → ClaudeUserGlobalAlt entries.
pub fn parse_claude_mcp_json(path: &Path) -> Vec<McpServerEntry> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!("mcp_servers: cannot read {}: {e}", path.display());
            return vec![];
        }
    };

    let root: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("mcp_servers: failed to parse {}: {e}", path.display());
            return vec![];
        }
    };

    // Detect heimdall managed marker
    let managed_by: Option<&'static str> = if root.get("_heimdall_mcp_version").is_some() {
        Some("heimdall")
    } else {
        None
    };

    let mut entries = Vec::new();
    if let Some(servers) = root.get("mcpServers").and_then(|v| v.as_object()) {
        for (name, val) in servers {
            if let Some(entry) = parse_server_entry(
                name,
                val,
                ScopeKind::ClaudeUserGlobalAlt,
                path,
                None,
                managed_by,
            ) {
                entries.push(entry);
            }
        }
    }

    entries
}

/// Parse `<root>/.mcp.json` → ClaudeProject entries.
pub fn parse_project_mcp_json(root: &Path, project_label: Option<&str>) -> Vec<McpServerEntry> {
    let path = root.join(".mcp.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!("mcp_servers: cannot read {}: {e}", path.display());
            return vec![];
        }
    };

    let doc: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("mcp_servers: failed to parse {}: {e}", path.display());
            return vec![];
        }
    };

    let mut entries = Vec::new();
    if let Some(servers) = doc.get("mcpServers").and_then(|v| v.as_object()) {
        for (name, val) in servers {
            if let Some(entry) = parse_server_entry(
                name,
                val,
                ScopeKind::ClaudeProject,
                &path,
                project_label.map(|s| s.to_string()),
                None,
            ) {
                entries.push(entry);
            }
        }
    }

    entries
}

/// Parse a single server entry from a JSON value.
pub fn parse_server_entry(
    name: &str,
    val: &Value,
    scope: ScopeKind,
    source_path: &Path,
    project_label: Option<String>,
    managed_by: Option<&'static str>,
) -> Option<McpServerEntry> {
    let transport_type = val
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("stdio");

    let transport = match transport_type {
        "http" => {
            let url = val
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Transport::Http {
                url: redact::redact_url_for_display(&url),
            }
        }
        "sse" => {
            let url = val
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Transport::Sse {
                url: redact::redact_url_for_display(&url),
            }
        }
        _ => {
            // stdio (default)
            let command = val
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let args: Vec<String> = val
                .get("args")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|a| a.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();
            Transport::Stdio { command, args }
        }
    };

    // Parse env
    let mut env: BTreeMap<String, RedactedValue> = BTreeMap::new();
    if let Some(env_obj) = val.get("env").and_then(|v| v.as_object()) {
        for (k, v) in env_obj {
            let value_str = match v {
                Value::String(s) => s.as_str(),
                _ => {
                    tracing::debug!("mcp_servers: env value for {k} is not a string, skipping");
                    continue;
                }
            };
            env.insert(k.clone(), redact::redact_env_value(k, value_str));
        }
    }

    let provider = "claude";

    Some(McpServerEntry {
        name: name.to_string(),
        provider,
        scope,
        project_label,
        source_path: source_path.to_path_buf(),
        managed_by,
        transport,
        env,
        runtime: RuntimeState::NotRunning,
        log_probe: None,
        usage: None,
        is_dormant: false,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn write_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let p = dir.path().join(name);
        fs::write(&p, content).unwrap();
        p
    }

    #[test]
    fn parse_global_mcp_servers() {
        let dir = TempDir::new().unwrap();
        let content = serde_json::json!({
            "mcpServers": {
                "my-server": {
                    "command": "npx",
                    "args": ["-y", "my-mcp"],
                    "env": { "API_KEY": "secret123456789", "PORT": "3000" }
                },
                "http-server": {
                    "type": "http",
                    "url": "https://api.example.com/mcp?token=abc"
                }
            }
        })
        .to_string();
        let path = write_file(&dir, ".claude.json", &content);

        let (global, projects) = parse_claude_dotjson(&path);
        assert_eq!(global.len(), 2);
        assert!(projects.is_empty());

        let stdio = global.iter().find(|e| e.name == "my-server").unwrap();
        assert!(matches!(&stdio.transport, Transport::Stdio { command, .. } if command == "npx"));
        // Secret key should be masked
        let api_key = stdio.env.get("API_KEY").unwrap();
        assert!(matches!(api_key, RedactedValue::Secret { .. }));
        // PORT should be plain
        let port = stdio.env.get("PORT").unwrap();
        assert!(matches!(port, RedactedValue::Plain { .. }));

        let http = global.iter().find(|e| e.name == "http-server").unwrap();
        // URL query stripped
        if let Transport::Http { url } = &http.transport {
            assert!(!url.contains("token="));
        } else {
            panic!("expected Http transport");
        }
    }

    #[test]
    fn parse_project_mcp_servers_from_dotjson() {
        let dir = TempDir::new().unwrap();
        let content = serde_json::json!({
            "projects": {
                "/home/user/myproject": {
                    "mcpServers": {
                        "proj-server": {
                            "command": "node",
                            "args": ["server.js"]
                        }
                    }
                }
            }
        })
        .to_string();
        let path = write_file(&dir, ".claude.json", &content);

        let (global, projects) = parse_claude_dotjson(&path);
        assert!(global.is_empty());
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "proj-server");
        assert_eq!(projects[0].project_label.as_deref(), Some("myproject"));
    }

    #[test]
    fn parse_mcp_json_heimdall_managed() {
        let dir = TempDir::new().unwrap();
        let content = serde_json::json!({
            "_heimdall_mcp_version": "0.1.0",
            "mcpServers": {
                "heimdall": {
                    "command": "heimdall",
                    "args": ["mcp", "serve"]
                }
            }
        })
        .to_string();
        let path = write_file(&dir, ".mcp.json", &content);

        let entries = parse_claude_mcp_json(&path);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].managed_by, Some("heimdall"));
        assert!(matches!(entries[0].scope, ScopeKind::ClaudeUserGlobalAlt));
    }

    #[test]
    fn parse_project_mcp_json_file() {
        let dir = TempDir::new().unwrap();
        let content = serde_json::json!({
            "mcpServers": {
                "local-server": {
                    "command": "python",
                    "args": ["-m", "my_mcp"]
                }
            }
        })
        .to_string();
        fs::write(dir.path().join(".mcp.json"), &content).unwrap();

        let entries = parse_project_mcp_json(dir.path(), Some("myproject"));
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "local-server");
        assert_eq!(entries[0].project_label.as_deref(), Some("myproject"));
        assert!(matches!(entries[0].scope, ScopeKind::ClaudeProject));
    }

    #[test]
    fn missing_file_returns_empty() {
        let dir = TempDir::new().unwrap();
        let nonexistent = dir.path().join("nonexistent.json");
        let (g, p) = parse_claude_dotjson(&nonexistent);
        assert!(g.is_empty());
        assert!(p.is_empty());

        let entries = parse_claude_mcp_json(&nonexistent);
        assert!(entries.is_empty());
    }
}
