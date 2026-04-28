//! Local-archive subsystem (Phase 1 of the chat-backup design).

pub mod discovery;
pub mod index;
pub mod manifest;
pub mod objects;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

use anyhow::{Context, Result};
use chrono::Utc;
use tracing::info;

use crate::scanner::provider::Provider;

use self::manifest::{FileEntry, Manifest, ProviderSection};
use self::objects::{ObjectStore, Sha256Hash};

/// Default archive root: `~/.heimdall/archive/`.
pub fn default_root() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".heimdall").join("archive")
}

/// Opaque handle to an archive at a particular root.
pub struct Archive {
    pub root: PathBuf,
}

impl Archive {
    pub fn at(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root)
            .with_context(|| format!("creating archive root at {}", root.display()))?;
        Ok(Self { root })
    }

    pub fn objects(&self) -> Result<ObjectStore> {
        ObjectStore::open(self.root.join("objects"))
    }

    pub fn snapshots_dir(&self) -> PathBuf {
        self.root.join("snapshots")
    }

    /// Take a content-addressed snapshot of every Provider's `archive_paths()`.
    ///
    /// Returns the snapshot ID (UTC timestamp string). Idempotent in the sense
    /// that running with no source changes only updates the manifest copy
    /// (objects already exist in the store and are not re-written).
    pub fn snapshot(&self, providers: &[Arc<dyn Provider>]) -> Result<String> {
        let store = self.objects()?;
        // Microsecond precision so back-to-back snapshots produce distinct ids
        // (relevant for tests and the CLI smoke flow).
        let snapshot_id = Utc::now().format("%Y-%m-%dT%H%M%S%.6fZ").to_string();
        let staging = self.snapshots_dir().join(&snapshot_id).join(".partial");
        fs::create_dir_all(&staging)?;

        let discovered = discovery::discover(providers)?;
        info!(
            target: "archive::snapshot",
            "discovered {} files across {} providers",
            discovered.len(),
            providers.len()
        );

        let mut sections: Vec<ProviderSection> = Vec::new();
        let mut last_root: Option<(String, PathBuf)> = None;

        for file in discovered {
            let bytes = fs::read(&file.absolute_path)
                .with_context(|| format!("reading {}", file.absolute_path.display()))?;
            let hash = store.put(&bytes)?;
            let mtime_ms = mtime_millis(&file.absolute_path).unwrap_or(0);
            let entry = FileEntry {
                logical_path: file.logical_path,
                sha256: hash.as_hex().to_string(),
                size: bytes.len() as u64,
                mtime_ms,
            };
            let section_key = (file.provider.to_string(), file.root.clone());
            let need_new_section = match last_root.as_ref() {
                Some((p, r)) => p != &section_key.0 || r != &section_key.1,
                None => true,
            };
            if need_new_section {
                sections.push(ProviderSection {
                    name: file.provider.to_string(),
                    // ProviderSection.root is a String (not PathBuf) for portability.
                    root: file.root.to_string_lossy().replace('\\', "/").to_string(),
                    files: Vec::new(),
                });
                last_root = Some(section_key);
            }
            sections.last_mut().expect("just pushed").files.push(entry);
        }

        let manifest = Manifest {
            snapshot_id: snapshot_id.clone(),
            created_at: Utc::now().to_rfc3339(),
            heimdall_version: env!("CARGO_PKG_VERSION").to_string(),
            providers: sections,
        };
        let summary = manifest.summary();

        write_json(&staging.join("manifest.json"), &manifest)?;
        write_json(&staging.join("summary.json"), &summary)?;

        // Atomically promote the .partial children to the parent <snapshot_id>/.
        let final_dir = self.snapshots_dir().join(&snapshot_id);
        for entry in fs::read_dir(&staging)? {
            let entry = entry?;
            let dst = final_dir.join(entry.file_name());
            fs::rename(entry.path(), dst)?;
        }
        fs::remove_dir(&staging)?;

        info!(
            target: "archive::snapshot",
            "snapshot {} complete: {} files, {} bytes",
            snapshot_id, summary.total_files, summary.total_bytes
        );
        Ok(snapshot_id)
    }
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes).with_context(|| format!("writing {}", path.display()))
}

fn mtime_millis(path: &Path) -> Option<i64> {
    let meta = fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    let dur = modified.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
    Some(dur.as_millis() as i64)
}

#[allow(dead_code)]
pub(crate) fn referenced_hashes_in_archive(root: &Path) -> Result<HashSet<Sha256Hash>> {
    let mut set = HashSet::new();
    let snapshots = root.join("snapshots");
    if !snapshots.is_dir() {
        return Ok(set);
    }
    for entry in fs::read_dir(&snapshots)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let manifest_path = entry.path().join("manifest.json");
        if !manifest_path.is_file() {
            continue;
        }
        let bytes = fs::read(&manifest_path)?;
        let manifest: Manifest = serde_json::from_slice(&bytes)?;
        for section in &manifest.providers {
            for file in &section.files {
                set.insert(Sha256Hash::from_hex(&file.sha256)?);
            }
        }
    }
    Ok(set)
}
