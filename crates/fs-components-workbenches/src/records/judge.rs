//! What the PLAN says about the records, as data: why a field is hidden, whether a page may be drawn, which commands are offered,
//! why a record is locked, what the reader is told when writing is not possible. The one source is `fs-forms-bind` (the plan:
//! `field_shown`, `admit_page`, `command_state`); the host implements [`Judge`] over it. fs-components renders the state it is
//! given and decides nothing (rule "frontend only components"): there is no default judge and no sentence of ours here.
//!
//! Pattern: Strategy (the host hands in the rules) + Value objects (`Shown`, `SlotReason`). The adapter calls the judge on the
//! worker thread and on the interface thread, hence `Send + Sync`.

use super::mirror::RecordRow;
use crate::board::SlotReason;

/// What a field shows for one record, said by the plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shown {
    /// The value, as the text the field component takes (the canonical text).
    Value(String),
    /// No value.
    Empty,
    /// Hidden for this reader: the reason, never a value.
    Hidden(SlotReason),
    /// The field or its block is not there: the reason.
    Missing(SlotReason),
}

/// The rules of the plan, as the adapter needs them.
pub trait Judge: Send + Sync {
    /// Whether a page of rows may be drawn at all. `Err`: the whole page is refused with this reason (a hidden field carried a
    /// value, the page was too long, ...); no row of it is drawn.
    ///
    /// # Errors
    /// The reason the page is refused.
    fn admit(&self, rows: &[RecordRow]) -> Result<(), SlotReason>;

    /// What `field` shows for `row`.
    fn shown(&self, row: &RecordRow, field: &str) -> Shown;

    /// Why `row` cannot be picked up (it is locked), if it cannot.
    fn locked(&self, row: &RecordRow) -> Option<SlotReason>;

    /// The ids of the commands the context menu of `row` offers (rights first, then "coming"), at most as many as the plan allows.
    fn commands(&self, row: &RecordRow) -> Vec<String>;

    /// What the reader is told when she moves or adds while the host only reads.
    fn write_later(&self) -> SlotReason;

    /// What the reader is told when the source does not answer (`code` is the code of the provider's fault).
    fn stale(&self, code: &str) -> SlotReason;
}
