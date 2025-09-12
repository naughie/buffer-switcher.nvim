use norm::Chars;
use norm::{NormAlloc, NormInplace};
mod norm {
    use std::str::CharIndices;

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub(super) struct NormAlloc {
        inner: String,
    }

    impl NormAlloc {
        pub(super) fn len(&self) -> usize {
            self.inner.len()
        }

        pub(super) fn with_capacity(cap: usize) -> Self {
            Self {
                inner: String::with_capacity(cap),
            }
        }

        pub(super) fn push(&mut self, c: char) {
            if let Some(c) = filter_char(c) {
                self.inner.push(c);
            }
        }

        pub(super) fn push_str(&mut self, s: &str) {
            for c in s.chars() {
                self.push(c);
            }
        }

        pub(super) fn from_str(s: &str) -> Self {
            let mut ret = Self::with_capacity(s.len());
            ret.push_str(s);
            ret
        }

        pub(super) fn as_str(&self) -> &str {
            &self.inner
        }

        pub(super) fn char_indices(&self) -> CharIndices<'_> {
            self.inner.char_indices()
        }
    }

    #[derive(Clone, Copy)]
    pub(super) struct NormInplace<'a> {
        inner: &'a str,
    }

    impl<'a> NormInplace<'a> {
        pub(super) fn from_str(s: &'a str) -> Self {
            let inner = s.trim_matches(|c: char| filter_char(c).is_none());
            Self { inner }
        }

        pub(super) fn is_empty(self) -> bool {
            self.inner.is_empty()
        }

        pub(super) fn chars(self) -> Chars<'a> {
            Chars {
                chars: self.inner.chars(),
            }
        }
    }

    pub(super) struct Chars<'a> {
        chars: std::str::Chars<'a>,
    }

    impl Iterator for Chars<'_> {
        type Item = char;

        fn next(&mut self) -> Option<Self::Item> {
            self.chars.find_map(filter_char)
        }
    }
    impl DoubleEndedIterator for Chars<'_> {
        fn next_back(&mut self) -> Option<Self::Item> {
            (&mut self.chars).filter_map(filter_char).next_back()
        }
    }

    fn filter_char(c: char) -> Option<char> {
        if c.is_whitespace() {
            Some(' ')
        } else if c.is_control() {
            None
        } else {
            Some(c)
        }
    }
}

type Range = std::ops::Range<usize>;

use std::ops::ControlFlow;
use std::str::CharIndices;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Target {
    display_name: NormAlloc,
}

impl Target {
    pub(super) fn with_capacity(cap: usize) -> Self {
        Self {
            display_name: NormAlloc::with_capacity(cap),
        }
    }

    pub(super) fn push(&mut self, c: char) {
        self.display_name.push(c);
    }

    pub(super) fn push_str(&mut self, s: &str) {
        self.display_name.push_str(s);
    }

    pub(super) fn from_str(s: &str) -> Self {
        Self {
            display_name: NormAlloc::from_str(s),
        }
    }

    pub(super) fn display_name(&self) -> &str {
        self.display_name.as_str()
    }

    pub(super) fn len(&self) -> usize {
        self.display_name.len()
    }
}

#[derive(Clone, Copy)]
pub(super) struct Pattern<'a> {
    inner: NormInplace<'a>,
}

