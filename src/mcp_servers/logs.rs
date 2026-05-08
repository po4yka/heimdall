use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use chrono::{DateTime, Utc};

use super::LogProbe;

/// Probe the log file for an MCP server by name.
/// Looks for `<log_dir>/mcp-server-<name>.log`.
pub fn probe_log(log_dir: &Path, name: &str) -> Option<LogProbe> {
    let log_path = log_dir.join(format!("mcp-server-{name}.log"));

    let mut file = match std::fs::File::open(&log_path) {
        Ok(f) => f,
        Err(_) => return None,
    };

    let metadata = match file.metadata() {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("mcp_servers: cannot stat {}: {e}", log_path.display());
            return None;
        }
    };

    let bytes = metadata.len();

    let modified = metadata
        .modified()
        .ok()
        .map(|t| {
            let dt: DateTime<Utc> = t.into();
            dt.to_rfc3339()
        })
        .unwrap_or_else(|| Utc::now().to_rfc3339());

    // Read last 4096 bytes and count newlines.
    let read_buf_size: u64 = 4096;
    let offset = bytes.saturating_sub(read_buf_size);

    let recent_line_count = if bytes == 0 {
        0
    } else {
        if let Err(e) = file.seek(SeekFrom::Start(offset)) {
            tracing::warn!("mcp_servers: seek failed on {}: {e}", log_path.display());
            0
        } else {
            let mut buf = Vec::new();
            if let Err(e) = file.read_to_end(&mut buf) {
                tracing::warn!("mcp_servers: read failed on {}: {e}", log_path.display());
                0
            } else {
                buf.iter().filter(|&&b| b == b'\n').count()
            }
        }
    };

    Some(LogProbe {
        path: log_path.to_string_lossy().into_owned(),
        bytes,
        modified,
        recent_line_count,
    })
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn probe_existing_log_file() {
        let dir = TempDir::new().unwrap();
        let name = "my-server";
        let log_path = dir.path().join(format!("mcp-server-{name}.log"));

        let mut f = std::fs::File::create(&log_path).unwrap();
        writeln!(f, "line 1").unwrap();
        writeln!(f, "line 2").unwrap();
        writeln!(f, "line 3").unwrap();
        writeln!(f, "line 4").unwrap();
        writeln!(f, "line 5").unwrap();
        drop(f);

        let probe = probe_log(dir.path(), name).unwrap();
        assert!(probe.bytes > 0);
        assert_eq!(probe.recent_line_count, 5);
        assert!(probe.path.contains("mcp-server-my-server.log"));
        // Check modified is valid RFC3339
        assert!(chrono::DateTime::parse_from_rfc3339(&probe.modified).is_ok());
    }

    #[test]
    fn probe_absent_log_returns_none() {
        let dir = TempDir::new().unwrap();
        let result = probe_log(dir.path(), "nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn probe_empty_log_file() {
        let dir = TempDir::new().unwrap();
        let name = "empty-server";
        let log_path = dir.path().join(format!("mcp-server-{name}.log"));
        std::fs::File::create(&log_path).unwrap();

        let probe = probe_log(dir.path(), name).unwrap();
        assert_eq!(probe.bytes, 0);
        assert_eq!(probe.recent_line_count, 0);
    }
}
