use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::Serialize;

use crate::skills::Tokenizer;

use super::{InstructionFile, InstructionScope, frontmatter, tokens};

// ---------------------------------------------------------------------------
// ScopeKind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    ClaudeGlobal,
    ClaudeProjectRoot,
    ClaudeProjectDotDir,
    ClaudeProjectNested,
    CodexGlobal,
    CodexProjectRoot,
    CodexProjectDotDir,
}

impl ScopeKind {
    pub fn provider(self) -> &'static str {
        match self {
            ScopeKind::ClaudeGlobal
            | ScopeKind::ClaudeProjectRoot
            | ScopeKind::ClaudeProjectDotDir
            | ScopeKind::ClaudeProjectNested => "claude",
            ScopeKind::CodexGlobal
            | ScopeKind::CodexProjectRoot
            | ScopeKind::CodexProjectDotDir => "codex",
        }
    }
}

// ---------------------------------------------------------------------------
// Directory names to skip during nested walk
// ---------------------------------------------------------------------------

static SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".venv",
    "venv",
    "__pycache__",
    ".gradle",
    "DerivedData",
    ".idea",
    ".vscode",
    "Pods",
];

// ---------------------------------------------------------------------------
// Public helpers
// ---------------------------------------------------------------------------

/// Read a single instruction file and return an `InstructionFile`, or `None`
/// if the path does not exist or cannot be read.
pub fn read_instruction_file(path: &Path, tok: Tokenizer) -> Option<InstructionFile> {
    // Use symlink_metadata so we don't follow the symlink for is_symlink/bytes.
    let lmeta = match path.symlink_metadata() {
        Ok(m) => m,
        Err(_) => return None,
    };

    // If the path is a symlink, get the real metadata for the follow-through
    // size, but keep is_symlink = true.
    let is_symlink = lmeta.file_type().is_symlink();
    let bytes = if is_symlink {
        // Follow the link for the actual byte size; fall back to lstat size.
        std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(lmeta.len())
    } else {
        lmeta.len()
    };

    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("instruction_files: cannot read {}: {e}", path.display());
            return None;
        }
    };

    let line_count = if text.is_empty() {
        0
    } else {
        text.chars().filter(|&c| c == '\n').count() + 1
    };

    let modified = {
        let meta = std::fs::metadata(path).or_else(|_| path.symlink_metadata());
        match meta.and_then(|m| m.modified()) {
            Ok(st) => {
                let secs = st.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
                chrono::DateTime::from_timestamp(secs, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_else(|| "1970-01-01T00:00:00+00:00".to_string())
            }
            Err(_) => "1970-01-01T00:00:00+00:00".to_string(),
        }
    };

    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let frontmatter_status = if file_name == "CLAUDE.md" {
        frontmatter::probe_status(&text)
    } else {
        frontmatter::FrontmatterStatus::NotApplicable
    };

    let token_count = tokens::count_text(&text, tok);

    Some(InstructionFile {
        path: path.to_path_buf(),
        bytes,
        line_count,
        tokens: token_count,
        modified,
        frontmatter_status,
        is_symlink,
    })
}

/// Build a single-file `InstructionScope` for a known path.
/// Returns `None` if the file does not exist.
pub fn enumerate_single_file(
    path: &Path,
    kind: ScopeKind,
    provider: &'static str,
    project_label: Option<String>,
    tok: Tokenizer,
) -> Option<InstructionScope> {
    let file = read_instruction_file(path, tok)?;
    let bytes = file.bytes;
    let file_tokens = file.tokens;
    Some(InstructionScope {
        provider,
        kind,
        root: path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".")),
        project_label,
        files: vec![file],
        bytes,
        tokens: file_tokens,
    })
}

