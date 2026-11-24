//! The conformance tests every [`Language`] has to pass: one collection, run by each implementation.
//!
//! Pattern: Template method – [`check`] runs the same cases against any language, which only names a fence line and a
//! line whose colours depend on the state.

use crate::language::{Language, LineState, Token, TokenKind};

/// Colour a line with the language that has `media_type`; no such language: the whole line is [`TokenKind::Plain`] and
/// the state is passed on unchanged.
#[must_use]
pub fn colour(
    languages: &[&dyn Language],
    media_type: &str,
    state: Option<LineState>,
    line: &str,
) -> (Vec<Token>, Option<LineState>) {
    if let Some(language) = languages
        .iter()
        .find(|l| l.media_types().contains(&media_type))
    {
        return language.line(state, line);
    }
    let tokens = if line.is_empty() {
        Vec::new()
    } else {
        vec![Token {
            start: 0,
            end: line.len(),
            kind: TokenKind::Plain,
        }]
    };
    (tokens, state)
}

/// Run all cases against `language`. `fence` is a line that opens (and closes) a state that lasts over lines;
/// `stateful` is a line whose tokens differ inside and outside that state. Panics on the first failure.
///
/// # Panics
/// When the language breaks a rule of the contract.
pub fn check(language: &dyn Language, fence: &str, stateful: &str) {
    case_media_types(language);
    case_empty_line(language);
    case_ranges(language, &["fn main() {", "  a  b ", "x", fence, stateful]);
    case_ranges(language, &["fn é€😀 fn", "日本語 fn 😀"]);
    case_state_over_fence(language, fence, stateful);
    case_same_input_same_output(language, stateful);
    case_unknown_media_type(language, stateful);
}

fn case_media_types(l: &dyn Language) {
    assert!(!l.media_types().is_empty(), "a language names a media type");
    assert!(l.media_types().iter().all(|m| m.contains('/')));
}

fn case_empty_line(l: &dyn Language) {
    let (tokens, _) = l.line(None, "");
    assert!(tokens.is_empty(), "an empty line has no tokens");
}

/// Sorted, without gaps, from 0 to the end, on char boundaries; also with the state of every line before.
fn case_ranges(l: &dyn Language, lines: &[&str]) {
    let mut state = None;
    for line in lines {
        let (tokens, after) = l.line(state, line);
        let mut at = 0;
        for t in &tokens {
            assert_eq!(t.start, at, "{line:?}: gap or overlap at {at}");
            assert!(t.end > t.start, "{line:?}: empty token");
            assert!(
                line.is_char_boundary(t.start) && line.is_char_boundary(t.end),
                "{line:?}: token {t:?} cuts a character"
            );
            at = t.end;
        }
        assert_eq!(at, line.len(), "{line:?}: tokens do not reach the end");
        state = after;
    }
}

fn case_state_over_fence(l: &dyn Language, fence: &str, stateful: &str) {
    let (outside, none) = l.line(None, stateful);
    let (_, open) = l.line(None, fence);
    assert!(open.is_some(), "the fence line opens a state");
    let (inside, still) = l.line(open.clone(), stateful);
    assert_ne!(outside, inside, "the state changes the colours");
    assert_eq!(still, open, "the state lasts until the fence closes");
    let (_, closed) = l.line(open, fence);
    assert_eq!(closed, none, "the second fence closes the state");
    let (again, _) = l.line(closed, stateful);
    assert_eq!(again, outside, "after the fence the colours are as before");
}

fn case_same_input_same_output(l: &dyn Language, line: &str) {
    assert_eq!(l.line(None, line), l.line(None, line));
}

fn case_unknown_media_type(l: &dyn Language, line: &str) {
    let (tokens, state) = colour(&[l], "application/x-nothing", None, line);
    assert_eq!(
        tokens,
        [Token {
            start: 0,
            end: line.len(),
            kind: TokenKind::Plain
        }]
    );
    assert_eq!(state, None, "no language, no state");
    let known = l.media_types()[0];
    assert_eq!(colour(&[l], known, None, line), l.line(None, line));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::FakeLanguage;

    #[test]
    fn the_fake_language_passes_the_collection() {
        check(&FakeLanguage, "```", "fn a");
    }

    #[test]
    fn the_fake_marks_the_word_fn_and_the_fence() {
        let (t, _) = FakeLanguage.line(None, "fn a");
        assert_eq!(t[0].kind, TokenKind::Keyword);
        let (t, s) = FakeLanguage.line(None, "```rust");
        assert_eq!(t[0].kind, TokenKind::Fence);
        assert!(s.is_some());
    }

    /// Counter-case: a language that ignores the state must fail the collection.
    #[test]
    #[should_panic(expected = "the fence line opens a state")]
    fn a_language_without_state_fails() {
        struct Flat;
        impl crate::modules::SourceModule for Flat {}
        impl Language for Flat {
            fn media_types(&self) -> &[&'static str] {
                &["text/x-flat"]
            }
            fn line(&self, _: Option<LineState>, line: &str) -> (Vec<Token>, Option<LineState>) {
                (FakeLanguage.line(None, line).0, None)
            }
        }
        check(&Flat, "```", "fn a");
    }

    /// Counter-case: tokens with a gap fail.
    #[test]
    #[should_panic(expected = "gap or overlap")]
    fn tokens_with_a_gap_fail() {
        struct Gap;
        impl crate::modules::SourceModule for Gap {}
        impl Language for Gap {
            fn media_types(&self) -> &[&'static str] {
                &["text/x-gap"]
            }
            fn line(&self, _: Option<LineState>, line: &str) -> (Vec<Token>, Option<LineState>) {
                let tokens = if line.len() > 1 {
                    vec![Token {
                        start: 1,
                        end: line.len(),
                        kind: TokenKind::Plain,
                    }]
                } else {
                    Vec::new()
                };
                (tokens, None)
            }
        }
        check(&Gap, "```", "fn a");
    }
}
