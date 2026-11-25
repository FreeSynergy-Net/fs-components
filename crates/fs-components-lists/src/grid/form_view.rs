//! The grid as a [`FormView`] of the form `rows`: the same columns and cells as the table (`rows_form`), walked cell by cell with the keys.
//!
//! Pattern: Abstract Factory (`View`) + Adapter (rows by channel in, a [`Grid`] out); the keys of the host go on to [`Grid::press`].

use std::sync::{Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};
use fs_components_core::keys::{Key, KeyEvent};

use super::{Grid, Row};
use crate::rows_form::{column_width, columns_and_cells};

/// The widget this view is (`widget` of the block `grid`).
pub const WIDGET: &str = "Grid";

/// The view of the block `grid`.
#[derive(Clone, Copy, Debug, Default)]
pub struct View;

impl FormView for View {
    fn widget(&self) -> &'static str {
        WIDGET
    }

    fn mount(&self, input: FormInput, ctx: DrawCtx) -> Box<dyn Mounted> {
        let grid = make(&input, &ctx);
        Box::new(MountedGrid(Mutex::new((ctx, grid))))
    }
}

/// The grid over `input`.
fn make(input: &FormInput, ctx: &DrawCtx) -> Grid {
    let (cols, rows) = columns_and_cells(input, ctx);
    let rows: Vec<Row> = rows.into_iter().map(|(k, c)| Row::new(k, c)).collect();
    Grid::new(cols, &rows).id("rows")
}

/// The context and the grid; a rebuild shares the state of the grid, so the cursor stays.
struct MountedGrid(Mutex<(DrawCtx, Grid)>);

impl Mounted for MountedGrid {
    fn build(&self, width: f32, _height: f32) -> Div {
        let grid = self
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .1
            .clone();
        let count = grid.column_count();
        div()
            .w(width)
            .child(grid.column_width(column_width(width, count)).build())
    }

    fn refresh(&self, input: FormInput) {
        let mut held = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        held.1 = make(&input, &held.0);
    }

    fn key(&self, key: Key) -> bool {
        let grid = self
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .1
            .clone();
        grid.press(KeyEvent::new(key))
    }
}
