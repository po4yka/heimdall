use serde::{Deserialize, Serialize};

/// Method used to estimate listing-token cost for a skill.
///
/// The listing is the XML-wrapped metadata (`name` + truncated `description`)
/// that Claude Code injects into every request.  Skill bodies are NOT counted
/// — they load on demand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tokenizer {
    /// `(chars + 3) / 4` — zero new dependencies, ±20 % accuracy for English.
    #[default]
    Heuristic,
    /// `cl100k_base` BPE — requires the `accurate-tokens` Cargo feature.
    #[cfg(feature = "accurate-tokens")]
    TiktokenCl100k,
}

impl Tokenizer {
    pub fn as_str(self) -> &'static str {
        match self {
            Tokenizer::Heuristic => "heuristic",
            #[cfg(feature = "accurate-tokens")]
            Tokenizer::TiktokenCl100k => "tiktoken-cl100k",
        }
    }
}

/// Fixed overhead for the XML wrapper Claude Code adds around each skill
/// listing entry: `<skill name="..."><description>...</description></skill>`
/// plus surrounding whitespace.
const WRAPPER_TOKENS: usize = 30;

/// Compute the listing-token cost for one skill.
///
/// Applies `max_desc_chars` truncation *before* counting to match
/// the agent's `skillListingMaxDescChars` behaviour (default: 1 536).
pub fn count_listing_tokens(
    name: &str,
    description: Option<&str>,
    max_desc_chars: usize,
    tok: Tokenizer,
) -> usize {
    let desc = description.unwrap_or("");
    let truncated: std::borrow::Cow<str> = if desc.chars().count() > max_desc_chars {
        std::borrow::Cow::Owned(desc.chars().take(max_desc_chars).collect())
    } else {
        std::borrow::Cow::Borrowed(desc)
    };

    WRAPPER_TOKENS + count_text(name, tok) + count_text(truncated.as_ref(), tok)
}

fn count_text(text: &str, tok: Tokenizer) -> usize {
    match tok {
        Tokenizer::Heuristic => (text.chars().count() + 3) / 4,
        #[cfg(feature = "accurate-tokens")]
        Tokenizer::TiktokenCl100k => tiktoken_rs::cl100k_base()
            .ok()
            .map(|bpe| bpe.encode_with_special_tokens(text).len())
            .unwrap_or_else(|| (text.chars().count() + 3) / 4),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heuristic_counts_chars_div_4() {
        // "hello world" = 11 chars → (11 + 3) / 4 = 3 tokens
        let tokens = count_text("hello world", Tokenizer::Heuristic);
        assert_eq!(tokens, 3);
    }

    #[test]
    fn empty_string_returns_zero() {
        assert_eq!(count_text("", Tokenizer::Heuristic), 0);
    }

    #[test]
    fn listing_tokens_includes_wrapper() {
        let tokens =
            count_listing_tokens("my-skill", Some("short desc"), 1536, Tokenizer::Heuristic);
        // wrapper (30) + name tokens + desc tokens
        assert!(tokens >= WRAPPER_TOKENS);
    }

    #[test]
    fn truncation_applied_before_counting() {
        let long_desc: String = "x".repeat(2000);
        let truncated = count_listing_tokens("s", Some(&long_desc), 1536, Tokenizer::Heuristic);
        let full = count_listing_tokens("s", Some(&long_desc), 10_000, Tokenizer::Heuristic);
        assert!(truncated < full);
    }

    #[test]
    fn none_description_treated_as_empty() {
        let none = count_listing_tokens("skill", None, 1536, Tokenizer::Heuristic);
        let empty = count_listing_tokens("skill", Some(""), 1536, Tokenizer::Heuristic);
        assert_eq!(none, empty);
    }
}
