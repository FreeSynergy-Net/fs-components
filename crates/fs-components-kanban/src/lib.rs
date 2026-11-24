//! `fs-components-kanban`: a board of columns with cards. Interim (one-board draft, until K3): `board` is the board `views()` draws (mouse
//! drag, keys, undo); the parts below (`KanbanBoard`, `state`, `view`) are the old board, no longer drawn by `View`, removed in K3.
//!
//! Plain data in (`Column`, `Card`), a trait for the data (`BoardSource`, which also says which
//! operations the host supports) and one for what the reader asks (`BoardEvents`), a pure model
//! (moves by grab/hover/drop, keys, move buttons) and a Blinc view in the look of
//! `fs-components-styles`. The old view draws unsupported operations locked with a "comes later" hint (the new board leaves them out until K2a).
//! Texts are translation keys answered by `Say`. Entry point of the old board: [`KanbanBoard::build`]; of the new one: [`View`] / `board::BoardBench`.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]
// Pixel arithmetic over a few dozen cards (counts far below 2^23) and short names for the style and
// palette in the view functions: the casts cannot lose anything that matters here.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::many_single_char_names,
    clippy::similar_names
)]

mod ask;
pub mod board;
mod form_dialogs;
mod form_events;
pub mod form_read;
mod form_source;
pub mod form_view;
pub mod handle;
mod keys;
pub mod layout;
pub mod limits;
pub mod model;
mod quick_add;
pub mod source;
pub mod state;
pub mod view;

pub use form_events::{
    COLUMN_CREATE, COLUMN_LIMIT, COLUMN_RENAME, CREATE, FormEvents, LANE, LIMIT, MOVE, NAME,
};
pub use form_read::columns_of;
pub use form_source::FormSource;
pub use form_view::View;
pub use fs_components_fields::Limit;
pub use fs_components_fields::{FieldSlot, SlotReason, SlotState};
pub use handle::KanbanBoard;
pub use layout::PROGRESS_BAR;
pub use limits::{DEFAULT_FACTS, FACTS_SETTING};
pub use model::{Card, Column, Due, DueState, Label, Operation, Tone};
pub use source::{Answer, BoardEvents, BoardSource, FixedBoard, NoEvents};
pub use state::{BoardState, Held, MoveRequest, Step};
