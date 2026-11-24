//! `records`: the adapter from the records of a program to the list widget (read only, H2 of the host plan). The adapter that put the
//! same records behind the board (`RecordsBoard`) fell away with K3: the board element has one way in, the forms adapter of
//! `fs-components-kanban` (`FormSource`, `FormEvents`).
//!
//! Pattern: none (declares modules and re-exports only). The adapter is [`RecordsRows`] (a `RowSource`). It reads one [`Snapshot`] that
//! a worker thread keeps current: the records are asked for through the narrow mirror [`RecordsRead`] (asynchronous, not `Send`), the
//! widget is synchronous and `Send + Sync`. The mirror is this crate's own copy of the read side of `fs-records` (not pinned until
//! that crate has a tag); the real client of `fs-rpc` implements it later. Nothing here knows a program: types and fields are names of
//! the binding. Nothing here decides why a field is hidden or a record locked: the plan does (`fs-forms-bind`), the host hands its
//! rules in as a [`Judge`] and this module only turns the states it names into the texts of the cells.

mod drive;
mod judge;
mod list;
mod map;
mod mirror;
mod shape;
mod snapshot;

pub use drive::{Driver, Worker, block_on_std};
pub use judge::{Judge, Shown};
pub use list::RecordsRows;
pub use map::Note;
pub use mirror::{
    Fault, FieldValue, Group, GroupSet, Hidden, HiddenReason, LockKind, Page, Pending, RecordLock,
    RecordQuery, RecordRow, RecordsRead,
};
pub use shape::{RecordsBinding, Slot};
pub use snapshot::{Outcome, Problem, Snapshot, Status, refresh};
