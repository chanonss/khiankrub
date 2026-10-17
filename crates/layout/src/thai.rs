//! Thai line breaking. Thai is written without spaces between words, so the Unicode line-break
//! algorithm finds no place to wrap inside a run of Thai letters; a dictionary word breaker
//! (ICU4X, Unicode-3.0) finds the word boundaries where a line may wrap, as Word does.

use icu_segmenter::LineSegmenter;
use icu_segmenter::options::LineBreakOptions;
use std::sync::OnceLock;

/// A Thai letter, vowel, tone mark or digit (U+0E01–U+0E5B).
pub(crate) fn is_thai(c: char) -> bool {
    matches!(u32::from(c), 0x0E01..=0x0E5B)
}

/// Byte offsets in `text` between two Thai characters where a line may break, at the boundaries
/// between dictionary words. Empty when `text` has no Thai.
pub(crate) fn breaks(text: &str) -> Vec<usize> {
    if !text.chars().any(is_thai) {
        return Vec::new();
    }
    static SEGMENTER: OnceLock<icu_segmenter::LineSegmenterBorrowed<'static>> = OnceLock::new();
    let seg = SEGMENTER.get_or_init(|| LineSegmenter::new_dictionary(LineBreakOptions::default()));
    seg.segment_str(text)
        .filter(|&i| {
            // Only inside Thai text: other scripts keep the breaks the line-break algorithm finds.
            text.get(..i).and_then(|t| t.chars().next_back()).is_some_and(is_thai)
                && text.get(i..).and_then(|t| t.chars().next()).is_some_and(is_thai)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breaks_between_thai_words_only() {
        let text = "ภาษาไทยไม่มีการเว้นวรรค";
        let b = breaks(text);
        let at = |w: &str| text.find(w).unwrap();
        for w in ["ไทย", "ไม่", "การ"] {
            assert!(b.contains(&at(w)), "no break before {w}: {b:?}");
        }
        // Never inside a word or syllable, and never at the ends of the text.
        let ok = |i: usize| text.is_char_boundary(i) && i > 0 && i < text.len();
        assert!(b.iter().all(|&i| ok(i)), "{b:?}");
        assert!(!b.contains(&(at("ภาษา") + "ภา".len())), "broke inside ภาษา: {b:?}");
    }

    #[test]
    fn leaves_other_text_alone() {
        assert!(breaks("plain English text").is_empty());
        assert!(breaks("").is_empty());
        // The space between Latin words and Thai is not a Thai-to-Thai break.
        assert!(breaks("Word ไทย").iter().all(|&i| i > "Word ".len()));
    }
}
