pub mod budget;
pub mod discovery;
pub mod frontmatter;
pub mod projects;
pub mod sizing;
pub mod tokens;

use std::path::PathBuf;

use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;
use serde::Serialize;

pub use discovery::{Skill, SkillScope, ScopeKind};
pub use tokens::Tokenizer;

// ---------------------------------------------------------------------------
// Report types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct SkillsTotals {
    pub skills_count: usize,
    pub total_bytes: u64,
    pub total_listing_tokens: usize,
    pub claude_bytes: u64,
    pub codex_bytes: u64,
    pub project_count: usize,
    pub duplicate_count: usize,
    pub duplicate_wasted_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuplicateOccurrence {
    pub provider: &'static str,
    pub scope_kind: ScopeKind,
    pub root: PathBuf,
    pub project_label: Option<String>,
    pub bytes: u64,
    pub listing_tokens: usize,
    pub frontmatter_status: frontmatter::FrontmatterStatus,
    /// First 120 chars of the description, for quick diff comparison in UIs.
    pub description_excerpt: Option<String>,
    pub is_symlink: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuplicateGroup {
    pub name: String,
    pub count: usize,
    /// Sum of all occurrence bytes minus the smallest one — bytes you could save.
    pub wasted_bytes: u64,
    /// Sum of all occurrence listing_tokens minus the smallest one.
    pub wasted_tokens: usize,
    pub occurrences: Vec<DuplicateOccurrence>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillsReport {
    pub generated_at: String,
    /// "heuristic" or "tiktoken-cl100k"
    pub tokenizer: &'static str,
    pub max_desc_chars: usize,
    pub budget_fraction: f64,
    pub scopes: Vec<SkillScope>,
    pub totals: SkillsTotals,
    pub budget: Vec<budget::BudgetRow>,
    /// Skills whose name appears in 2+ scopes, sorted by wasted_bytes descending.
    pub duplicates: Vec<DuplicateGroup>,
}

// ---------------------------------------------------------------------------
// Scan options
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub include_global: bool,
    pub include_plugins: bool,
    pub include_projects: bool,
    /// Explicit project paths from `--path`; if non-empty, replaces DB discovery.
    pub project_paths: Vec<PathBuf>,
    /// Override `~/.claude/` for tests (keeps real home untouched).
    pub claude_home_override: Option<PathBuf>,
    /// Override `~/.codex/` for tests.
    pub codex_home_override: Option<PathBuf>,
    pub tokenizer: Tokenizer,
    pub max_desc_chars: usize,
    pub budget_fraction: f64,
    /// DB path used to discover project CWDs (optional; omit to skip per-project scan).
    pub db_path: Option<PathBuf>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            include_global: true,
            include_plugins: true,
            include_projects: true,
            project_paths: Vec::new(),
            claude_home_override: None,
            codex_home_override: None,
            tokenizer: Tokenizer::Heuristic,
            max_desc_chars: 1536,
            budget_fraction: 0.01,
            db_path: None,
        }
    }
}

