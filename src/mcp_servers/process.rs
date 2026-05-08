use sysinfo::System;

use super::{McpServerEntry, RuntimeState, Transport};

/// Match a running process to an MCP server entry and update its runtime state.
pub fn match_runtime(entry: &mut McpServerEntry, sys: &System) {
    let (command, args) = match &entry.transport {
        Transport::Stdio { command, args } => (command.clone(), args.clone()),
        Transport::Http { .. } | Transport::Sse { .. } => {
            entry.runtime = RuntimeState::NotApplicable;
            return;
        }
    };

    if command.is_empty() {
        entry.runtime = RuntimeState::NotRunning;
        return;
    }

    // Resolve the command to an absolute path if possible.
    let resolved = which::which(&command)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| command.clone());

    // Search system processes for a matching command+args prefix.
    for (pid, proc) in sys.processes() {
        let cmd_vec: Vec<String> = proc
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();

        if cmd_vec.is_empty() {
            continue;
        }

        // Match if first element equals resolved path or original command name,
        // and remaining elements start with our args as prefix.
        let first_match = cmd_vec[0] == resolved
            || cmd_vec[0] == command
            || cmd_vec[0].ends_with(&format!("/{command}"));

        if !first_match {
            continue;
        }

        // Check args prefix
        let proc_args = &cmd_vec[1..];
        let our_args = &args[..];

        if our_args.len() > proc_args.len() {
            continue;
        }

        let prefix_match = our_args.iter().zip(proc_args.iter()).all(|(a, b)| a == b);

        if prefix_match {
            let parent_pid = proc.parent().map(|p| p.as_u32());
            entry.runtime = RuntimeState::Running {
                pid: pid.as_u32(),
                parent_pid,
                started_at: None,
                cpu_percent: proc.cpu_usage(),
                memory_bytes: proc.memory(),
            };
            return;
        }
    }

    entry.runtime = RuntimeState::NotRunning;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp_servers::ScopeKind;
    use std::collections::BTreeMap;

    fn make_entry(command: &str, args: Vec<&str>) -> McpServerEntry {
        McpServerEntry {
            name: "test".to_string(),
            provider: "claude",
            scope: ScopeKind::ClaudeUserGlobal,
            project_label: None,
            source_path: std::path::PathBuf::from("/fake/.claude.json"),
            managed_by: None,
            transport: Transport::Stdio {
                command: command.to_string(),
                args: args.into_iter().map(|s| s.to_string()).collect(),
            },
            env: BTreeMap::new(),
            runtime: RuntimeState::NotRunning,
            log_probe: None,
            usage: None,
            is_dormant: false,
        }
    }

    fn make_http_entry() -> McpServerEntry {
        McpServerEntry {
            name: "http-test".to_string(),
            provider: "claude",
            scope: ScopeKind::ClaudeUserGlobal,
            project_label: None,
            source_path: std::path::PathBuf::from("/fake/.claude.json"),
            managed_by: None,
            transport: Transport::Http {
                url: "https://example.com/mcp".to_string(),
            },
            env: BTreeMap::new(),
            runtime: RuntimeState::NotRunning,
            log_probe: None,
            usage: None,
            is_dormant: false,
        }
    }

    #[test]
    fn http_transport_sets_not_applicable() {
        let sys = System::new();
        let mut entry = make_http_entry();
        match_runtime(&mut entry, &sys);
        assert!(matches!(entry.runtime, RuntimeState::NotApplicable));
    }

    #[test]
    fn empty_system_sets_not_running() {
        // An empty System has no processes, so any stdio command won't match.
        let sys = System::new();
        let mut entry = make_entry("npx", vec!["-y", "some-mcp-server"]);
        match_runtime(&mut entry, &sys);
        assert!(matches!(entry.runtime, RuntimeState::NotRunning));
    }

    #[test]
    fn empty_command_sets_not_running() {
        let sys = System::new();
        let mut entry = make_entry("", vec![]);
        match_runtime(&mut entry, &sys);
        assert!(matches!(entry.runtime, RuntimeState::NotRunning));
    }
}
