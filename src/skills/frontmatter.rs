use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct RawFrontmatter {
    name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FrontmatterStatus {
    Ok,
    Missing,
    Invalid,
}

#[derive(Debug, Clone)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: Option<String>,
    pub raw_desc_chars: usize,
    pub status: FrontmatterStatus,
}

/// Parse the YAML frontmatter block from a SKILL.md file.
///
/// Never returns an error — read/parse failures surface as `Missing` or
/// `Invalid` status so callers always get a usable name (falls back to the
/// parent directory's name).
pub fn parse_skill_md(path: &Path) -> SkillFrontmatter {
    let fallback_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!("skills: cannot read {}: {e}", path.display());
            return SkillFrontmatter {
                name: fallback_name,
                description: None,
                raw_desc_chars: 0,
                status: FrontmatterStatus::Missing,
            };
        }
    };

    let Some(rest) = contents.strip_prefix("---") else {
        return SkillFrontmatter {
            name: fallback_name,
            description: None,
            raw_desc_chars: 0,
            status: FrontmatterStatus::Missing,
        };
    };

    let Some(end) = rest.find("\n---") else {
        return SkillFrontmatter {
            name: fallback_name,
            description: None,
            raw_desc_chars: 0,
            status: FrontmatterStatus::Missing,
        };
    };

    let yaml = &rest[..end];

    match serde_yaml::from_str::<RawFrontmatter>(yaml) {
        Ok(raw) => {
            let raw_desc_chars = raw
                .description
                .as_ref()
                .map(|d| d.chars().count())
                .unwrap_or(0);
            SkillFrontmatter {
                name: raw.name.unwrap_or(fallback_name),
                description: raw.description,
                raw_desc_chars,
                status: FrontmatterStatus::Ok,
            }
        }
        Err(e) => {
            tracing::debug!("skills: invalid frontmatter in {}: {e}", path.display());
            SkillFrontmatter {
                name: fallback_name,
                description: None,
                raw_desc_chars: 0,
                status: FrontmatterStatus::Invalid,
            }
        }
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

    fn write(dir: &TempDir, name: &str, contents: &str) -> std::path::PathBuf {
        let path = dir.path().join(name);
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn parses_name_and_description() {
        let dir = TempDir::new().unwrap();
        let path = write(
            &dir,
            "SKILL.md",
            "---\nname: my-skill\ndescription: Does things.\n---\n\n# Body",
        );
        let fm = parse_skill_md(&path);
        assert_eq!(fm.name, "my-skill");
        assert_eq!(fm.description.as_deref(), Some("Does things."));
        assert_eq!(fm.raw_desc_chars, "Does things.".chars().count());
        assert_eq!(fm.status, FrontmatterStatus::Ok);
    }

    #[test]
    fn missing_frontmatter_uses_dir_name() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("my-cool-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let path = skill_dir.join("SKILL.md");
        fs::write(&path, "# Just a body\nno frontmatter").unwrap();
        let fm = parse_skill_md(&path);
        assert_eq!(fm.name, "my-cool-skill");
        assert!(fm.description.is_none());
        assert_eq!(fm.status, FrontmatterStatus::Missing);
    }

    #[test]
    fn invalid_yaml_reports_invalid_status() {
        let dir = TempDir::new().unwrap();
        let path = write(
            &dir,
            "SKILL.md",
            "---\nname: [unclosed bracket\ndescription: x\n---\n",
        );
        let fm = parse_skill_md(&path);
        assert_eq!(fm.status, FrontmatterStatus::Invalid);
    }

    #[test]
    fn multiline_description_counts_chars() {
        let dir = TempDir::new().unwrap();
        let path = write(
            &dir,
            "SKILL.md",
            "---\nname: foo\ndescription: |\n  Line one.\n  Line two.\n---\n",
        );
        let fm = parse_skill_md(&path);
        assert_eq!(fm.status, FrontmatterStatus::Ok);
        // serde_yaml collapses the literal block scalar into a single string with a trailing newline
        assert!(fm.raw_desc_chars > 0);
    }

    #[test]
    fn missing_file_returns_missing_status() {
        let path = std::path::PathBuf::from("/nonexistent/SKILL.md");
        let fm = parse_skill_md(&path);
        assert_eq!(fm.status, FrontmatterStatus::Missing);
    }
}
