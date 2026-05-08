use std::path::{Path, PathBuf};

use serde::Serialize;

use super::{
    frontmatter::{FrontmatterStatus, parse_skill_md},
    sizing::bundle_bytes,
    tokens::{Tokenizer, count_listing_tokens},
};

// ---------------------------------------------------------------------------
// Scope kind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    ClaudeGlobal,
    ClaudePlugin,
    ClaudeProject,
    CodexGlobal,
    CodexPrompts,
    CodexProject,
}

impl ScopeKind {
    pub fn provider(self) -> &'static str {
        match self {
            ScopeKind::ClaudeGlobal | ScopeKind::ClaudePlugin | ScopeKind::ClaudeProject => {
                "claude"
            }
            ScopeKind::CodexGlobal | ScopeKind::CodexPrompts | ScopeKind::CodexProject => "codex",
        }
    }
}

// ---------------------------------------------------------------------------
// Skill
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct Skill {
    pub name: String,
    pub path: PathBuf,
    pub description: Option<String>,
    /// Raw character count of the description before truncation.
    pub description_chars: usize,
    /// True when the raw description exceeds `max_desc_chars`.
    pub description_truncated: bool,
    pub bytes: u64,
    pub file_count: u32,
    /// Estimated listing-token cost (wrapper + truncated name + truncated description).
    pub listing_tokens: usize,
    pub frontmatter_status: FrontmatterStatus,
    pub is_symlink: bool,
    pub symlink_target: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<crate::skills::SkillInvocationStats>,
    pub is_dormant: bool,
}

// ---------------------------------------------------------------------------
// Scope
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct SkillScope {
    pub provider: &'static str,
    pub kind: ScopeKind,
    pub root: PathBuf,
    /// Set for per-project scopes; the last component of the project path.
    pub project_label: Option<String>,
    pub skills: Vec<Skill>,
    pub bytes: u64,
    pub listing_tokens: usize,
}

// ---------------------------------------------------------------------------
// Discovery
// ---------------------------------------------------------------------------

/// Enumerate skills under `root` where each first-level subdirectory that
/// contains a `SKILL.md` is treated as one skill bundle.
///
/// Used for: `~/.claude/skills/`, `<project>/.claude/skills/`,
/// `~/.codex/skills/`, `<project>/.codex/skills/`.
///
/// For Codex, symlinks at the first level are recorded as skills with
/// `is_symlink = true` and are NOT traversed (lstat size only).
pub fn enumerate_skill_dirs(
    root: &Path,
    kind: ScopeKind,
    project_label: Option<&str>,
    max_desc_chars: usize,
    tok: Tokenizer,
) -> Option<SkillScope> {
    if !root.exists() {
        return None;
    }

    let entries = match std::fs::read_dir(root) {
        Ok(e) => e,
        Err(e) => {
            tracing::debug!("skills: cannot read dir {}: {e}", root.display());
            return None;
        }
    };

    let mut skills = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();

        // Skip hidden files, .system reserved dir, and non-directories.
        if name_str.starts_with('.') || name_str == "system" {
            continue;
        }

        // Detect symlinks without following them.
        let lstat = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if lstat.is_symlink() {
            let target = std::fs::read_link(&path).ok();
            let skill_md = path.join("SKILL.md");
            let fm = if skill_md.exists() {
                parse_skill_md(&skill_md)
            } else {
                super::frontmatter::SkillFrontmatter {
                    name: name_str.into_owned(),
                    description: None,
                    raw_desc_chars: 0,
                    status: FrontmatterStatus::Missing,
                }
            };
            let tokens =
                count_listing_tokens(&fm.name, fm.description.as_deref(), max_desc_chars, tok);
            skills.push(Skill {
                name: fm.name,
                path: path.clone(),
                description: fm.description,
                description_chars: fm.raw_desc_chars,
                description_truncated: fm.raw_desc_chars > max_desc_chars,
                bytes: lstat.len(),
                file_count: 1,
                listing_tokens: tokens,
                frontmatter_status: fm.status,
                is_symlink: true,
                symlink_target: target,
                usage: None,
                is_dormant: false,
            });
            continue;
        }

        if !lstat.is_dir() {
            continue;
        }

        let skill_md = path.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }

        let fm = parse_skill_md(&skill_md);
        let (file_count, bytes) = bundle_bytes(&path);
        let tokens = count_listing_tokens(&fm.name, fm.description.as_deref(), max_desc_chars, tok);

        skills.push(Skill {
            name: fm.name,
            path,
            description: fm.description,
            description_chars: fm.raw_desc_chars,
            description_truncated: fm.raw_desc_chars > max_desc_chars,
            bytes,
            file_count,
            listing_tokens: tokens,
            frontmatter_status: fm.status,
            is_symlink: false,
            symlink_target: None,
            usage: None,
            is_dormant: false,
        });
    }

    if skills.is_empty() && !root.exists() {
        return None;
    }

    let total_bytes: u64 = skills.iter().map(|s| s.bytes).sum();
    let total_tokens: usize = skills.iter().map(|s| s.listing_tokens).sum();

    Some(SkillScope {
        provider: kind.provider(),
        kind,
        root: root.to_path_buf(),
        project_label: project_label.map(|s| s.to_string()),
        skills,
        bytes: total_bytes,
        listing_tokens: total_tokens,
    })
}