impl<'a> Pattern<'a> {
    pub(super) fn from_str(inner: &'a str) -> Self {
        Self {
            inner: NormInplace::from_str(inner),
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub(super) fn test<'p, 't>(&'p self, target: &'t Target) -> Match<'p, 't> {
        let mut pat = self.inner.chars();
        let pat_peek = pat.next_back().unwrap_or_default();
        Match {
            pat,
            pat_peek,
            target: target.display_name.char_indices(),
            target_len: target.display_name.len(),
        }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub(super) struct MatchItem {
    pub(super) range: Range,
    pub(super) roffset: usize,
}

pub(super) struct Match<'p, 't> {
    pat: Chars<'p>,
    pat_peek: char,
    target: CharIndices<'t>,
    target_len: usize,
}

impl Iterator for Match<'_, '_> {
    type Item = ControlFlow<MatchItem, MatchItem>;

    fn next(&mut self) -> Option<Self::Item> {
        fn eq_char(target: char, pat: char) -> bool {
            if pat.is_uppercase() {
                target == pat
            } else {
                target.to_lowercase().eq(pat.to_lowercase())
            }
        }

        if self.pat_peek == '\0' {
            return None;
        }
        let (i, target) = self.target.rfind(|&(_, c)| eq_char(c, self.pat_peek))?;

        let mut item = {
            let end = i + target.len_utf8();
            MatchItem {
                range: i..end,
                roffset: self.target_len - end,
            }
        };

        for pat in (&mut self.pat).rev() {
            let (i, target) = self.target.next_back()?;
            if eq_char(target, pat) {
                item.range.start = i;
            } else {
                self.pat_peek = pat;
                return Some(ControlFlow::Continue(item));
            }
        }

        Some(ControlFlow::Break(item))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expect_matches(target: &str, pat: &str, expected: impl IntoIterator<Item = (Range, usize)>) {
        let target = Target::from_str(target);
        let pat = Pattern::from_str(pat);

        let expected = expected
            .into_iter()
            .map(|(range, roffset)| MatchItem { range, roffset })
            .collect::<Vec<_>>();

        let mut matches = Vec::new();
        let mut broken = false;
        for match_item in pat.test(&target) {
            match match_item {
                ControlFlow::Continue(item) => {
                    matches.push(item);
                }
                ControlFlow::Break(item) => {
                    matches.push(item);
                    broken = true;
                    break;
                }
            }
        }

        if broken {
            assert_eq!(expected, matches);
        } else {
            assert_eq!(expected, []);
        }
    }

    #[test]
    fn no_matches() {
        expect_matches("", "", []);
        expect_matches("foo", "", []);
        expect_matches("", "foo", []);

        expect_matches("abcd", "xyz", []);
        expect_matches("abcd", "zbcd", []);
        expect_matches("abcd", "zd", []);
        expect_matches("abcd", "zabcd", []);
        expect_matches("abcd", "zacd", []);
    }

    #[test]
    fn substr() {
        expect_matches("abcd", "d", [(3..4, 0)]);
        expect_matches("abcd", "cd", [(2..4, 0)]);
        expect_matches("abcd", "abcd", [(0..4, 0)]);

        expect_matches("abcd", "c", [(2..3, 1)]);
        expect_matches("abcd", "bc", [(1..3, 1)]);
        expect_matches("abcd", "ab", [(0..2, 2)]);
    }

    #[test]
    fn fuzzy() {
        expect_matches("abcdefgh", "ac", [(2..3, 5), (0..1, 7)]);
        expect_matches("abcdefgh", "cdfgh", [(5..8, 0), (2..4, 4)]);
        expect_matches("abcdefgh", "abch", [(7..8, 0), (0..3, 5)]);
    }

    #[test]
    fn normalization() {
        expect_matches("ABCD", "abcd", [(0..4, 0)]);
        expect_matches("A\tB\0\u{3}CD", "a bcd", [(0..5, 0)]);
        expect_matches("ΑΒΗ", "αβη", [(0..("ΑΒΗ".len()), 0)]);

        expect_matches("aBH", "Abh", []);
        expect_matches("ABH", "Abh", [(0..3, 0)]);
        expect_matches("αΒΗ", "Αβη", []);
        expect_matches("ΑΒΗ", "Αβη", [(0..("ΑΒΗ".len()), 0)]);
    }

    #[test]
    fn pattern_is_empty() {
        assert!(Pattern::from_str("").is_empty());
        assert!(Pattern::from_str("\0\0\0").is_empty());
        assert!(!Pattern::from_str("abc").is_empty());
    }
}
