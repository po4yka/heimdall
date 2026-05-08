use std::collections::HashSet;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use tracing::warn;

use crate::skills::Tokenizer;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub struct ClaudeMdRevision {
    pub commit_sha: String,
    pub commit_ts: i64,
    pub byte_size: i64,
    pub token_count: i64,
    pub line_count: i64,
}

pub struct TrackedClaudeMd {
    pub repo_root: PathBuf,
    pub rel_path: PathBuf,
    pub label: String,
}

// ---------------------------------------------------------------------------
// Git helpers
// ---------------------------------------------------------------------------

/// Resolve the git repo root for `path`. Returns `None` if not a git repo or
/// if `git` is not available.
pub fn git_toplevel(path: &Path) -> Option<PathBuf> {
    let path_str = path.to_str()?;
    let output = std::process::Command::new("git")
        .args(["-C", path_str, "rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let trimmed = stdout.trim_end_matches(['\n', '\r', ' ']);
    if trimmed.is_empty() {
        return None;
    }
    Some(PathBuf::from(trimmed))
}

/// Returns `true` if `rel_file` is currently tracked in `repo_root`.
pub fn git_file_is_tracked(repo_root: &Path, rel_file: &Path) -> bool {
    let root_str = match repo_root.to_str() {
        Some(s) => s,
        None => return false,
    };
    let file_str = match rel_file.to_str() {
        Some(s) => s,
        None => return false,
    };
    std::process::Command::new("git")
        .args(["-C", root_str, "ls-files", "--error-unmatch", file_str])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Walk git log for `rel_file` in `repo_root`, returning revisions oldest → newest.
///
/// Stops when `since_sha` is encountered (exclusive — the SHA is not re-inserted).
/// Caps at `max_commits`. Skips non-UTF-8 or >2 MiB file content at any revision.
/// Returns an empty `Vec` on any git error.
pub fn walk_claude_md_history(
    repo_root: &Path,
    rel_file: &Path,
    since_sha: Option<&str>,
    tokenizer: Tokenizer,
    max_commits: usize,
) -> Vec<ClaudeMdRevision> {
    const MAX_BYTES: usize = 2 * 1024 * 1024; // 2 MiB

    let root_str = match repo_root.to_str() {
        Some(s) => s,
        None => return vec![],
    };
    let file_str = match rel_file.to_str() {
        Some(s) => s,
        None => return vec![],
    };

    // Step 1: collect all (sha, epoch) pairs from git log, oldest first.
    let log_out = match std::process::Command::new("git")
        .args([
            "-C",
            root_str,
            "log",
            "--follow",
            "--format=%H\t%ct",
            "--",
            file_str,
        ])
        .output()
    {
        Ok(o) if o.status.success() => o.stdout,
        _ => return vec![],
    };

    let log_text = match String::from_utf8(log_out) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    // git log outputs newest first — collect then reverse for oldest-first.
    let mut entries: Vec<(String, i64)> = Vec::new();
    for line in log_text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(2, '\t');
        let sha = match parts.next() {
            Some(s) => s.trim().to_string(),
            None => continue,
        };
        let epoch: i64 = parts
            .next()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        entries.push((sha, epoch));
    }
    entries.reverse(); // oldest first

    // Step 2: skip all entries up to and including since_sha, then collect newer ones.
    // since_sha is the most-recently-stored commit (exclusive — don't re-insert it).
    let entries: Vec<(String, i64)> = if let Some(stop) = since_sha {
        let pos = entries.iter().position(|(sha, _)| sha == stop);
        match pos {
            Some(idx) => entries.into_iter().skip(idx + 1).collect(),
            None => entries, // since_sha not found in history — collect everything
        }
    } else {
        entries
    };

    let mut revisions = Vec::new();
    for (sha, epoch) in entries {
        if revisions.len() >= max_commits {
            break;
        }

        // Step 3: fetch file content at this revision.
        let blob_ref = format!("{sha}:{file_str}");
        let show_out = match std::process::Command::new("git")
            .args(["-C", root_str, "show", &blob_ref])
            .output()
        {
            Ok(o) if o.status.success() => o.stdout,
            _ => {
                // File missing at this revision — skip.
                continue;
            }
        };

        if show_out.len() > MAX_BYTES {
            warn!(
                "claude_md history: skipping {sha}:{file_str} — {} bytes > 2MiB limit",
                show_out.len()
            );
            continue;
        }

        let text = match String::from_utf8(show_out) {
            Ok(s) => s,
            Err(_) => {
                // Binary content — skip.
                continue;
            }
        };

        let byte_size = text.len() as i64;
        let token_count =
            crate::instruction_files::tokens::count_text(&text, tokenizer) as i64;
        let line_count = text.lines().count() as i64;

        revisions.push(ClaudeMdRevision {
            commit_sha: sha,
            commit_ts: epoch,
            byte_size,
            token_count,
            line_count,
        });
    }

    revisions
}

// ---------------------------------------------------------------------------
// Discovery
// ---------------------------------------------------------------------------

/// Discover all git-tracked CLAUDE.md files across known projects.
///
/// Per project: probes `"CLAUDE.md"` and `".claude/CLAUDE.md"`.
/// Also probes `~/.claude/CLAUDE.md` if `~/.claude` is a git repo.
/// Deduplicates by `(canonical repo_root, rel_path)`.
pub fn discover_tracked_claude_md(
    conn: &Connection,
    projects_override: &[PathBuf],
) -> Vec<TrackedClaudeMd> {
    let project_paths =
        crate::skills::projects::discover_project_paths(conn, projects_override);

    let mut seen: HashSet<(PathBuf, PathBuf)> = HashSet::new();
    let mut results: Vec<TrackedClaudeMd> = Vec::new();

    // Helper that adds a candidate if tracked and not yet seen.
    let try_add =
        |repo_root: PathBuf, rel_path: PathBuf, label: String, seen: &mut HashSet<(PathBuf, PathBuf)>, results: &mut Vec<TrackedClaudeMd>| {
            if !git_file_is_tracked(&repo_root, &rel_path) {
                return;
            }
            let canonical = repo_root.canonicalize().unwrap_or_else(|_| repo_root.clone());
            let key = (canonical, rel_path.clone());
            if seen.insert(key) {
                results.push(TrackedClaudeMd {
                    repo_root,
                    rel_path,
                    label,
                });
            }
        };

    // Per-project files.
    for project in &project_paths {
        let repo_root = match git_toplevel(project) {
            Some(r) => r,
            None => continue,
        };
        let repo_name = repo_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let candidates = [
            (PathBuf::from("CLAUDE.md"), format!("{repo_name}/CLAUDE.md")),
            (
                PathBuf::from(".claude/CLAUDE.md"),
                format!("{repo_name}/.claude/CLAUDE.md"),
            ),
        ];
        for (rel, label) in candidates {
            try_add(repo_root.clone(), rel, label, &mut seen, &mut results);
        }
    }

    // ~/.claude/CLAUDE.md
    if let Some(home) = dirs::home_dir() {
        let claude_dir = home.join(".claude");
        if let Some(repo_root) = git_toplevel(&claude_dir) {
            try_add(
                repo_root,
                PathBuf::from("CLAUDE.md"),
                "~/.claude/CLAUDE.md".to_string(),
                &mut seen,
                &mut results,
            );
        }
    }

    results
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    fn init_git_repo(dir: &Path) {
        for args in [
            vec!["init"],
            vec!["config", "user.email", "test@test.com"],
            vec!["config", "user.name", "Test"],
        ] {
            Command::new("git")
                .args(&args)
                .current_dir(dir)
                .output()
                .unwrap();
        }
    }

    fn git_commit(dir: &Path, msg: &str) {
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", msg, "--allow-empty"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn walk_returns_empty_for_non_repo() {
        let dir = TempDir::new().unwrap();
        let revs = walk_claude_md_history(
            dir.path(),
            &PathBuf::from("CLAUDE.md"),
            None,
            Tokenizer::Heuristic,
            100,
        );
        assert!(revs.is_empty(), "non-repo should produce empty vec");
    }

    #[test]
    fn walk_returns_revisions_oldest_first() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());

        // 3 commits, each with a different CLAUDE.md content.
        for i in 1u8..=3 {
            std::fs::write(dir.path().join("CLAUDE.md"), format!("version {i}")).unwrap();
            git_commit(dir.path(), &format!("commit {i}"));
        }

        let revs = walk_claude_md_history(
            dir.path(),
            &PathBuf::from("CLAUDE.md"),
            None,
            Tokenizer::Heuristic,
            100,
        );
        assert_eq!(revs.len(), 3, "expected 3 revisions, got {}", revs.len());
        // Verify timestamps are non-decreasing (oldest first).
        for w in revs.windows(2) {
            assert!(
                w[0].commit_ts <= w[1].commit_ts,
                "revisions not in ascending order: {} > {}",
                w[0].commit_ts,
                w[1].commit_ts
            );
        }
    }

    #[test]
    fn walk_stops_at_since_sha() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());

        // 3 commits.
        for i in 1u8..=3 {
            std::fs::write(dir.path().join("CLAUDE.md"), format!("version {i}")).unwrap();
            git_commit(dir.path(), &format!("commit {i}"));
        }

        // Get all revisions first to find the middle SHA.
        let all_revs = walk_claude_md_history(
            dir.path(),
            &PathBuf::from("CLAUDE.md"),
            None,
            Tokenizer::Heuristic,
            100,
        );
        assert_eq!(all_revs.len(), 3);

        let middle_sha = all_revs[1].commit_sha.clone(); // index 1 = second oldest

        // Pass the middle SHA as since_sha — expect only the newest (1 result).
        let filtered = walk_claude_md_history(
            dir.path(),
            &PathBuf::from("CLAUDE.md"),
            Some(&middle_sha),
            Tokenizer::Heuristic,
            100,
        );
        assert_eq!(
            filtered.len(),
            1,
            "expected 1 revision after since_sha, got {}",
            filtered.len()
        );
        assert_eq!(filtered[0].commit_sha, all_revs[2].commit_sha);
    }

    #[test]
    fn git_file_is_tracked_when_tracked() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());
        std::fs::write(dir.path().join("CLAUDE.md"), "hello").unwrap();
        git_commit(dir.path(), "add CLAUDE.md");

        assert!(
            git_file_is_tracked(dir.path(), &PathBuf::from("CLAUDE.md")),
            "tracked file should return true"
        );
    }

    #[test]
    fn git_file_is_tracked_when_not_tracked() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());
        // Don't add any file.
        assert!(
            !git_file_is_tracked(dir.path(), &PathBuf::from("CLAUDE.md")),
            "untracked file should return false"
        );
    }
}
