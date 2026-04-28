//! Folder watcher for inbound export ZIPs. Implementation lands in Task 6.

use std::path::Path;
use anyhow::Result;

pub fn run_watch(_archive_root: &Path, _watch_dir: &Path) -> Result<()> {
    anyhow::bail!("watch mode not yet implemented (Phase 2 Task 6)");
}
