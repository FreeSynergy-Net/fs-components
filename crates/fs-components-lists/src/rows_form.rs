//! The form `rows` read as columns and cells: the one place that maps the channels to the columns of `table` and `grid`.
//!
//! The first column is the channel `label`, then one column per field of the channel `columns`, then the channel `state` as the
//! last column. A cell holds the values of ITS field (found by the path of the value, never by position): several values are
//! several lines in the cell, none is `""`. The head of a column is the word of the field (`FieldInfo.label`, from the type
//! catalog of the module), else the name of the field.
//!
//! Pattern: Adapter (rows by channel in, columns and cells out) – one function for every view of `rows`, so no view keeps a copy.

use fs_components_core::form_read::{FieldInfo, FormInput};
use fs_components_core::form_view::{DrawCtx, said_or};

use crate::grid::Column;

/// The words of the head of a column of a channel (`label`, `state`): the host's sentence for `channel.<name>`, else the name.
fn channel_head(ctx: &DrawCtx, name: &str) -> String {
    let key = format!("channel.{name}");
    said_or((ctx.say)(&key), &key, name)
}

/// The columns of `input` and, per row, its key and cells (one cell per column).
pub(crate) fn columns_and_cells(
    input: &FormInput,
    ctx: &DrawCtx,
) -> (Vec<Column>, Vec<(String, Vec<String>)>) {
    let fields: &[FieldInfo] = input.channel("columns").map_or(&[], |c| &c.fields);
    // the channel `state` (the mark of a row, e.g. a level) is the last column, named by its field
    let state: Option<&FieldInfo> = input.channel("state").and_then(|c| c.fields.first());
    let mut cols = vec![Column::new("label", channel_head(ctx, "label"))];
    cols.extend(
        fields
            .iter()
            .map(|f| Column::new(f.name.as_str(), f.shown(&*ctx.say))),
    );
    cols.extend(
        state
            .iter()
            .map(|f| Column::new(f.name.as_str(), f.shown(&*ctx.say))),
    );
    let rows = input
        .rows
        .iter()
        .map(|r| {
            let mut cells = vec![r.get("label").unwrap_or(&r.key).to_owned()];
            cells.extend(
                fields
                    .iter()
                    .map(|f| r.field("columns", &f.name).join("\n")),
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
