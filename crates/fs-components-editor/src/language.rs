//! The language of a source text: colours for the lines of one text, chosen by media type.
//!
//! Pattern: Strategy – a language colours the lines of a source text; the editor picks one by media type. No Blinc type
//! and no colour is named here: a [`TokenKind`] is a kind, the theme gives the colour (ADR 0074).

use crate::modules::SourceModule;

/// A language of the `SourceEditor`: colours one line at a time and carries a state from line to line.
///
/// Pattern: Strategy – one trait, one implementation per language (the adapter in the workbenches, the fake in `check`).
pub trait Language: SourceModule {
    /// The media types this language colours (`text/markdown`, `application/toml`, ...).
    fn media_types(&self) -> &[&'static str];

    /// Colour one line. `state` comes from the line before (`None` = start of the text); the state after the line is
    /// returned with the tokens. The tokens are sorted, do not overlap, cover the whole line and lie on char boundaries.
    fn line(&self, state: Option<LineState>, line: &str) -> (Vec<Token>, Option<LineState>);
}

/// A byte range of a line with its kind.
///
/// Pattern: Value object – a range and a kind; equal values are equal tokens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    /// First byte (inclusive).
    pub start: usize,
    /// End byte (exclusive).
    pub end: usize,
    /// What the range is.
    pub kind: TokenKind,
}

/// What a range of source text is; the theme maps a kind to a colour.
///
/// Pattern: Value object – a closed set of kinds shared by all languages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// Nothing special.
    Plain,
    /// A keyword.
    Keyword,
    /// A type name.
    Type,
    /// A function name.
    Function,
    /// A string.
    String,
    /// A number.
    Number,
    /// A comment.
    Comment,
    /// Punctuation.
    Punct,
    /// A key (TOML, JSON).
    Key,
    /// A heading (Markdown).
    Heading,
    /// Emphasis (Markdown).
    Emphasis,
    /// Strong emphasis (Markdown).
    Strong,
    /// A link.
    Link,
    /// Code inside a text (a code span or the inside of a fence).
    Code,
    /// A code fence line.
    Fence,
    /// Something that is wrong.
    Invalid,
}

/// The state of a language after a line, opaque for everybody but the language that made it.
///
/// Pattern: Memento – the language hands out its state and takes it back for the next line; the editor never looks inside.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineState(pub Vec<u8>);

mod conformance;
mod fake;

pub use conformance::{check, colour};
pub use fake::FakeLanguage;

#[cfg(feature = "syntect")]
mod syntect_language;
#[cfg(feature = "syntect")]
pub use syntect_language::SyntectLanguage;
