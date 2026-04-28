//! Account-export ZIP importer (Phase 2 / Tier 2).

pub mod anthropic;
pub mod detect;
pub mod openai;
pub mod storage;
pub mod watch;

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;
use tracing::info;

use crate::archive::Archive;

use self::detect::Vendor;
use self::storage::{ImportDir, ImportMetadata};

const PARSER_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct ImportReport {
    pub vendor: Vendor,
    pub import_id: String,
    pub conversation_count: usize,
    pub parse_warnings: Vec<String>,
    pub root: PathBuf,
}

/// Import a single ZIP into the archive at `archive_root`.
pub fn import_zip(archive_root: &Path, zip_path: &Path) -> Result<ImportReport> {
    // Ensure archive root exists; reuses Archive::at semantics.
    let _archive = Archive::at(archive_root.to_path_buf())?;

    let f = File::open(zip_path).with_context(|| format!("opening {}", zip_path.display()))?;
    let mut zip = zip::ZipArchive::new(BufReader::new(f))
        .with_context(|| format!("reading zip {}", zip_path.display()))?;

    let vendor = detect::detect_archive(&mut zip)?;
    if vendor == Vendor::Unknown {
        anyhow::bail!(
            "{}: not a recognised export ZIP (no conversations.json or vendor JSON)",
            zip_path.display()
        );
    }

    let dir = ImportDir::create(archive_root, vendor.slug())?;
    dir.copy_original(zip_path)?;

    let (count, fingerprint, warnings) = match vendor {
        Vendor::OpenAI => write_openai_conversations(&mut zip, &dir)?,
        Vendor::Anthropic => write_anthropic_conversations(&mut zip, &dir)?,
        Vendor::Unknown => unreachable!(),
    };

    let meta = ImportMetadata {
        import_id: dir.import_id.clone(),
        vendor: vendor.slug().to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        heimdall_version: env!("CARGO_PKG_VERSION").to_string(),
        parser_version: PARSER_VERSION,
        schema_fingerprint: fingerprint,
        conversation_count: count,
        parse_warnings: warnings.clone(),
    };
    dir.write_metadata(&meta)?;
    dir.write_parse_errors(&warnings)?;

    info!(
        target: "archive::imports",
        "imported {} conversations from {} ({})",
        count,
        zip_path.display(),
        vendor.slug(),
    );

    Ok(ImportReport {
        vendor,
        import_id: dir.import_id,
        conversation_count: count,
        parse_warnings: warnings,
        root: dir.root,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImportSummary {
    pub import_id: String,
    pub vendor: String,
    pub created_at: String,
    pub conversation_count: usize,
    pub parser_version: u32,
    pub schema_fingerprint: Option<String>,
}

/// List every import under `<archive_root>/exports/<vendor>/*`.
pub fn list_imports(archive_root: &Path) -> Result<Vec<ImportSummary>> {
    let exports = archive_root.join("exports");
    if !exports.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for vendor_entry in std::fs::read_dir(&exports)? {
        let vendor_entry = vendor_entry?;
        if !vendor_entry.file_type()?.is_dir() {
            continue;
        }
        for import_entry in std::fs::read_dir(vendor_entry.path())? {
            let import_entry = import_entry?;
            if !import_entry.file_type()?.is_dir() {
                continue;
            }
            let meta_path = import_entry.path().join("metadata.json");
            if !meta_path.is_file() {
                continue;
            }
            let bytes = std::fs::read(&meta_path)?;
            let meta: storage::ImportMetadata = serde_json::from_slice(&bytes)
                .with_context(|| format!("parsing {}", meta_path.display()))?;
            out.push(ImportSummary {
                import_id: meta.import_id,
                vendor: meta.vendor,
                created_at: meta.created_at,
                conversation_count: meta.conversation_count,
                parser_version: meta.parser_version,
                schema_fingerprint: meta.schema_fingerprint,
            });
        }
    }
    out.sort_by(|a, b| b.import_id.cmp(&a.import_id));
    Ok(out)
}

fn write_openai_conversations<R: std::io::Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
    dir: &ImportDir,
) -> Result<(usize, Option<String>, Vec<String>)> {
    let convs = openai::read_conversations_from_zip(zip)?;
    let mut warnings = Vec::new();
    let mut count = 0;
    for c in &convs {
        let key = openai::conversation_key(c).unwrap_or_else(|| format!("conv-{count}"));
        let value = serde_json::to_value(c)?;
        if let Err(e) = dir.write_conversation_json(&key, &value) {
            warnings.push(format!("{key}: {e}"));
        } else {
            count += 1;
        }
    }
    Ok((count, None, warnings))
}

fn write_anthropic_conversations<R: std::io::Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
    dir: &ImportDir,
) -> Result<(usize, Option<String>, Vec<String>)> {
    let entries = anthropic::read_json_entries_from_zip(zip)?;
    let mut all_convs: Vec<Value> = Vec::new();
    for (_name, value) in entries {
        all_convs.extend(anthropic::collect_conversations(&value));
    }
    let fingerprint = anthropic::schema_fingerprint(&all_convs);

    let mut warnings = Vec::new();
    let mut count = 0;
    for value in all_convs {
        match anthropic::normalize(value.clone()) {
            Some(c) => {
                if let Err(e) = dir.write_conversation_json(&c.id, &value) {
                    warnings.push(format!("{}: {e}", c.id));
                } else {
                    count += 1;
                }
            }
            None => {
                warnings.push(format!(
                    "skipped value without id-field: {}",
                    serde_json::to_string(&value)
                        .unwrap_or_default()
                        .chars()
                        .take(80)
                        .collect::<String>()
                ));
            }
        }
    }
    Ok((count, Some(fingerprint), warnings))
}
