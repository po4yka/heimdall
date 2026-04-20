use std::path::{Path, PathBuf};

use anyhow::Result;
use walkdir::WalkDir;

use crate::models::Turn;
use crate::scanner::parser::parse_claude_jsonl_file;
use crate::scanner::provider::{Provider, SessionSource};

pub struct ClaudeProvider {
    pub dirs: Vec<PathBuf>,
}

impl ClaudeProvider {
    pub fn new(dirs: Vec<PathBuf>) -> Self {
        Self { dirs }
    }
}

impl Provider for ClaudeProvider {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn discover_sessions(&self) -> Result<Vec<SessionSource>> {
        let mut sources = Vec::new();
        for dir in &self.dirs {
            if !dir.exists() {
                continue;
            }
            for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
                if entry.path().extension().is_some_and(|ext| ext == "jsonl") {
                    sources.push(SessionSource {
                        path: entry.path().to_path_buf(),
                    });
                }
            }
        }
        Ok(sources)
    }

    fn parse(&self, path: &Path) -> Result<Vec<Turn>> {
        Ok(parse_claude_jsonl_file(path, 0).turns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_jsonl(dir: &TempDir, name: &str, lines: &[String]) -> PathBuf {
        let path = dir.path().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        for line in lines {
            writeln!(f, "{}", line).unwrap();
        }
        path
    }

    #[test]
    fn claude_provider_name() {
        assert_eq!(ClaudeProvider::new(vec![]).name(), "claude");
    }

    #[test]
    fn claude_parse_delegates_to_existing_logic() {
        let dir = TempDir::new().unwrap();
        let path = write_jsonl(
            &dir,
            "test.jsonl",
            &[
                serde_json::json!({
                    "type": "user",
                    "sessionId": "s1",
                    "timestamp": "2026-04-08T09:59:00Z",
                    "cwd": "/home/user/project",
                })
                .to_string(),
                serde_json::json!({
                    "type": "assistant",
                    "sessionId": "s1",
                    "timestamp": "2026-04-08T10:00:00Z",
                    "cwd": "/home/user/project",
                    "message": {
                        "model": "claude-sonnet-4-6",
                        "usage": {
                            "input_tokens": 100,
                            "output_tokens": 50,
                            "cache_read_input_tokens": 0,
                            "cache_creation_input_tokens": 0,
                        },
                        "content": [],
                    }
                })
                .to_string(),
            ],
        );

        let provider = ClaudeProvider::new(vec![]);
        let turns = provider.parse(&path).unwrap();
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].provider, "claude");
        assert_eq!(turns[0].input_tokens, 100);
    }
}
