//! Detector: ghost agents.
//!
//! Lists `*.md` files in `~/.claude/agents/` (or an injected override for tests).
//! Each filename without the `.md` extension is treated as an agent name.
//!
//! The detector then queries `turns.agent_id` for the set of agents that were
//! actually observed in the database. Any configured agent that never appeared
//! in a turn is a "ghost" — it exists in the filesystem but has never been used.
//!
//! # Severity and waste
//! Ghost agents are configuration clutter rather than a direct spend driver.
//! Severity is always `Low` and `estimated_monthly_waste_nanos` is 0.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::Result;
use rusqlite::Connection;

use super::{Detector, Finding, Severity};

pub struct GhostAgentDetector {
    /// Override path for tests. If `None`, defaults to `~/.claude/agents/`.
    agents_dir_override: Option<PathBuf>,
}

impl GhostAgentDetector {
    pub fn new(agents_dir_override: Option<PathBuf>) -> Self {
        Self {
            agents_dir_override,
        }
    }

    fn agents_dir(&self) -> Option<PathBuf> {
        if let Some(p) = &self.agents_dir_override {
            return Some(p.clone());
        }
        dirs::home_dir().map(|h| h.join(".claude").join("agents"))
    }
}

/// List agent names from `*.md` files in `dir`.
/// Returns an empty set if the directory doesn't exist.
fn configured_agents(dir: &Path) -> HashSet<String> {
    let mut names = HashSet::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return names,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            names.insert(stem.to_string());
        }
    }
    names
}

/// Query the DB for the set of agent IDs that appear in any turn.
fn observed_agents(conn: &Connection) -> Result<HashSet<String>> {
    let mut agents = HashSet::new();
    // SQL kept here for detector-local cohesion; see scanner/db.rs for shared queries.
    let result = conn.prepare(
        "SELECT DISTINCT agent_id FROM turns WHERE agent_id IS NOT NULL AND agent_id != ''",
    );
    match result {
        Ok(mut stmt) => {
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            for row in rows.flatten() {
                agents.insert(row);
            }
        }
        Err(e) => {
            tracing::warn!("GhostAgentDetector: could not query turns.agent_id: {e}");
        }
    }
    Ok(agents)
}

impl Detector for GhostAgentDetector {
    fn name(&self) -> &'static str {
        "ghost_agent"
    }

    fn run(&self, conn: &Connection) -> Result<Vec<Finding>> {
        let dir = match self.agents_dir() {
            Some(d) => d,
            None => return Ok(vec![]),
        };

        let configured = configured_agents(&dir);
        if configured.is_empty() {
            return Ok(vec![]);
        }

        let observed = observed_agents(conn)?;

        let mut ghosts: Vec<&String> = configured
            .iter()
            .filter(|name| !observed.contains(*name))
            .collect();
        // Sort for deterministic output.
        ghosts.sort();

        let findings = ghosts
            .into_iter()
            .map(|name| Finding {
                detector: self.name().into(),
                severity: Severity::Low,
                title: format!("Agent '{name}' is configured but never invoked"),
                detail: format!(
                    "The agent definition '{name}.md' exists in ~/.claude/agents/ but \
                     this agent's ID has never appeared in a recorded turn. \
                     If this agent is no longer needed, removing its definition file \
                     reduces configuration clutter."
                ),
                estimated_monthly_waste_nanos: 0,
            })
            .collect();

        Ok(findings)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;
    use crate::optimizer::mod_tests::empty_db;

    fn agents_dir(dir: &TempDir) -> PathBuf {
        let p = dir.path().join("agents");
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn write_agent(dir: &Path, name: &str) {
        fs::write(dir.join(format!("{name}.md")), format!("# {name}")).unwrap();
    }

    fn det(dir: &Path) -> GhostAgentDetector {
        GhostAgentDetector::new(Some(dir.to_path_buf()))
    }

    // -----------------------------------------------------------------------
    // No finding cases
    // -----------------------------------------------------------------------

    #[test]
    fn missing_agents_dir_produces_no_finding() {
        let (dir, conn) = empty_db();
        let d = GhostAgentDetector::new(Some(dir.path().join("nonexistent")));
        let findings = d.run(&conn).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn empty_agents_dir_produces_no_finding() {
        let (dir, conn) = empty_db();
        let adir = agents_dir(&dir);
        let findings = det(&adir).run(&conn).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn non_md_files_ignored() {
        let (dir, conn) = empty_db();
        let adir = agents_dir(&dir);
        fs::write(adir.join("readme.txt"), "ignored").unwrap();
        let findings = det(&adir).run(&conn).unwrap();
        assert!(findings.is_empty());
    }

    // -----------------------------------------------------------------------
    // Detection: ghost agents
    // -----------------------------------------------------------------------

    #[test]
    fn agent_never_in_db_is_ghost() {
        let (dir, conn) = empty_db();
        let adir = agents_dir(&dir);
        write_agent(&adir, "my-agent");
        let findings = det(&adir).run(&conn).unwrap();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].title.contains("my-agent"));
        assert_eq!(findings[0].severity, Severity::Low);
        assert_eq!(findings[0].estimated_monthly_waste_nanos, 0);
    }

    #[test]
    fn agent_present_in_db_not_reported() {
        let (dir, conn) = empty_db();
        let adir = agents_dir(&dir);
        write_agent(&adir, "active-agent");

        // Insert a turn that references this agent.
        conn.execute(
            "INSERT INTO turns \
             (session_id, provider, timestamp, agent_id) \
             VALUES ('claude:s1', 'claude', '2024-01-01T00:00:00Z', 'active-agent')",
            [],
        )
        .unwrap();

        let findings = det(&adir).run(&conn).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn mixed_active_and_ghost_agents() {
        let (dir, conn) = empty_db();
        let adir = agents_dir(&dir);
        write_agent(&adir, "active");
        write_agent(&adir, "ghost-a");
        write_agent(&adir, "ghost-b");

        conn.execute(
            "INSERT INTO turns \
             (session_id, provider, timestamp, agent_id) \
             VALUES ('claude:s1', 'claude', '2024-01-01T00:00:00Z', 'active')",
            [],
        )
        .unwrap();

        let findings = det(&adir).run(&conn).unwrap();
        assert_eq!(findings.len(), 2);
        let titles: Vec<&str> = findings.iter().map(|f| f.title.as_str()).collect();
        assert!(titles.iter().any(|t| t.contains("ghost-a")));
        assert!(titles.iter().any(|t| t.contains("ghost-b")));
        assert!(!titles.iter().any(|t| t.contains("active")));
    }

    #[test]
    fn findings_are_sorted_deterministically() {
        let (dir, conn) = empty_db();
        let adir = agents_dir(&dir);
        // Write in reverse order.
        write_agent(&adir, "zzz");
        write_agent(&adir, "aaa");
        write_agent(&adir, "mmm");

        let findings = det(&adir).run(&conn).unwrap();
        assert_eq!(findings.len(), 3);
        let names: Vec<&str> = findings
            .iter()
            .map(|f| {
                // Extract name from title "Agent 'X' is configured but never invoked"
                f.title.split('\'').nth(1).unwrap_or("")
            })
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }
}
