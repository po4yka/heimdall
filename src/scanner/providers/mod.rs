pub mod claude;
pub mod codex;
pub mod cursor;
pub mod cursor_cache;
pub mod xcode;

pub use claude::ClaudeProvider;
pub use codex::CodexProvider;
pub use cursor::CursorProvider;
pub use xcode::XcodeProvider;

use crate::scanner::provider::Provider;
use std::path::PathBuf;

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Returns the default provider set for a full scan.
pub fn all() -> Vec<Box<dyn Provider>> {
    let home = home_dir();
    let mut providers: Vec<Box<dyn Provider>> = vec![
        Box::new(ClaudeProvider::new(vec![
            home.join(".claude").join("projects"),
        ])),
        Box::new(CodexProvider::new(vec![
            home.join(".codex").join("sessions"),
            home.join(".codex").join("archived_sessions"),
        ])),
        Box::new(CursorProvider::new()),
    ];
    #[cfg(target_os = "macos")]
    providers.push(Box::new(XcodeProvider::new(vec![home.join(
        "Library/Developer/Xcode/CodingAssistant/ClaudeAgentConfig/projects",
    )])));
    providers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_contains_cursor_provider() {
        let providers = all();
        let names: Vec<&str> = providers.iter().map(|p| p.name()).collect();
        assert!(
            names.contains(&"cursor"),
            "providers::all() must include a provider with name 'cursor', got: {names:?}"
        );
    }
}
