//! Snapshot manifest schema.
//!
//! `manifest.json` is the authoritative record of *what* was in a snapshot.
//! `summary.json` is a small derived rollup for fast dashboard listing.
//! Both files live under `<archive_root>/snapshots/<snapshot_id>/`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub snapshot_id: String,
    pub created_at: String, // RFC3339
    pub heimdall_version: String,
    pub providers: Vec<ProviderSection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderSection {
    pub name: String,
    /// Absolute provider root as a UTF-8 string with forward slashes, so
    /// snapshots written on one OS round-trip cleanly on another.
    pub root: String,
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    /// Path relative to the provider's `root`, with forward slashes.
    pub logical_path: String,
    /// Lowercase hex SHA-256 of the file contents.
    pub sha256: String,
    /// Bytes on disk at snapshot time.
    pub size: u64,
    /// Last-modified time as Unix milliseconds at snapshot time.
    pub mtime_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    pub snapshot_id: String,
    pub created_at: String,
    pub total_bytes: u64,
    pub total_files: u64,
    pub providers: Vec<SummaryProvider>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryProvider {
    pub name: String,
    pub file_count: u64,
    pub byte_count: u64,
}

impl Manifest {
    /// Compute the rollup summary from a manifest.
    pub fn summary(&self) -> Summary {
        let providers: Vec<SummaryProvider> = self
            .providers
            .iter()
            .map(|p| SummaryProvider {
                name: p.name.clone(),
                file_count: p.files.len() as u64,
                byte_count: p.files.iter().map(|f| f.size).sum(),
            })
            .collect();
        let total_files = providers.iter().map(|p| p.file_count).sum();
        let total_bytes = providers.iter().map(|p| p.byte_count).sum();
        Summary {
            snapshot_id: self.snapshot_id.clone(),
            created_at: self.created_at.clone(),
            total_bytes,
            total_files,
            providers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_round_trips_through_json() {
        let m = Manifest {
            snapshot_id: "2026-04-28T080000Z".into(),
            created_at: "2026-04-28T08:00:00Z".into(),
            heimdall_version: "1.0.0".into(),
            providers: vec![ProviderSection {
                name: "claude".into(),
                root: "/home/u/.claude/projects".into(),
                files: vec![FileEntry {
                    logical_path: "proj-a/sess-1.jsonl".into(),
                    sha256: "abcd".repeat(16),
                    size: 1234,
                    mtime_ms: 1_700_000_000_000,
                }],
            }],
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: Manifest = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn summary_aggregates_per_provider_byte_counts() {
        let m = Manifest {
            snapshot_id: "x".into(),
            created_at: "x".into(),
            heimdall_version: "x".into(),
            providers: vec![
                ProviderSection {
                    name: "claude".into(),
                    root: "/r1".into(),
                    files: vec![
                        FileEntry { logical_path: "a".into(), sha256: "0".into(), size: 100, mtime_ms: 0 },
                        FileEntry { logical_path: "b".into(), sha256: "0".into(), size: 50, mtime_ms: 0 },
                    ],
                },
                ProviderSection {
                    name: "codex".into(),
                    root: "/r2".into(),
                    files: vec![
                        FileEntry { logical_path: "c".into(), sha256: "0".into(), size: 25, mtime_ms: 0 },
                    ],
                },
            ],
        };
        let s = m.summary();
        assert_eq!(s.total_files, 3);
        assert_eq!(s.total_bytes, 175);
        let claude = s.providers.iter().find(|p| p.name == "claude").unwrap();
        assert_eq!(claude.file_count, 2);
        assert_eq!(claude.byte_count, 150);
    }
}
