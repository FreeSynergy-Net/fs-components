//! Tests of `rows_form` and of the two views that use it: one mapping, so table and grid cannot differ.
//!
//! Pattern: Test double (a `NoWrite` host and a `say` that knows one sentence).

use std::sync::Arc;

use fs_components_core::form_read::{ChannelInfo, FieldInfo, FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView};
use fs_components_core::form_write::NoWrite;
use fs_components_core::keys::Key;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_words::Locale;

use crate::rows_form::columns_and_cells;

fn channel(name: &str, fields: &[&str]) -> ChannelInfo {
    ChannelInfo {
        name: name.to_owned(),
        fields: fields
            .iter()
            .map(|f| FieldInfo {
                name: (*f).to_owned(),
                block: "field-text".to_owned(),
                label: format!("person.{f}"),
            })
            .collect(),
        choices: Vec::new(),
    }
}

pub(crate) fn input() -> FormInput {
    FormInput {
        form: "rows@1.0.0".to_owned(),
        rows: vec![
            FormRow::of("a", &[("label", "Alpha"), ("state", "high")]),
            FormRow::of("b", &[("state", "low")]),
        ],
        channels: vec![
            channel("columns", &["owner", "due"]),
            channel("state", &["level"]),
        ],
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

pub(crate) fn ctx() -> DrawCtx {
    DrawCtx::new(
        fs_components_core::form_view::PlaceInfo::default(),
        (StyleId::Admin, false),
        Arc::new(|k| {
            if k == "person.owner" {
                "Owner".to_owned()
            } else {
                k.to_owned()
            }
        }),
        Locale::english(),
        Arc::new(NoWrite),
        Arc::new(input),
    )
}

#[test]
fn the_columns_are_label_then_columns_then_state_named_by_the_host_or_the_field() {
    let (cols, rows) = columns_and_cells(&input(), &ctx());
    assert_eq!(cols.len(), 4);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, "a");
    assert_eq!(rows[0].1, ["Alpha", "", "", "high"]);
    // a row without a label shows its key
    assert_eq!(rows[1].1[0], "b");
}

#[test]
fn the_grid_view_takes_the_keys_of_the_host_and_the_table_view_does_not() {
    let grid = crate::grid::form_view::View.mount(input(), ctx());
    assert!(grid.key(Key::Down), "the cursor moves");
    let table = crate::table::form_view::View.mount(input(), ctx());
    assert!(!table.key(Key::Down), "a table is read, not walked");
}

#[test]
fn the_two_views_are_two_widgets() {
    assert_eq!(crate::grid::form_view::View.widget(), "Grid");
    assert_eq!(crate::table::form_view::View.widget(), "Table");
}

#[test]
fn a_cell_holds_the_values_of_its_field_and_the_head_is_the_word_of_the_field() {
    let mut i = input();
    i.rows = vec![
        FormRow::of("a", &[("label", "Alpha")])
            .with_values("columns", &[("due", "d1"), ("due", "d2")]),
    ];
    let (cols, rows) = columns_and_cells(&i, &ctx());
    assert_eq!(
        cols[1].label, "Owner",
        "the word of the field, not its name"
    );
    assert_eq!(cols[2].label, "due", "no sentence: the name of the field");
    // `owner` has no value: its cell is empty and `due` is NOT shifted into it
    assert_eq!(rows[0].1, ["Alpha", "", "d1\nd2", ""]);
}
