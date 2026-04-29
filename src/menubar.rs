//! ROADMAP Phase 8 -- SwiftBar menubar widget output.
//!
//! Emits SwiftBar-formatted stdout for the `menubar` subcommand.
//! The protocol has two sections separated by `---`:
//!   - Line 1: title shown in the macOS menu bar
//!   - Lines after `---`: drill-down submenu entries
//!
//! Security: any user-controlled text (project names, session titles, etc.)
//! must pass through `sanitize_swiftbar_text` before being included in the
//! output. This prevents injection of SwiftBar control sequences.
//!
//! # Char budgets
//!
//! The menu bar surface is length-capped by the medium (macOS menu bar clips
//! long titles; submenu rows look ragged on small screens). Two budgets are
//! enforced as `pub const` and asserted in unit tests so future edits cannot
//! silently bust them:
//!
//! - [`MENUBAR_TITLE_MAX_CHARS`] (30): title line shown in the menu bar.
//!   Worst realistic case `$99999.99 · 9999` is 18 chars, giving ~12-char
//!   headroom for future title content.
//! - [`MENUBAR_ROW_MAX_CHARS`] (80): submenu content rows (Cost, Sessions,
//!   One-shot rate, etc.). Action rows containing SwiftBar param keywords
//!   (`href=`, `bash=`) are excluded from this budget — those are by-design
//!   long.
//!
//! Pattern borrowed from talk-normal's `prompt-chatgpt.md` budget convention:
//! when output overflows, either compress the content or bump the budget with
//! rationale documented here. Never silently truncate at runtime — UTF-8
//! truncation can split codepoints and break the security sanitizer.

use std::path::Path;

use anyhow::Result;

use crate::scanner::db::open_db;

/// Maximum chars in the menu bar title line (Unicode scalar values).
///
/// macOS clips long titles in the menu bar; 30 chars holds the worst
/// realistic case (`$99999.99 · 9999` = 18 chars) with margin. Asserted in
/// `tests::test_title_fits_budget`.
pub const MENUBAR_TITLE_MAX_CHARS: usize = 30;

/// Maximum chars per submenu content row (Unicode scalar values).
///
/// Excludes SwiftBar action rows (`href=`, `bash=` keywords) which are
/// by-design long. Asserted in `tests::test_submenu_rows_fit_budget`.
pub const MENUBAR_ROW_MAX_CHARS: usize = 80;

/// Data extracted from the DB for the menubar display.
#[derive(Debug, Default)]
pub struct MenubarData {
    /// Today's total cost in USD.
    pub today_cost_usd: f64,
    /// Number of distinct sessions today.
    pub today_sessions: i64,
    /// One-shot rate across all sessions with data (0.0–1.0), or None.
    pub one_shot_rate: Option<f64>,
    /// When the most recent archive snapshot was taken (RFC3339), or `None`
    /// if the archive is empty.
    pub last_snapshot_at: Option<String>,
    /// Total bytes in the most recent snapshot, or `None`.
    pub last_snapshot_bytes: Option<u64>,
}

