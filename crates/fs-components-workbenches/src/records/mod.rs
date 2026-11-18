//! `records`: the adapter from the records of a program to the workbenches that are placed with them (board, list, side frame),
//! read only (H2 of the host plan).
//!
//! Pattern: none (declares modules and re-exports only). The adapter is [`RecordsBoard`] (a `BoardSource`, `BoardEvents` and
//! `DetailContent`) and [`RecordsRows`] (a `RowSource`). Both read one [`Snapshot`] that a worker thread keeps current: the
//! records are asked for through the narrow mirror [`RecordsRead`] (asynchronous, not `Send`), the workbenches are synchronous and
//! `Send + Sync`. The mirror is this crate's own copy of the read side of `fs-records` (not pinned until that crate has a tag);
//! the real client of `fs-rpc` implements it later. Nothing here knows a program: types and fields are names of the binding.

mod board;
mod drive;
mod list;
mod map;
mod mirror;
mod present;
mod shape;
mod snapshot;
mod texts;

pub use board::RecordsBoard;
pub use drive::{Driver, Worker, block_on_std};
pub use list::RecordsRows;
pub use map::Note;
pub use mirror::{
    Fault, FieldValue, Group, GroupSet, Hidden, HiddenReason, LockKind, Page, Pending, RecordLock,
    RecordQuery, RecordRow, RecordsRead,
};
pub use present::{Present, join_values};
pub use shape::{RecordsBinding, Role, Slot};
pub use snapshot::{Outcome, Problem, Snapshot, Status, refresh};
pub use texts::{RecordsTexts, TEXTS};
