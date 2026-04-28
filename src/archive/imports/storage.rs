//! Per-import storage layout writer.
//!
//! Each successful import lands at:
//!   <archive_root>/exports/<vendor>/<import_id>/
//!     original.zip
//!     conversations/<conv_id>.json
//!     metadata.json
//!     parse-errors.json   (only if any conversation failed to parse)

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportMetadata {
    pub import_id: String,
    pub vendor: String,
    pub created_at: String,
    pub heimdall_version: String,
    pub parser_version: u32,
    pub schema_fingerprint: Option<String>,
    pub conversation_count: usize,
    pub parse_warnings: Vec<String>,
}

pub struct ImportDir {
    pub root: PathBuf,
    pub import_id: String,
    pub vendor: String,
}

impl ImportDir {
    pub fn create(archive_root: &Path, vendor: &str) -> Result<Self> {
        let import_id = Utc::now().format("%Y-%m-%dT%H%M%S%.6fZ").to_string();
        let root = archive_root
            .join("exports")
            .join(vendor)
            .join(&import_id);
        fs::create_dir_all(root.join("conversations"))?;
        Ok(Self { root, import_id, vendor: vendor.to_string() })
    }

    pub fn copy_original(&self, src: &Path) -> Result<()> {
        fs::copy(src, self.root.join("original.zip"))
            .with_context(|| format!("copying {} into archive", src.display()))?;
        Ok(())
    }

    pub fn write_conversation_json(&self, conv_id: &str, value: &serde_json::Value) -> Result<()> {
        let safe = sanitize_conv_id(conv_id);
        let path = self.root.join("conversations").join(format!("{safe}.json"));
        let bytes = serde_json::to_vec_pretty(value)?;
        fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
        Ok(())
    }

    pub fn write_metadata(&self, meta: &ImportMetadata) -> Result<()> {
        let path = self.root.join("metadata.json");
        let bytes = serde_json::to_vec_pretty(meta)?;
        fs::write(&path, bytes)?;
        Ok(())
    }

    pub fn write_parse_errors(&self, errors: &[String]) -> Result<()> {
        if errors.is_empty() {
            return Ok(());
        }
        let path = self.root.join("parse-errors.json");
        let bytes = serde_json::to_vec_pretty(errors)?;
        fs::write(&path, bytes)?;
        Ok(())
    }
}

/// Replace any path-unsafe character in a vendor-supplied conversation id.
fn sanitize_conv_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') { c } else { '_' })
        .collect::<String>()
        .chars()
        .take(80)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn create_lays_out_directory_tree() {
        let tmp = TempDir::new().unwrap();
        let dir = ImportDir::create(tmp.path(), "openai").unwrap();
        assert!(dir.root.join("conversations").is_dir());
        assert!(dir.root.starts_with(tmp.path().join("exports").join("openai")));
    }

    #[test]
    fn sanitize_strips_path_traversal() {
        // '/' is unsafe and becomes '_'; '.' is allowed (valid in filenames).
        // "../../etc/passwd" → ".._.._ etc_passwd" with dots preserved.
        assert_eq!(sanitize_conv_id("../../etc/passwd"), ".._.._etc_passwd");
        assert_eq!(sanitize_conv_id("conv-1234_abc.def"), "conv-1234_abc.def");
        // Verify that slashes are the only unsafe chars here.
        assert_eq!(sanitize_conv_id("a/b/c"), "a_b_c");
        // Null bytes and spaces become underscores.
        assert_eq!(sanitize_conv_id("bad id!"), "bad_id_");
    }
}
