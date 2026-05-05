pub mod agent_sessions;
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
    compute_tool_events_for_turn, delete_agent_sessions_by_path, delete_processed_file,
    delete_tool_events_by_source_path, delete_tool_invocations_by_source_path,
    delete_turns_by_source_path, get_processed_file, init_db, insert_agent_session,
    insert_tool_events, insert_tool_invocations, insert_turns, list_processed_files, open_db,
    recompute_session_totals, sync_session_titles, upsert_codex_plan_daily, upsert_processed_file,
    upsert_sessions,
};
use parser::{PROVIDER_CLAUDE, PROVIDER_CODEX, PROVIDER_XCODE, aggregate_sessions};
use provider::Provider;
use usage_limits::{discover_usage_limits_files, insert_usage_limits_snapshot, parse_usage_limits};

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

// ---------------------------------------------------------------------------
// Agent-sessions incremental ingest
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct AgentScanStats {
    pub files_seen: usize,
    pub files_parsed: usize,
    pub files_skipped_unchanged: usize,
    pub files_failed_sanity: usize,
    pub records_inserted: usize,
    pub stale_files_removed: usize,
}

fn scan_agent_sessions(
    conn: &rusqlite::Connection,
    projects_dirs: Vec<PathBuf>,
) -> Result<AgentScanStats> {
    use agent_sessions::{DiscoveryConfig, discover_files, parse_one};

    let mut stats = AgentScanStats::default();

    let cfg = DiscoveryConfig::from_projects_dirs(projects_dirs);
    let discovered = discover_files(&cfg);

    // Use a prefixed key in processed_files so agent-session entries don't
    // collide with the main scanner's entries for the same .jsonl paths.
    // (The main scanner walks all .jsonl files under projects_dirs, which
    // includes subagent files.  Without a distinct key the main scanner would
    // set the mtime entry first and this pass would skip every file.)
    fn agent_pf_key(path: &std::path::Path) -> String {
        format!("agent_session:{}", path.to_string_lossy())
    }

    // Build set of currently-discovered pf-keys for stale detection.
    let current_keys: HashSet<String> = discovered.iter().map(|f| agent_pf_key(&f.path)).collect();

    // Remove stale agent_sessions rows for files that no longer exist on disk.
    let all_processed = list_processed_files(conn)?;
    for key in all_processed {
        if !key.starts_with("agent_session:") {
            continue;
        }
        if !current_keys.contains(&key) {
            // Recover the raw path by stripping the prefix.
            let raw_path = key.strip_prefix("agent_session:").unwrap_or(&key);
            debug!("[AGENT-DEL] {}", raw_path);
            let _ = delete_agent_sessions_by_path(conn, raw_path);
            delete_processed_file(conn, &key)?;
            stats.stale_files_removed += 1;
        }
    }

    // Process each discovered file.
    for file in &discovered {
        stats.files_seen += 1;
        let path_str = file.path.to_string_lossy().to_string();
        let pf_key = agent_pf_key(&file.path);

        let mtime = match std::fs::metadata(&file.path) {
            Ok(m) => m
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0),
            Err(e) => {
                warn!("agent_sessions: stat failed for {}: {}", path_str, e);
                continue;
            }
        };

        let prev = get_processed_file(conn, &pf_key)?;
        if let Some((prev_mtime, _)) = prev
            && (prev_mtime - mtime).abs() < 0.01
        {
            stats.files_skipped_unchanged += 1;
            continue;
        }

        // Delete prior rows for this source before reinserting.
        let _ = delete_agent_sessions_by_path(conn, &path_str);

        match parse_one(file) {
            Ok(Some(rec)) => {
                insert_agent_session(conn, &rec).map_err(anyhow::Error::from)?;
                stats.records_inserted += 1;
                stats.files_parsed += 1;
            }
            Ok(None) => {
                // Empty / failed sanity gate — still update processed_files so
                // we don't retry on every scan.
                stats.files_failed_sanity += 1;
            }
            Err(e) => {
                warn!("agent_sessions: parse error for {}: {:#}", path_str, e);
                stats.files_failed_sanity += 1;
            }
        }

        upsert_processed_file(conn, &pf_key, mtime, 0)?;
    }

    Ok(stats)
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
    // Retain the explicit projects-dirs list so the agent-sessions pass can
    // use the same roots.  When `projects_dirs` is None the agent-sessions
    // discovery falls back to its own default roots (home_dir/.claude/projects).
    let explicit_dirs: Option<Vec<PathBuf>> = projects_dirs.clone();
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
    // Feature 1: accumulate Codex plan data across all Codex files.
    let mut all_codex_snapshots: Vec<crate::models::CodexPlanSnapshot> = Vec::new();
    let mut all_codex_limit_hits: Vec<crate::scanner::parser::CodexLimitHit> = Vec::new();

    for stale_path in list_processed_files(&conn)?
        .into_iter()
        .filter(|path| !current_files.contains(path))
    {
        // Skip agent-session entries — they use the "agent_session:" prefix and
        // are managed exclusively by scan_agent_sessions below.
        if stale_path.starts_with("agent_session:") {
            continue;
        }
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

        // Feature 1: accumulate Codex plan snapshots from every Codex file.
        if provider.name() == PROVIDER_CODEX {
            all_codex_snapshots.extend(parsed.codex_plan_snapshots.iter().cloned());
            all_codex_limit_hits.extend(parsed.codex_limit_hits.iter().cloned());
        }

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

    // Feature 1: aggregate and persist Codex plan daily rows.
    // Run unconditionally (cheap upsert; always reflects current scan state).
    if !all_codex_snapshots.is_empty() || !all_codex_limit_hits.is_empty() {
        let daily_rows = providers::codex::aggregate_codex_plan_daily(
            &all_codex_snapshots,
            &all_codex_limit_hits,
            0, // UTC; client tz applied at query time
        );
        for row in &daily_rows {
            if let Err(e) = upsert_codex_plan_daily(&conn, row) {
                warn!("codex_plan_daily upsert failed for {}: {}", row.day, e);
            }
        }
        if verbose && !daily_rows.is_empty() {
            info!(
                "codex_plan: aggregated {} day(s) from {} snapshots",
                daily_rows.len(),
                all_codex_snapshots.len()
            );
        }
    }

    // Phase 21: ingest agent-sessions (subagent JSONL + task-tool outputs).
    // Runs after main session ingest so agent_sessions.session_id can soft-FK
    // existing sessions rows.  Uses processed_files for mtime-based incremental
    // ingest identical to the main pipeline.
    {
        let agent_dirs =
            explicit_dirs.unwrap_or_else(|| vec![home_dir().join(".claude").join("projects")]);
        match scan_agent_sessions(&conn, agent_dirs) {
            Ok(stats) => {
                info!(
                    files_seen = stats.files_seen,
                    inserted = stats.records_inserted,
                    skipped_unchanged = stats.files_skipped_unchanged,
                    failed = stats.files_failed_sanity,
                    stale_removed = stats.stale_files_removed,
                    "agent_sessions scan complete"
                );
            }
            Err(e) => {
                warn!("agent_sessions scan failed: {:#}", e);
            }
        }
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
