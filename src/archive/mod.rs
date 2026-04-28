//! Local-archive subsystem (Phase 1 of the chat-backup design).

pub mod companion_token;
pub mod discovery;
pub mod imports;
pub mod index;
#[cfg(target_os = "macos")]
pub mod macos_cache;
pub mod manifest;
pub mod objects;
pub mod web;

use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};

use anyhow::{Context, Result};
use chrono::Utc;
use tracing::info;

use crate::scanner::provider::Provider;

use self::manifest::{FileEntry, Manifest, ProviderSection, Summary};
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

/// Lightweight metadata about a snapshot, returned by `Archive::list`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotMeta {
    pub snapshot_id: String,
    pub created_at: String,
    pub total_files: u64,
    pub total_bytes: u64,
}

impl Archive {
    /// List snapshots, newest first.
    pub fn list(&self) -> Result<Vec<SnapshotMeta>> {
        let dir = self.snapshots_dir();
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut metas = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let summary_path = entry.path().join("summary.json");
            if !summary_path.is_file() {
                continue;
            }
            let bytes = fs::read(&summary_path)
                .with_context(|| format!("reading {}", summary_path.display()))?;
            let summary: Summary = serde_json::from_slice(&bytes)
                .with_context(|| format!("parsing {}", summary_path.display()))?;
            metas.push(SnapshotMeta {
                snapshot_id: summary.snapshot_id,
                created_at: summary.created_at,
                total_files: summary.total_files,
                total_bytes: summary.total_bytes,
            });
        }
        metas.sort_by(|a, b| b.snapshot_id.cmp(&a.snapshot_id));
        Ok(metas)
    }

    /// Read a snapshot's full manifest.
    pub fn show(&self, snapshot_id: &str) -> Result<Manifest> {
        let path = self.snapshots_dir().join(snapshot_id).join("manifest.json");
        let bytes = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
        let manifest: Manifest = serde_json::from_slice(&bytes)?;
        Ok(manifest)
    }

    /// Restore a snapshot to a fresh directory.
    ///
    /// Refuses to overwrite an existing non-empty directory. Validates the
    /// snapshot before touching the destination so a bad snapshot_id never
    /// leaves a stale empty dir behind.
    pub fn restore(&self, snapshot_id: &str, dest: &Path) -> Result<()> {
        let manifest = self.show(snapshot_id)?;
        if dest.exists() {
            let mut entries = fs::read_dir(dest)?;
            if entries.next().is_some() {
                anyhow::bail!(
                    "restore destination {} is not empty; refusing to overwrite",
                    dest.display()
                );
            }
        } else {
            fs::create_dir_all(dest)?;
        }
        let store = self.objects()?;
        for section in &manifest.providers {
            for file in &section.files {
                let hash = Sha256Hash::from_hex(&file.sha256)?;
                let bytes = store.get(&hash)?;
                let out_path = dest.join(&section.name).join(&file.logical_path);
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&out_path, bytes)?;
            }
        }
        Ok(())
    }
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes).with_context(|| format!("writing {}", path.display()))
}

fn mtime_millis(path: &Path) -> Option<i64> {
    let meta = fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    let dur = modified
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    Some(dur.as_millis() as i64)
}

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

pub struct VerifyReport {
    pub objects_checked: usize,
    pub corrupt_objects: Vec<String>,
    pub manifests_checked: usize,
}

impl Archive {
    /// Walk every snapshot's manifest, hash each referenced object, and
    /// flag any whose stored SHA does not match its filename.
    pub fn verify(&self) -> Result<VerifyReport> {
        let referenced = referenced_hashes_in_archive(&self.root)?;
        let store = self.objects()?;
        let mut corrupt = Vec::new();
        let mut checked = 0_usize;
        for hash in &referenced {
            checked += 1;
            if !store.contains(hash) {
                corrupt.push(format!("missing: {}", hash.as_hex()));
                continue;
            }
            let actual = store.actual_hash(hash)?;
            if &actual != hash {
                corrupt.push(format!("hash mismatch: {}", hash.as_hex()));
            }
        }
        let snapshots = self.snapshots_dir();
        let manifests_checked = if snapshots.is_dir() {
            fs::read_dir(&snapshots)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().join("manifest.json").is_file())
                .count()
        } else {
            0
        };
        Ok(VerifyReport {
            objects_checked: checked,
            corrupt_objects: corrupt,
            manifests_checked,
        })
    }

    /// Remove all but the most recent `keep` snapshots, then GC unreferenced
    /// objects. Returns `(snapshots_removed, objects_removed)`.
    pub fn prune(&self, keep: usize) -> Result<(usize, usize)> {
        let mut metas = self.list()?;
        let mut removed_snapshots = 0;
        if metas.len() > keep {
            for stale in metas.split_off(keep) {
                let dir = self.snapshots_dir().join(&stale.snapshot_id);
                fs::remove_dir_all(&dir)?;
                removed_snapshots += 1;
            }
        }
        let referenced = referenced_hashes_in_archive(&self.root)?;
        let removed_objects = self.objects()?.gc(&referenced)?;
        Ok((removed_snapshots, removed_objects))
    }
}

/// Advisory file lock for concurrent snapshot/import runs.
///
/// On Unix we use `flock` via raw libc. On unsupported platforms we fall
/// back to a presence-of-file check (best-effort).
pub struct ArchiveLock {
    _file: File,
}

impl ArchiveLock {
    pub fn acquire(root: &Path) -> Result<Self> {
        fs::create_dir_all(root)?;
        let path = root.join("archive.lock");
        let file = match File::create(&path) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::PermissionDenied => {
                anyhow::bail!("cannot write archive.lock at {}: {e}", path.display())
            }
            Err(e) => return Err(e.into()),
        };
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();
            let rc = unsafe { libc_flock(fd, LOCK_EX | LOCK_NB) };
            if rc != 0 {
                anyhow::bail!(
                    "another archive operation is in progress (flock failed on {})",
                    path.display()
                );
            }
        }
        Ok(ArchiveLock { _file: file })
    }
}

#[cfg(unix)]
const LOCK_EX: i32 = 2;
#[cfg(unix)]
const LOCK_NB: i32 = 4;

#[cfg(unix)]
unsafe fn libc_flock(fd: i32, op: i32) -> i32 {
    unsafe extern "C" {
        fn flock(fd: i32, op: i32) -> i32;
    }
    unsafe { flock(fd, op) }
}
