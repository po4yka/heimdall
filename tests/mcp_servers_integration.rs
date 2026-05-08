use std::fs;

use tempfile::TempDir;

use claude_usage_tracker::mcp_servers::{ScanOptions, scan};

fn make_claude_dotjson(dir: &TempDir) -> std::path::PathBuf {
    let path = dir.path().join("claude.json");
    fs::write(
        &path,
        r#"{
  "mcpServers": {
    "github": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": { "GITHUB_TOKEN": "ghp_real_secret_value" }
    },
    "web-search": {
      "type": "http",
      "url": "https://search.example.com/mcp?api_key=sk-real-value"
    }
  }
}"#,
    )
    .unwrap();
    path
}

fn make_codex_home(dir: &TempDir) {
    let config_path = dir.path().join("config.toml");
    fs::write(
        &config_path,
        r#"
[mcp_servers.code-search]
command = "node"
args = ["./mcp-server.js"]

[mcp_servers.docs]
url = "https://docs.example.com/mcp"
"#,
    )
    .unwrap();
}

#[test]
fn full_scan_configured_count() {
    let tmp = TempDir::new().unwrap();
    let claude_dotjson = make_claude_dotjson(&tmp);
    let claude_home_tmp = TempDir::new().unwrap(); // empty — no .mcp.json here
    let codex_tmp = TempDir::new().unwrap();
    make_codex_home(&codex_tmp);

    let opts = ScanOptions {
        include_claude_global: true,
        include_claude_projects: false,
        include_codex_global: true,
        claude_dotjson_override: Some(claude_dotjson),
        claude_home_override: Some(claude_home_tmp.path().to_path_buf()),
        codex_home_override: Some(codex_tmp.path().to_path_buf()),
        probe_processes: false,
        probe_logs: false,
        ..Default::default()
    };

    let report = scan(opts).unwrap();

    // 2 Claude global + 2 Codex
    assert_eq!(report.totals.configured_count, 4, "expected 4 configured servers");
    assert_eq!(report.claude.len(), 2);
    assert_eq!(report.codex.len(), 2);
}

#[test]
fn secrets_are_redacted_in_env() {
    let tmp = TempDir::new().unwrap();
    let claude_dotjson = make_claude_dotjson(&tmp);
    let claude_home_tmp = TempDir::new().unwrap();

    let opts = ScanOptions {
        include_claude_global: true,
        include_claude_projects: false,
        include_codex_global: false,
        claude_dotjson_override: Some(claude_dotjson),
        claude_home_override: Some(claude_home_tmp.path().to_path_buf()),
        probe_processes: false,
        probe_logs: false,
        ..Default::default()
    };

    let report = scan(opts).unwrap();
    let github = report.claude.iter().find(|e| e.name == "github").unwrap();

    // Env must contain GITHUB_TOKEN but value must NOT be the plaintext secret
    let token_val = github.env.get("GITHUB_TOKEN").expect("GITHUB_TOKEN must be in env");
    let json = serde_json::to_string(token_val).unwrap();
    assert!(!json.contains("ghp_real_secret_value"), "secret value leaked: {json}");
}

#[test]
fn http_url_has_query_stripped() {
    let tmp = TempDir::new().unwrap();
    let claude_dotjson = make_claude_dotjson(&tmp);
    let claude_home_tmp = TempDir::new().unwrap();

    let opts = ScanOptions {
        include_claude_global: true,
        include_claude_projects: false,
        include_codex_global: false,
        claude_dotjson_override: Some(claude_dotjson),
        claude_home_override: Some(claude_home_tmp.path().to_path_buf()),
        probe_processes: false,
        probe_logs: false,
        ..Default::default()
    };

    let report = scan(opts).unwrap();
    let web = report.claude.iter().find(|e| e.name == "web-search").unwrap();

    let url = match &web.transport {
        claude_usage_tracker::mcp_servers::Transport::Http { url } => url,
        other => panic!("expected http transport, got {:?}", other),
    };
    assert!(!url.contains("api_key"), "query string with secret not stripped: {url}");
    assert!(url.contains("search.example.com"), "host should be retained");
}

#[test]
fn same_name_in_different_scopes_not_deduped() {
    let tmp = TempDir::new().unwrap();
    // global entry
    let claude_dotjson = tmp.path().join("claude.json");
    let project_root = TempDir::new().unwrap();
    let project_mcp = project_root.path().join(".mcp.json");

    fs::write(
        &claude_dotjson,
        r#"{"mcpServers": {"shared": {"type": "stdio", "command": "node", "args": []}}}"#,
    )
    .unwrap();
    fs::write(
        &project_mcp,
        r#"{"mcpServers": {"shared": {"type": "stdio", "command": "deno", "args": []}}}"#,
    )
    .unwrap();

    let claude_home_tmp = TempDir::new().unwrap();
    let opts = ScanOptions {
        include_claude_global: true,
        include_claude_projects: true,
        include_codex_global: false,
        claude_dotjson_override: Some(claude_dotjson),
        claude_home_override: Some(claude_home_tmp.path().to_path_buf()),
        project_paths: vec![project_root.path().to_path_buf()],
        probe_processes: false,
        probe_logs: false,
        ..Default::default()
    };

    let report = scan(opts).unwrap();
    let shared: Vec<_> = report.claude.iter().filter(|e| e.name == "shared").collect();
    assert_eq!(shared.len(), 2, "same name in global+project must appear twice (no dedup)");
}

#[test]
fn generated_at_is_rfc3339() {
    let tmp = TempDir::new().unwrap();
    let opts = ScanOptions {
        include_claude_global: false,
        include_claude_projects: false,
        include_codex_global: false,
        probe_processes: false,
        probe_logs: false,
        claude_dotjson_override: Some(tmp.path().join("missing.json")),
        ..Default::default()
    };
    let report = scan(opts).unwrap();
    let parsed = chrono::DateTime::parse_from_rfc3339(&report.generated_at);
    assert!(parsed.is_ok(), "generated_at not RFC3339: {}", report.generated_at);
}
