//! The table as a [`FormView`] of the form `rows`: the first column is the channel `label`, then one column per value of the channel
//! `columns`, named by the field of the module that fills it (the sentence for a head cell is `channel.<name>` when the host has one).
//!
//! Pattern: Abstract Factory (`View`) + Adapter (rows by channel in, a [`Table`] out).

use std::sync::{Mutex, PoisonError};

use blinc_layout::div::{Div, div};
use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};

use super::Table;
use crate::grid::Column;

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

/// The words of a head cell: the host's sentence for `channel.<name>`, else the name itself.
fn head(ctx: &DrawCtx, name: &str) -> String {
    let key = format!("channel.{name}");
    let said = (ctx.say)(&key);
    if said == key { name.to_owned() } else { said }
}

impl Mounted for MountedTable {
    fn build(&self, width: f32, _height: f32) -> Div {
        let held = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let (input, ctx) = (&held.0, &held.1);
        let names: Vec<String> = input
            .channel("columns")
            .map(|c| c.fields.iter().map(|f| f.name.clone()).collect())
            .unwrap_or_default();
        let mut cols = vec![Column::new("label", head(ctx, "label"))];
        cols.extend(names.iter().map(|n| Column::new(n.as_str(), head(ctx, n))));
        let cells: Vec<Vec<String>> = input
            .rows
            .iter()
            .map(|r| {
                let mut cells = vec![r.get("label").unwrap_or(&r.key).to_owned()];
                cells.extend(
                    (0..names.len()).map(|i| r.all("columns").get(i).cloned().unwrap_or_default()),
                );
                cells
            })
            .collect();
        let column_w = (width / f32::from(u8::try_from(cols.len()).unwrap_or(u8::MAX))).max(80.0);
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
