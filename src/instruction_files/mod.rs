pub mod claude_md_history;
pub mod discovery;
pub mod frontmatter;
pub mod git_history;
pub mod tokens;

use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;
use serde::Serialize;

use crate::skills::{Tokenizer, budget, projects};

pub use frontmatter::FrontmatterStatus;

// ---------------------------------------------------------------------------
// Report types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct InstructionFilesReport {
    pub generated_at: String,
    pub tokenizer: &'static str,
    pub budget_fraction: f64,
    pub scopes: Vec<InstructionScope>,
    pub totals: InstructionTotals,
    pub budget: Vec<budget::BudgetRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstructionScope {
    pub provider: &'static str,
    pub kind: discovery::ScopeKind,
    pub root: PathBuf,
    pub project_label: Option<String>,
    pub files: Vec<InstructionFile>,
    pub bytes: u64,
    pub tokens: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstructionFile {
    pub path: PathBuf,
    pub bytes: u64,
    pub line_count: usize,
    pub tokens: usize,
    pub modified: String,
    pub frontmatter_status: FrontmatterStatus,
    pub is_symlink: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstructionTotals {
    pub file_count: usize,
    pub total_bytes: u64,
    pub total_tokens: usize,
    pub claude_bytes: u64,
    pub codex_bytes: u64,
    pub project_count: usize,
    pub nested_count: usize,
}

// ---------------------------------------------------------------------------
// Scan options
// ---------------------------------------------------------------------------

pub struct ScanOptions {
    pub include_global: bool,
    pub include_projects: bool,
    pub include_nested: bool,
    /// Explicit project paths from `--path`; if non-empty, replaces DB discovery.
    pub project_paths: Vec<PathBuf>,
    /// Override `~/.claude/` for tests (keeps real home untouched).
    pub claude_home_override: Option<PathBuf>,
    /// Override `~/.codex/` for tests.
    pub codex_home_override: Option<PathBuf>,
    pub tokenizer: Tokenizer,
    pub budget_fraction: f64,
    pub max_walk_depth: usize,
    /// DB path used to discover project CWDs (optional; omit to skip per-project scan).
    pub db_path: Option<PathBuf>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            include_global: true,
            include_projects: true,
            include_nested: true,
            project_paths: Vec::new(),
            claude_home_override: None,
            codex_home_override: None,
            tokenizer: Tokenizer::Heuristic,
            budget_fraction: 0.05,
            max_walk_depth: 8,
            db_path: None,
        }
    }
}

// ---------------------------------------------------------------------------
// scan()
// ---------------------------------------------------------------------------

