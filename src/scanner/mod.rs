pub mod classifier;
pub mod db;
pub mod parser;
pub mod provider;
pub mod providers;
#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use crate::models::ScanResult;
use db::{
    delete_processed_file, delete_tool_invocations_by_source_path, delete_turns_by_source_path,
    get_processed_file, init_db, insert_tool_invocations, insert_turns, list_processed_files,
    open_db, recompute_session_totals, sync_session_titles, upsert_processed_file, upsert_sessions,
};
use parser::{
    PROVIDER_CLAUDE, PROVIDER_CODEX, PROVIDER_XCODE, aggregate_sessions, parse_jsonl_file,
};

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn provider_for_dir(path: &Path) -> &'static str {
    let normalized = path.to_string_lossy().replace('\\', "/");
    if normalized.contains("/.codex/") {
        PROVIDER_CODEX
    } else if normalized.contains("/CodingAssistant/") {
        PROVIDER_XCODE
    } else {
        PROVIDER_CLAUDE
    }
}

pub fn default_db_path() -> PathBuf {
    home_dir().join(".claude").join("usage.db")
}

pub fn scan(
    projects_dirs: Option<Vec<PathBuf>>,
    db_path: &Path,
    verbose: bool,
) -> Result<ScanResult> {
    let conn = open_db(db_path)?;
    init_db(&conn)?;

    // Collect `(provider_name, jsonl_file_path)` tuples. Two inputs:
    //   - Explicit override via `--projects-dir`: walk each dir for .jsonl
    //     files and tag by `provider_for_dir`.
    //   - Registry default: consume `SessionSource`s directly from each
    //     provider's `discover_sessions()`. The source already carries
    //     both the file path and the provider slug, so no second walk
    //     and no parent-directory round-trip is needed.
    let mut jsonl_files: Vec<(String, PathBuf)> = Vec::new();
    if let Some(dirs) = projects_dirs {
        for dir in dirs {
            if !dir.exists() {
                continue;
            }
            let provider = provider_for_dir(&dir).to_string();
            if verbose {
                info!("Scanning {} {} ...", provider, dir.display());
            }
            for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
                if entry.path().extension().is_some_and(|ext| ext == "jsonl") {
                    jsonl_files.push((provider.clone(), entry.path().to_path_buf()));
                }
            }
        }
    } else {
        for p in providers::all() {
            let provider = p.name().to_string();
            let sources = match p.discover_sessions() {
                Ok(s) => s,
                Err(e) => {
                    warn!("provider {}: discover_sessions failed: {}", provider, e);
                    continue;
                }
            };
            if verbose && !sources.is_empty() {
                info!("Scanning {} ({} sessions) ...", provider, sources.len());
            }
            for src in sources {
                debug_assert_eq!(src.provider_name, p.name());
                jsonl_files.push((provider.clone(), src.path));
            }
        }
    }
    jsonl_files.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    let current_files: HashSet<String> = jsonl_files
        .iter()
        .map(|(_, path)| path.to_string_lossy().to_string())
        .collect();

    let mut result = ScanResult::default();
    let mut any_changes = false;

    for stale_path in list_processed_files(&conn)?
        .into_iter()
        .filter(|path| !current_files.contains(path))
    {
        debug!("[DEL] {}", stale_path);
        delete_turns_by_source_path(&conn, &stale_path)?;
        delete_tool_invocations_by_source_path(&conn, &stale_path)?;
        delete_processed_file(&conn, &stale_path)?;
        any_changes = true;
    }

    for (provider, filepath) in &jsonl_files {
        let filepath_str = filepath.to_string_lossy().to_string();
        let mtime = match std::fs::metadata(filepath) {
            Ok(m) => m
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0),
            Err(_) => continue,
        };

        let prev = get_processed_file(&conn, &filepath_str)?;

        if let Some((prev_mtime, _)) = prev
            && (prev_mtime - mtime).abs() < 0.01
        {
            result.skipped += 1;
            continue;
        }

        let is_new = prev.is_none();

        debug!("[{}] {}", if is_new { "NEW" } else { "UPD" }, filepath_str);

        if !is_new {
            delete_turns_by_source_path(&conn, &filepath_str)?;
            delete_tool_invocations_by_source_path(&conn, &filepath_str)?;
            any_changes = true;
        }

        let parsed = parse_jsonl_file(provider, filepath, 0);

        if !parsed.turns.is_empty() || !parsed.session_metas.is_empty() {
            let sessions = aggregate_sessions(&parsed.session_metas, &parsed.turns);
            let session_ids: Vec<String> = sessions
                .iter()
                .map(|session| session.session_id.clone())
                .collect();
            upsert_sessions(&conn, &sessions)?;
            insert_turns(&conn, &parsed.turns)?;
            insert_tool_invocations(&conn, &parsed.turns, &parsed.tool_results)?;
            sync_session_titles(&conn, &session_ids, &parsed.session_titles)?;

            result.sessions += sessions.len();
            result.turns += parsed.turns.len();
            any_changes = true;
        }

        if is_new {
            result.new += 1;
        } else {
            result.updated += 1;
        }

        upsert_processed_file(&conn, &filepath_str, mtime, parsed.line_count)?;
    }

    // Recompute session totals from turns for dedup correctness
    if any_changes {
        recompute_session_totals(&conn)?;
    }

    if verbose {
        info!(
            "Scan complete: {} new, {} updated, {} skipped, {} turns",
            result.new, result.updated, result.skipped, result.turns
        );
    }

    Ok(result)
}
