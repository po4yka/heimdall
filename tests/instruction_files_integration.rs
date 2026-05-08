use std::fs;

use chrono::DateTime;
use tempfile::TempDir;

use claude_usage_tracker::instruction_files::{ScanOptions, scan};

#[test]
fn full_scan_global_and_nested() {
    let claude_home = TempDir::new().unwrap();
    let codex_home = TempDir::new().unwrap();

    // Global CLAUDE.md
    fs::write(
        claude_home.path().join("CLAUDE.md"),
        "# Global\nHello world.",
    )
    .unwrap();
    // Global AGENTS.md
    fs::write(codex_home.path().join("AGENTS.md"), "# Codex\nHello.").unwrap();

    let opts = ScanOptions {
        include_global: true,
        include_projects: false,
        include_nested: false,
        claude_home_override: Some(claude_home.path().to_path_buf()),
        codex_home_override: Some(codex_home.path().to_path_buf()),
        ..Default::default()
    };
    let report = scan(opts).unwrap();
    assert_eq!(report.totals.file_count, 2);
    assert!(report.totals.claude_bytes > 0);
    assert!(report.totals.codex_bytes > 0);
    assert!(!report.budget.is_empty());
    let parsed = DateTime::parse_from_rfc3339(&report.generated_at);
    assert!(parsed.is_ok());
}

#[test]
fn nested_walk_skips_node_modules() {
    let claude_home = TempDir::new().unwrap();
    let project_root = TempDir::new().unwrap();

    // Project root CLAUDE.md
    fs::write(project_root.path().join("CLAUDE.md"), "# Root").unwrap();
    // Nested (should be found)
    let nested = project_root.path().join("subdir");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("CLAUDE.md"), "# Nested").unwrap();
    // node_modules (should be skipped)
    let nm = project_root.path().join("node_modules").join("pkg");
    fs::create_dir_all(&nm).unwrap();
    fs::write(nm.join("CLAUDE.md"), "# Skipped").unwrap();

    let opts = ScanOptions {
        include_global: false,
        include_projects: true,
        include_nested: true,
        project_paths: vec![project_root.path().to_path_buf()],
        claude_home_override: Some(claude_home.path().to_path_buf()),
        codex_home_override: Some(claude_home.path().to_path_buf()),
        ..Default::default()
    };
    let report = scan(opts).unwrap();
    let nested_scope = report.scopes.iter().find(|s| {
        matches!(
            s.kind,
            claude_usage_tracker::instruction_files::discovery::ScopeKind::ClaudeProjectNested
        )
    });
    assert_eq!(nested_scope.map(|s| s.files.len()).unwrap_or(0), 1);
}
