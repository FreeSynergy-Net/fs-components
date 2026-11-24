//! A language for tests: the word `fn` is a keyword, a line that starts with three backticks opens or closes a fence.
//!
//! Pattern: Test double (fake) – the smallest language that has a word kind and a state over lines.

use crate::language::{Language, LineState, Token, TokenKind};
use crate::modules::SourceModule;

/// The state byte of "inside a fence".
const IN_FENCE: u8 = 1;

/// The fake language, media type `text/x-fake`.
#[derive(Clone, Copy, Debug, Default)]
pub struct FakeLanguage;

impl SourceModule for FakeLanguage {}

impl Language for FakeLanguage {
    fn media_types(&self) -> &[&'static str] {
        &["text/x-fake"]
    }

    fn line(&self, state: Option<LineState>, line: &str) -> (Vec<Token>, Option<LineState>) {
        let inside = state.is_some_and(|s| s.0 == [IN_FENCE]);
        let whole = |kind| {
            if line.is_empty() {
                Vec::new()
            } else {
                vec![Token {
                    start: 0,
                    end: line.len(),
                    kind,
                }]
            }
        };
        if line.starts_with("```") {
            let after = (!inside).then(|| LineState(vec![IN_FENCE]));
            return (whole(TokenKind::Fence), after);
        }
        if inside {
            return (whole(TokenKind::Code), Some(LineState(vec![IN_FENCE])));
        }
        (words(line), None)
    }
}

/// Split into runs of word characters and runs of the rest; the word `fn` is a keyword.
fn words(line: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut prev: Option<bool> = None;
    for (i, c) in line.char_indices() {
        let word = c.is_alphanumeric() || c == '_';
        if prev.is_some_and(|p| p != word) {
            out.push(token(line, start, i));
            start = i;
        }
        prev = Some(word);
    }
    if prev.is_some() {
        out.push(token(line, start, line.len()));
    }
    out
}

fn token(line: &str, start: usize, end: usize) -> Token {
    let kind = if &line[start..end] == "fn" {
        TokenKind::Keyword
    } else {
        TokenKind::Plain
    };
    Token { start, end, kind }
}