/// Walk `plugins_root` (e.g. `~/.claude/plugins/`) for any `SKILL.md` files
/// and build one `SkillScope` containing all plugin skills.
///
/// Each `SKILL.md` parent dir is the bundle dir.  Nested SKILL.md files (e.g.
/// `version/skill-name/SKILL.md` and `version/skill-name/template/SKILL.md`)
/// are each reported independently; their bundle sizes will overlap for the
/// `template/` case but this is documented and acceptable.
pub fn enumerate_plugin_skills(
    plugins_root: &Path,
    max_desc_chars: usize,
    tok: Tokenizer,
) -> Option<SkillScope> {
    if !plugins_root.exists() {
        return None;
    }

    let mut skills = Vec::new();

    for entry in walkdir::WalkDir::new(plugins_root)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        if entry.file_name() != "SKILL.md" {
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }

        let skill_md = entry.path();
        let bundle_dir = match skill_md.parent() {
            Some(p) => p,
            None => continue,
        };

        let fm = parse_skill_md(skill_md);
        let (file_count, bytes) = bundle_bytes(bundle_dir);
        let tokens = count_listing_tokens(&fm.name, fm.description.as_deref(), max_desc_chars, tok);

        skills.push(Skill {
            name: fm.name,
            path: bundle_dir.to_path_buf(),
            description: fm.description,
            description_chars: fm.raw_desc_chars,
            description_truncated: fm.raw_desc_chars > max_desc_chars,
            bytes,
            file_count,
            listing_tokens: tokens,
            frontmatter_status: fm.status,
            is_symlink: false,
            symlink_target: None,
            usage: None,
            is_dormant: false,
        });
    }

    if skills.is_empty() {
        return None;
    }

    let total_bytes: u64 = skills.iter().map(|s| s.bytes).sum();
    let total_tokens: usize = skills.iter().map(|s| s.listing_tokens).sum();

    Some(SkillScope {
        provider: ScopeKind::ClaudePlugin.provider(),
        kind: ScopeKind::ClaudePlugin,
        root: plugins_root.to_path_buf(),
        project_label: None,
        skills,
        bytes: total_bytes,
        listing_tokens: total_tokens,
    })
}

