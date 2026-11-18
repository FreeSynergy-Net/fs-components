//! The grey line under the title of a card: the subtitle, then the facts, then the due text, then the assignee.
//!
//! Pattern: pure function (a card in, the pieces of the line out); the view joins them. The facts are the compact lines of
//! the field components of the blocks the catalog chose (`FieldSlot::line`), in the order the program gave them. Facts beyond the
//! limit of the card are not cut silently: a piece "+ N more" stands right after the facts.

use fs_components_kanban::{Card, FieldSlot};

use super::labels::Labels;

/// The pieces of the line of `card`, in order; empty = no line. `labels` says "+ N more" for the facts beyond the limit.
#[must_use]
pub fn card_meta(card: &Card, labels: &Labels) -> Vec<String> {
    let mut meta: Vec<String> = Vec::new();
    if let Some(sub) = &card.subtitle {
        meta.push(sub.clone());
    }
    // the same facts the kanban card draws: no value (`Unset`) takes no room, hidden or missing shows its reason
    meta.extend(card.shown_facts().map(FieldSlot::line));
    if card.more_facts() > 0 {
        meta.push(labels.more(card.more_facts()));
    }
    if let Some(due) = &card.due {
        meta.push(due.text.clone());
    }
    if let Some(who) = &card.assignee {
        meta.push(who.clone());
    }
    meta
}
