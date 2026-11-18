//! The grey line under the title of a card: the subtitle, then the facts, then the due text, then the assignee.
//!
//! Pattern: pure function (a card in, the pieces of the line out); the view joins them. The facts are the compact lines of
//! the field components of the blocks the catalog chose (`FieldSlot::line`), in the order the program gave them. Facts beyond the
//! limit of the card are not cut silently: the piece "+ N more" ([`card_more`]) is not part of the line, the view draws it as an
//! element of its own at the right end, so a long line can never push it out of the card.

use fs_components_kanban::{Card, FieldSlot};

use super::labels::Labels;

/// The pieces of the line of `card`, in order; empty = no line.
#[must_use]
pub fn card_meta(card: &Card) -> Vec<String> {
    let mut meta: Vec<String> = Vec::new();
    if let Some(sub) = &card.subtitle {
        meta.push(sub.clone());
    }
    // the same facts the kanban card draws: no value (`Unset`) takes no room, hidden or missing shows its reason.
    // "+ N more" is not part of the line: `card_more`, drawn apart at the right end.
    meta.extend(card.shown_facts().map(FieldSlot::line));
    if let Some(due) = &card.due {
        meta.push(due.text.clone());
    }
    if let Some(who) = &card.assignee {
        meta.push(who.clone());
    }
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
