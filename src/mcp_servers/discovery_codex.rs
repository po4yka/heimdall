use std::collections::BTreeMap;
use std::path::Path;

use super::{McpServerEntry, RedactedValue, RuntimeState, ScopeKind, Transport};
use crate::mcp_servers::redact;

const OMC_BLOCK_START: &str = "# BEGIN OMC MANAGED MCP REGISTRY";
const OMC_BLOCK_END: &str = "# END OMC MANAGED MCP REGISTRY";

/// Parse `~/.codex/config.toml` → Codex MCP server entries.
pub fn parse_codex_config_toml(path: &Path) -> Vec<McpServerEntry> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!("mcp_servers: cannot read {}: {e}", path.display());
            return vec![];
        }
    };

    let doc: toml::Value = match toml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("mcp_servers: failed to parse {}: {e}", path.display());
            return vec![];
        }
    };

    // Determine which server names are inside the OMC managed block
    let omc_names = find_omc_managed_names(&content);

    let servers = match doc.get("mcp_servers").and_then(|v| v.as_table()) {
        Some(t) => t,
        None => return vec![],
    };

    let mut entries = Vec::new();

    for (name, val) in servers {
        let managed_by: Option<&'static str> = if omc_names.contains(name.as_str()) {
            Some("omc")
        } else {
            None
        };

        let Some(server_table) = val.as_table() else {
            tracing::warn!("mcp_servers: codex server {name} is not a table");
            continue;
        };

        let transport = build_transport(name, server_table);

        let mut env: BTreeMap<String, RedactedValue> = BTreeMap::new();
        if let Some(env_table) = server_table.get("env").and_then(|v| v.as_table()) {
            for (k, v) in env_table {
                let value_str = match v.as_str() {
                    Some(s) => s,
                    None => {
                        tracing::debug!("mcp_servers: codex env value for {k} is not a string");
                        continue;
                    }
                };
                env.insert(k.clone(), redact::redact_env_value(k, value_str));
            }
        }

        entries.push(McpServerEntry {
            name: name.clone(),
            provider: "codex",
            scope: ScopeKind::CodexUserGlobal,
            project_label: None,
            source_path: path.to_path_buf(),
            managed_by,
            transport,
            env,
            runtime: RuntimeState::NotRunning,
            log_probe: None,
            usage: None,
            is_dormant: false,
        });
    }

    entries
}

fn build_transport(name: &str, table: &toml::map::Map<String, toml::Value>) -> Transport {
    // If url is present, determine http vs sse via optional `type` field
    if let Some(url) = table.get("url").and_then(|v| v.as_str()) {
        let transport_type = table
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("http");
        let clean_url = redact::redact_url_for_display(url);
        if transport_type == "sse" {
            return Transport::Sse { url: clean_url };
        }
        return Transport::Http { url: clean_url };
    }

    // Stdio
    let command = table
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
            tracing::warn!("mcp_servers: codex server {name} has no command or url");
            ""
        })
        .to_string();

    let args: Vec<String> = table
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

/// Scan the raw TOML text for OMC-managed block boundaries and return
/// the set of server names declared within.
fn find_omc_managed_names(content: &str) -> std::collections::HashSet<&str> {
    let mut in_block = false;
    let mut names = std::collections::HashSet::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == OMC_BLOCK_START {
            in_block = true;
            continue;
        }
        if trimmed == OMC_BLOCK_END {
            in_block = false;
            continue;
        }
        if in_block {
            // Look for `[mcp_servers.<name>]` table header lines
            if let Some(inner) = trimmed
                .strip_prefix("[mcp_servers.")
                .and_then(|s| s.strip_suffix(']'))
            {
                names.insert(inner);
            }
        }
    }

    names
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn write_toml(dir: &TempDir, content: &str) -> std::path::PathBuf {
        let p = dir.path().join("config.toml");
        std::fs::write(&p, content).unwrap();
        p
    }

    const SAMPLE_TOML: &str = r#"
[mcp_servers.outside-server]
command = "npx"
args = ["-y", "outside-mcp"]

# BEGIN OMC MANAGED MCP REGISTRY
[mcp_servers.omc-server-one]
command = "node"
args = ["server.js"]

[mcp_servers.omc-server-two]
url = "https://api.example.com/mcp?token=abc"
type = "sse"
# END OMC MANAGED MCP REGISTRY
"#;

    #[test]
    fn parses_three_servers() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(&dir, SAMPLE_TOML);
        let entries = parse_codex_config_toml(&path);
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn outside_server_not_managed() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(&dir, SAMPLE_TOML);
        let entries = parse_codex_config_toml(&path);
        let outside = entries.iter().find(|e| e.name == "outside-server").unwrap();
        assert_eq!(outside.managed_by, None);
    }

    #[test]
    fn omc_servers_marked_managed() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(&dir, SAMPLE_TOML);
        let entries = parse_codex_config_toml(&path);

        let one = entries.iter().find(|e| e.name == "omc-server-one").unwrap();
        assert_eq!(one.managed_by, Some("omc"));

        let two = entries.iter().find(|e| e.name == "omc-server-two").unwrap();
        assert_eq!(two.managed_by, Some("omc"));
    }

    #[test]
    fn omc_server_two_is_sse_transport() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(&dir, SAMPLE_TOML);
        let entries = parse_codex_config_toml(&path);
        let two = entries.iter().find(|e| e.name == "omc-server-two").unwrap();
        assert!(
            matches!(&two.transport, Transport::Sse { url } if !url.contains("token=")),
            "expected Sse transport with stripped URL"
        );
    }

    #[test]
    fn missing_file_returns_empty() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.toml");
        let entries = parse_codex_config_toml(&path);
        assert!(entries.is_empty());
    }

    #[test]
    fn env_secrets_redacted() {
        let content = r#"
[mcp_servers.secret-server]
command = "node"
[mcp_servers.secret-server.env]
API_KEY = "supersecretvalue1234"
HOST = "localhost"
"#;
        let dir = TempDir::new().unwrap();
        let path = write_toml(&dir, content);
        let entries = parse_codex_config_toml(&path);
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert!(matches!(e.env.get("API_KEY"), Some(RedactedValue::Secret { .. })));
        assert!(matches!(e.env.get("HOST"), Some(RedactedValue::Plain { .. })));
    }

    #[test]
    fn all_entries_have_codex_provider_and_global_scope() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(&dir, SAMPLE_TOML);
        let entries = parse_codex_config_toml(&path);
        for e in &entries {
            assert_eq!(e.provider, "codex");
            assert!(matches!(e.scope, ScopeKind::CodexUserGlobal));
        }
    }
}
