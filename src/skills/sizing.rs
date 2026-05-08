use std::path::Path;

use walkdir::WalkDir;

/// Recursively sum file sizes under `dir` without following symlinks.
///
/// Returns `(file_count, total_bytes)`. Symlink entries are counted as their
/// link size (via `lstat`), not the size of the target, to avoid double-
/// counting and out-of-tree escapes.
pub fn bundle_bytes(dir: &Path) -> (u32, u64) {
    let mut file_count = 0u32;
    let mut total_bytes = 0u64;

    for entry in WalkDir::new(dir).follow_links(false).into_iter().flatten() {
        let ft = entry.file_type();
        if ft.is_file() || ft.is_symlink() {
            // Use metadata() which gives lstat for symlinks when follow_links is false.
            if let Ok(meta) = entry.metadata() {
                total_bytes = total_bytes.saturating_add(meta.len());
                file_count = file_count.saturating_add(1);
            }
        }
    }

    (file_count, total_bytes)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn sums_nested_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.txt"), "hello").unwrap(); // 5 bytes
        let sub = dir.path().join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("b.txt"), "world!").unwrap(); // 6 bytes
        let (count, bytes) = bundle_bytes(dir.path());
        assert_eq!(count, 2);
        assert_eq!(bytes, 11);
    }

    #[test]
    fn empty_dir_returns_zeros() {
        let dir = TempDir::new().unwrap();
        let (count, bytes) = bundle_bytes(dir.path());
        assert_eq!(count, 0);
        assert_eq!(bytes, 0);
    }

    #[test]
    fn nonexistent_dir_returns_zeros() {
        let (count, bytes) = bundle_bytes(std::path::Path::new("/nonexistent/path/abc"));
        assert_eq!(count, 0);
        assert_eq!(bytes, 0);
    }
}
