//! The grey line under the title of a card: the subtitle, then the facts, then the due text, then the assignee.
//!
//! Pattern: pure function (a card in, the pieces of the line out); the view joins them. The facts are the compact lines of
//! the field components of the blocks the catalog chose (`FieldSlot::line`), in the order the program gave them.

use fs_components_kanban::{Card, FieldSlot};

/// The pieces of the line of `card`, in order; empty = no line.
#[must_use]
pub fn card_meta(card: &Card) -> Vec<String> {
    let mut meta: Vec<String> = Vec::new();
    if let Some(sub) = &card.subtitle {
        meta.push(sub.clone());
    }
    meta.extend(card.facts().iter().map(FieldSlot::line));
    if let Some(due) = &card.due {
        meta.push(due.text.clone());
    }
    if let Some(who) = &card.assignee {
        meta.push(who.clone());
    }
    meta
}
