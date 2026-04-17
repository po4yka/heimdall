use std::path::Path;

use anyhow::Result;

use crate::models::Turn;

/// A file path that a provider has identified as a parseable session log.
#[derive(Debug, Clone)]
pub struct SessionSource {
    pub path: std::path::PathBuf,
    pub provider_name: &'static str,
}

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
