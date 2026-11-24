//! Records to what the list draws: the cells of a row, the lock mark, and the notes that say what is wrong with the data. Every
//! state of a field (value, none, hidden, missing) comes from the plan through the [`Judge`]; nothing is decided here and no sentence is
//! ours.
//!
//! Pattern: Mapper (pure functions over one row, one binding and the judge). A hidden or missing slot carries no value.

use super::judge::{Judge, Shown};
use super::mirror::RecordRow;
use super::shape::RecordsBinding;
use super::snapshot::{Snapshot, Status};
use fs_components_fields::SlotReason as Reason;

/// One thing the host says next to the data: a code (for tests and logs) and what goes with it. The sentence is the host's (the
/// plan's words), not ours.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    /// The code (what the plan gave a refused page, `group-unknown`, `truncated`, `stale`, ...).
    pub code: String,
    /// The reason the plan gave, a count, a cursor or the code of the provider's fault.
    pub detail: Option<String>,
}

/// The text of a field where one line is all there is (a cell): the value or the reason.
fn line_of(row: &RecordRow, field: &str, judge: &dyn Judge) -> Option<String> {
    match judge.shown(row, field) {
        Shown::Value(v) => Some(v),
        Shown::Empty => None,
        Shown::Hidden(r) | Shown::Missing(r) => Some(r.said().to_owned()),
    }
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

/// The notes for a snapshot and the state of the worker, in a fixed order: the state first, then the problems.
pub(super) fn notes(snap: &Snapshot, status: &Status) -> Vec<Note> {
    let mut out = Vec::new();
    if let Status::Stale(fault) = status {
        out.push(Note {
            code: "stale".into(),
            detail: Some(fault.code.clone()),
        });
    }
    out.extend(snap.problems.iter().map(|p| Note {
        code: p.code.clone(),
        detail: p.detail.clone(),
    }));
    out
}
