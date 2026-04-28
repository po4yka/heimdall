//! Local-archive subsystem (Phase 1 of the chat-backup design).
//!
//! Owns:
//! - content-addressed object store under `<archive_root>/objects/sha256/...`
//! - snapshot manifests under `<archive_root>/snapshots/<id>/` (Task 3+)
//! - per-archive `index.sqlite` rebuildable from on-disk state (Task 9)
//!
//! The archive root is `~/.heimdall/archive/` by default; tests pass an
//! explicit `tempdir` to keep the user's real archive untouched.

pub mod index;
pub mod objects;
