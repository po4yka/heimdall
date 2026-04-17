//! Cowork ephemeral label resolution.
//!
//! Claude Desktop's Cowork feature creates ephemeral sessions under
//! `~/.claude/local-agent-mode-sessions/<generated-slug>/`. The slugs are
//! procedurally generated names like `wizardly-charming-thompson` -- zero
//! human meaning. Alongside the normal JSONL files, each slug directory
//! contains an `audit.jsonl` whose first `user` record carries the original
//! user prompt. Extracting that prompt gives a human-readable project label.
//!
//! # Dedup strategy
//!
//! Label resolution is idempotent: the first user message in `audit.jsonl`
//! never changes, so resolving once per session at parse time is enough.
//! The resolved label is written directly into `SessionMeta.project_name`.
//! No new DB columns are needed; the existing `project_name` column is reused.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use tracing::debug;

const MAX_LABEL_CHARS: usize = 80;
const ELLIPSIS: char = '\u{2026}'; // U+2026 HORIZONTAL ELLIPSIS

/// Resolve a human-readable label for a Cowork ephemeral session.
///
/// Walks `slug_dir` for a file named exactly `audit.jsonl` (top-level only,
/// not recursive). Reads line by line looking for the first record where
/// `type == "user"` and `content` is a non-empty string or an array whose
/// first text block is non-empty.
///
/// Returns `Some(label)` truncated to 80 characters (character-wise, not
/// byte-wise) with a trailing `…` appended if truncation occurred. Returns
/// `None` on any error or if no usable user record is found.
pub fn resolve_cowork_label(slug_dir: &Path) -> Option<String> {
    let audit_path = slug_dir.join("audit.jsonl");
    if !audit_path.exists() {
        debug!("cowork: no audit.jsonl in {}", slug_dir.display());
        return None;
    }

    let file = match std::fs::File::open(&audit_path) {
        Ok(f) => f,
        Err(e) => {
            debug!("cowork: cannot open {}: {}", audit_path.display(), e);
            return None;
        }
    };

    let reader = BufReader::new(file);
    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                debug!("cowork: read error in {}: {}", audit_path.display(), e);
                continue;
            }
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let record: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                debug!(
                    "cowork: JSON parse error in {}: {}",
                    audit_path.display(),
                    e
                );
                continue;
            }
        };

        let rtype = record.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if rtype != "user" {
            continue;
        }

        let content = record.get("content");
        let text = extract_content_text(content);

        match text {
            Some(t) if !t.is_empty() => return Some(truncate_label(&t)),
            _ => {
                debug!(
                    "cowork: user record has empty content in {}",
                    audit_path.display()
                );
                return None;
            }
        }
    }

    debug!("cowork: no user record found in {}", audit_path.display());
    None
}

/// Extract the text content from a `content` JSON value.
///
/// Handles two shapes:
/// - String: `"content": "hello world"`
/// - Array: `"content": [{"type": "text", "text": "hello world"}, ...]`
fn extract_content_text(content: Option<&serde_json::Value>) -> Option<String> {
    let content = content?;

    if let Some(s) = content.as_str() {
        return Some(s.to_string());
    }

    if let Some(arr) = content.as_array() {
        for item in arr {
            let block_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if block_type == "text"
                && let Some(text) = item.get("text").and_then(|v| v.as_str())
            {
                return Some(text.to_string());
            }
        }
    }

    None
}

/// Truncate `text` to at most `MAX_LABEL_CHARS` characters (character-wise,
/// never byte-wise). Appends `…` only when truncation actually occurred.
/// Trailing whitespace is stripped before the ellipsis is appended.
fn truncate_label(text: &str) -> String {
    let char_count = text.chars().count();
    if char_count <= MAX_LABEL_CHARS {
        return text.trim_end().to_string();
    }
    let truncated: String = text.chars().take(MAX_LABEL_CHARS).collect();
    let trimmed = truncated.trim_end();
    format!("{trimmed}{ELLIPSIS}")
}

