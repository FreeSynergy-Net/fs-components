//! Records to what the workbenches draw: cards with facts (K5), the frame at the side with element slots (K6), the cells of a row
//! (K8), and the notes that say what is wrong with the data.
//!
//! Pattern: Mapper (pure functions over one row, one binding and the sentences). A hidden field shows its reason and never its
//! value; an empty field is "unset", never a blank that looks like data.

use super::mirror::{HiddenReason, LockKind, RecordRow};
use super::present::Present;
use super::shape::{MAX_FACTS, RecordsBinding, Slot};
use super::snapshot::{Snapshot, Status};
use super::texts::RecordsTexts;
use crate::board::{Card, DetailFrame, DetailSlot, Fact, Reason, SlotValue};

/// One thing the host says next to the data: a code (for tests and logs) and the sentence for the reader.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    /// The code (`page-refused`, `group-unknown`, `truncated`, `stale`).
    pub code: String,
    /// The sentence.
    pub text: String,
    /// A field, a count or the code of the provider's fault.
    pub detail: Option<String>,
}

fn hidden(reason: HiddenReason, t: &RecordsTexts) -> Reason {
    match reason {
        HiddenReason::NoRight => Reason::new("hidden-no-right", t.hidden_no_right.clone()),
        HiddenReason::Sensitive => Reason::new("hidden-sensitive", t.hidden_sensitive.clone()),
    }
}

pub(super) fn value(
    row: &RecordRow,
    slot: &Slot,
    present: &Present,
    t: &RecordsTexts,
) -> SlotValue {
    if let Some(why) = row.hidden_reason(&slot.field) {
        return SlotValue::Hidden(hidden(why, t));
    }
    match row.values_of(&slot.field) {
        Some(v) => SlotValue::Text(present(&slot.field, v)),
        None => SlotValue::Unset,
    }
}

fn plain(row: &RecordRow, field: &str, present: &Present, t: &RecordsTexts) -> Option<String> {
    if let Some(why) = row.hidden_reason(field) {
        return Some(hidden(why, t).text);
    }
    row.values_of(field).map(|v| present(field, v))
}

pub(super) fn card(row: &RecordRow, b: &RecordsBinding, p: &Present, t: &RecordsTexts) -> Card {
    let title = plain(row, &b.title, p, t).unwrap_or_else(|| row.id.clone());
    let mut card = Card::new(&row.id, &title);
    card.subtitle = b.subtitle.as_deref().and_then(|f| plain(row, f, p, t));
    for slot in b.facts.iter().take(MAX_FACTS) {
        match value(row, slot, p, t) {
            SlotValue::Text(text) => card = card.fact(Fact::new(&slot.field, text)),
            SlotValue::Hidden(r) | SlotValue::Missing(r) => {
                card = card.fact(Fact::new(&slot.field, r.text));
            }
            _ => {} // an empty fact takes no room on the card
        }
    }
    card
}

pub(super) fn lock_reason(row: &RecordRow, t: &RecordsTexts) -> Option<Reason> {
    let lock = row.lock.as_ref()?;
    let (code, sentence) = match lock.kind {
        LockKind::Claimed => ("locked-claimed", &t.lock_claimed),
        LockKind::ReadOnly => ("locked-read-only", &t.lock_read_only),
        LockKind::Admin => ("locked-admin", &t.lock_admin),
    };
    // the reason as the locker wrote it is user text and wins over the stock sentence
    Some(Reason::new(
        code,
        lock.reason.clone().unwrap_or_else(|| sentence.clone()),
    ))
}

pub(super) fn frame(
    row: &RecordRow,
    path: &str,
    b: &RecordsBinding,
    p: &Present,
    t: &RecordsTexts,
) -> DetailFrame {
    let title = plain(row, &b.title, p, t).unwrap_or_else(|| row.id.clone());
    let mut frame = DetailFrame::new(title).path(path);
    for slot in &b.detail {
        frame = frame.slot(DetailSlot {
            field: slot.field.clone(),
            name: slot.label.clone(),
            value: value(row, slot, p, t),
        });
    }
    frame
}

pub(super) fn cells(
    row: &RecordRow,
    b: &RecordsBinding,
    p: &Present,
    t: &RecordsTexts,
) -> Vec<String> {
    b.cells
        .iter()
        .map(|slot| match value(row, slot, p, t) {
            SlotValue::Text(s) => s,
            SlotValue::Unset => String::new(),
            SlotValue::Hidden(r) | SlotValue::Missing(r) => r.text,
        })
        .collect()
}

/// The notes for a snapshot and the state of the worker, in a fixed order: the state first, then the problems.
pub(super) fn notes(snap: &Snapshot, status: &Status, t: &RecordsTexts) -> Vec<Note> {
    let mut out = Vec::new();
    if let Status::Stale(fault) = status {
        out.push(Note {
            code: "stale".into(),
            text: t.stale.clone(),
            detail: Some(fault.code.clone()),
        });
    }
    for p in &snap.problems {
        let text = match p.code.as_str() {
            "page-refused" => &t.page_refused,
            "group-unknown" => &t.group_unknown,
            _ => &t.truncated,
        };
        out.push(Note {
            code: p.code.clone(),
            text: text.clone(),
            detail: p.detail.clone(),
        });
    }
    out
}
