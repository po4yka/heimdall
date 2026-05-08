pub mod discovery_claude;
pub mod discovery_codex;
pub mod logs;
pub mod process;
pub mod redact;
pub mod usage;

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;
use serde::Serialize;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    ClaudeUserGlobal,
    ClaudeUserGlobalAlt,
    ClaudeProject,
    CodexUserGlobal,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Transport {
    Stdio { command: String, args: Vec<String> },
    Http { url: String },
    Sse { url: String },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuntimeState {
    Running {
        pid: u32,
        parent_pid: Option<u32>,
        started_at: Option<String>,
        cpu_percent: f32,
        memory_bytes: u64,
    },
    NotRunning,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RedactedValue {
    Plain {
        value: String,
    },
    Secret {
        masked: String,
    },
    EnvFromFile {
        path: String,
        exists: bool,
        bytes: u64,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct LogProbe {
    pub path: String,
    pub bytes: u64,
    pub modified: String,
    pub recent_line_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpUsageStats {
    pub total_calls: u64,
    pub last_used: Option<String>,
    pub distinct_sessions: u64,
    pub distinct_tools: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpServerEntry {
    pub name: String,
    pub provider: &'static str,
    pub scope: ScopeKind,
    pub project_label: Option<String>,
    pub source_path: PathBuf,
    pub managed_by: Option<&'static str>,
    pub transport: Transport,
    pub env: BTreeMap<String, RedactedValue>,
    pub runtime: RuntimeState,
    pub log_probe: Option<LogProbe>,
    pub usage: Option<McpUsageStats>,
    pub is_dormant: bool,
}

impl McpServerEntry {
    pub fn scope_kind_label(&self) -> &'static str {
        match self.scope {
            ScopeKind::ClaudeUserGlobal => "(global)",
            ScopeKind::ClaudeUserGlobalAlt => "(global-alt)",
            ScopeKind::ClaudeProject => "(project)",
            ScopeKind::CodexUserGlobal => "(global)",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct McpServerTotals {
    pub configured_count: usize,
    pub running_count: usize,
    pub never_invoked_count: usize,
    pub claude_count: usize,
    pub codex_count: usize,
    pub project_count: usize,
    pub dormant_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpServerReport {
    pub generated_at: String,
    pub claude: Vec<McpServerEntry>,
    pub codex: Vec<McpServerEntry>,
    pub totals: McpServerTotals,
}

// ---------------------------------------------------------------------------
// Scan options
// ---------------------------------------------------------------------------

pub struct ScanOptions {
    pub include_claude_global: bool,
    pub include_claude_projects: bool,
    pub include_codex_global: bool,
    /// Explicit project roots; if non-empty, replaces DB discovery.
    pub project_paths: Vec<PathBuf>,
    /// Override `~/.claude/` for tests.
    pub claude_home_override: Option<PathBuf>,
    /// Override `~/.codex/` for tests.
    pub codex_home_override: Option<PathBuf>,
    /// Override `~/.claude.json` for tests.
    pub claude_dotjson_override: Option<PathBuf>,
    /// Override Claude Desktop log directory for tests.
    pub claude_desktop_log_dir_override: Option<PathBuf>,
    pub probe_processes: bool,
    pub probe_logs: bool,
    pub db_path: Option<PathBuf>,
    /// Days without invocation before a server is considered dormant (default 30).
    pub dormant_threshold_days: u32,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            include_claude_global: true,
            include_claude_projects: true,
            include_codex_global: true,
            project_paths: vec![],
            claude_home_override: None,
            codex_home_override: None,
            claude_dotjson_override: None,
            claude_desktop_log_dir_override: None,
            probe_processes: true,
            probe_logs: true,
            db_path: None,
            dormant_threshold_days: 30,
        }
    }
}

// ---------------------------------------------------------------------------
// scan()
// ---------------------------------------------------------------------------

/// Run a full MCP server inventory and return the report.
pub fn scan(opts: ScanOptions) -> Result<McpServerReport> {
    let claude_home = opts
        .claude_home_override
        .clone()
        .or_else(|| dirs::home_dir().map(|h| h.join(".claude")));

    let codex_home = opts
        .codex_home_override
        .clone()
        .or_else(|| dirs::home_dir().map(|h| h.join(".codex")));

    let claude_dotjson = opts
        .claude_dotjson_override
        .clone()
        .or_else(|| dirs::home_dir().map(|h| h.join(".claude.json")));

    let mut claude_entries: Vec<McpServerEntry> = Vec::new();
    let mut codex_entries: Vec<McpServerEntry> = Vec::new();

    // --- Claude ~/.claude.json (global + projects) ---
    if opts.include_claude_global
        && let Some(ref p) = claude_dotjson
    {
        let (global, project_es) = discovery_claude::parse_claude_dotjson(p);
        claude_entries.extend(global);
        if opts.include_claude_projects {
            claude_entries.extend(project_es);
        }
    }

    // --- Claude ~/.claude/.mcp.json ---
    if opts.include_claude_global
        && let Some(ref home) = claude_home
    {
        let alt = home.join(".mcp.json");
        claude_entries.extend(discovery_claude::parse_claude_mcp_json(&alt));
    }

    // --- Per-project <root>/.mcp.json ---
    if opts.include_claude_projects {
        let roots = resolve_project_paths(&opts);
        for root in &roots {
            let label = root
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string());
            claude_entries.extend(discovery_claude::parse_project_mcp_json(
                root,
                label.as_deref(),
            ));
        }
    }

    // --- Codex ~/.codex/config.toml ---
    if opts.include_codex_global
        && let Some(ref home) = codex_home
    {
        codex_entries.extend(discovery_codex::parse_codex_config_toml(
            &home.join("config.toml"),
        ));
    }

    // --- Usage join ---
    let usage_map = if let Some(ref db_path) = opts.db_path {
        if db_path.exists() {
            match rusqlite::Connection::open(db_path) {
                Ok(conn) => usage::fetch_usage_stats(&conn).unwrap_or_default(),
                Err(e) => {
                    tracing::warn!("mcp_servers: cannot open db: {e}");
                    Default::default()
                }
            }
        } else {
            Default::default()
        }
    } else {
        Default::default()
    };

    // Join usage into entries and compute dormancy
    let dormant_days = opts.dormant_threshold_days;
    let threshold_dt = chrono::Utc::now() - chrono::Duration::days(dormant_days as i64);

    for entry in claude_entries.iter_mut().chain(codex_entries.iter_mut()) {
        let key = entry.name.to_lowercase();
        if let Some(stats) = usage_map.get(&key) {
            entry.usage = Some(stats.clone());
        }
        entry.is_dormant = match &entry.usage {
            None => true,
            Some(u) => u
                .last_used
                .as_deref()
                .and_then(|ts| chrono::DateTime::parse_from_rfc3339(ts).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc) < threshold_dt)
                .unwrap_or(true),
        };
    }

    // --- Log probe ---
    if opts.probe_logs {
        let log_dir = opts.claude_desktop_log_dir_override.clone().or_else(|| {
            #[cfg(target_os = "macos")]
            {
                dirs::home_dir().map(|h| h.join("Library/Logs/Claude"))
            }
            #[cfg(not(target_os = "macos"))]
            {
                None
            }
        });
        if let Some(ref dir) = log_dir {
            for entry in claude_entries.iter_mut().chain(codex_entries.iter_mut()) {
                entry.log_probe = logs::probe_log(dir, &entry.name);
            }
        }
    }

    // --- Live process matching ---
    if opts.probe_processes {
        let sys = sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::new().with_processes(sysinfo::ProcessRefreshKind::new()),
        );
        for entry in claude_entries.iter_mut().chain(codex_entries.iter_mut()) {
            process::match_runtime(entry, &sys);
        }
    }

    // --- Totals ---
    let project_count = resolve_project_paths(&opts).len();
    let running_count = claude_entries
        .iter()
        .chain(codex_entries.iter())
        .filter(|e| matches!(e.runtime, RuntimeState::Running { .. }))
        .count();
    let never_invoked = claude_entries
        .iter()
        .chain(codex_entries.iter())
        .filter(|e| e.usage.is_none())
        .count();
    let configured_count = claude_entries.len() + codex_entries.len();
    let dormant_count = claude_entries
        .iter()
        .chain(codex_entries.iter())
        .filter(|e| e.is_dormant)
        .count();

    Ok(McpServerReport {
        generated_at: Utc::now().to_rfc3339(),
        totals: McpServerTotals {
            configured_count,
            running_count,
            never_invoked_count: never_invoked,
            claude_count: claude_entries.len(),
            codex_count: codex_entries.len(),
            project_count,
            dormant_count,
        },
        claude: claude_entries,
        codex: codex_entries,
    })
}

fn resolve_project_paths(opts: &ScanOptions) -> Vec<PathBuf> {
    if !opts.project_paths.is_empty() {
        return opts
            .project_paths
            .iter()
            .filter(|p| p.is_dir())
            .cloned()
            .collect();
    }

    let db_path = match &opts.db_path {
        Some(p) => p.clone(),
        None => return vec![],
    };

    if !db_path.exists() {
        return vec![];
    }

    match rusqlite::Connection::open(&db_path) {
        Ok(conn) => crate::skills::projects::discover_project_paths(&conn, &[]),
        Err(e) => {
            tracing::warn!("mcp_servers: db open failed: {e}");
            vec![]
        }
    }
}
