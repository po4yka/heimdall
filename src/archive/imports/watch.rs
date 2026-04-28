//! Folder watcher: ingest export ZIPs as they appear.
//!
//! Implementation: poll-style watcher (notify crate). On Create + Modify
//! events for `.zip` files, runs `import_zip` and logs the outcome. The
//! foreground process blocks until Ctrl-C.

use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::{Context, Result};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::{info, warn};

use super::import_zip;

pub fn run_watch(archive_root: &Path, watch_dir: &Path) -> Result<()> {
    if !watch_dir.is_dir() {
        anyhow::bail!(
            "watch dir {} does not exist or is not a directory",
            watch_dir.display()
        );
    }

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(tx).context("starting fs watcher")?;
    watcher
        .watch(watch_dir, RecursiveMode::NonRecursive)
        .with_context(|| format!("watching {}", watch_dir.display()))?;

    info!(
        target: "archive::imports::watch",
        "watching {} for export ZIPs (press Ctrl-C to stop)",
        watch_dir.display()
    );
    eprintln!(
        "Watching {} for *.zip — press Ctrl-C to stop.",
        watch_dir.display()
    );

    // 2-second debounce so we don't fire on a Create event before the
    // browser has finished writing the file.
    let debounce = Duration::from_secs(2);

    loop {
        let event = match rx.recv() {
            Ok(Ok(e)) => e,
            Ok(Err(e)) => {
                warn!("watch error: {e}");
                continue;
            }
            Err(_) => break, // channel closed
        };

        if !matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
            continue;
        }

        for path in event.paths {
            if !is_zip(&path) {
                continue;
            }
            std::thread::sleep(debounce);
            if !path.is_file() {
                continue;
            }
            match import_zip(archive_root, &path) {
                Ok(report) => info!(
                    target: "archive::imports::watch",
                    "imported {} ({} convs) from {}",
                    report.vendor.slug(),
                    report.conversation_count,
                    path.display()
                ),
                Err(e) => warn!(
                    target: "archive::imports::watch",
                    "import failed for {}: {e}",
                    path.display()
                ),
            }
        }
    }

    Ok(())
}

fn is_zip(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("zip"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_zip_recognises_extension() {
        assert!(is_zip(Path::new("/tmp/x.zip")));
        assert!(is_zip(Path::new("/tmp/x.ZIP")));
        assert!(!is_zip(Path::new("/tmp/x.tar.gz")));
        assert!(!is_zip(Path::new("/tmp/x")));
    }
}