/// Walk `root` recursively (up to `max_depth`) for `CLAUDE.md` files that are
/// NOT the project-root `CLAUDE.md` or `.claude/CLAUDE.md` (those have their
/// own scopes).  Skips known build/tooling directories.
///
/// Returns `None` if no nested files are found.
pub fn enumerate_nested_claude_md(
    root: &Path,
    project_label: Option<String>,
    max_depth: usize,
    tok: Tokenizer,
) -> Option<InstructionScope> {
    let root_claude_md = root.join("CLAUDE.md");
    let dot_claude_claude_md = root.join(".claude").join("CLAUDE.md");

    let mut files: Vec<InstructionFile> = Vec::new();

    for entry in walkdir::WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Allow the root itself; skip known noisy directories.
            if e.depth() == 0 {
                return true;
            }
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                if SKIP_DIRS.contains(&name.as_ref()) {
                    return false;
                }
            }
            true
        })
        .flatten()
    {
        if entry.file_type().is_dir() {
            continue;
        }
        if entry.file_name() != "CLAUDE.md" {
            continue;
        }

        let path = entry.path();

        // Canonically skip the two root-level scopes.
        if path == root_claude_md || path == dot_claude_claude_md {
            continue;
        }

        if let Some(f) = read_instruction_file(path, tok) {
            files.push(f);
        }
    }

    if files.is_empty() {
        return None;
    }

    let bytes: u64 = files.iter().map(|f| f.bytes).sum();
    let tok_sum: usize = files.iter().map(|f| f.tokens).sum();

    Some(InstructionScope {
        provider: ScopeKind::ClaudeProjectNested.provider(),
        kind: ScopeKind::ClaudeProjectNested,
        root: root.to_path_buf(),
        project_label,
        files,
        bytes,
        tokens: tok_sum,
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

    #[test]
    fn read_instruction_file_basic() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("CLAUDE.md");
        fs::write(&path, "# Title\nLine two.\nLine three.").unwrap();
        let f = read_instruction_file(&path, Tokenizer::Heuristic).unwrap();
        assert_eq!(f.line_count, 3);
        assert!(f.bytes > 0);
        assert!(f.tokens > 0);
        assert!(!f.is_symlink);
        assert_eq!(f.frontmatter_status, frontmatter::FrontmatterStatus::Ok);
    }

    #[test]
    fn read_instruction_file_empty_string_is_zero_lines() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("CLAUDE.md");
        fs::write(&path, "").unwrap();
        let f = read_instruction_file(&path, Tokenizer::Heuristic).unwrap();
        assert_eq!(f.line_count, 0);
    }

    #[test]
    fn read_instruction_file_nonexistent_returns_none() {
        let result =
            read_instruction_file(Path::new("/nonexistent/CLAUDE.md"), Tokenizer::Heuristic);
        assert!(result.is_none());
    }

    #[test]
    fn agents_md_gets_not_applicable_status() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("AGENTS.md");
        fs::write(&path, "# Global agents").unwrap();
        let f = read_instruction_file(&path, Tokenizer::Heuristic).unwrap();
        assert_eq!(
            f.frontmatter_status,
            frontmatter::FrontmatterStatus::NotApplicable
        );
    }

    #[test]
    fn enumerate_nested_skips_root_and_dot_claude() {
        let dir = TempDir::new().unwrap();
        // Root CLAUDE.md — should be skipped
        fs::write(dir.path().join("CLAUDE.md"), "# Root").unwrap();
        // .claude/CLAUDE.md — should be skipped
        fs::create_dir_all(dir.path().join(".claude")).unwrap();
        fs::write(dir.path().join(".claude").join("CLAUDE.md"), "# Dot").unwrap();
        // Nested — should be found
        let sub = dir.path().join("subdir");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("CLAUDE.md"), "# Sub").unwrap();

        let scope = enumerate_nested_claude_md(dir.path(), None, 8, Tokenizer::Heuristic).unwrap();
        assert_eq!(scope.files.len(), 1);
        assert!(scope.files[0].path.ends_with("subdir/CLAUDE.md"));
    }

    #[test]
    fn enumerate_nested_skips_node_modules() {
        let dir = TempDir::new().unwrap();
        let nm = dir.path().join("node_modules").join("pkg");
        fs::create_dir_all(&nm).unwrap();
        fs::write(nm.join("CLAUDE.md"), "# Should be skipped").unwrap();

        let result = enumerate_nested_claude_md(dir.path(), None, 8, Tokenizer::Heuristic);
        assert!(result.is_none());
    }
}
