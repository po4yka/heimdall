//! Detect the vendor of an export ZIP by probing its top-level entries.
//!
//! Detection rules (in order):
//!   1. ZIP contains a top-level `conversations.json` → OpenAI ChatGPT.
//!   2. ZIP contains any other `.json` file (heuristic until a real
//!      Anthropic export is observed) → Anthropic.
//!   3. Otherwise → `Unknown`.

use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;

use anyhow::{Context, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vendor {
    OpenAI,
    Anthropic,
    Unknown,
}

impl Vendor {
    pub fn slug(self) -> &'static str {
        match self {
            Vendor::OpenAI => "openai",
            Vendor::Anthropic => "anthropic",
            Vendor::Unknown => "unknown",
        }
    }
}

pub fn detect_zip(path: &Path) -> Result<Vendor> {
    let f = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut zip = zip::ZipArchive::new(BufReader::new(f))
        .with_context(|| format!("reading zip {}", path.display()))?;
    detect_archive(&mut zip)
}

pub(crate) fn detect_archive<R: Read + Seek>(zip: &mut zip::ZipArchive<R>) -> Result<Vendor> {
    let mut names: Vec<String> = (0..zip.len())
        .filter_map(|i| zip.by_index(i).ok().map(|e| e.name().to_string()))
        .collect();
    names.sort();
    if names
        .iter()
        .any(|n| n == "conversations.json" || n.ends_with("/conversations.json"))
    {
        return Ok(Vendor::OpenAI);
    }
    let openai_known = [
        "conversations.json",
        "chat.html",
        "message_feedback.json",
        "model_comparisons.json",
        "shared_conversations.json",
        "user.json",
    ];
    if names
        .iter()
        .any(|n| n.ends_with(".json") && !openai_known.iter().any(|k| n.ends_with(k)))
    {
        return Ok(Vendor::Anthropic);
    }
    Ok(Vendor::Unknown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    fn build_zip(entries: &[(&str, &[u8])]) -> Cursor<Vec<u8>> {
        let buf = Vec::new();
        let mut writer = ZipWriter::new(Cursor::new(buf));
        for (name, content) in entries {
            writer
                .start_file(*name, SimpleFileOptions::default())
                .unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap()
    }

    #[test]
    fn detects_openai_via_conversations_json() {
        let mut zip_bytes = build_zip(&[("conversations.json", b"[]")]);
        zip_bytes.set_position(0);
        let mut zip = zip::ZipArchive::new(zip_bytes).unwrap();
        assert_eq!(detect_archive(&mut zip).unwrap(), Vendor::OpenAI);
    }

    #[test]
    fn detects_anthropic_via_other_json() {
        let mut zip_bytes = build_zip(&[("conversations_data.json", b"{}")]);
        zip_bytes.set_position(0);
        let mut zip = zip::ZipArchive::new(zip_bytes).unwrap();
        assert_eq!(detect_archive(&mut zip).unwrap(), Vendor::Anthropic);
    }

    #[test]
    fn unknown_zip_returns_unknown() {
        let mut zip_bytes = build_zip(&[("README.txt", b"hi")]);
        zip_bytes.set_position(0);
        let mut zip = zip::ZipArchive::new(zip_bytes).unwrap();
        assert_eq!(detect_archive(&mut zip).unwrap(), Vendor::Unknown);
    }
}