/// Strip or escape any character that SwiftBar would interpret as a control
/// sequence. Removes: pipe, newline, carriage return, tab, and SwiftBar param
/// keywords if they appear adjacent to whitespace (defence in depth).
///
/// Removed sequences:
/// - `|`  (SwiftBar attribute separator)
/// - `\n`, `\r` (multi-line injection)
/// - `\t` (tab, used in injections alongside keywords)
/// - Leading-space SwiftBar keywords: ` bash=`, ` href=`, ` refresh=`,
///   ` terminal=`, ` shell=`, ` param1=`, ` templateImage=`
/// - Collapses consecutive whitespace to a single space and trims.
pub fn sanitize_swiftbar_text(input: &str) -> String {
    // Step 1: replace control characters with spaces (pipe is removed; newlines
    // and tabs become spaces so adjacent words stay separated and keyword
    // detection in step 2 still works correctly).
    let no_ctrl: String = input
        .chars()
        .map(|c| match c {
            '|' => ' ',
            '\n' | '\r' | '\t' => ' ',
            other => other,
        })
        .collect();

    // Step 2: strip SwiftBar param keywords that start with a space.
    // These are the injection vectors from the security spec.
    let keywords = [
        " bash=",
        " href=",
        " refresh=",
        " terminal=",
        " shell=",
        " param1=",
        " templateImage=",
    ];

    let mut result = no_ctrl;
    for kw in &keywords {
        // Remove every occurrence; loop until none remain (handles adjacent injections).
        while result.contains(kw) {
            // Remove the keyword and everything after it up to (but not including)
            // the next space-prefixed keyword or end-of-string. This is defensive:
            // we simply truncate from the first occurrence of the keyword.
            if let Some(pos) = result.find(kw) {
                result.truncate(pos);
            }
        }
    }

    // Step 3: collapse consecutive whitespace to a single space and trim.
    let collapsed: String = result.split_whitespace().collect::<Vec<&str>>().join(" ");

    collapsed
}

/// Format the menubar title line (short, fits ~20 chars).
fn format_title(data: &MenubarData) -> String {
    let cost = format!("${:.2}", data.today_cost_usd);
    format!("{} · {}", cost, data.today_sessions)
}

/// Render the full SwiftBar output from `MenubarData`.
/// Returns the complete string the caller should print to stdout.
pub fn render(data: &MenubarData) -> String {
    let title = format_title(data);

    // sanitize_swiftbar_text is applied defensively to all display strings so
    // that future additions of user-controlled fields (project names, session
    // titles, etc.) automatically pass through the sanitizer.
    let one_shot_line = sanitize_swiftbar_text(&match data.one_shot_rate {
        Some(rate) => format!("One-shot rate: {}%", (rate * 100.0).round() as u64),
        None => "One-shot rate: n/a".to_string(),
    });

    let snapshot_section = match (&data.last_snapshot_at, data.last_snapshot_bytes) {
        (Some(at), Some(bytes)) => format!(
            "Snapshots\nLast: {at} ({bytes} bytes)\nSnapshot now | bash=claude-usage-tracker param0=archive param1=snapshot terminal=false",
            at = sanitize_swiftbar_text(at),
            bytes = bytes,
        ),
        _ => "Snapshots\nNo snapshots yet\nSnapshot now | bash=claude-usage-tracker param0=archive param1=snapshot terminal=false".to_string(),
    };

    format!(
        "{title}\n---\nToday\nCost: ${cost:.2}\nSessions: {sessions}\n{one_shot_line}\n---\n{snapshot_section}\n---\nOpen dashboard | href=http://localhost:8080\nRescan | bash=claude-usage-tracker terminal=false",
        title = title,
        cost = data.today_cost_usd,
        sessions = data.today_sessions,
        one_shot_line = one_shot_line,
        snapshot_section = snapshot_section,
    )
}

