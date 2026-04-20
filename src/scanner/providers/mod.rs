pub mod amp;
pub mod claude;
pub mod codex;
pub mod copilot;
pub mod cursor;
pub mod cursor_cache;
pub mod opencode;
pub mod pi;
pub mod xcode;

pub use amp::AmpProvider;
pub use claude::ClaudeProvider;
pub use codex::CodexProvider;
pub use copilot::CopilotProvider;
pub use cursor::CursorProvider;
pub use opencode::OpenCodeProvider;
pub use pi::PiProvider;
pub use xcode::XcodeProvider;

use crate::scanner::provider::Provider;
use std::path::PathBuf;
use std::sync::Arc;

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Returns the default provider set for a full scan.
///
/// Registry order: claude, codex, xcode, cursor, opencode, pi, copilot.
///
/// Provider backend notes:
/// - JSONL-backed: Claude, Codex, Xcode, Pi — the dispatcher in `parser.rs`
///   routes these through `parse_jsonl_file`.
/// - SQLite-backed: Cursor, OpenCode — these providers parse via their trait
///   `parse()` directly; `parse_jsonl_file` is not involved.
/// - Mixed-format / best-effort probe: Copilot — uses its trait `parse()`
///   directly; JSON and JSONL files are both probed.
pub fn all() -> Vec<Arc<dyn Provider>> {
    let home = home_dir();
    #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
    let mut providers: Vec<Arc<dyn Provider>> = vec![
        Arc::new(ClaudeProvider::new(vec![
            home.join(".claude").join("projects"),
        ])),
        Arc::new(CodexProvider::new(vec![
            home.join(".codex").join("sessions"),
            home.join(".codex").join("archived_sessions"),
        ])),
        Arc::new(CursorProvider::new()),
        Arc::new(OpenCodeProvider::new()),
        Arc::new(PiProvider::new()),
        Arc::new(CopilotProvider::new()),
        Arc::new(AmpProvider::new()),
    ];
    #[cfg(target_os = "macos")]
    providers.push(Arc::new(XcodeProvider::new(vec![home.join(
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

    #[test]
    fn all_contains_opencode_provider() {
        let providers = all();
        let names: Vec<&str> = providers.iter().map(|p| p.name()).collect();
        assert!(
            names.contains(&"opencode"),
            "providers::all() must include 'opencode', got: {names:?}"
        );
    }

    #[test]
    fn all_contains_pi_provider() {
        let providers = all();
        let names: Vec<&str> = providers.iter().map(|p| p.name()).collect();
        assert!(
            names.contains(&"pi"),
            "providers::all() must include 'pi', got: {names:?}"
        );
    }

    #[test]
    fn all_contains_copilot_provider() {
        let providers = all();
        let names: Vec<&str> = providers.iter().map(|p| p.name()).collect();
        assert!(
            names.contains(&"copilot"),
            "providers::all() must include 'copilot', got: {names:?}"
        );
    }

    #[test]
    fn all_contains_amp_provider() {
        let providers = all();
        let names: Vec<&str> = providers.iter().map(|p| p.name()).collect();
        assert!(
            names.contains(&"amp"),
            "providers::all() must include 'amp', got: {names:?}"
        );
    }

    #[test]
    fn all_registry_order() {
        let providers = all();
        let names: Vec<&str> = providers.iter().map(|p| p.name()).collect();
        // Verify the known non-platform-gated providers appear in the correct order.
        let claude_pos = names.iter().position(|&n| n == "claude").unwrap();
        let codex_pos = names.iter().position(|&n| n == "codex").unwrap();
        let cursor_pos = names.iter().position(|&n| n == "cursor").unwrap();
        let opencode_pos = names.iter().position(|&n| n == "opencode").unwrap();
        let pi_pos = names.iter().position(|&n| n == "pi").unwrap();
        let copilot_pos = names.iter().position(|&n| n == "copilot").unwrap();
        let amp_pos = names.iter().position(|&n| n == "amp").unwrap();
        assert!(claude_pos < codex_pos);
        assert!(codex_pos < cursor_pos);
        assert!(cursor_pos < opencode_pos);
        assert!(opencode_pos < pi_pos);
        assert!(pi_pos < copilot_pos);
        assert!(copilot_pos < amp_pos);
    }
}
