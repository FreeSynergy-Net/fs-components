//! `SyntectLanguage`: the [`Language`] of the four media types of the editor (Markdown, TOML, Rust, JSON), made of the
//! syntax definitions of `two-face` and the parser of `syntect` (pure Rust regex, no onig, no C).
//!
//! Pattern: Adapter – a foreign parser (syntect) behind our trait [`Language`]; its scopes become our [`TokenKind`],
//! no colour and no theme of syntect is used. Deviation: the state is a [`Memento`](LineState) that is a handle, not a
//! serialisation, because the parser state of syntect cannot be serialised (private fields, no serde); see [`States`].

mod kinds;
mod states;

use std::sync::OnceLock;

use syntect::parsing::{ParseState, ScopeStack, ScopeStackOp, SyntaxReference, SyntaxSet};

use crate::language::{Language, LineState, Token, TokenKind};
use crate::modules::SourceModule;

use self::kinds::kind_of;
use self::states::States;

/// The syntaxes of two-face (with newlines, as the parser wants them), loaded once.
fn syntaxes() -> &'static SyntaxSet {
    static SET: OnceLock<SyntaxSet> = OnceLock::new();
    SET.get_or_init(two_face::syntax::extra_newlines)
}

/// What one language is: the name of its syntax and the media types it colours.
const LANGUAGES: [(&str, &[&str]); 4] = [
    ("Markdown", &["text/markdown"]),
    ("TOML", &["application/toml", "text/x-toml"]),
    ("Rust", &["text/x-rust", "text/rust"]),
    ("JSON", &["application/json"]),
];

/// One language of syntect: Markdown, TOML, Rust or JSON. A media type that no language has is not coloured here
/// ([`language::colour`](crate::language::colour) makes it Plain).
pub struct SyntectLanguage {
    syntax: &'static SyntaxReference,
    media_types: &'static [&'static str],
    states: States,
}

impl SourceModule for SyntectLanguage {}

impl SyntectLanguage {
    /// All four languages, to put into a [`Registry`](crate::modules::Registry) or to hand to `language::colour`.
    #[must_use]
    pub fn all() -> Vec<Self> {
        LANGUAGES
            .iter()
            .filter_map(|(name, types)| Self::named(name, types))
            .collect()
    }

    /// The language that colours `media_type`, or `None` for an unknown one.
    #[must_use]
    pub fn for_media_type(media_type: &str) -> Option<Self> {
        let (name, types) = LANGUAGES.iter().find(|(_, t)| t.contains(&media_type))?;
        Self::named(name, types)
    }

    fn named(name: &str, media_types: &'static [&'static str]) -> Option<Self> {
        let syntax = syntaxes().find_syntax_by_name(name)?;
        Some(Self {
            syntax,
            media_types,
            states: States::default(),
        })
    }

    /// The whole line as one Plain token (an empty line has none); used when the parser fails.
    fn plain(line: &str) -> Vec<Token> {
        if line.is_empty() {
            return Vec::new();
        }
        vec![Token {
            start: 0,
            end: line.len(),
            kind: TokenKind::Plain,
        }]
    }
}

impl Language for SyntectLanguage {
    fn media_types(&self) -> &[&'static str] {
        self.media_types
    }

    fn line(&self, state: Option<LineState>, line: &str) -> (Vec<Token>, Option<LineState>) {
        let (mut parse, mut stack) = state
            .and_then(|s| self.states.get(&s))
            .unwrap_or_else(|| (ParseState::new(self.syntax), ScopeStack::new()));
        let text = format!("{line}\n");
        let Ok(ops) = parse.parse_line(&text, syntaxes()) else {
            return (Self::plain(line), None);
        };
        let tokens = tokens(line, &ops, &mut stack);
        (tokens, Some(self.states.put(parse, stack)))
    }
}

/// Cut the line at the operations of the parser; the kind of a piece is the one of the scope stack in force there.
/// Equal neighbours are one token. Offsets behind the line (the added newline) are cut off.
fn tokens(line: &str, ops: &[(usize, ScopeStackOp)], stack: &mut ScopeStack) -> Vec<Token> {
    let mut out: Vec<Token> = Vec::new();
    let mut at = 0;
    let mut put = |start: usize, end: usize, kind: TokenKind| {
        if end <= start {
            return;
        }
        match out.last_mut() {
            Some(last) if last.kind == kind && last.end == start => last.end = end,
            _ => out.push(Token { start, end, kind }),
        }
    };
    for (index, op) in ops {
        let index = (*index).min(line.len());
        put(at, index, kind_of(stack));
        at = at.max(index);
        if stack.apply(op).is_err() {
            break;
        }
    }
    put(at, line.len(), kind_of(stack));
    out
}

#[cfg(test)]
mod tests;
