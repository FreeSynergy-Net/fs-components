//! The table as a [`FormView`] of the form `rows`: the columns and cells come from `rows_form` (shared with the grid).
//!
//! Pattern: Abstract Factory (`View`) + Adapter (rows by channel in, a [`Table`] out).

use std::sync::{Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};

use super::Table;
use crate::rows_form::{column_width, columns_and_cells};

/// The widget this view is (`widget` of the block `table`).
pub const WIDGET: &str = "Table";

/// The view of the block `table`.
#[derive(Clone, Copy, Debug, Default)]
pub struct View;

impl FormView for View {
    fn widget(&self) -> &'static str {
        WIDGET
    }

    fn mount(&self, input: FormInput, ctx: DrawCtx) -> Box<dyn Mounted> {
        Box::new(MountedTable(Mutex::new((input, ctx))))
    }
}

struct MountedTable(Mutex<(FormInput, DrawCtx)>);

impl Mounted for MountedTable {
    fn build(&self, width: f32, _height: f32) -> Div {
        let held = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let (cols, rows) = columns_and_cells(&held.0, &held.1);
        let column_w = column_width(width, cols.len());
        let cells: Vec<Vec<String>> = rows.into_iter().map(|(_, c)| c).collect();
        div().w(width).child(
            Table::new(cols, cells)
                .id("rows")
                .column_width(column_w)
                .build(),
        )
    }

    fn refresh(&self, input: FormInput) {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).0 = input;
    }
}