/// Return the slug directory if `path` matches the Cowork session pattern:
/// `.../local-agent-mode-sessions/<slug>/...`
///
/// The slug directory is the component immediately after
/// `local-agent-mode-sessions` in the path.
pub fn is_cowork_session_path(path: &Path) -> Option<PathBuf> {
    let path_str = path.to_string_lossy();
    // Normalise to forward-slashes for a single pattern match on all platforms.
    let normalised = path_str.replace('\\', "/");

    let marker = "/local-agent-mode-sessions/";
    let marker_pos = normalised.find(marker)?;

    let after_marker = &normalised[marker_pos + marker.len()..];
    // The slug is everything up to the next `/` (or the whole remaining
    // segment if the path ends at the slug component).
    let slug_end = after_marker.find('/').unwrap_or(after_marker.len());
    if slug_end == 0 {
        return None;
    }

    let slug = &after_marker[..slug_end];
    let slug_dir_str = &normalised[..marker_pos + marker.len() + slug_end];

    // Reconstruct as a PathBuf using the original path's prefix up to the
    // slug so we don't break OS-specific roots.
    // On non-Windows the normalised string == original string for these paths.
    // Use the normalised prefix up to and including the slug component.
    let _ = slug; // already validated non-empty above
    Some(PathBuf::from(slug_dir_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    // -------------------------------------------------------------------------
    // resolve_cowork_label tests
    // -------------------------------------------------------------------------

    /// Create a slug directory with an `audit.jsonl` containing the given lines.
    fn make_audit(dir: &TempDir, lines: &[serde_json::Value]) -> PathBuf {
        let slug_dir = dir.path().join("wizardly-charming-thompson");
        std::fs::create_dir_all(&slug_dir).unwrap();
        let audit_path = slug_dir.join("audit.jsonl");
        let mut f = std::fs::File::create(&audit_path).unwrap();
        for line in lines {
            writeln!(f, "{}", line).unwrap();
        }
        slug_dir
    }

    #[test]
    fn no_audit_jsonl_returns_none() {
        let dir = TempDir::new().unwrap();
        let slug_dir = dir.path().join("some-slug");
        std::fs::create_dir_all(&slug_dir).unwrap();
        assert!(resolve_cowork_label(&slug_dir).is_none());
    }

    #[test]
    fn string_content_returns_label() {
        let dir = TempDir::new().unwrap();
        let slug_dir = make_audit(
            &dir,
            &[serde_json::json!({
                "type": "user",
                "content": "hello world"
            })],
        );
        assert_eq!(resolve_cowork_label(&slug_dir), Some("hello world".into()));
    }

    #[test]
    fn long_prompt_truncated_to_80_chars_with_ellipsis() {
        let dir = TempDir::new().unwrap();
        let long_prompt: String = "a".repeat(200);
        let slug_dir = make_audit(
            &dir,
            &[serde_json::json!({
                "type": "user",
                "content": long_prompt
            })],
        );
        let result = resolve_cowork_label(&slug_dir).unwrap();
        // The result should be exactly 81 chars: 80 'a's + the ellipsis character.
        assert_eq!(result.chars().count(), 81);
        assert!(result.ends_with('\u{2026}'));
        assert_eq!(&result[..80], "a".repeat(80));
    }

    #[test]
    fn non_user_records_before_first_user_are_skipped() {
        let dir = TempDir::new().unwrap();
        let slug_dir = make_audit(
            &dir,
            &[
                serde_json::json!({"type": "system", "content": "system message"}),
                serde_json::json!({"type": "assistant", "content": "assistant reply"}),
                serde_json::json!({"type": "user", "content": "real user prompt"}),
            ],
        );
        assert_eq!(
            resolve_cowork_label(&slug_dir),
            Some("real user prompt".into())
        );
    }

    #[test]
    fn empty_content_user_record_returns_none() {
        let dir = TempDir::new().unwrap();
        let slug_dir = make_audit(
            &dir,
            &[serde_json::json!({
                "type": "user",
                "content": ""
            })],
        );
        assert!(resolve_cowork_label(&slug_dir).is_none());
    }

    #[test]
    fn malformed_json_returns_none_no_panic() {
        let dir = TempDir::new().unwrap();
        let slug_dir = dir.path().join("bad-slug");
        std::fs::create_dir_all(&slug_dir).unwrap();
        let audit_path = slug_dir.join("audit.jsonl");
        let mut f = std::fs::File::create(&audit_path).unwrap();
        writeln!(f, "{{not valid json").unwrap();
        writeln!(f, "also not json").unwrap();
        // Should not panic and should return None
        assert!(resolve_cowork_label(&slug_dir).is_none());
    }

    #[test]
    fn array_content_first_text_block_returned() {
        let dir = TempDir::new().unwrap();
        let slug_dir = make_audit(
            &dir,
            &[serde_json::json!({
                "type": "user",
                "content": [
                    {"type": "text", "text": "first text block"},
                    {"type": "text", "text": "second text block"}
                ]
            })],
        );
        assert_eq!(
            resolve_cowork_label(&slug_dir),
            Some("first text block".into())
        );
    }

    #[test]
    fn exactly_80_char_prompt_no_ellipsis() {
        let dir = TempDir::new().unwrap();
        let prompt: String = "b".repeat(80);
        let slug_dir = make_audit(
            &dir,
            &[serde_json::json!({
                "type": "user",
                "content": prompt
            })],
        );
        let result = resolve_cowork_label(&slug_dir).unwrap();
        assert_eq!(result.chars().count(), 80);
        assert!(!result.ends_with('\u{2026}'));
    }

    #[test]
    fn trailing_whitespace_stripped_before_ellipsis() {
        let dir = TempDir::new().unwrap();
        // 80 chars of 'x' followed by spaces; truncation happens at 80 chars,
        // then trailing whitespace is stripped, then ellipsis is NOT added
        // (since we only add it when original was > 80 chars).
        // Test the trailing-whitespace stripping for a non-truncated case:
        let slug_dir = make_audit(
            &dir,
            &[serde_json::json!({
                "type": "user",
                "content": "hello   "
            })],
        );
        let result = resolve_cowork_label(&slug_dir).unwrap();
        assert_eq!(result, "hello");
    }

    // -------------------------------------------------------------------------
    // is_cowork_session_path tests
    // -------------------------------------------------------------------------

    #[test]
    fn is_cowork_positive_case() {
        let path = Path::new(
            "/Users/someone/.claude/local-agent-mode-sessions/wizardly-charming-thompson/abc.jsonl",
        );
        let slug_dir = is_cowork_session_path(path).unwrap();
        assert_eq!(
            slug_dir,
            PathBuf::from(
                "/Users/someone/.claude/local-agent-mode-sessions/wizardly-charming-thompson"
            )
        );
    }

    #[test]
    fn is_cowork_negative_case_regular_project() {
        let path = Path::new("/Users/someone/.claude/projects/my-project/session.jsonl");
        assert!(is_cowork_session_path(path).is_none());
    }

    #[test]
    fn is_cowork_negative_case_empty() {
        let path = Path::new("");
        assert!(is_cowork_session_path(path).is_none());
    }

    #[test]
    fn is_cowork_path_at_slug_boundary() {
        // Path ends exactly at the slug directory (no file component)
        let path = Path::new("/home/user/.claude/local-agent-mode-sessions/my-slug");
        let slug_dir = is_cowork_session_path(path).unwrap();
        assert_eq!(
            slug_dir,
            PathBuf::from("/home/user/.claude/local-agent-mode-sessions/my-slug")
        );
    }
}
