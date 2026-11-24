//! From the scopes of syntect to the kinds of ours: the one table that decides what a range is called.
//!
//! Pattern: Adapter (mapping table) – the names of the scopes are the `TextMate` ones; the first rule that fits the innermost
//! scope wins, punctuation only counts when no scope around it has a kind of its own.

use syntect::parsing::ScopeStack;

use crate::language::TokenKind;

/// The rules, most specific first: scope prefix to kind.
const RULES: &[(&str, TokenKind)] = &[
    ("invalid", TokenKind::Invalid),
    ("comment", TokenKind::Comment),
    ("punctuation.definition.raw.code-fence", TokenKind::Fence),
    ("markup.raw", TokenKind::Code),
    ("markup.heading", TokenKind::Heading),
    ("markup.italic", TokenKind::Emphasis),
    ("markup.bold", TokenKind::Strong),
    ("markup.underline.link", TokenKind::Link),
    ("meta.link", TokenKind::Link),
    ("string.other.link", TokenKind::Link),
    ("string", TokenKind::String),
    ("constant.numeric", TokenKind::Number),
    ("entity.name.function", TokenKind::Function),
    ("support.function", TokenKind::Function),
    ("meta.function-call", TokenKind::Function),
    ("entity.name.type", TokenKind::Type),
    ("entity.name.struct", TokenKind::Type),
    ("entity.name.enum", TokenKind::Type),
    ("entity.name.trait", TokenKind::Type),
    ("support.type", TokenKind::Type),
    ("storage.type.numeric", TokenKind::Type),
    ("storage.type.primitive", TokenKind::Type),
    ("keyword", TokenKind::Keyword),
    ("storage", TokenKind::Keyword),
    ("constant.language", TokenKind::Keyword),
];

/// The scopes of a key: they win over what lies inside them (the string of a JSON key is a Key).
const KEYS: &[&str] = &[
    "support.type.property-name",
    "meta.mapping.key",
    "entity.name.tag",
];

/// The kind of the text under `stack`: the innermost scope with a rule decides; punctuation stands for itself only when
/// nothing around it has a kind (so the quote of a string is a String, a `#` of a heading is a Heading).
pub fn kind_of(stack: &ScopeStack) -> TokenKind {
    let mut punct = false;
    let names: Vec<String> = stack.as_slice().iter().map(|s| s.build_string()).collect();
    if names.iter().any(|n| KEYS.iter().any(|k| has_prefix(n, k))) {
        return TokenKind::Key;
    }
    for name in names.iter().rev() {
        if let Some((_, kind)) = RULES.iter().find(|(prefix, _)| has_prefix(name, prefix)) {
            return *kind;
        }
        punct |= name.starts_with("punctuation");
    }
    if punct {
        TokenKind::Punct
    } else {
        TokenKind::Plain
    }
}

/// `prefix` is the name or its front, cut at a dot (`string` fits `string.quoted`, not `stringy`).
fn has_prefix(name: &str, prefix: &str) -> bool {
    name.strip_prefix(prefix)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
}
