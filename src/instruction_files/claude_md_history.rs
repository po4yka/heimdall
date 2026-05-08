use std::path::PathBuf;

use rusqlite::Connection;
use tracing::warn;

use crate::skills::Tokenizer;

use super::git_history::{discover_tracked_claude_md, walk_claude_md_history};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub struct RefreshOutcome {
    pub files_visited: usize,
    pub revisions_inserted: usize,
    pub errors: Vec<String>,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Refresh the `claude_md_history` table.
///
/// Discovers all git-tracked CLAUDE.md files, determines which commits have
/// not yet been stored, walks their history, and inserts new revisions.
///
/// When `rebuild` is `true` the existing rows for each file are deleted first,
/// causing a full re-import.
///
/// Failures are collected into `RefreshOutcome::errors` — the function itself
/// never propagates errors.
pub fn refresh_claude_md_history(
    conn: &Connection,
    projects_override: &[PathBuf],
    tokenizer: Tokenizer,
    rebuild: bool,
) -> RefreshOutcome {
    let files = discover_tracked_claude_md(conn, projects_override);
    let mut outcome = RefreshOutcome {
        files_visited: 0,
        revisions_inserted: 0,
        errors: Vec::new(),
    };

    for file in &files {
        outcome.files_visited += 1;

        let repo_root_str = file.repo_root.to_string_lossy().into_owned();
        let rel_path_str = file.rel_path.to_string_lossy().into_owned();

        // Optionally wipe existing data for a full rebuild.
        if rebuild
            && let Err(e) = crate::scanner::db::delete_claude_md_history_for_file(
                conn,
                &repo_root_str,
                &rel_path_str,
            )
        {
            warn!(
                "claude_md history: delete failed for {}/{}: {e}",
                repo_root_str, rel_path_str
            );
            // Continue — a failed delete is not fatal.
        }

        // Find the most recent commit already stored so we can skip it.
        let since_sha =
            crate::scanner::db::last_claude_md_commit_sha(conn, &repo_root_str, &rel_path_str);

        let revs = walk_claude_md_history(
            &file.repo_root,
            &file.rel_path,
            since_sha.as_deref(),
            tokenizer,
            1000,
        );

        match crate::scanner::db::insert_claude_md_revisions(
            conn,
            &repo_root_str,
            &rel_path_str,
            &revs,
        ) {
            Ok(n) => outcome.revisions_inserted += n,
            Err(e) => {
                outcome.errors.push(format!(
                    "insert failed for {}/{}: {e}",
                    repo_root_str, rel_path_str
                ));
            }
        }
    }

    outcome
}
