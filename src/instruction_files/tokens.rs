use crate::skills::Tokenizer;

pub fn count_text(text: &str, tok: Tokenizer) -> usize {
    match tok {
        Tokenizer::Heuristic => text.chars().count().div_ceil(4),
        #[cfg(feature = "accurate-tokens")]
        Tokenizer::TiktokenCl100k => {
            use tiktoken_rs::cl100k_base;
            cl100k_base()
                .ok()
                .map(|bpe| bpe.encode_with_special_tokens(text).len())
                .unwrap_or_else(|| (text.chars().count() + 3) / 4)
        }
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
        // "hello world" = 11 chars → (11 + 3) / 4 = 3
        assert_eq!(count_text("hello world", Tokenizer::Heuristic), 3);
    }

    #[test]
    fn empty_string_returns_zero() {
        assert_eq!(count_text("", Tokenizer::Heuristic), 0);
    }
}
