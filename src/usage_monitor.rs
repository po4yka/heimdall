use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use regex::Regex;

use crate::models::ClaudeUsageFactor;
use crate::scanner::db;

pub const INVOCATION_MODE: &str = "print_slash_command";
pub const PERIOD_TODAY: &str = "today";
pub const STATUS_SUCCESS: &str = "success";
pub const STATUS_UNPARSED: &str = "unparsed";
pub const STATUS_FAILED: &str = "failed";
pub const PARSER_VERSION: &str = "v1";

#[derive(Debug, Clone)]
pub struct CaptureOptions {
    pub db_path: PathBuf,
    pub claude_binary: Option<PathBuf>,
    pub working_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct CaptureResult {
    pub run_id: i64,
    pub status: String,
}

#[derive(Debug, Clone)]
struct PersistedCapture {
    status: &'static str,
    exit_code: Option<i32>,
    stdout_raw: String,
    stderr_raw: String,
    error_summary: Option<String>,
    factors: Vec<ClaudeUsageFactor>,
}

pub fn capture_snapshot(options: &CaptureOptions) -> Result<CaptureResult> {
    let conn = db::open_db(&options.db_path)?;
    db::init_db(&conn)?;

    let binary = options
        .claude_binary
        .clone()
        .unwrap_or_else(|| PathBuf::from("claude"));
    let working_dir = options
        .working_dir
        .clone()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."));

    let persisted = match execute_usage_command(&binary, &working_dir) {
        Ok(output) => persist_capture_result(
            &String::from_utf8_lossy(&output.stdout),
            &String::from_utf8_lossy(&output.stderr),
            output.status.code(),
        ),
        Err(err) => PersistedCapture {
            status: STATUS_FAILED,
            exit_code: None,
            stdout_raw: String::new(),
            stderr_raw: String::new(),
            error_summary: Some(err.to_string()),
            factors: Vec::new(),
        },
    };

    let run_id = db::insert_claude_usage_run(
        &conn,
        persisted.status,
        persisted.exit_code,
        &persisted.stdout_raw,
        &persisted.stderr_raw,
        INVOCATION_MODE,
        PERIOD_TODAY,
        PARSER_VERSION,
        persisted.error_summary.as_deref(),
    )?;
    if persisted.status == STATUS_SUCCESS && !persisted.factors.is_empty() {
        db::insert_claude_usage_factors(&conn, run_id, &persisted.factors)?;
    }

    Ok(CaptureResult {
        run_id,
        status: persisted.status.to_string(),
    })
}

fn execute_usage_command(binary: &Path, working_dir: &Path) -> Result<std::process::Output> {
    Command::new(binary)
        .current_dir(working_dir)
        .env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1")
        .env("CLAUDE_CODE_DISABLE_TERMINAL_TITLE", "1")
        .args(["-p", "/usage", "--output-format", "text"])
        .output()
        .with_context(|| {
            format!(
                "failed to run Claude `/usage` via {} in {}",
                binary.display(),
                working_dir.display()
            )
        })
}

fn persist_capture_result(stdout: &str, stderr: &str, exit_code: Option<i32>) -> PersistedCapture {
    let stdout_raw = stdout.to_string();
    let stderr_raw = stderr.to_string();

    if exit_code.unwrap_or(1) != 0 {
        return PersistedCapture {
            status: STATUS_FAILED,
            exit_code,
            stdout_raw,
            stderr_raw: stderr_raw.clone(),
            error_summary: Some(compose_error_summary(exit_code, stdout, stderr)),
            factors: Vec::new(),
        };
    }

    match parse_usage_output(stdout) {
        Ok(factors) => PersistedCapture {
            status: STATUS_SUCCESS,
            exit_code,
            stdout_raw,
            stderr_raw,
            error_summary: None,
            factors,
        },
        Err(err) => PersistedCapture {
            status: STATUS_UNPARSED,
            exit_code,
            stdout_raw,
            stderr_raw,
            error_summary: Some(err.to_string()),
            factors: Vec::new(),
        },
    }
}

pub fn parse_usage_output(stdout: &str) -> Result<Vec<ClaudeUsageFactor>> {
    let factor_re = Regex::new(r"^(?P<pct>\d+(?:\.\d+)?)%\s+of your usage\s+(?P<label>.+?)\s*$")
        .context("failed to compile `/usage` factor regex")?;
    let lines: Vec<&str> = stdout.lines().collect();
    let mut factors = Vec::new();
    let mut idx = 0_usize;

    while idx < lines.len() {
        let line = lines[idx].trim();
        let Some(caps) = factor_re.captures(line) else {
            idx += 1;
            continue;
        };

        let percent = caps["pct"]
            .parse::<f64>()
            .with_context(|| format!("invalid percent in `/usage` factor line: {line}"))?;
        let label = caps["label"].trim().to_string();
        idx += 1;

        let mut detail_lines = Vec::new();
        while idx < lines.len() {
            let next_trimmed = lines[idx].trim();
            if next_trimmed.is_empty() {
                if !detail_lines.is_empty() {
                    idx += 1;
                    break;
                }
                idx += 1;
                continue;
            }
            if factor_re.is_match(next_trimmed) {
                break;
            }
            detail_lines.push(next_trimmed.to_string());
            idx += 1;
        }

        let advice_text = detail_lines.join(" ");
        factors.push(ClaudeUsageFactor {
            factor_key: slugify_factor_key(&label, factors.len()),
            display_label: label,
            percent,
            description: advice_text.clone(),
            advice_text,
            display_order: factors.len() as i64,
        });
    }

    if factors.is_empty() {
        bail!("no contributor rows found in Claude `/usage` output");
    }

    Ok(factors)
}

fn compose_error_summary(exit_code: Option<i32>, stdout: &str, stderr: &str) -> String {
    let stderr_trimmed = stderr.trim();
    if !stderr_trimmed.is_empty() {
        return stderr_trimmed.to_string();
    }
    let stdout_trimmed = stdout.trim();
    if !stdout_trimmed.is_empty() {
        return stdout_trimmed.to_string();
    }
    match exit_code {
        Some(code) => format!("Claude `/usage` exited with status {code}"),
        None => "Claude `/usage` exited without a status code".into(),
    }
}

fn slugify_factor_key(label: &str, order: usize) -> String {
    let mut key = String::with_capacity(label.len());
    let mut prev_was_sep = false;
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            key.push(ch.to_ascii_lowercase());
            prev_was_sep = false;
        } else if !prev_was_sep {
            key.push('_');
            prev_was_sep = true;
        }
    }
    let key = key.trim_matches('_').to_string();
    if key.is_empty() {
        return format!("factor_{order}");
    }
    key
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn sample_usage_output() -> &'static str {
        "What's contributing to your limits usage?\n\
Approximate, based on local sessions on this machine — does not include other devices or claude.ai\n\n\
Last 24h · these are independent characteristics of your usage, not a breakdown\n\n\
98% of your usage was while 4+ sessions ran in parallel\n\
  All sessions share one limit. If you don't need them all at once, queueing uses it more evenly.\n\n\
90% of your usage came from sessions that ran 3+ subagents\n\
  Each subagent runs its own requests, using up your limits faster.\n\n\
19% of your usage hit a >100k-token cache miss\n\
  Uncached input is expensive, and often happens when sending a message to a session that has gone idle.\n"
    }

    #[test]
    fn parse_usage_output_extracts_factors() {
        let factors = parse_usage_output(sample_usage_output()).unwrap();
        assert_eq!(factors.len(), 3);
        assert_eq!(
            factors[0].display_label,
            "was while 4+ sessions ran in parallel"
        );
        assert!((factors[0].percent - 98.0).abs() < 0.001);
        assert!(
            factors[0]
                .advice_text
                .contains("All sessions share one limit")
        );
        assert_eq!(
            factors[0].factor_key,
            "was_while_4_sessions_ran_in_parallel"
        );
    }

    #[test]
    fn parse_usage_output_handles_wrapped_whitespace() {
        let output = "75% of your usage was at >150k context\n\
    Longer sessions are more expensive even when cached.\n\
    /compact mid-task, /clear when switching to new tasks.\n";
        let factors = parse_usage_output(output).unwrap();
        assert_eq!(factors.len(), 1);
        assert!(factors[0].advice_text.contains("/compact mid-task"));
    }

    #[test]
    fn parse_usage_output_unknown_format_errors() {
        let err = parse_usage_output("/usage isn't available in this environment.")
            .expect_err("unexpected parse success");
        assert!(err.to_string().contains("no contributor rows found"));
    }

    #[test]
    fn persist_capture_result_records_failed_exit() {
        let persisted =
            persist_capture_result("/usage isn't available in this environment.\n", "", Some(1));
        assert_eq!(persisted.status, STATUS_FAILED);
        assert_eq!(
            persisted.error_summary.as_deref(),
            Some("/usage isn't available in this environment.")
        );
        assert!(persisted.factors.is_empty());
    }

    #[test]
    fn capture_snapshot_persists_failed_run_when_binary_is_missing() {
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("usage.db");
        let result = capture_snapshot(&CaptureOptions {
            db_path: db_path.clone(),
            claude_binary: Some(PathBuf::from("/definitely/missing/claude")),
            working_dir: Some(tmp.path().to_path_buf()),
        })
        .unwrap();
        assert_eq!(result.status, STATUS_FAILED);

        let conn = db::open_db(&db_path).unwrap();
        db::init_db(&conn).unwrap();
        let response = db::get_latest_claude_usage_response(&conn).unwrap();
        assert!(!response.available);
        let last_run = response.last_run.expect("missing last run");
        assert_eq!(last_run.status, STATUS_FAILED);
        assert!(
            last_run
                .error_summary
                .unwrap_or_default()
                .contains("failed to run Claude")
        );
    }
}
