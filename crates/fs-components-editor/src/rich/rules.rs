//! The input rules: Markdown shortcuts that turn a paragraph into a typed block the moment the trigger is typed.
//!
//! Pattern: Specification — each rule is a predicate over what stands in front of the cursor and where the paragraph is, and
//! says which block it becomes; one table, each row tested alone. The triggers are the Space after the shortcut (`## `, `- `,
//! `1. `, `> `, `[] `, `[x] `, ```` ``` ````, `---`) and, for the fence and the rule, Enter.

/// What a rule makes of the paragraph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Turn {
    /// A heading of this level.
    Heading(u8),
    /// A bullet list with this paragraph as its first item.
    Bullet,
    /// A numbered list that starts at this number.
    Numbered(u32),
    /// A quotation.
    Quote,
    /// A task item (a list of one, or the box at the start of an item that is none yet); `true` when done.
    Task(bool),
    /// A horizontal rule.
    Rule,
    /// A code block with this language hint.
    Code(String),
}

/// Where the paragraph stands: a rule that makes a block of its own only fires in a paragraph of the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// A block of the page itself.
    Page,
    /// The text of a list item that has no task box.
    Item,
    /// Anywhere else (in a quote, in an item that is a task already): no rule fires.
    Inside,
}

/// The shortcut that `typed` (the text of the paragraph from its start up to and including the Space just typed) makes.
#[must_use]
pub fn on_space(typed: &str, place: Place) -> Option<Turn> {
    let head = typed.strip_suffix(' ')?;
    let task = match head {
        "[]" | "[ ]" => Some(Turn::Task(false)),
        "[x]" | "[X]" => Some(Turn::Task(true)),
        _ => None,
    };
    match place {
        Place::Inside => None,
        Place::Item => task,
        Place::Page => {
            if task.is_some() {
                return task;
            }
            let hashes = head.len();
            if (1..=6).contains(&hashes) && head.chars().all(|c| c == '#') {
                return Some(Turn::Heading(u8::try_from(hashes).ok()?));
            }
            match head {
                "-" | "*" | "+" => return Some(Turn::Bullet),
                ">" => return Some(Turn::Quote),
                "---" => return Some(Turn::Rule),
                _ => {}
            }
            if let Some(lang) = head.strip_prefix("```") {
                return Some(Turn::Code(lang.to_owned()));
            }
            let digits = head.strip_suffix('.')?;
            ((1..10).contains(&digits.len()) && digits.chars().all(|c| c.is_ascii_digit()))
                .then(|| Turn::Numbered(digits.parse().unwrap_or(1)))
        }
    }
}

/// The rule of Enter at the end of a paragraph of the page: a fence line (```` ```rust ````) opens a code block, `---` makes a rule.
#[must_use]
pub fn on_enter(text: &str) -> Option<Turn> {
    if let Some(lang) = text.strip_prefix("```")
        && !lang.contains(char::is_whitespace)
        && !lang.contains('`')
    {
        return Some(Turn::Code(lang.to_owned()));
    }
    (text == "---").then_some(Turn::Rule)
}