/// Run a full instruction-files inventory and return the report.
///
/// The function never touches `~/.claude/` or `~/.codex/` in tests: pass
/// `claude_home_override` / `codex_home_override` in `opts` instead.
pub fn scan(opts: ScanOptions) -> Result<InstructionFilesReport> {
    let claude_home = opts
        .claude_home_override
        .clone()
        .or_else(|| dirs::home_dir().map(|h| h.join(".claude")));

    let codex_home = opts
        .codex_home_override
        .clone()
        .or_else(|| dirs::home_dir().map(|h| h.join(".codex")));

    let tok = opts.tokenizer;
    let mut scopes: Vec<InstructionScope> = Vec::new();

    // --- Global files ---
    if opts.include_global {
        if let Some(ref claude) = claude_home {
            let path = claude.join("CLAUDE.md");
            if let Some(scope) = discovery::enumerate_single_file(
                &path,
                discovery::ScopeKind::ClaudeGlobal,
                "claude",
                None,
                tok,
            ) {
                scopes.push(scope);
            }
        }

        if let Some(ref codex) = codex_home {
            let path = codex.join("AGENTS.md");
            if let Some(scope) = discovery::enumerate_single_file(
                &path,
                discovery::ScopeKind::CodexGlobal,
                "codex",
                None,
                tok,
            ) {
                scopes.push(scope);
            }
        }
    }

    // --- Per-project files ---
    if opts.include_projects {
        let project_roots = resolve_project_paths(&opts);

        for root in &project_roots {
            let label = root
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string());

            // Claude project root CLAUDE.md
            let claude_root_md = root.join("CLAUDE.md");
            if let Some(scope) = discovery::enumerate_single_file(
                &claude_root_md,
                discovery::ScopeKind::ClaudeProjectRoot,
                "claude",
                label.clone(),
                tok,
            ) {
                scopes.push(scope);
            }

            // Claude .claude/CLAUDE.md
            let claude_dot_dir_md = root.join(".claude").join("CLAUDE.md");
            if let Some(scope) = discovery::enumerate_single_file(
                &claude_dot_dir_md,
                discovery::ScopeKind::ClaudeProjectDotDir,
                "claude",
                label.clone(),
                tok,
            ) {
                scopes.push(scope);
            }

            // Nested CLAUDE.md files
            if opts.include_nested
                && let Some(scope) = discovery::enumerate_nested_claude_md(
                    root,
                    label.clone(),
                    opts.max_walk_depth,
                    tok,
                )
            {
                scopes.push(scope);
            }

            // Codex project root AGENTS.md
            let codex_root_md = root.join("AGENTS.md");
            if let Some(scope) = discovery::enumerate_single_file(
                &codex_root_md,
                discovery::ScopeKind::CodexProjectRoot,
                "codex",
                label.clone(),
                tok,
            ) {
                scopes.push(scope);
            }

            // Codex .codex/AGENTS.md
            let codex_dot_dir_md = root.join(".codex").join("AGENTS.md");
            if let Some(scope) = discovery::enumerate_single_file(
                &codex_dot_dir_md,
                discovery::ScopeKind::CodexProjectDotDir,
                "codex",
                label.clone(),
                tok,
            ) {
                scopes.push(scope);
            }
        }
    }

    // --- Totals ---
    let totals = compute_totals(&scopes, &opts);

    // --- Budget ---
    let all_files: Vec<&InstructionFile> = scopes.iter().flat_map(|s| s.files.iter()).collect();
    let names: Vec<&str> = all_files
        .iter()
        .map(|f| {
            f.path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
        })
        .collect();
    let token_counts: Vec<usize> = all_files.iter().map(|f| f.tokens).collect();
    let budget_rows = budget::compute_budget(
        &names,
        &token_counts,
        opts.budget_fraction,
        budget::DEFAULT_CONTEXT_SIZES,
    );

    Ok(InstructionFilesReport {
        generated_at: Utc::now().to_rfc3339(),
        tokenizer: tok.as_str(),
        budget_fraction: opts.budget_fraction,
        scopes,
        totals,
        budget: budget_rows,
    })
}

fn resolve_project_paths(opts: &ScanOptions) -> Vec<PathBuf> {
    // If explicit paths given, use them (no DB needed).
    if !opts.project_paths.is_empty() {
        return opts
            .project_paths
            .iter()
            .filter(|p| p.exists() && p.is_dir())
            .cloned()
            .collect();
    }

    // Fall back to DB-backed discovery.
    let db_path = match &opts.db_path {
        Some(p) => p.clone(),
        None => return vec![],
    };

    if !db_path.exists() {
        tracing::debug!(
            "instruction_files: db at {} not found, skipping project scan",
            db_path.display()
        );
        return vec![];
    }

    match Connection::open(&db_path) {
        Ok(conn) => projects::discover_project_paths(&conn, &[]),
        Err(e) => {
            tracing::warn!("instruction_files: cannot open db for project discovery: {e}");
            vec![]
        }
    }
}

