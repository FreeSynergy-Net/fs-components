//! Records to what the workbenches draw: cards with facts (K5), the frame at the side with elements (K6), the cells of a row
//! (K8), and the notes that say what is wrong with the data. Every state of a field (value, none, hidden, missing) comes from
//! the plan through the [`Judge`] and is only turned into a [`FieldSlot`]; nothing is decided here and no sentence is ours.
//!
//! Pattern: Mapper (pure functions over one row, one binding and the judge). A hidden or missing slot carries no value.

use super::judge::{Judge, Shown};
use super::mirror::RecordRow;
use super::shape::{RecordsBinding, Slot};
use super::snapshot::{Problem, Snapshot, Status};
use crate::board::{Card, DetailFrame, FieldSlot, Reason};

/// One thing the host says next to the data: a code (for tests and logs) and what goes with it. The sentence is the host's (the
/// plan's words), not ours.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    /// The code (what the plan gave a refused page, `group-unknown`, `truncated`, `stale`, `too-many-facts`, ...).
    pub code: String,
    /// The reason the plan gave, a count, a cursor or the code of the provider's fault.
    pub detail: Option<String>,
}

pub(super) fn slot_of(row: &RecordRow, slot: &Slot, judge: &dyn Judge) -> FieldSlot {
    match judge.shown(row, &slot.field) {
        Shown::Value(v) => FieldSlot::new(&slot.block, &slot.label, &v),
        Shown::Empty => FieldSlot::unset(&slot.block, &slot.label),
        Shown::Hidden(r) => FieldSlot::hidden(&slot.block, &slot.label, r),
        Shown::Missing(r) => FieldSlot::missing(&slot.block, &slot.label, r),
    }
}

/// The text of a field where one line is all there is (title, subtitle, a cell): the value or the reason.
fn line_of(row: &RecordRow, field: &str, judge: &dyn Judge) -> Option<String> {
    match judge.shown(row, field) {
        Shown::Value(v) => Some(v),
        Shown::Empty => None,
        Shown::Hidden(r) | Shown::Missing(r) => Some(r.said().to_owned()),
    }
}

pub(super) fn card(row: &RecordRow, b: &RecordsBinding, judge: &dyn Judge) -> Card {
    let title = line_of(row, &b.title, judge).unwrap_or_else(|| row.id.clone());
    let mut card = Card::limited(&row.id, &title, b.facts_limit);
    card.subtitle = b.subtitle.as_deref().and_then(|f| line_of(row, f, judge));
    for slot in &b.facts {
        // more than the card takes: the card itself says "+ N more" (and `binding_problems` says it once for the binding)
        card = card.fact_or_more(slot_of(row, slot, judge));
    }
    card
}

pub(super) fn frame(
    row: &RecordRow,
    path: &str,
    b: &RecordsBinding,
    judge: &dyn Judge,
) -> DetailFrame {
    let title = line_of(row, &b.title, judge).unwrap_or_else(|| row.id.clone());
    let mut frame = DetailFrame::limited(title, b.elements_limit).path(path);
    for slot in &b.detail {
        frame = frame.element_or_more(slot.label.clone(), slot_of(row, slot, judge));
    }
    frame
}

pub(super) fn cells(row: &RecordRow, b: &RecordsBinding, judge: &dyn Judge) -> Vec<String> {
    b.cells
        .iter()
        .map(|s| line_of(row, &s.field, judge).unwrap_or_default())
        .collect()
}

pub(super) fn lock_reason(row: &RecordRow, judge: &dyn Judge) -> Option<Reason> {
    judge.locked(row)
}

/// Problems of the binding itself: more slots than the card or the frame takes. The plan's `accepts` should have caught it; if it
/// did not, it is said here ONCE (a note for the binding); the card and the frame say "+ N more" on themselves for every record.
pub(super) fn binding_problems(b: &RecordsBinding) -> Vec<Problem> {
    let mut out = Vec::new();
    if b.facts.len() > b.facts_limit.get() {
        out.push(Problem::new(
            "too-many-facts",
            Some(b.facts.len().to_string()),
        ));
    }
    if b.detail.len() > b.elements_limit.get() {
        out.push(Problem::new(
            "too-many-elements",
            Some(b.detail.len().to_string()),
        ));
    }
    out
}

/// The notes for a snapshot and the state of the worker, in a fixed order: the state first, then the problems.
pub(super) fn notes(snap: &Snapshot, status: &Status, own: &[Problem]) -> Vec<Note> {
    let mut out = Vec::new();
    if let Status::Stale(fault) = status {
        out.push(Note {
            code: "stale".into(),
            detail: Some(fault.code.clone()),
        });
    }
    out.extend(own.iter().chain(&snap.problems).map(|p| Note {
        code: p.code.clone(),
        detail: p.detail.clone(),
    }));
    out
}
