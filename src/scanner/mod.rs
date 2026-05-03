pub mod classifier;
pub mod cowork;
pub mod db;
pub mod oneshot;
pub mod parser;
pub mod provider;
pub mod providers;
#[cfg(test)]
mod tests;
pub mod usage_limits;
pub mod watcher;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use crate::models::ScanResult;
use db::{
    compute_tool_events_for_turn, delete_processed_file, delete_tool_events_by_source_path,
    delete_tool_invocations_by_source_path, delete_turns_by_source_path, get_processed_file,
    init_db, insert_tool_events, insert_tool_invocations, insert_turns, list_processed_files,
    open_db, recompute_session_totals, sync_session_titles, upsert_processed_file, upsert_sessions,
};
use parser::{PROVIDER_CLAUDE, PROVIDER_CODEX, PROVIDER_XCODE, aggregate_sessions};
use provider::Provider;
use usage_limits::{discover_usage_limits_files, insert_usage_limits_snapshot, parse_usage_limits};

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

    // Collect `(provider, source_path)` tuples. Two inputs:
    //   - Explicit override via `--projects-dir`: walk each dir for .jsonl
    //     files and tag by `provider_for_dir`.
    //   - Registry default: consume `SessionSource`s directly from each
    //     provider's `discover_sessions()` and keep the provider object
    //     attached for the parse step.
    let providers = providers::all();
    let mut source_files: Vec<(Arc<dyn Provider>, PathBuf)> = Vec::new();
    if let Some(dirs) = projects_dirs {
        let provider_lookup: std::collections::HashMap<&'static str, Arc<dyn Provider>> = providers
            .iter()
            .map(|provider| (provider.name(), Arc::clone(provider)))
            .collect();
        for dir in dirs {
            if !dir.exists() {
                continue;
            }
            let provider_name = provider_for_dir(&dir);
            let Some(provider) = provider_lookup.get(provider_name).cloned() else {
                warn!("provider {}: not registered", provider_name);
                continue;
            };
            if verbose {
                info!("Scanning {} {} ...", provider.name(), dir.display());
            }
            for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
                if entry.path().extension().is_some_and(|ext| ext == "jsonl") {
                    source_files.push((Arc::clone(&provider), entry.path().to_path_buf()));
                }
            }
        }
    } else {
        for provider in providers {
            let sources = match provider.discover_sessions() {
                Ok(s) => s,
                Err(e) => {
                    warn!(
                        "provider {}: discover_sessions failed: {}",
                        provider.name(),
                        e
                    );
                    continue;
                }
            };
            if verbose && !sources.is_empty() {
                info!(
                    "Scanning {} ({} sessions) ...",
                    provider.name(),
                    sources.len()
                );
            }
            for src in sources {
                source_files.push((Arc::clone(&provider), src.path));
            }
        }
    }
    source_files.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.name().cmp(b.0.name())));
    let current_files: HashSet<String> = source_files
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
        delete_tool_events_by_source_path(&conn, &stale_path)?;
        delete_processed_file(&conn, &stale_path)?;
        any_changes = true;
    }

    for (provider, filepath) in &source_files {
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
            delete_tool_events_by_source_path(&conn, &filepath_str)?;
            any_changes = true;
        }

        let parsed = provider.parse_source(filepath, 0);

        if !parsed.turns.is_empty() || !parsed.session_metas.is_empty() {
            let sessions = aggregate_sessions(&parsed.session_metas, &parsed.turns);
            let session_ids: Vec<String> = sessions
                .iter()
                .map(|session| session.session_id.clone())
                .collect();
            upsert_sessions(&conn, &sessions)?;
            insert_turns(&conn, &parsed.turns)?;
            insert_tool_invocations(
                &conn,
                &parsed.turns,
                &parsed.tool_results,
                &parsed.tool_error_texts,
                &parsed.tool_input_jsons,
            )?;
            sync_session_titles(&conn, &session_ids, &parsed.session_titles)?;

            // Phase 12: compute and insert tool-event cost attribution rows.
            // Look up the project name from the sessions we just upserted so
            // tool_events.project is populated even on first-scan.
            let session_project: std::collections::HashMap<String, String> = sessions
                .iter()
                .map(|s| (s.session_id.clone(), s.project_name.clone()))
                .collect();
            let tool_events: Vec<crate::models::ToolEvent> = parsed
                .turns
                .iter()
                .flat_map(|t| {
                    let project = session_project
                        .get(&t.session_id)
                        .map(|s| s.as_str())
                        .unwrap_or("");
                    compute_tool_events_for_turn(t, project)
                })
                .collect();
            insert_tool_events(&conn, &tool_events)?;

            result.sessions += sessions.len();
            result.turns += parsed.turns.len();
            any_changes = true;
        }

        if is_new {
            result.new += 1;
        } else {
            result.updated += 1;
        }

        upsert_processed_file(&conn, &filepath_str, mtime, parsed.progress_marker)?;
    }

    // Recompute session totals from turns for dedup correctness
    if any_changes {
        recompute_session_totals(&conn)?;
    }

    // Phase 20: ingest usage-limits files from ~/.claude/ subtree.
    // Runs on every scan (cheap: no re-reading already-seen identical values).
    let claude_dir = home_dir().join(".claude");
    let usage_limit_files = discover_usage_limits_files(&claude_dir);
    if !usage_limit_files.is_empty() {
        let now_iso = chrono::Utc::now().to_rfc3339();
        for ulf in &usage_limit_files {
            if let Some(snapshot) = parse_usage_limits(ulf)
                && let Err(e) = insert_usage_limits_snapshot(&conn, &snapshot, &now_iso)
            {
                warn!("usage_limits: insert failed for {}: {}", ulf.display(), e);
            }
        }
        if verbose {
            info!(
                "usage_limits: processed {} file(s)",
                usage_limit_files.len()
            );
        }
    }

    if verbose {
        info!(
            "Scan complete: {} new, {} updated, {} skipped, {} turns",
            result.new, result.updated, result.skipped, result.turns
        );
    }

    Ok(result)
}
