//! Content-addressed object store.
//!
//! Layout: `<archive_root>/objects/sha256/<aa>/<bb>/<rest-of-hex>`.
//! The 2-level fanout caps directory size for users with millions of objects.

use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

/// 64-character lowercase hex SHA-256 digest.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sha256Hash(String);

impl Sha256Hash {
    /// Compute a SHA-256 over the given bytes.
    pub fn from_bytes(content: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let bytes = hasher.finalize();
        Self(hex_lower(&bytes))
    }

    /// Borrow the 64-char hex string.
    pub fn as_hex(&self) -> &str {
        &self.0
    }

    /// Construct a hash from an existing hex string. Validates length and charset.
    pub fn from_hex(hex: &str) -> Result<Self> {
        if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("invalid sha256 hex: {hex}");
        }
        Ok(Self(hex.to_ascii_lowercase()))
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Object store rooted at `<archive_root>/objects/`.
pub struct ObjectStore {
    root: PathBuf,
}

impl ObjectStore {
    /// Open (creating directory on demand) an object store rooted at `objects_dir`.
    pub fn open(objects_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&objects_dir)
            .with_context(|| format!("creating object store at {}", objects_dir.display()))?;
        Ok(Self { root: objects_dir })
    }

    /// Return the path where a given hash is stored, without creating it.
    pub fn path_for(&self, hash: &Sha256Hash) -> PathBuf {
        let hex = hash.as_hex();
        // sha256/aa/bb/ccdd...
        self.root
            .join("sha256")
            .join(&hex[0..2])
            .join(&hex[2..4])
            .join(&hex[4..])
    }

    /// Whether the object exists on disk.
    pub fn contains(&self, hash: &Sha256Hash) -> bool {
        self.path_for(hash).is_file()
    }

    /// Atomically write `content` to its content-addressed path, returning the
    /// computed hash. No-op if an object with the same hash already exists.
    pub fn put(&self, content: &[u8]) -> Result<Sha256Hash> {
        let hash = Sha256Hash::from_bytes(content);
        let final_path = self.path_for(&hash);
        if final_path.is_file() {
            return Ok(hash);
        }
        let parent = final_path.parent().context("object path missing parent")?;
        fs::create_dir_all(parent)?;
        // Write to a sibling tempfile then rename for atomicity.
        let tmp_path = parent.join(format!(".tmp-{}", hash.as_hex()));
        {
            let mut f = fs::File::create(&tmp_path)
                .with_context(|| format!("creating {}", tmp_path.display()))?;
            f.write_all(content)?;
            f.sync_all()?;
        }
        fs::rename(&tmp_path, &final_path)?;
        Ok(hash)
    }

    /// Read an object back. Errors if the hash is unknown.
    pub fn get(&self, hash: &Sha256Hash) -> Result<Vec<u8>> {
        let path = self.path_for(hash);
        fs::read(&path).with_context(|| format!("reading object {}", path.display()))
    }

    /// Remove every object whose hash is not in `referenced`. Returns the
    /// number of objects deleted.
    pub fn gc(&self, referenced: &HashSet<Sha256Hash>) -> Result<usize> {
        let sha_root = self.root.join("sha256");
        if !sha_root.is_dir() {
            return Ok(0);
        }
        let mut removed = 0_usize;
        for outer in fs::read_dir(&sha_root)? {
            let outer = outer?;
            if !outer.file_type()?.is_dir() {
                continue;
            }
            for inner in fs::read_dir(outer.path())? {
                let inner = inner?;
                if !inner.file_type()?.is_dir() {
                    continue;
                }
                for entry in fs::read_dir(inner.path())? {
                    let entry = entry?;
                    if !entry.file_type()?.is_file() {
                        continue;
                    }
                    let outer_name = outer.file_name().to_string_lossy().to_string();
                    let inner_name = inner.file_name().to_string_lossy().to_string();
                    let entry_name = entry.file_name().to_string_lossy().to_string();
                    if entry_name.starts_with(".tmp-") {
                        // Stale tmp file from a crashed put; clean up.
                        let _ = fs::remove_file(entry.path());
                        continue;
                    }
                    let hex = format!("{outer_name}{inner_name}{entry_name}");
                    let hash = Sha256Hash(hex);
                    if !referenced.contains(&hash) {
                        fs::remove_file(entry.path())?;
                        removed += 1;
                    }
                }
            }
        }
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn sha256_from_empty_input_matches_known_value() {
        let h = Sha256Hash::from_bytes(b"");
        assert_eq!(
            h.as_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn put_then_get_round_trips_bytes() {
        let tmp = TempDir::new().unwrap();
        let store = ObjectStore::open(tmp.path().join("objects")).unwrap();

        let content = b"hello, archive";
        let hash = store.put(content).unwrap();

        assert!(store.contains(&hash));
        assert_eq!(store.get(&hash).unwrap(), content);
    }

    #[test]
    fn put_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let store = ObjectStore::open(tmp.path().join("objects")).unwrap();
        let content = b"same bytes";
        let h1 = store.put(content).unwrap();
        let h2 = store.put(content).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn fanout_layout_has_two_level_directories() {
        let tmp = TempDir::new().unwrap();
        let store = ObjectStore::open(tmp.path().join("objects")).unwrap();
        let hash = store.put(b"x").unwrap();
        let path = store.path_for(&hash);
        let rel = path.strip_prefix(tmp.path().join("objects")).unwrap();
        let components: Vec<_> = rel.components().collect();
        // sha256 / aa / bb / rest
        assert_eq!(components.len(), 4);
    }

    #[test]
    fn from_hex_rejects_invalid_length() {
        assert!(Sha256Hash::from_hex("abcd").is_err());
    }

    #[test]
    fn from_hex_rejects_non_hex_chars() {
        let bad = "g".repeat(64);
        assert!(Sha256Hash::from_hex(&bad).is_err());
    }

    #[test]
    fn from_hex_normalizes_to_lowercase() {
        let upper = "ABCD".repeat(16);
        let h = Sha256Hash::from_hex(&upper).unwrap();
        assert_eq!(h.as_hex(), upper.to_ascii_lowercase());
    }

    #[test]
    fn gc_removes_unreferenced_and_keeps_referenced() {
        let tmp = TempDir::new().unwrap();
        let store = ObjectStore::open(tmp.path().join("objects")).unwrap();
        let keep = store.put(b"keep me").unwrap();
        let drop = store.put(b"drop me").unwrap();
        assert!(store.contains(&keep) && store.contains(&drop));

        let mut referenced = HashSet::new();
        referenced.insert(keep.clone());
        let removed = store.gc(&referenced).unwrap();
        assert_eq!(removed, 1);
        assert!(store.contains(&keep));
        assert!(!store.contains(&drop));
    }

    #[test]
    fn gc_cleans_stale_tmp_files() {
        let tmp = TempDir::new().unwrap();
        let store = ObjectStore::open(tmp.path().join("objects")).unwrap();
        // Put a real object so the fanout dirs exist.
        let real = store.put(b"real").unwrap();
        // Drop a stale tmp sibling next to it.
        let parent = store.path_for(&real).parent().unwrap().to_path_buf();
        let stale = parent.join(".tmp-stale");
        fs::write(&stale, b"junk").unwrap();
        assert!(stale.is_file());

        let mut referenced = HashSet::new();
        referenced.insert(real);
        store.gc(&referenced).unwrap();
        assert!(!stale.is_file(), "GC must remove stale .tmp-* files");
    }
}
