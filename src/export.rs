//! ROADMAP Phase 1 — multi-period export (CSV / JSON / JSONL).
//!
//! Exposes a `claude-usage-tracker export` path that aggregates turns
//! by (date, provider, project, model) over a fixed time window and
//! writes them to disk in one of three formats.
//!
//! Storage stays in integer nanos; floats are derived for display only.

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result, anyhow};
use chrono::{Duration, Local, NaiveDate};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::jq as jq_mod;
use crate::scanner::db::open_db;

/// Returns `true` when the output path means "write to stdout" (the path is literally `-`).
fn is_stdout(path: &Path) -> bool {
    path == Path::new("-")
}

/// Open a `BufWriter` over either stdout (when `path` is `-`) or the given file path.
///
/// The caller receives a `Box<dyn Write>` so the rest of the code is sink-agnostic.
fn open_writer(path: &Path) -> Result<Box<dyn Write>> {
    if is_stdout(path) {
        Ok(Box::new(std::io::BufWriter::new(std::io::stdout())))
    } else {
        let f =
            std::fs::File::create(path).with_context(|| format!("creating {}", path.display()))?;
        Ok(Box::new(std::io::BufWriter::new(f)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Csv,
    Json,
    Jsonl,
}

impl FromStr for ExportFormat {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "csv" => Ok(Self::Csv),
            "json" => Ok(Self::Json),
            "jsonl" => Ok(Self::Jsonl),
            other => Err(anyhow!(
                "unknown --format: {} (expected csv|json|jsonl)",
                other
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportPeriod {
    Today,
    Week,
    Month,
    Year,
    All,
}

impl FromStr for ExportPeriod {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "today" => Ok(Self::Today),
            "week" => Ok(Self::Week),
            "month" => Ok(Self::Month),
            "year" => Ok(Self::Year),
            "all" => Ok(Self::All),
            other => Err(anyhow!(
                "unknown --period: {} (expected today|week|month|year|all)",
                other
            )),
        }
    }
}

/// Inclusive date bounds for a period; None means "no bound" (all time).
fn period_bounds(period: ExportPeriod, today: NaiveDate) -> Option<(NaiveDate, NaiveDate)> {
    let end = today;
    let start = match period {
        ExportPeriod::Today => today,
        ExportPeriod::Week => today - Duration::days(6),
        ExportPeriod::Month => today - Duration::days(29),
        ExportPeriod::Year => today - Duration::days(364),
        ExportPeriod::All => return None,
    };
    Some((start, end))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportRow {
    pub date: String,
    pub provider: String,
    /// Raw project slug — canonical, scriptable, never aliased.
    pub project: String,
    /// Human-readable alias for `project`.  Equals `project` when no alias is configured.
    pub project_display_name: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read: i64,
    pub cache_write: i64,
    /// Nanos — preserved as integer across all output formats.
    pub cost_usd_nanos: i64,
    /// Display-only float. Never used for math.
    pub cost_usd_display: f64,
}

pub struct ExportOptions {
    pub format: ExportFormat,
    pub period: ExportPeriod,
    pub output: PathBuf,
    pub provider: Option<String>,
    pub project: Option<String>,
    /// Optional jq filter applied to each row's JSON representation.
    /// When set, only JSON and JSONL outputs are produced (CSV is unaffected).
    pub jq: Option<String>,
    /// Map of raw project slug → display name.  Applied to `project_display_name` column.
    pub project_aliases: HashMap<String, String>,
}

pub fn run_export(db_path: &Path, opts: &ExportOptions) -> Result<usize> {
    if !db_path.exists() {
        anyhow::bail!(
            "Database not found: {}. Run `claude-usage-tracker scan` first.",
            db_path.display()
        );
    }
    let conn = open_db(db_path)?;
    let today = Local::now().date_naive();
    let rows = query_rows(
        &conn,
        opts.period,
        today,
        opts.provider.as_deref(),
        opts.project.as_deref(),
        &opts.project_aliases,
    )?;

    if let Some(filter) = &opts.jq {
        // Apply jq filter per-row for JSON/JSONL; for CSV output jq is ignored.
        // Compile the filter once; reuse across all rows (FIX 3).
        match opts.format {
            ExportFormat::Csv => {
                // jq is ignored for CSV — write normally.
                write_rows(&rows, opts.format, &opts.output)
                    .with_context(|| format!("writing {}", opts.output.display()))?;
            }
            ExportFormat::Json => {
                // Build JSON array, apply filter to whole array value.
                let compiled = jq_mod::CompiledJqFilter::compile(filter)
                    .with_context(|| "compiling --jq filter")?;
                let arr = serde_json::to_value(&rows)?;
                let result = compiled
                    .apply(&arr)
                    .with_context(|| "applying --jq filter to JSON export")?;
                let out_str = match result {
                    jq_mod::JqResult::Empty => String::new(),
                    jq_mod::JqResult::Single(s) => s,
                    jq_mod::JqResult::Multiple(vs) => vs.join("\n"),
                };
                let mut w = open_writer(&opts.output)?;
                if !out_str.is_empty() {
                    writeln!(w, "{out_str}")?;
                }
            }
            ExportFormat::Jsonl => {
                // Compile filter once, apply per row.
                let compiled = jq_mod::CompiledJqFilter::compile(filter)
                    .with_context(|| "compiling --jq filter")?;
                let mut w = open_writer(&opts.output)?;
                for row in &rows {
                    let row_val = serde_json::to_value(row)?;
                    let result = compiled
                        .apply(&row_val)
                        .with_context(|| "applying --jq filter to JSONL export row")?;
                    match result {
                        jq_mod::JqResult::Empty => {}
                        jq_mod::JqResult::Single(s) => writeln!(w, "{s}")?,
                        jq_mod::JqResult::Multiple(vs) => {
                            for v in vs {
                                writeln!(w, "{v}")?;
                            }
                        }
                    }
                }
            }
        }
    } else {
        write_rows(&rows, opts.format, &opts.output)
            .with_context(|| format!("writing {}", opts.output.display()))?;
    }

    Ok(rows.len())
}

fn query_rows(
    conn: &Connection,
    period: ExportPeriod,
    today: NaiveDate,
    provider: Option<&str>,
    project: Option<&str>,
    aliases: &HashMap<String, String>,
) -> Result<Vec<ExportRow>> {
    let bounds = period_bounds(period, today);

    let mut sql = String::from(
        "SELECT substr(t.timestamp, 1, 10) AS date, \
                t.provider, \
                COALESCE(s.project_name, '') AS project, \
                COALESCE(t.model, 'unknown') AS model, \
                COALESCE(SUM(t.input_tokens), 0), \
                COALESCE(SUM(t.output_tokens), 0), \
                COALESCE(SUM(t.cache_read_tokens), 0), \
                COALESCE(SUM(t.cache_creation_tokens), 0), \
                COALESCE(SUM(t.estimated_cost_nanos), 0) \
         FROM turns t \
         LEFT JOIN sessions s ON s.session_id = t.session_id \
         WHERE 1=1 ",
    );
    let mut params: Vec<String> = Vec::new();
    if let Some((start, end)) = bounds {
        sql.push_str("AND substr(t.timestamp, 1, 10) BETWEEN ? AND ? ");
        params.push(start.format("%Y-%m-%d").to_string());
        params.push(end.format("%Y-%m-%d").to_string());
    }
    if let Some(p) = provider {
        sql.push_str("AND t.provider = ? ");
        params.push(p.to_string());
    }
    if let Some(p) = project {
        sql.push_str("AND s.project_name = ? ");
        params.push(p.to_string());
    }
    sql.push_str(
        "GROUP BY date, t.provider, project, t.model ORDER BY date, t.provider, project, t.model",
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            let cost_nanos: i64 = row.get(8)?;
            let project: String = row.get(2)?;
            Ok((
                project,
                cost_nanos,
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let result = rows
        .into_iter()
        .map(
            |(
                project,
                cost_nanos,
                date,
                provider,
                model,
                input_tokens,
                output_tokens,
                cache_read,
                cache_write,
            )| {
                let project_display_name = aliases
                    .get(&project)
                    .cloned()
                    .unwrap_or_else(|| project.clone());
                ExportRow {
                    date,
                    provider,
                    project_display_name,
                    project,
                    model,
                    input_tokens,
                    output_tokens,
                    cache_read,
                    cache_write,
                    cost_usd_nanos: cost_nanos,
                    cost_usd_display: cost_nanos as f64 / 1_000_000_000.0,
                }
            },
        )
        .collect();
    Ok(result)
}

fn write_rows(rows: &[ExportRow], format: ExportFormat, output: &Path) -> Result<()> {
    match format {
        ExportFormat::Csv => write_csv(rows, output),
        ExportFormat::Json => write_json(rows, output),
        ExportFormat::Jsonl => write_jsonl(rows, output),
    }
}

fn write_csv(rows: &[ExportRow], output: &Path) -> Result<()> {
    // csv::Writer::from_path cannot write to stdout; fall back to a generic writer.
    let w = open_writer(output)?;
    let mut wtr = csv::Writer::from_writer(w);
    for r in rows {
        wtr.serialize(r)?;
    }
    wtr.flush()?;
    Ok(())
}

fn write_json(rows: &[ExportRow], output: &Path) -> Result<()> {
    let mut w = open_writer(output)?;
    serde_json::to_writer_pretty(&mut w, rows)?;
    Ok(())
}

fn write_jsonl(rows: &[ExportRow], output: &Path) -> Result<()> {
    let mut w = open_writer(output)?;
    for r in rows {
        serde_json::to_writer(&mut w, r)?;
        writeln!(w)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_row(date: &str, provider: &str, nanos: i64) -> ExportRow {
        ExportRow {
            date: date.into(),
            provider: provider.into(),
            project: "user/proj".into(),
            project_display_name: "user/proj".into(),
            model: "claude-sonnet-4-6".into(),
            input_tokens: 100,
            output_tokens: 50,
            cache_read: 10,
            cache_write: 0,
            cost_usd_nanos: nanos,
            cost_usd_display: nanos as f64 / 1_000_000_000.0,
        }
    }

    #[test]
    fn format_parses_case_insensitively() {
        assert_eq!(ExportFormat::from_str("CSV").unwrap(), ExportFormat::Csv);
        assert_eq!(ExportFormat::from_str("Json").unwrap(), ExportFormat::Json);
        assert_eq!(
            ExportFormat::from_str("jsonl").unwrap(),
            ExportFormat::Jsonl
        );
        assert!(ExportFormat::from_str("xml").is_err());
    }

    #[test]
    fn period_parses_case_insensitively() {
        assert_eq!(
            ExportPeriod::from_str("Today").unwrap(),
            ExportPeriod::Today
        );
        assert_eq!(ExportPeriod::from_str("ALL").unwrap(), ExportPeriod::All);
        assert!(ExportPeriod::from_str("forever").is_err());
    }

    #[test]
    fn period_bounds_cover_expected_spans() {
        let today = NaiveDate::from_ymd_opt(2026, 4, 17).unwrap();
        assert_eq!(
            period_bounds(ExportPeriod::Today, today),
            Some((today, today))
        );
        let (start, end) = period_bounds(ExportPeriod::Week, today).unwrap();
        assert_eq!(end - start, Duration::days(6));
        let (start, end) = period_bounds(ExportPeriod::Month, today).unwrap();
        assert_eq!(end - start, Duration::days(29));
        let (start, end) = period_bounds(ExportPeriod::Year, today).unwrap();
        assert_eq!(end - start, Duration::days(364));
        assert_eq!(period_bounds(ExportPeriod::All, today), None);
    }

    #[test]
    fn csv_round_trip_preserves_rows_and_integer_nanos() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("out.csv");
        let rows = vec![
            sample_row("2026-04-17", "claude", 1_234_000_000),
            sample_row("2026-04-16", "codex", 5_678_000_000),
        ];
        write_csv(&rows, &path).unwrap();

        let mut rdr = csv::Reader::from_path(&path).unwrap();
        let parsed: Vec<ExportRow> = rdr.deserialize().map(|r| r.unwrap()).collect();
        assert_eq!(parsed, rows);
        // Assert nanos survive as the exact integer via text round-trip.
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(
            text.contains("1234000000"),
            "csv should encode nanos as integer: {text}"
        );
        assert!(text.contains("5678000000"));
    }

    #[test]
    fn json_round_trip_preserves_rows() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("out.json");
        let rows = vec![sample_row("2026-04-17", "claude", 1_234_000_000)];
        write_json(&rows, &path).unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        let parsed: Vec<ExportRow> = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed, rows);
        assert!(text.contains("\"cost_usd_nanos\": 1234000000"));
    }

    #[test]
    fn jsonl_writes_one_object_per_line() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("out.jsonl");
        let rows = vec![
            sample_row("2026-04-17", "claude", 100),
            sample_row("2026-04-16", "codex", 200),
        ];
        write_jsonl(&rows, &path).unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        let parsed: Vec<ExportRow> = lines
            .iter()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(parsed, rows);
    }

    fn seed_db(db_path: &Path) {
        let conn = open_db(db_path).unwrap();
        crate::scanner::db::init_db(&conn).unwrap();
        // Seed two sessions and three turns spanning three dates.
        conn.execute(
            "INSERT INTO sessions (session_id, provider, project_name, project_slug,
                                   first_timestamp, last_timestamp)
             VALUES ('claude:s1', 'claude', 'user/alpha', 'alpha', '2026-04-17', '2026-04-17'),
                    ('codex:s2',  'codex',  'user/beta',  'beta',  '2026-04-16', '2026-04-16')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO turns (session_id, provider, timestamp, model,
                                input_tokens, output_tokens,
                                cache_read_tokens, cache_creation_tokens,
                                reasoning_output_tokens,
                                estimated_cost_nanos,
                                cwd, message_id, source_path,
                                pricing_version, pricing_model, billing_mode, cost_confidence)
             VALUES
                ('claude:s1','claude','2026-04-17T10:00:00Z','claude-sonnet-4-6',
                 100,50,0,0,0, 1000000000,
                 '/x','m1','/p', '', '', '', ''),
                ('claude:s1','claude','2026-04-17T11:00:00Z','claude-sonnet-4-6',
                 200,80,0,0,0, 2000000000,
                 '/x','m2','/p', '', '', '', ''),
                ('codex:s2','codex','2026-04-16T09:00:00Z','gpt-5',
                 300,120,0,0,0, 3000000000,
                 '/y','m3','/q', '', '', '', '')",
            [],
        )
        .unwrap();
    }

    #[test]
    fn query_rows_aggregates_by_day_provider_project_model() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("t.db");
        seed_db(&db);

        let conn = open_db(&db).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 4, 17).unwrap();
        let rows =
            query_rows(&conn, ExportPeriod::All, today, None, None, &HashMap::new()).unwrap();

        // Two logical buckets: (2026-04-16, codex, beta, gpt-5) and
        // (2026-04-17, claude, alpha, claude-sonnet-4-6) with the two
        // same-day turns summed.
        assert_eq!(rows.len(), 2);
        let claude_row = rows.iter().find(|r| r.provider == "claude").unwrap();
        assert_eq!(claude_row.date, "2026-04-17");
        assert_eq!(claude_row.input_tokens, 300);
        assert_eq!(claude_row.output_tokens, 130);
        assert_eq!(claude_row.cost_usd_nanos, 3_000_000_000);
        assert_eq!(claude_row.project, "user/alpha");

        let codex_row = rows.iter().find(|r| r.provider == "codex").unwrap();
        assert_eq!(codex_row.date, "2026-04-16");
        assert_eq!(codex_row.cost_usd_nanos, 3_000_000_000);
    }

    #[test]
    fn query_rows_applies_provider_filter() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("t.db");
        seed_db(&db);

        let conn = open_db(&db).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 4, 17).unwrap();
        let rows = query_rows(
            &conn,
            ExportPeriod::All,
            today,
            Some("codex"),
            None,
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].provider, "codex");
    }

    #[test]
    fn query_rows_applies_period_today_bound() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("t.db");
        seed_db(&db);

        let conn = open_db(&db).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 4, 17).unwrap();
        let rows = query_rows(
            &conn,
            ExportPeriod::Today,
            today,
            None,
            None,
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].date, "2026-04-17");
    }

    #[test]
    fn run_export_writes_file_and_returns_row_count() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("t.db");
        seed_db(&db);
        let out = dir.path().join("out.csv");

        let opts = ExportOptions {
            format: ExportFormat::Csv,
            period: ExportPeriod::All,
            output: out.clone(),
            provider: None,
            project: None,
            jq: None,
            project_aliases: HashMap::new(),
        };
        let n = run_export(&db, &opts).unwrap();
        assert_eq!(n, 2);
        assert!(out.exists());
        let text = std::fs::read_to_string(&out).unwrap();
        assert!(text.lines().count() >= 3); // header + 2 rows
    }

    #[test]
    fn run_export_errors_when_db_missing() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("does-not-exist.db");
        let out = dir.path().join("out.csv");
        let opts = ExportOptions {
            format: ExportFormat::Csv,
            period: ExportPeriod::All,
            output: out,
            provider: None,
            project: None,
            jq: None,
            project_aliases: HashMap::new(),
        };
        assert!(run_export(&db, &opts).is_err());
    }

    /// Verify that `open_writer("-")` returns a writer (stdout) rather than
    /// creating a file literally named `-`.
    #[test]
    fn open_writer_dash_does_not_create_file() {
        // We cannot easily capture stdout in a unit test, but we can assert
        // that no file named `-` was created in the current directory and
        // that the function succeeds without error.
        let result = open_writer(Path::new("-"));
        assert!(result.is_ok(), "open_writer(\"-\") must succeed");
        // No file named `-` should exist (we only write to stdout).
        assert!(
            !Path::new("-").exists(),
            "file literally named `-` must not be created"
        );
    }

    /// Verify that `is_stdout` correctly identifies the `-` sentinel.
    #[test]
    fn is_stdout_detects_dash() {
        assert!(is_stdout(Path::new("-")));
        assert!(!is_stdout(Path::new("out.json")));
        assert!(!is_stdout(Path::new("--")));
    }

    /// Verify that jq filter is compiled once and applied per-row (compile-once path).
    /// We exercise this via `run_export` with JSONL + jq and confirm correctness.
    #[test]
    fn run_export_jsonl_jq_compiles_once_and_applies_per_row() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("t.db");
        seed_db(&db);
        let out = dir.path().join("out.jsonl");

        let opts = ExportOptions {
            format: ExportFormat::Jsonl,
            period: ExportPeriod::All,
            output: out.clone(),
            provider: None,
            project: None,
            jq: Some(".model".to_string()),
            project_aliases: HashMap::new(),
        };
        let n = run_export(&db, &opts).unwrap();
        assert_eq!(n, 2);

        let text = std::fs::read_to_string(&out).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        // Each row should produce one model name (as a JSON string).
        assert_eq!(lines.len(), 2, "expected 2 model lines, got: {text}");
        for line in &lines {
            // Each line should be a JSON string value.
            let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
            assert!(parsed.is_string(), "expected JSON string, got {line}");
        }
    }

    /// Verify that a jq filter producing `null` (missing field) outputs `null` per row.
    #[test]
    fn run_export_jsonl_jq_null_field_outputs_null() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("t.db");
        seed_db(&db);
        let out = dir.path().join("out.jsonl");

        let opts = ExportOptions {
            format: ExportFormat::Jsonl,
            period: ExportPeriod::All,
            output: out.clone(),
            provider: None,
            project: None,
            jq: Some(".nonexistent_field".to_string()),
            project_aliases: HashMap::new(),
        };
        let n = run_export(&db, &opts).unwrap();
        assert_eq!(n, 2);

        let text = std::fs::read_to_string(&out).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        // Each row yields `null` — two rows means two `null` lines.
        assert_eq!(lines.len(), 2, "expected 2 null lines, got: {text}");
        for line in &lines {
            assert_eq!(*line, "null", "expected null output, got {line}");
        }
    }

    // ── Phase 11: project_display_name in export ─────────────────────────────

    #[test]
    fn csv_export_with_aliases_populates_project_display_name() {
        let dir = TempDir::new().unwrap();
        let db = dir.path().join("t.db");
        seed_db(&db);
        let out = dir.path().join("out.csv");

        let mut aliases = HashMap::new();
        aliases.insert("user/alpha".to_string(), "Alpha Project".to_string());

        let opts = ExportOptions {
            format: ExportFormat::Csv,
            period: ExportPeriod::All,
            output: out.clone(),
            provider: None,
            project: None,
            jq: None,
            project_aliases: aliases,
        };
        let n = run_export(&db, &opts).unwrap();
        assert_eq!(n, 2);

        let text = std::fs::read_to_string(&out).unwrap();
        // project_display_name column must be present
        assert!(
            text.contains("project_display_name"),
            "csv should have project_display_name header: {text}"
        );
        // aliased row: user/alpha → Alpha Project
        assert!(
            text.contains("Alpha Project"),
            "csv should contain alias 'Alpha Project': {text}"
        );
        // non-aliased row: user/beta stays as-is
        assert!(
            text.contains("user/beta"),
            "csv should contain raw slug for non-aliased project: {text}"
        );
    }
}
