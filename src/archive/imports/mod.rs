//! Account-export ZIP importer (Phase 2 / Tier 2).
//!
//! Vendor exports — claude.ai's "Export data" and chatgpt.com's
//! "Export data" — are delivered as ZIP archives. This module probes for
//! a vendor signature, dispatches to a vendor-specific parser, and writes
//! the result under `<archive_root>/exports/<vendor>/<id>/`.

pub mod detect;

use detect::Vendor;

/// Outcome of importing a single ZIP.
#[derive(Debug, Clone)]
pub struct ImportReport {
    pub vendor: Vendor,
    pub import_id: String,
    pub conversation_count: usize,
    pub parse_warnings: Vec<String>,
}
