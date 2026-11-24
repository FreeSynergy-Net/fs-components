//! Tests of `SyntectLanguage`: the collection `language::check`, the kinds of the four languages and the state over lines.
//!
//! Pattern: Template method – the shared collection runs against the adapter, the rest checks what only syntect knows.

use super::SyntectLanguage;
use crate::language::{Language, LineState, Token, TokenKind, check, colour};

fn kinds(
    language: &dyn Language,
    state: Option<LineState>,
    line: &str,
) -> Vec<(String, TokenKind)> {
    let (tokens, _) = language.line(state, line);
    tokens
        .iter()
        .map(|t| (line[t.start..t.end].to_string(), t.kind))
        .collect()
}

fn of(media_type: &str) -> SyntectLanguage {
    SyntectLanguage::for_media_type(media_type).expect("a language for the media type")
}

fn has(found: &[(String, TokenKind)], text: &str, kind: TokenKind) -> bool {
    found.iter().any(|(t, k)| t.contains(text) && *k == kind)
}

#[test]
fn markdown_passes_the_collection() {
    check(&of("text/markdown"), "```", "fn main() {");
}

#[test]
fn all_four_languages_exist_and_keep_their_media_types() {
    let all = SyntectLanguage::all();
    assert_eq!(all.len(), 4);
    for media in [
        "text/markdown",
        "application/toml",
        "text/x-toml",
        "text/x-rust",
        "text/rust",
        "application/json",
    ] {
        assert!(of(media).media_types().contains(&media));
    }
    assert!(SyntectLanguage::for_media_type("application/x-nothing").is_none());
}

#[test]
fn markdown_heading_and_fence_have_their_kinds() {
    let md = of("text/markdown");
    assert!(has(
        &kinds(&md, None, "# Title"),
        "Title",
        TokenKind::Heading
    ));
    let (fence, open) = md.line(None, "```");
    assert_eq!(fence[0].kind, TokenKind::Fence);
    assert!(has(
        &kinds(&md, open, "plain words"),
        "plain",
        TokenKind::Code
    ));
}

#[test]
fn toml_has_keys_strings_numbers_and_a_multiline_string_over_lines() {
    let toml = of("application/toml");
    let line = "name = \"x\" # c";
    let found = kinds(&toml, None, line);
    assert!(has(&found, "name", TokenKind::Key), "{found:?}");
    assert!(has(&found, "\"x\"", TokenKind::String), "{found:?}");
    assert!(has(&found, "# c", TokenKind::Comment), "{found:?}");
    assert!(has(&kinds(&toml, None, "n = 42"), "42", TokenKind::Number));
    let (_, open) = toml.line(None, "text = \"\"\"first");
    assert!(has(
        &kinds(&toml, open, "second # not a comment"),
        "second",
        TokenKind::String
    ));
}

#[test]
fn rust_has_keywords_types_and_a_block_comment_over_lines() {
    let rust = of("text/x-rust");
    let found = kinds(&rust, None, "pub fn main() -> u32 { 1 }");
    assert!(has(&found, "fn", TokenKind::Keyword), "{found:?}");
    assert!(has(&found, "main", TokenKind::Function), "{found:?}");
    let (_, open) = rust.line(None, "/* a block");
    assert!(has(
        &kinds(&rust, open, "fn still comment"),
        "fn",
        TokenKind::Comment
    ));
}

#[test]
fn json_has_keys_strings_and_numbers() {
    let json = of("application/json");
    let found = kinds(&json, None, "{\"a\": \"b\", \"c\": 1.5, \"d\": true}");
    assert!(has(&found, "\"a\"", TokenKind::Key), "{found:?}");
    assert!(has(&found, "\"b\"", TokenKind::String), "{found:?}");
    assert!(has(&found, "1.5", TokenKind::Number), "{found:?}");
}

#[test]
fn an_unknown_media_type_is_plain_and_a_known_one_is_coloured() {
    let md = of("text/markdown");
    let all: Vec<&dyn Language> = vec![&md];
    let (plain, state) = colour(&all, "application/x-nothing", None, "# Title");
    assert_eq!(
        plain,
        [Token {
            start: 0,
            end: 7,
            kind: TokenKind::Plain
        }]
    );
    assert_eq!(state, None);
    assert_eq!(
        colour(&all, "text/markdown", None, "# Title"),
        md.line(None, "# Title")
    );
}

#[test]
fn a_foreign_state_starts_afresh_and_equal_states_are_equal() {
    let rust = of("text/x-rust");
    let garbage = Some(LineState(vec![9, 9, 9]));
    assert_eq!(rust.line(garbage, "fn a"), rust.line(None, "fn a"));
    let (_, a) = rust.line(None, "let x = 1;");
    let (_, b) = rust.line(None, "let x = 1;");
    assert_eq!(a, b);
}
