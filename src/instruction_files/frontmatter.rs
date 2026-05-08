use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FrontmatterStatus {
    Ok,
    Invalid,
    NotApplicable,
}

/// Probe YAML frontmatter validity in a CLAUDE.md file.
///
/// Files with no frontmatter (`---` opening block) are considered valid (`Ok`).
/// Files with a malformed block are `Invalid`.
/// Non-CLAUDE.md files should be passed `FrontmatterStatus::NotApplicable` by
/// callers instead of calling this function.
pub fn probe_status(text: &str) -> FrontmatterStatus {
    if !text.starts_with("---\n") {
        return FrontmatterStatus::Ok;
    }
    let rest = &text[4..];
    if let Some(close) = rest.find("\n---\n").or_else(|| rest.find("\n---")) {
        let yaml_text = &rest[..close];
        match serde_yaml::from_str::<serde_yaml::Value>(yaml_text) {
            Ok(_) => FrontmatterStatus::Ok,
            Err(_) => FrontmatterStatus::Invalid,
        }
    } else {
        FrontmatterStatus::Invalid
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_frontmatter_is_ok() {
        let text = "# CLAUDE.md\nHello world.";
        assert_eq!(probe_status(text), FrontmatterStatus::Ok);
    }

    #[test]
    fn valid_yaml_frontmatter_is_ok() {
        let text = "---\ntitle: My Project\nversion: 1\n---\n# Body";
        assert_eq!(probe_status(text), FrontmatterStatus::Ok);
    }

    #[test]
    fn opening_dashes_with_no_close_is_invalid() {
        let text = "---\ntitle: Missing close\n";
        assert_eq!(probe_status(text), FrontmatterStatus::Invalid);
    }

    #[test]
    fn bad_yaml_inside_block_is_invalid() {
        let text = "---\nname: [unclosed\n---\n# Body";
        assert_eq!(probe_status(text), FrontmatterStatus::Invalid);
    }
}
