//! The input rules: Markdown shortcuts that turn a paragraph into a typed block the moment the trigger is typed.
//!
//! Pattern: Table of rules — one function maps "what stands in front of the cursor" to a change of kind; each rule is one row,
//! tested alone. The triggers are the Space after the shortcut (`## `, `- `, `1. `, `> `, `[] `, ```` ``` ````, `---`) and, for
//! the fence and the rule, Enter.

use super::block::Kind;

/// What a rule does to the block: the new kind and the text that stays.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Turn {
    /// The kind the block gets.
    pub kind: Kind,
    /// The text of the block afterwards (what stood behind the shortcut).
    pub text: String,
}

/// The shortcut that `typed` (the text of a block from its start up to and including the Space just typed) makes, with
/// `rest` the text behind the cursor. A paragraph only; the rules never fire in code or after other text.
#[must_use]
pub fn on_space(kind: &Kind, typed: &str, rest: &str) -> Option<Turn> {
    let turn = |kind| {
        Some(Turn {
            kind,
            text: rest.to_owned(),
        })
    };
    match kind {
        Kind::Paragraph => {
            let head = typed.strip_suffix(' ')?;
            let hashes = head.len();
            if (1..=6).contains(&hashes) && head.chars().all(|c| c == '#') {
                return turn(Kind::Heading(u8::try_from(hashes).ok()?));
            }
            match head {
                "-" | "*" | "+" => return turn(Kind::Bullet),
                ">" => return turn(Kind::Quote),
                "[]" | "[ ]" => return turn(Kind::Task(false)),
                "[x]" | "[X]" => return turn(Kind::Task(true)),
                "---" => return turn(Kind::Rule),
                _ => {}
            }
            if let Some(lang) = head.strip_prefix("```") {
                return turn(Kind::Code(lang.to_owned()));
            }
            let digits = head.strip_suffix('.')?;
            if (1..10).contains(&digits.len()) && digits.chars().all(|c| c.is_ascii_digit()) {
                return turn(Kind::Numbered(digits.parse().ok()?));
            }
            None
        }
        // "- [] " typed as a bullet first: the box follows.
        Kind::Bullet if typed == "[] " || typed == "[ ] " => turn(Kind::Task(false)),
        _ => None,
    }
}

/// The rule of Enter at the end of a paragraph: a fence line (```` ```rust ````) opens a code block, `---` makes a rule.
#[must_use]
pub fn on_enter(kind: &Kind, text: &str) -> Option<Turn> {
    if *kind != Kind::Paragraph {
        return None;
    }
    if let Some(lang) = text.strip_prefix("```")
        && !lang.contains(char::is_whitespace)
        && !lang.contains('`')
    {
        return Some(Turn {
            kind: Kind::Code(lang.to_owned()),
            text: String::new(),
        });
    }
    (text == "---").then(|| Turn {
        kind: Kind::Rule,
        text: String::new(),
    })
}
