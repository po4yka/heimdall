use std::path::Path;

use anyhow::Result;

use crate::models::Turn;
use crate::scanner::parser::{ParseResult, parse_jsonl_file};

/// A file path that a provider has identified as a parseable session log.
#[derive(Debug, Clone)]
pub struct SessionSource {
    pub path: std::path::PathBuf,
}

/// Per-source-type plug-in contract. Registered in `providers::all()`.
///
/// `scan()` uses `discover_sessions()` to enumerate per-provider files and then
/// calls `parse_source()` on the same provider object. JSONL-backed providers
/// inherit the default dispatcher; custom backends override `parse_source()`
/// to wrap their native parse results into the common scanner shape.
#[allow(dead_code)]
pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;
    fn discover_sessions(&self) -> Result<Vec<SessionSource>>;
    fn parse(&self, path: &Path) -> Result<Vec<Turn>>;

    fn parse_source(&self, path: &Path, skip_lines: i64) -> ParseResult {
        parse_jsonl_file(self.name(), path, skip_lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_trait_is_object_safe() {
        // Verify dyn dispatch compiles — no implementation needed, just type-check.
        fn _accepts(_p: &dyn Provider) {}
        // If this file compiles, the trait is object-safe.
    }
}