impl ScanOptions {
    pub fn all() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// scan()
// ---------------------------------------------------------------------------

/// Run a full skills inventory and return the report.
///
/// The function never touches `~/.claude/` or `~/.codex/` in tests: pass
/// `claude_home_override` / `codex_home_override` in `opts` instead.
pub fn scan(opts: ScanOptions) -> Result<SkillsReport> {
    let claude_home = opts
        .claude_home_override
        .clone()
        .or_else(|| dirs::home_dir().map(|h| h.join(".claude")));

    let codex_home = opts
        .codex_home_override
        .clone()
        .or_else(|| dirs::home_dir().map(|h| h.join(".codex")));

    let tok = opts.tokenizer;
    let max_desc = opts.max_desc_chars;

    let mut scopes: Vec<SkillScope> = Vec::new();

    // --- Claude global skills ---
    if opts.include_global {
        if let Some(ref claude) = claude_home {
            let skills_dir = claude.join("skills");
            if let Some(scope) = discovery::enumerate_skill_dirs(
                &skills_dir,
                ScopeKind::ClaudeGlobal,
                None,
                max_desc,
                tok,
            ) {
                scopes.push(scope);
            }
        }
    }

    // --- Claude plugin skills ---
    if opts.include_plugins {
        if let Some(ref claude) = claude_home {
            let plugins_dir = claude.join("plugins");
            if let Some(scope) = discovery::enumerate_plugin_skills(&plugins_dir, max_desc, tok) {
                scopes.push(scope);
            }
        }
    }

    // --- Codex global skills ---
    if opts.include_global {
        if let Some(ref codex) = codex_home {
            let skills_dir = codex.join("skills");
            if let Some(scope) = discovery::enumerate_skill_dirs(
                &skills_dir,
                ScopeKind::CodexGlobal,
                None,
                max_desc,
                tok,
            ) {
                scopes.push(scope);
            }
        }
    }

    // --- Codex prompts proxy ---
    if opts.include_global {
        if let Some(ref codex) = codex_home {
            let prompts_dir = codex.join("prompts");
            if let Some(scope) = discovery::enumerate_codex_prompts(&prompts_dir, max_desc, tok) {
                scopes.push(scope);
            }
        }
    }

    // --- Per-project skills ---
    if opts.include_projects {
        let project_roots = resolve_project_paths(&opts);
        let project_count = project_roots.len();

        for root in &project_roots {
            let label = root
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string());

            // Claude per-project
            let claude_skills = root.join(".claude").join("skills");
            if let Some(scope) = discovery::enumerate_skill_dirs(
                &claude_skills,
                ScopeKind::ClaudeProject,
                label.as_deref(),
                max_desc,
                tok,
            ) {
                scopes.push(scope);
            }

            // Codex per-project
            let codex_skills = root.join(".codex").join("skills");
            if let Some(scope) = discovery::enumerate_skill_dirs(
                &codex_skills,
                ScopeKind::CodexProject,
                label.as_deref(),
                max_desc,
                tok,
            ) {
                scopes.push(scope);
            }
        }

        // Record project_count in totals via a side channel.
        let _ = project_count; // used below in totals computation
    }

    // --- Totals ---
    let mut totals = compute_totals(&scopes, &opts);

    // --- Duplicates ---
    let duplicates = detect_duplicates(&scopes);
    totals.duplicate_count = duplicates.len();
    totals.duplicate_wasted_bytes = duplicates.iter().map(|d| d.wasted_bytes).sum();

    // --- Budget ---
    let all_skills: Vec<&Skill> = scopes.iter().flat_map(|s| s.skills.iter()).collect();
    let names: Vec<&str> = all_skills.iter().map(|s| s.name.as_str()).collect();
    let skill_tokens: Vec<usize> = all_skills.iter().map(|s| s.listing_tokens).collect();
    let budget_rows = budget::compute_budget(
        &names,
        &skill_tokens,
        opts.budget_fraction,
        budget::DEFAULT_CONTEXT_SIZES,
    );

    Ok(SkillsReport {
        generated_at: Utc::now().to_rfc3339(),
        tokenizer: tok.as_str(),
        max_desc_chars: opts.max_desc_chars,
        budget_fraction: opts.budget_fraction,
        scopes,
        totals,
        budget: budget_rows,
        duplicates,
    })
}

fn detect_duplicates(scopes: &[SkillScope]) -> Vec<DuplicateGroup> {
    use std::collections::HashMap;
    let mut by_name: HashMap<String, Vec<DuplicateOccurrence>> = HashMap::new();

    for scope in scopes {
        for skill in &scope.skills {
            let excerpt = skill.description.as_deref().map(|d| {
                d.chars().take(120).collect::<String>()
            });
            by_name.entry(skill.name.clone()).or_default().push(DuplicateOccurrence {
                provider: scope.provider,
                scope_kind: scope.kind,
                root: scope.root.clone(),
                project_label: scope.project_label.clone(),
                bytes: skill.bytes,
                listing_tokens: skill.listing_tokens,
                frontmatter_status: skill.frontmatter_status,
                description_excerpt: excerpt,
                is_symlink: skill.is_symlink,
            });
        }
    }

    let mut groups: Vec<DuplicateGroup> = by_name
        .into_iter()
        .filter(|(_, occs)| occs.len() >= 2)
        .map(|(name, occs)| {
            let total_bytes: u64 = occs.iter().map(|o| o.bytes).sum();
            let min_bytes = occs.iter().map(|o| o.bytes).min().unwrap_or(0);
            let total_tokens: usize = occs.iter().map(|o| o.listing_tokens).sum();
            let min_tokens = occs.iter().map(|o| o.listing_tokens).min().unwrap_or(0);
            DuplicateGroup {
                count: occs.len(),
                wasted_bytes: total_bytes.saturating_sub(min_bytes),
                wasted_tokens: total_tokens.saturating_sub(min_tokens),
                name,
                occurrences: occs,
            }
        })
        .collect();

    groups.sort_by(|a, b| b.wasted_bytes.cmp(&a.wasted_bytes));
    groups
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
        tracing::debug!("skills: db at {} not found, skipping project scan", db_path.display());
        return vec![];
    }

    match Connection::open(&db_path) {
        Ok(conn) => projects::discover_project_paths(&conn, &[]),
        Err(e) => {
            tracing::warn!("skills: cannot open db for project discovery: {e}");
            vec![]
        }
    }
}