fn compute_totals(scopes: &[InstructionScope], opts: &ScanOptions) -> InstructionTotals {
    let project_roots = if opts.include_projects {
        resolve_project_paths(opts)
    } else {
        vec![]
    };

    let file_count: usize = scopes.iter().map(|s| s.files.len()).sum();
    let total_bytes: u64 = scopes.iter().map(|s| s.bytes).sum();
    let total_tokens: usize = scopes.iter().map(|s| s.tokens).sum();

    let claude_bytes: u64 = scopes
        .iter()
        .filter(|s| s.provider == "claude")
        .map(|s| s.bytes)
        .sum();
    let codex_bytes: u64 = scopes
        .iter()
        .filter(|s| s.provider == "codex")
        .map(|s| s.bytes)
        .sum();

    let nested_count: usize = scopes
        .iter()
        .filter(|s| matches!(s.kind, discovery::ScopeKind::ClaudeProjectNested))
        .map(|s| s.files.len())
        .sum();

    InstructionTotals {
        file_count,
        total_bytes,
        total_tokens,
        claude_bytes,
        codex_bytes,
        project_count: project_roots.len(),
        nested_count,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn global_scan_finds_claude_md() {
        let claude_home = TempDir::new().unwrap();
        let codex_home = TempDir::new().unwrap();

        fs::write(claude_home.path().join("CLAUDE.md"), "# Global\nHello.").unwrap();

        let opts = ScanOptions {
            include_global: true,
            include_projects: false,
            include_nested: false,
            claude_home_override: Some(claude_home.path().to_path_buf()),
            codex_home_override: Some(codex_home.path().to_path_buf()),
            ..Default::default()
        };

        let report = scan(opts).unwrap();
        assert_eq!(report.totals.file_count, 1);
        assert!(report.totals.claude_bytes > 0);
        assert_eq!(report.totals.codex_bytes, 0);
        assert!(!report.budget.is_empty());
    }

    #[test]
    fn global_scan_finds_both_files() {
        let claude_home = TempDir::new().unwrap();
        let codex_home = TempDir::new().unwrap();

        fs::write(
            claude_home.path().join("CLAUDE.md"),
            "# Global\nHello world.",
        )
        .unwrap();
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
    }

    #[test]
    fn empty_dirs_produce_no_scopes() {
        let claude_home = TempDir::new().unwrap();
        let codex_home = TempDir::new().unwrap();

        let opts = ScanOptions {
            include_global: true,
            include_projects: false,
            include_nested: false,
            claude_home_override: Some(claude_home.path().to_path_buf()),
            codex_home_override: Some(codex_home.path().to_path_buf()),
            ..Default::default()
        };

        let report = scan(opts).unwrap();
        assert_eq!(report.totals.file_count, 0);
        assert!(report.scopes.is_empty());
    }

    #[test]
    fn generated_at_is_rfc3339() {
        let dir = TempDir::new().unwrap();
        let opts = ScanOptions {
            include_global: true,
            include_projects: false,
            include_nested: false,
            claude_home_override: Some(dir.path().to_path_buf()),
            codex_home_override: Some(dir.path().to_path_buf()),
            ..Default::default()
        };
        let report = scan(opts).unwrap();
        let parsed = chrono::DateTime::parse_from_rfc3339(&report.generated_at);
        assert!(
            parsed.is_ok(),
            "generated_at is not RFC3339: {}",
            report.generated_at
        );
    }

    #[test]
    fn project_scan_finds_claude_md() {
        let claude_home = TempDir::new().unwrap();
        let codex_home = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();

        fs::write(project.path().join("CLAUDE.md"), "# Project\nContent.").unwrap();

        let opts = ScanOptions {
            include_global: false,
            include_projects: true,
            include_nested: false,
            project_paths: vec![project.path().to_path_buf()],
            claude_home_override: Some(claude_home.path().to_path_buf()),
            codex_home_override: Some(codex_home.path().to_path_buf()),
            ..Default::default()
        };

        let report = scan(opts).unwrap();
        assert_eq!(report.totals.file_count, 1);
        let scope = &report.scopes[0];
        assert_eq!(scope.provider, "claude");
        assert!(matches!(
            scope.kind,
            discovery::ScopeKind::ClaudeProjectRoot
        ));
    }

    #[test]
    fn nested_count_reflects_nested_scopes() {
        let claude_home = TempDir::new().unwrap();
        let codex_home = TempDir::new().unwrap();
        let project = TempDir::new().unwrap();

        fs::write(project.path().join("CLAUDE.md"), "# Root").unwrap();
        let sub = project.path().join("docs");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("CLAUDE.md"), "# Sub docs").unwrap();

        let opts = ScanOptions {
            include_global: false,
            include_projects: true,
            include_nested: true,
            project_paths: vec![project.path().to_path_buf()],
            claude_home_override: Some(claude_home.path().to_path_buf()),
            codex_home_override: Some(codex_home.path().to_path_buf()),
            ..Default::default()
        };

        let report = scan(opts).unwrap();
        assert_eq!(report.totals.nested_count, 1);
    }
}
