use std::path::Path;

use anyhow::Result;

use crate::models::Turn;

/// A file path that a provider has identified as a parseable session log.
/// `provider_name` duplicates the owning `Provider::name()` so callers that
/// consume a flattened `Vec<SessionSource>` still know which provider
/// emitted each entry without threading the provider alongside.
#[derive(Debug, Clone)]
pub struct SessionSource {
    pub path: std::path::PathBuf,
    pub provider_name: &'static str,
}

/// Per-source-type plug-in contract. Registered in `providers::all()`.
///
/// `scan()` uses `discover_sessions()` to enumerate per-provider files
/// and then dispatches parsing through `parser::parse_jsonl_file`. The
/// `parse()` method on this trait is a contract placeholder for a future
/// refactor that lifts parsing fully behind the trait — keeping it here
/// documents the intended provider shape without forcing the lift today.
#[allow(dead_code)]
pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;
    fn discover_sessions(&self) -> Result<Vec<SessionSource>>;
    fn parse(&self, path: &Path) -> Result<Vec<Turn>>;
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
