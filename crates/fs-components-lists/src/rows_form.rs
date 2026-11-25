//! The form `rows` read as columns and cells: the one place that maps the channels to the columns of `table` and `grid`.
//!
//! The first column is the channel `label`, then one column per value of the channel `columns`, then the channel `state` as the
//! last column; each is named by the field of the module that fills it (the sentence for a head cell is `channel.<name>` when the
//! host has one).
//!
//! Pattern: Adapter (rows by channel in, columns and cells out) – one function for every view of `rows`, so no view keeps a copy.

use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, said_or};

use crate::grid::Column;

/// The words of a head cell: the host's sentence for `channel.<name>`, else the name itself.
fn head(ctx: &DrawCtx, name: &str) -> String {
    let key = format!("channel.{name}");
    said_or((ctx.say)(&key), &key, name)
}

/// The columns of `input` and, per row, its key and cells (one cell per column).
pub(crate) fn columns_and_cells(
    input: &FormInput,
    ctx: &DrawCtx,
) -> (Vec<Column>, Vec<(String, Vec<String>)>) {
    let names: Vec<String> = input
        .channel("columns")
        .map(|c| c.fields.iter().map(|f| f.name.clone()).collect())
        .unwrap_or_default();
    // the channel `state` (the mark of a row, e.g. a level) is the last column, named by its field
    let state: Option<String> = input
        .channel("state")
        .and_then(|c| c.fields.first().map(|f| f.name.clone()));
    let mut cols = vec![Column::new("label", head(ctx, "label"))];
    cols.extend(names.iter().map(|n| Column::new(n.as_str(), head(ctx, n))));
    cols.extend(state.iter().map(|n| Column::new(n.as_str(), head(ctx, n))));
    let rows = input
        .rows
        .iter()
        .map(|r| {
            let mut cells = vec![r.get("label").unwrap_or(&r.key).to_owned()];
            cells.extend(
                (0..names.len()).map(|i| r.all("columns").get(i).cloned().unwrap_or_default()),
            );
            if state.is_some() {
                cells.push(r.get("state").unwrap_or_default().to_owned());
            }
            (r.key.clone(), cells)
        })
        .collect();
    (cols, rows)
}

/// The width of one column when `width` is shared by `count` columns (not below 80).
pub(crate) fn column_width(width: f32, count: usize) -> f32 {
    (width / f32::from(u8::try_from(count).unwrap_or(u8::MAX))).max(80.0)
}
