//! The grey line under the title of a card, as the pieces it is made of: the subtitle, the facts, the due text, the assignee.
//!
//! Pattern: pure function (a card in, the pieces out). The view draws the facts by their field components
//! (`FieldSlot::compact_view`, see `card_line.rs`); [`card_meta`] is the same line as plain text (the label for assistive technology,
//! logs, tests). Both come from the same pieces, so the text and the picture cannot disagree. Facts beyond the limit of the card are
//! not cut silently: the piece "+ N more" ([`card_more`]) is not part of the line, the view draws it as an element of its own at the
//! right end, so a long line can never push it out of the card.

use crate::model::MAX_ASSIGNEES;
use crate::{Card, FieldSlot};

use crate::labels::Labels;

/// The pieces before the facts: the subtitle.
#[must_use]
pub fn card_lead(card: &Card) -> Vec<String> {
    card.subtitle.iter().cloned().collect()
}

/// The initials of a name: the first letter of its first and last word, upper case ("Ada Lovelace" is "AL", "Ada" is "A").
fn initials(name: &str) -> String {
    let mut words = name.split_whitespace();
    let first = words.next().and_then(|w| w.chars().next());
    let last = words.next_back().and_then(|w| w.chars().next());
    first
        .into_iter()
        .chain(last)
        .flat_map(char::to_uppercase)
        .collect()
}

/// The assignees as pieces: the initials of the first [`MAX_ASSIGNEES`], then "+N" for the rest (the number is never dropped).
fn assignee_pieces(card: &Card) -> Vec<String> {
    let mut pieces: Vec<String> = card
        .assignees
        .iter()
        .take(MAX_ASSIGNEES)
        .map(|n| initials(n))
        .filter(|i| !i.is_empty())
        .collect();
    let over = card.assignees.len().saturating_sub(MAX_ASSIGNEES);
    if over > 0 {
        pieces.push(format!("+{over}"));
    }
    pieces
}

/// The pieces after the facts: the due text, the assignee.
#[must_use]
pub fn card_tail(card: &Card) -> Vec<String> {
    let mut tail = Vec::new();
    if let Some(due) = &card.due {
        tail.push(due.text.clone());
    }
    tail.extend(assignee_pieces(card));
    tail
}

/// The pieces of the line of `card` as text, in order; empty = no line. A fact without a value (`Unset`) takes no room, a hidden
/// or missing one shows its reason.
#[must_use]
pub fn card_meta(card: &Card) -> Vec<String> {
    let mut meta = card_lead(card);
    meta.extend(card.shown_facts().map(FieldSlot::line));
    meta.extend(card_tail(card));
    meta
}

/// "+ N more": how many facts the limit of `card` left out (none = no piece). Drawn apart from the line, at its right end.
#[must_use]
pub fn card_more(card: &Card, labels: &Labels) -> Option<String> {
    if card.more_facts() > 0 {
        return Some(labels.more(card.more_facts()));
    }
    None
}
