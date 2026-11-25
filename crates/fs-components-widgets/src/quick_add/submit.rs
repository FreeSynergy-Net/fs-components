//! Quick add: what Enter, Shift+Enter and a pasted text come to, for anything that is created from typed lines (a card, a task, a list item).
//!
//! Pattern: none - a pure function and its result type. It knows lines of text and nothing about what they become; the one who
//! calls it maps the titles onto its own request (kanban: `NewCard`). Empty lines are nothing; more than one line asks
//! "N entries?" first (never a silent flood).

/// What a submit comes to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Quick {
    /// Nothing to create (no letter in it).
    Nothing,
    /// Create these now.
    Create(Vec<String>),
    /// More than one line: ask first, then create these.
    Confirm(Vec<String>),
}

/// The titles `text` stands for; `top` = Shift+Enter. One title per non-empty line (trimmed), **in the order to ask the program**:
/// as typed for the end of a list; reversed for the top, so that each one lands above the one before and the first line ends up first.
pub fn submit(text: &str, top: bool) -> Quick {
    let mut titles: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect();
    if top {
        titles.reverse();
    }
    match titles.len() {
        0 => Quick::Nothing,
        1 => Quick::Create(titles),
        _ => Quick::Confirm(titles),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_for_blank_text() {
        assert_eq!(submit("   ", false), Quick::Nothing);
        assert_eq!(submit("\n\n", true), Quick::Nothing);
    }

    #[test]
    fn one_line_is_created_trimmed() {
        assert_eq!(
            submit("  Write letter ", false),
            Quick::Create(vec!["Write letter".into()])
        );
    }

    #[test]
    fn several_lines_ask_first_and_skip_empty_ones() {
        assert_eq!(
            submit("x\n\ny\nz", false),
            Quick::Confirm(vec!["x".into(), "y".into(), "z".into()])
        );
    }

    #[test]
    fn at_the_top_the_order_is_reversed() {
        assert_eq!(
            submit("first\nsecond", true),
            Quick::Confirm(vec!["second".into(), "first".into()])
        );
    }
}