/// Query the DB and build `MenubarData`. Returns a degraded default if the
/// DB does not exist (prints a minimal placeholder rather than erroring).
pub fn run_menubar(db_path: &Path) -> Result<String> {
    if !db_path.exists() {
        return Ok("heimdall: no data\n---\n".to_string());
    }

    let conn = open_db(db_path)?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Today's cost and session count.
    let (today_cost_nanos, today_sessions) =
        crate::scanner::db::query_day_cost_and_session_count(&conn, &today).unwrap_or((0, 0));

    let today_cost_usd = today_cost_nanos as f64 / 1_000_000_000.0;

    // One-shot rate across all sessions (not just today — same as stats command).
    let one_shot_rate: Option<f64> = crate::scanner::db::query_oneshot_rate(&conn).unwrap_or(None);

    let archive_root = crate::archive::default_root();
    let archive_metas = crate::archive::Archive::at(archive_root)
        .ok()
        .and_then(|a| a.list().ok());
    let (last_snapshot_at, last_snapshot_bytes) =
        match archive_metas.and_then(|metas| metas.into_iter().next()) {
            Some(m) => (Some(m.created_at), Some(m.total_bytes)),
            None => (None, None),
        };

    let data = MenubarData {
        today_cost_usd,
        today_sessions,
        one_shot_rate,
        last_snapshot_at,
        last_snapshot_bytes,
    };

    Ok(render(&data))
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Unit tests: sanitize_swiftbar_text
    // -----------------------------------------------------------------------

    /// Positive case: plain text passes through unchanged (modulo whitespace collapse).
    #[test]
    fn test_sanitize_plain_text_passthrough() {
        let input = "my-project";
        assert_eq!(sanitize_swiftbar_text(input), "my-project");
    }

    /// Positive case: text with leading/trailing spaces is trimmed.
    #[test]
    fn test_sanitize_trims_whitespace() {
        let input = "  hello world  ";
        assert_eq!(sanitize_swiftbar_text(input), "hello world");
    }

    // -----------------------------------------------------------------------
    // Security tests: injection prevention
    // -----------------------------------------------------------------------

    /// SECURITY: pipe injection — `bash=` must not survive in sanitized output.
    ///
    /// SwiftBar interprets `| bash=<cmd>` as a button action. If project names
    /// contain a pipe followed by a bash= directive the menu bar item would
    /// execute arbitrary shell commands on click.
    #[test]
    fn test_security_pipe_injection() {
        let malicious = "proj | bash=rm -rf /";
        let sanitized = sanitize_swiftbar_text(malicious);
        assert!(
            !sanitized.contains("bash="),
            "pipe injection: output must not contain 'bash=', got: {sanitized:?}"
        );
        assert!(
            !sanitized.contains('|'),
            "pipe injection: output must not contain '|', got: {sanitized:?}"
        );
    }

    /// SECURITY: newline injection — output must collapse to a single line.
    ///
    /// A newline in a SwiftBar item text creates a second menu item on a new
    /// line, enabling injection of arbitrary entries (including `sudo evil`).
    #[test]
    fn test_security_newline_injection() {
        let malicious = "proj\nsudo evil";
        let sanitized = sanitize_swiftbar_text(malicious);
        assert!(
            !sanitized.contains('\n'),
            "newline injection: output must not contain newline, got: {sanitized:?}"
        );
        // The text should be collapsed onto one line.
        assert_eq!(sanitized, "proj sudo evil");
    }

    /// SECURITY: href injection — `href=` must be stripped.
    ///
    /// `href=javascript:alert(1)` would open an attacker-controlled URL when
    /// the menu item is clicked.
    #[test]
    fn test_security_href_injection() {
        let malicious = "proj href=javascript:alert(1)";
        let sanitized = sanitize_swiftbar_text(malicious);
        assert!(
            !sanitized.contains("href="),
            "href injection: output must not contain 'href=', got: {sanitized:?}"
        );
    }

    /// SECURITY: shell + param keyword injection.
    ///
    /// `shell=` and `param1=` together form the alternate SwiftBar action
    /// syntax. Both must be stripped even when they appear without a pipe.
    #[test]
    fn test_security_shell_param_injection() {
        let malicious = "proj shell=/bin/zsh param1=-c param2=evil";
        let sanitized = sanitize_swiftbar_text(malicious);
        assert!(
            !sanitized.contains("shell="),
            "shell injection: output must not contain 'shell=', got: {sanitized:?}"
        );
        assert!(
            !sanitized.contains("param1="),
            "param1 injection: output must not contain 'param1=', got: {sanitized:?}"
        );
    }

    /// SECURITY: tab + bash= injection.
    ///
    /// Tabs can be used in place of spaces before SwiftBar keywords when the
    /// pipe character has already been stripped by a naive first pass.
    #[test]
    fn test_security_tab_bash_injection() {
        let malicious = "proj\tbash=evil";
        let sanitized = sanitize_swiftbar_text(malicious);
        assert!(
            !sanitized.contains("bash="),
            "tab+bash injection: output must not contain 'bash=', got: {sanitized:?}"
        );
        assert!(
            !sanitized.contains('\t'),
            "tab injection: output must not contain tab, got: {sanitized:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Unit tests: render
    // -----------------------------------------------------------------------

    #[test]
    fn test_render_title_contains_dollar() {
        let data = MenubarData {
            today_cost_usd: 2.47,
            today_sessions: 12,
            one_shot_rate: Some(0.78),
            last_snapshot_at: None,
            last_snapshot_bytes: None,
        };
        let output = render(&data);
        let first_line = output.lines().next().unwrap_or("");
        assert!(
            first_line.contains('$'),
            "title must contain '$', got: {first_line:?}"
        );
    }

    #[test]
    fn test_render_title_contains_session_count() {
        let data = MenubarData {
            today_cost_usd: 1.23,
            today_sessions: 7,
            one_shot_rate: None,
            last_snapshot_at: None,
            last_snapshot_bytes: None,
        };
        let output = render(&data);
        let first_line = output.lines().next().unwrap_or("");
        assert!(
            first_line.contains("7"),
            "title must contain session count, got: {first_line:?}"
        );
    }

    #[test]
    fn test_render_contains_separator() {
        let data = MenubarData::default();
        let output = render(&data);
        assert!(
            output.contains("---"),
            "output must contain SwiftBar separator '---'"
        );
    }

    #[test]
    fn test_render_one_shot_rate_present() {
        let data = MenubarData {
            today_cost_usd: 0.5,
            today_sessions: 3,
            one_shot_rate: Some(0.75),
            last_snapshot_at: None,
            last_snapshot_bytes: None,
        };
        let output = render(&data);
        assert!(
            output.contains("75%"),
            "render must show one-shot rate as percentage, got: {output:?}"
        );
    }

    #[test]
    fn test_render_one_shot_rate_absent() {
        let data = MenubarData {
            today_cost_usd: 0.0,
            today_sessions: 0,
            one_shot_rate: None,
            last_snapshot_at: None,
            last_snapshot_bytes: None,
        };
        let output = render(&data);
        assert!(
            output.contains("n/a"),
            "render must show 'n/a' when one_shot_rate is None, got: {output:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Integration tests: run_menubar
    // -----------------------------------------------------------------------

    /// Missing DB returns degraded display string, not an error.
    #[test]
    fn test_run_menubar_missing_db_returns_degraded() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().with_extension("nonexistent.db");
        // path does not exist
        let result = run_menubar(&path);
        assert!(result.is_ok(), "missing DB must not error");
        let text = result.unwrap();
        assert!(
            text.contains("heimdall: no data"),
            "degraded output must contain 'heimdall: no data', got: {text:?}"
        );
    }

    /// Seeded DB with two turns produces a title with '$' and non-zero cost.
    // -----------------------------------------------------------------------
    // Char-budget tests (see module docblock § "Char budgets")
    // -----------------------------------------------------------------------

    /// The menu-bar title fits MENUBAR_TITLE_MAX_CHARS across realistic ranges.
    ///
    /// macOS clips long titles silently; busting the budget leaves users
    /// staring at a chopped cost number with no error. Failing CI here is
    /// the early warning.
    #[test]
    fn test_title_fits_budget() {
        // Default (zero) fixture.
        let small = MenubarData::default();
        let title = format_title(&small);
        assert!(
            title.chars().count() <= MENUBAR_TITLE_MAX_CHARS,
            "default title overflowed budget: {:?} ({} chars > {})",
            title,
            title.chars().count(),
            MENUBAR_TITLE_MAX_CHARS,
        );

        // Worst realistic case: large daily cost, many sessions.
        let huge = MenubarData {
            today_cost_usd: 99_999.99,
            today_sessions: 9_999,
            one_shot_rate: None,
            last_snapshot_at: None,
            last_snapshot_bytes: None,
        };
        let title = format_title(&huge);
        assert!(
            title.chars().count() <= MENUBAR_TITLE_MAX_CHARS,
            "worst-case title overflowed budget: {:?} ({} chars > {})",
            title,
            title.chars().count(),
            MENUBAR_TITLE_MAX_CHARS,
        );
    }

    /// Each submenu content row fits MENUBAR_ROW_MAX_CHARS. Action rows
    /// containing SwiftBar param keywords (`href=`, `bash=`) are by-design
    /// long and excluded from this budget.
    #[test]
    fn test_submenu_rows_fit_budget() {
        let data = MenubarData {
            today_cost_usd: 99_999.99,
            today_sessions: 9_999,
            one_shot_rate: Some(0.99),
            last_snapshot_at: None,
            last_snapshot_bytes: None,
        };
        let output = render(&data);

        // SwiftBar layout: <title>\n---\n<content>\n---\n<actions>
        // Split on the separator and pick the content section (index 1).
        let sections: Vec<&str> = output.split("\n---\n").collect();
        assert!(
            sections.len() >= 2,
            "render output missing SwiftBar separator: {output:?}"
        );
        let content = sections[1];

        for line in content.lines() {
            // Skip any line that contains a SwiftBar param keyword — those
            // are action rows, not content rows.
            if line.contains("href=") || line.contains("bash=") {
                continue;
            }
            assert!(
                line.chars().count() <= MENUBAR_ROW_MAX_CHARS,
                "submenu row overflowed budget: {:?} ({} chars > {})",
                line,
                line.chars().count(),
                MENUBAR_ROW_MAX_CHARS,
            );
        }
    }

    #[test]
    fn test_run_menubar_with_seeded_db() {
        use crate::scanner::db::{init_db, open_db};
        use tempfile::NamedTempFile;

        let tmp = NamedTempFile::new().unwrap();
        let db_path = tmp.path();

        let conn = open_db(db_path).unwrap();
        init_db(&conn).unwrap();

        // Seed two turns for today with non-zero cost.
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let ts = format!("{}T10:00:00Z", today);
        conn.execute_batch(&format!(
            "INSERT INTO sessions (session_id, provider, first_timestamp, last_timestamp)
             VALUES ('claude:sess-test-1', 'claude', '{ts}', '{ts}');
             INSERT INTO turns (session_id, provider, timestamp, model,
                                input_tokens, output_tokens, estimated_cost_nanos)
             VALUES ('claude:sess-test-1', 'claude', '{ts}', 'claude-sonnet-4-5',
                     1000, 500, 1500000000);
             INSERT INTO turns (session_id, provider, timestamp, model,
                                input_tokens, output_tokens, estimated_cost_nanos)
             VALUES ('claude:sess-test-1', 'claude', '{ts}', 'claude-sonnet-4-5',
                     2000, 800, 750000000);",
        ))
        .unwrap();

        drop(conn);

        let result = run_menubar(db_path);
        assert!(
            result.is_ok(),
            "seeded DB must not error: {:?}",
            result.err()
        );
        let text = result.unwrap();
        let first_line = text.lines().next().unwrap_or("");

        assert!(
            first_line.contains('$'),
            "title must contain '$', got: {first_line:?}"
        );
        // Total cost nanos = 1_500_000_000 + 750_000_000 = 2_250_000_000 => $2.25
        assert!(
            first_line.contains("2.25"),
            "title must reflect cost $2.25, got: {first_line:?}"
        );
        // Session count = 1 (one distinct session_id)
        assert!(
            first_line.contains("1"),
            "title must reflect 1 session, got: {first_line:?}"
        );
    }

    #[test]
    fn render_includes_snapshots_section() {
        let data = MenubarData {
            today_cost_usd: 1.0,
            today_sessions: 1,
            one_shot_rate: None,
            last_snapshot_at: Some("2026-04-28T08:00:00Z".into()),
            last_snapshot_bytes: Some(1234),
        };
        let out = render(&data);
        assert!(out.contains("Snapshots"));
        assert!(out.contains("2026-04-28T08:00:00Z"));
        assert!(out.contains("1234"));
        assert!(out.contains("Snapshot now"));
    }
}