fn compute_totals(scopes: &[SkillScope], opts: &ScanOptions) -> SkillsTotals {
    let project_roots = if opts.include_projects {
        resolve_project_paths(opts)
    } else {
        vec![]
    };

    let skills_count: usize = scopes.iter().map(|s| s.skills.len()).sum();
    let total_bytes: u64 = scopes.iter().map(|s| s.bytes).sum();
    let total_listing_tokens: usize = scopes.iter().map(|s| s.listing_tokens).sum();

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

    SkillsTotals {
        skills_count,
        total_bytes,
        total_listing_tokens,
        claude_bytes,
        codex_bytes,
        project_count: project_roots.len(),
        duplicate_count: 0,
        duplicate_wasted_bytes: 0,
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

    fn make_skill(root: &std::path::Path, name: &str, desc: &str) {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: {desc}\n---\n# body"),
        )
        .unwrap();
    }

    #[test]
    fn full_scan_with_overrides() {
        let claude_home = TempDir::new().unwrap();
        let codex_home = TempDir::new().unwrap();

        // Two Claude global skills.
        let claude_skills = claude_home.path().join("skills");
        fs::create_dir_all(&claude_skills).unwrap();
        make_skill(&claude_skills, "skill-a", "First skill.");
        make_skill(&claude_skills, "skill-b", "Second skill.");

        // One Codex global skill.
        let codex_skills = codex_home.path().join("skills");
        fs::create_dir_all(&codex_skills).unwrap();
        make_skill(&codex_skills, "codex-skill", "Codex skill.");

        let opts = ScanOptions {
            claude_home_override: Some(claude_home.path().to_path_buf()),
            codex_home_override: Some(codex_home.path().to_path_buf()),
            include_projects: false,
            ..Default::default()
        };

        let report = scan(opts).unwrap();
        assert_eq!(report.totals.skills_count, 3);
        assert_eq!(report.totals.claude_bytes, report.scopes.iter()
            .filter(|s| s.provider == "claude")
            .map(|s| s.bytes)
            .sum::<u64>());
        assert!(!report.budget.is_empty());
        assert_eq!(report.tokenizer, "heuristic");
    }

    #[test]
    fn empty_dirs_produce_no_scopes() {
        let claude_home = TempDir::new().unwrap();
        let codex_home = TempDir::new().unwrap();

        let opts = ScanOptions {
            claude_home_override: Some(claude_home.path().to_path_buf()),
            codex_home_override: Some(codex_home.path().to_path_buf()),
            include_projects: false,
            ..Default::default()
        };

        let report = scan(opts).unwrap();
        assert_eq!(report.totals.skills_count, 0);
        assert!(report.scopes.is_empty());
    }

    #[test]
    fn generated_at_is_rfc3339() {
        let dir = TempDir::new().unwrap();
        let opts = ScanOptions {
            claude_home_override: Some(dir.path().to_path_buf()),
            codex_home_override: Some(dir.path().to_path_buf()),
            include_projects: false,
            ..Default::default()
        };
        let report = scan(opts).unwrap();
        // Must parse as RFC3339 without panic.
        let parsed = chrono::DateTime::parse_from_rfc3339(&report.generated_at);
        assert!(parsed.is_ok(), "generated_at is not RFC3339: {}", report.generated_at);
    }
}
