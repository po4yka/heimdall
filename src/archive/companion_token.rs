//! Companion bearer token stored at `~/.heimdall/companion-token`.
//!
//! Used by the Phase 3a CLI scrape path (so the local HTTP endpoint can
//! distinguish loopback callers from random localhost software) and by
//! the Phase 3b browser extension (paired once via the token's hex value).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rand::Rng;
use subtle::ConstantTimeEq;

const TOKEN_FILE: &str = "companion-token";
const TOKEN_BYTES: usize = 32;

pub struct CompanionToken {
    hex: String,
}

impl CompanionToken {
    pub fn as_hex(&self) -> &str {
        &self.hex
    }

    pub fn matches(&self, candidate: &str) -> bool {
        let a = self.hex.as_bytes();
        let b = candidate.as_bytes();
        if a.len() != b.len() {
            return false;
        }
        a.ct_eq(b).into()
    }
}

/// Default location: `~/.heimdall/companion-token`.
pub fn default_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".heimdall").join(TOKEN_FILE)
}

/// Read the token; create one with cryptographically random bytes if absent.
pub fn read_or_init(path: &Path) -> Result<CompanionToken> {
    if path.is_file() {
        let hex = fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?
            .trim()
            .to_string();
        if hex.len() == TOKEN_BYTES * 2 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(CompanionToken { hex });
        }
    }
    rotate(path)
}

/// Generate a fresh token and persist it (mode 0600 on Unix).
pub fn rotate(path: &Path) -> Result<CompanionToken> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = [0_u8; TOKEN_BYTES];
    rand::rng().fill_bytes(&mut bytes);
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();

    let mut f = fs::File::create(path).with_context(|| format!("creating {}", path.display()))?;
    f.write_all(hex.as_bytes())?;
    f.write_all(b"\n")?;
    f.sync_all()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms)?;
    }

    Ok(CompanionToken { hex })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn read_or_init_creates_token_when_missing() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("token");
        let t = read_or_init(&p).unwrap();
        assert_eq!(t.as_hex().len(), TOKEN_BYTES * 2);
        assert!(p.is_file());
    }

    #[test]
    fn rotate_generates_a_new_token() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("token");
        let a = read_or_init(&p).unwrap();
        let b = rotate(&p).unwrap();
        assert_ne!(a.as_hex(), b.as_hex());
    }

    #[test]
    fn matches_is_constant_time_correct() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("token");
        let t = read_or_init(&p).unwrap();
        assert!(t.matches(t.as_hex()));
        assert!(!t.matches("not-the-token"));
        assert!(!t.matches(""));
    }

    #[cfg(unix)]
    #[test]
    fn token_file_is_mode_0600() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("token");
        read_or_init(&p).unwrap();
        let mode = fs::metadata(&p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
