//! `column-table`: the settings of a board's columns as a table: a handle to drag, name, status, stage, limit, cards now, a row
//! menu, "+ Add column", and a Save that is blocked with its reason while the program finds something wrong.
//!
//! Pattern: none (declares modules and re-exports only). The model [`Rows`] is pure; the program gives the check
//! ([`ColumnCheck`]) and the choices of the status, and does the saving. The same table fits other lists of "named places with a
//! limit" (lanes of a picture board, steps of a form); it knows no board.

mod model;
mod rows;
mod styles;
mod view;

pub use model::{AllowAll, ColumnCheck, ColumnRow, Problem, RemoveError, Rows, Save, parse_wip};
pub use styles::STYLES;
pub use view::{ColumnTable, TableLabels};
