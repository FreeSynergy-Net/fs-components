//! `grid` and `treegrid`: rows and columns the person walks cell by cell with the keys; with the second, rows that open and close.
//!
//! Pattern: State (`model.rs`, pure: the columns, the rows in an arena, the open marks, the cursor cell) + Observer (`view.rs`: `Live`
//! draws the rows again, the grid's own element stays so the keyboard stays) + Adapter. WAI-ARIA APG, data grid: role `grid`
//! (`treegrid`), `aria-rowcount` and `aria-colcount` (the header counts as a row), rows `row` with `aria-rowindex`, cells `gridcell`
//! with `aria-colindex`, the head cells `columnheader`. Arrows move one cell (no wrap), Home and End go to the first and last cell of
//! the row, Ctrl+Home and Ctrl+End to the first and last cell of the grid, Page Up and Page Down by a page of rows (a setting), Enter
//! or Space activates the cell. Treegrid (APG): a parent row has `aria-expanded`, `aria-level`, `aria-setsize`, `aria-posinset`;
//! on the first cell Right opens a closed parent and Left closes an open one. Limits: the grid is ONE tab stop and the cursor is the
//! `fs-grid-cell-current` mark (like `segmented`); the head cells are not on the cursor's way; cells are text (a cell that holds
//! controls would need a tab stop of its own); no sorting, no editing, no virtual scrolling (`list_view` is the big one).

pub mod form_view;
mod model;
mod styles;
mod view;

pub use model::{Column, Outcome, Row, State};
pub use styles::STYLES;
pub use view::Grid;
