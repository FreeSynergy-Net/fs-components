//! `table`: static rows and columns for reading; no keys of its own.
//!
//! Pattern: Builder (a table is described, then drawn once) + Adapter. WAI-ARIA APG, table: role `table` named by its caption
//! (`aria-labelledby`) or `aria-label`, rows `row`, head cells `columnheader`, cells `cell`, and with `row_headers` the first cell of
//! every row is `rowheader`; `aria-rowcount` and `aria-colcount` are set when the table shows only part of a larger one (`Table::of`).
//! A table is read, not walked: its cells take no keyboard (a table that is walked is `grid`). The head and the body are `rowgroup`s.

mod styles;
mod view;

pub use styles::STYLES;
pub use view::Table;
