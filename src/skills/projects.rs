use std::collections::HashSet;
use std::path::PathBuf;

use rusqlite::Connection;

/// Return a deduplicated, existence-checked list of project root directories.
///
/// If `overrides` is non-empty they are used directly (existence-filtered),
/// replacing DB discovery entirely — this corresponds to the `--path` CLI flag.
///
/// Otherwise the function queries the `turns` table for distinct `cwd` values
/// recorded by all providers and filters to directories that still exist on
/// disk.
pub fn discover_project_paths(conn: &Connection, overrides: &[PathBuf]) -> Vec<PathBuf> {
    if !overrides.is_empty() {
        return overrides
            .iter()
            .filter(|p| p.exists() && p.is_dir())
            .cloned()
            .collect();
    }

    let mut stmt = match conn
        .prepare("SELECT DISTINCT cwd FROM turns WHERE cwd IS NOT NULL AND cwd != '' ORDER BY cwd")
    {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("skills: cannot query project cwds: {e}");
            return vec![];
        }
    };

    let mut seen = HashSet::new();
    let mut paths = Vec::new();

    let rows = match stmt.query_map([], |row| row.get::<_, String>(0)) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("skills: cwd query failed: {e}");
            return vec![];
        }
    };

    for row in rows.flatten() {
        let p = PathBuf::from(&row);
        if p.exists() && p.is_dir() && seen.insert(row) {
            paths.push(p);
        }
    }

    paths
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use tempfile::TempDir;

    use super::*;

    fn init_db() -> (TempDir, Connection) {
        let dir = TempDir::new().unwrap();
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE turns (
                id INTEGER PRIMARY KEY,
                session_id TEXT,
                provider TEXT,
                timestamp TEXT,
                cwd TEXT
            );",
        )
        .unwrap();
        (dir, conn)
    }

    #[test]
    fn overrides_bypass_db() {
        let (dir, conn) = init_db();
        let existing = dir.path().to_path_buf();
        let paths = discover_project_paths(&conn, std::slice::from_ref(&existing));
        assert_eq!(paths, vec![existing]);
    }

    #[test]
    fn nonexistent_override_excluded() {
        let (_dir, conn) = init_db();
        let paths = discover_project_paths(&conn, &[PathBuf::from("/nonexistent/abc/xyz")]);
        assert!(paths.is_empty());
    }

    #[test]
    fn db_discovery_filters_nonexistent() {
        let (dir, conn) = init_db();
        let existing = dir.path().display().to_string();
        conn.execute(
            "INSERT INTO turns (session_id, provider, timestamp, cwd) VALUES ('s1', 'claude', '2024-01-01', ?1)",
            [&existing],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO turns (session_id, provider, timestamp, cwd) VALUES ('s2', 'codex', '2024-01-02', '/nonexistent/path')",
            [],
        )
        .unwrap();
        let paths = discover_project_paths(&conn, &[]);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], dir.path());
    }

    #[test]
    fn deduplicates_same_cwd() {
        let (dir, conn) = init_db();
        let cwd = dir.path().display().to_string();
        for _ in 0..3 {
            conn.execute(
                "INSERT INTO turns (session_id, provider, timestamp, cwd) VALUES ('s1', 'claude', '2024-01-01', ?1)",
                [&cwd],
            )
            .unwrap();
        }
        let paths = discover_project_paths(&conn, &[]);
        assert_eq!(paths.len(), 1);
    }
}