/// Walk `prompts_root` for `*.md` files (Codex prompts proxy).
pub fn enumerate_codex_prompts(
    prompts_root: &Path,
    max_desc_chars: usize,
    tok: Tokenizer,
) -> Option<SkillScope> {
    if !prompts_root.exists() {
        return None;
    }

    let mut skills = Vec::new();

    for entry in walkdir::WalkDir::new(prompts_root)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let path = entry.path();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let tokens = count_listing_tokens(&name, None, max_desc_chars, tok);

        skills.push(Skill {
            name,
            path: path.to_path_buf(),
            description: None,
            description_chars: 0,
            description_truncated: false,
            bytes,
            file_count: 1,
            listing_tokens: tokens,
            frontmatter_status: FrontmatterStatus::Missing,
            is_symlink: false,
            symlink_target: None,
            usage: None,
            is_dormant: false,
        });
    }

    if skills.is_empty() {
        return None;
    }

    let total_bytes: u64 = skills.iter().map(|s| s.bytes).sum();
    let total_tokens: usize = skills.iter().map(|s| s.listing_tokens).sum();

    Some(SkillScope {
        provider: "codex",
        kind: ScopeKind::CodexPrompts,
        root: prompts_root.to_path_buf(),
        project_label: None,
        skills,
        bytes: total_bytes,
        listing_tokens: total_tokens,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn skill_dir(root: &Path, name: &str, desc: &str) {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("SKILL.md"),
            format!("---\nname: {name}\ndescription: {desc}\n---\n# body"),
        )
        .unwrap();
    }

    #[test]
    fn finds_skills_with_skill_md() {
        let dir = TempDir::new().unwrap();
        skill_dir(dir.path(), "my-skill", "Does things.");
        skill_dir(dir.path(), "other-skill", "Also useful.");

        let scope = enumerate_skill_dirs(
            dir.path(),
            ScopeKind::ClaudeGlobal,
            None,
            1536,
            Tokenizer::Heuristic,
        )
        .unwrap();
        assert_eq!(scope.skills.len(), 2);
        let names: Vec<&str> = scope.skills.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"my-skill"));
        assert!(names.contains(&"other-skill"));
    }

    #[test]
    fn skips_dirs_without_skill_md() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("not-a-skill")).unwrap();
        skill_dir(dir.path(), "real-skill", "desc");
        let scope = enumerate_skill_dirs(
            dir.path(),
            ScopeKind::ClaudeGlobal,
            None,
            1536,
            Tokenizer::Heuristic,
        )
        .unwrap();
        assert_eq!(scope.skills.len(), 1);
        assert_eq!(scope.skills[0].name, "real-skill");
    }

    #[test]
    fn nonexistent_root_returns_none() {
        let result = enumerate_skill_dirs(
            Path::new("/nonexistent/skills"),
            ScopeKind::ClaudeGlobal,
            None,
            1536,
            Tokenizer::Heuristic,
        );
        assert!(result.is_none());
    }

    #[test]
    fn plugin_walk_finds_nested_skill_md() {
        let dir = TempDir::new().unwrap();
        // Simulate: plugins/cache/marketplace/plugin/1.0.0/my-skill/SKILL.md
        let nested = dir
            .path()
            .join("cache")
            .join("marketplace")
            .join("plugin")
            .join("1.0.0")
            .join("my-skill");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            nested.join("SKILL.md"),
            "---\nname: nested-skill\ndescription: Found it.\n---\n",
        )
        .unwrap();

        let scope = enumerate_plugin_skills(dir.path(), 1536, Tokenizer::Heuristic).unwrap();
        assert_eq!(scope.skills.len(), 1);
        assert_eq!(scope.skills[0].name, "nested-skill");
    }

    #[test]
    fn description_truncated_flag() {
        let dir = TempDir::new().unwrap();
        let long_desc: String = "x".repeat(2000);
        let inner = dir.path().join("s");
        fs::create_dir_all(&inner).unwrap();
        fs::write(
            inner.join("SKILL.md"),
            format!("---\nname: s\ndescription: {long_desc}\n---\n"),
        )
        .unwrap();
        let scope = enumerate_skill_dirs(
            dir.path(),
            ScopeKind::ClaudeGlobal,
            None,
            1536,
            Tokenizer::Heuristic,
        )
        .unwrap();
        assert!(scope.skills[0].description_truncated);
    }
}
