/// File-based cache + PID semaphore for the statusline subcommand.
///
/// Cache file: `$XDG_CACHE_HOME/heimdall/statusline.json`
///             or `~/.cache/heimdall/statusline.json`
/// Lock file:  `~/.cache/heimdall/statusline.lock`
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

// ── Cache directory ──────────────────────────────────────────────────────────

pub fn cache_dir() -> PathBuf {
    // Respect XDG_CACHE_HOME if set.
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME")
        && !xdg.is_empty()
    {
        return PathBuf::from(xdg).join("heimdall");
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache")
        .join("heimdall")
}

fn cache_file() -> PathBuf {
    cache_dir().join("statusline.json")
}

fn lock_file() -> PathBuf {
    cache_dir().join("statusline.lock")
}

// ── Cache entry ───────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    pub session_id: String,
    pub computed_at: DateTime<Utc>,
    pub transcript_mtime_secs: i64,
    pub output: String,
}

pub fn read_cache() -> Option<CacheEntry> {
    let text = fs::read_to_string(cache_file()).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn write_cache(entry: &CacheEntry) -> Result<()> {
    let dir = cache_dir();
    fs::create_dir_all(&dir).with_context(|| format!("creating cache dir {}", dir.display()))?;
    let path = cache_file();
    // Write to a temp file then atomically rename so a concurrent reader never
    // observes a partially-written (truncated) JSON blob.
    let tmp_path = path.with_extension("tmp");
    {
        let mut f = fs::File::create(&tmp_path)
            .with_context(|| format!("creating tmp cache file {}", tmp_path.display()))?;
        serde_json::to_writer(&mut f, entry)?;
        f.sync_all()?;
    }
    fs::rename(&tmp_path, &path)
        .with_context(|| format!("renaming tmp cache to {}", path.display()))?;
    Ok(())
}

/// Return `true` when the cached entry is still valid:
/// - same session_id
/// - within TTL
/// - transcript mtime has not changed
pub fn is_fresh(
    entry: &CacheEntry,
    session_id: &str,
    transcript_path: &Path,
    ttl: Duration,
) -> bool {
    if entry.session_id != session_id {
        return false;
    }
    let age = Utc::now().signed_duration_since(entry.computed_at);
    // age.num_seconds() < 0 means computed_at is in the future (clock skew) —
    // treat as stale.  The to_std() conversion only fails for negative
    // durations, which the prior guard already excludes, so we treat a
    // conversion failure as stale rather than carrying an unreachable
    // `unwrap_or(Duration::MAX)`.
    if age.num_seconds() < 0 {
        return false;
    }
    let Ok(age_std) = age.to_std() else {
        return false;
    };
    if age_std >= ttl {
        return false;
    }
    // Check transcript mtime.
    let current_mtime = transcript_mtime(transcript_path).unwrap_or(-1);
    current_mtime == entry.transcript_mtime_secs
}

/// Read the current mtime of a file as Unix seconds.
pub fn transcript_mtime(path: &Path) -> Option<i64> {
    use std::time::UNIX_EPOCH;
    let meta = fs::metadata(path).ok()?;
    let mtime = meta.modified().ok()?;
    let secs = mtime.duration_since(UNIX_EPOCH).ok()?.as_secs();
    Some(secs as i64)
}

// ── PID lock ─────────────────────────────────────────────────────────────────

/// RAII guard: removes the lock file when dropped.
pub struct LockGuard {
    path: PathBuf,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Acquire the PID lock file.
///
/// Algorithm:
/// 1. Atomic create via O_EXCL. On success, write own PID.
/// 2. If `AlreadyExists`: read stored PID and check liveness.
///    - Dead PID → delete and retry once.
///    - Alive PID → sleep 10 ms, retry until `timeout`.
/// 3. If lock cannot be acquired within `timeout`, return `Err`.
pub fn acquire_lock(timeout: Duration) -> Result<LockGuard> {
    let lock_path = lock_file();
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let deadline = std::time::Instant::now() + timeout;

    loop {
        match try_create_lock(&lock_path) {
            Ok(guard) => return Ok(guard),
            Err(e) if is_already_exists(&e) => {
                // Check liveness of existing lock holder.
                let stale = match read_lock_pid(&lock_path) {
                    Some(pid) => !pid_alive(pid),
                    None => true, // unreadable / corrupt → treat as stale
                };
                if stale {
                    // Delete stale lock and retry immediately.
                    let _ = fs::remove_file(&lock_path);
                    if let Ok(guard) = try_create_lock(&lock_path) {
                        return Ok(guard);
                    }
                    // else: lost the race; fall through to sleep
                }
            }
            Err(e) => return Err(e).context("acquiring statusline lock"),
        }

        if std::time::Instant::now() >= deadline {
            anyhow::bail!("could not acquire statusline lock within {:?}", timeout);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn try_create_lock(path: &Path) -> std::io::Result<LockGuard> {
    let mut opts = fs::OpenOptions::new();
    opts.write(true).create_new(true);
    let mut f = opts.open(path)?;
    // Write own PID as plain text.
    let pid = std::process::id();
    let _ = write!(f, "{}", pid);
    Ok(LockGuard {
        path: path.to_path_buf(),
    })
}

fn is_already_exists(e: &std::io::Error) -> bool {
    e.kind() == std::io::ErrorKind::AlreadyExists
}

fn read_lock_pid(path: &Path) -> Option<u32> {
    let text = fs::read_to_string(path).ok()?;
    text.trim().parse::<u32>().ok()
}

/// Check whether a process with the given PID is alive.
///
/// On Linux, checks `/proc/<pid>` (no subprocess needed).
/// On other Unix, runs `kill -0 <pid>` via a shell-less subprocess.
/// On non-Unix, conservatively returns `true` (assume alive).
fn pid_alive(pid: u32) -> bool {
    #[cfg(target_os = "linux")]
    {
        std::path::Path::new(&format!("/proc/{}", pid)).exists()
    }
    #[cfg(all(unix, not(target_os = "linux")))]
    {
        // `kill -0 pid` exits 0 if the process exists and we have permission.
        std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn lock_guard_drop_removes_file() {
        let dir = TempDir::new().unwrap();
        let lock_path = dir.path().join("test.lock");

        // Manually create a lock file and wrap it.
        fs::File::create(&lock_path).unwrap();
        {
            let _guard = LockGuard {
                path: lock_path.clone(),
            };
            assert!(lock_path.exists());
        }
        assert!(!lock_path.exists(), "lock file must be removed on drop");
    }

    #[test]
    fn is_fresh_false_on_session_mismatch() {
        let entry = CacheEntry {
            session_id: "a".into(),
            computed_at: Utc::now(),
            transcript_mtime_secs: 0,
            output: "x".into(),
        };
        assert!(!is_fresh(
            &entry,
            "b",
            Path::new("/nonexistent"),
            Duration::from_secs(60)
        ));
    }

    #[test]
    fn is_fresh_false_when_ttl_expired() {
        let old = Utc::now() - chrono::Duration::seconds(120);
        let entry = CacheEntry {
            session_id: "s".into(),
            computed_at: old,
            transcript_mtime_secs: 0,
            output: "x".into(),
        };
        assert!(!is_fresh(
            &entry,
            "s",
            Path::new("/nonexistent"),
            Duration::from_secs(30)
        ));
    }

    #[test]
    fn is_fresh_false_when_mtime_changed() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("transcript.jsonl");
        fs::write(&path, b"data").unwrap();

        let mtime = transcript_mtime(&path).unwrap();
        let entry = CacheEntry {
            session_id: "s".into(),
            computed_at: Utc::now(),
            transcript_mtime_secs: mtime + 1, // simulate change
            output: "x".into(),
        };
        assert!(!is_fresh(&entry, "s", &path, Duration::from_secs(60)));
    }

    #[test]
    fn is_fresh_true_when_all_valid() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("transcript.jsonl");
        fs::write(&path, b"data").unwrap();

        let mtime = transcript_mtime(&path).unwrap();
        let entry = CacheEntry {
            session_id: "s".into(),
            computed_at: Utc::now(),
            transcript_mtime_secs: mtime,
            output: "x".into(),
        };
        assert!(is_fresh(&entry, "s", &path, Duration::from_secs(60)));
    }

    #[cfg(unix)]
    #[test]
    fn stale_pid_is_detected() {
        // PID u32::MAX / 2_147_483_647 is almost never alive.
        let very_large_pid: u32 = 2_147_483_647;
        // On Linux/macOS PIDs are bounded well below this.
        assert!(!pid_alive(very_large_pid));
    }

    #[cfg(unix)]
    #[test]
    fn own_pid_is_alive() {
        let own_pid = std::process::id();
        assert!(pid_alive(own_pid));
    }
}
