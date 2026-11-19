//! `table`: role, caption naming the table, row and cell roles, row headers, the counts of a partial table, no tab stop.
//!
//! Tests: attribute reads on the built element.

mod common;

use blinc_layout::prelude::RenderTree;
use common::{attr, layout_with, serial};
use fs_components_widgets::grid::Column;
use fs_components_widgets::table::{STYLES, Table};

fn cols() -> Vec<Column> {
    vec![Column::new("n", "Name"), Column::new("s", "Size")]
}

fn rows() -> Vec<Vec<String>> {
    vec![vec!["a".into(), "1".into()], vec!["b".into()]]
}

fn show(t: &Table) -> RenderTree {
    layout_with(t.build(), 500.0, 300.0, STYLES)
}

#[test]
fn a_table_with_a_caption_is_named_by_it_and_takes_no_keyboard() {
    let _g = serial();
    let tree = show(&Table::new(cols(), rows()).id("tb").caption("Files"));
    assert_eq!(attr(&tree, "tb", "role").as_deref(), Some("table"));
    assert_eq!(attr(&tree, "tb", "aria-labelledby").as_deref(), Some("tb-caption"));
    assert_eq!(attr(&tree, "tb", "aria-label"), None, "the caption names it, not a label");
    assert!(tree.query_by_id("tb-caption").is_some());
    assert_eq!(attr(&tree, "tb", "tabindex"), None, "a table is read, not walked");
    assert_eq!(attr(&tree, "tb-row-0", "role").as_deref(), Some("row"));
}

#[test]
fn a_table_without_a_caption_may_have_a_name() {
    let _g = serial();
    let tree = show(&Table::new(cols(), rows()).id("tn").name("Sizes"));
    assert_eq!(attr(&tree, "tn", "aria-label").as_deref(), Some("Sizes"));
    assert_eq!(attr(&tree, "tn", "aria-labelledby"), None);
    assert!(tree.query_by_id("tn-caption").is_none());
}

#[test]
fn counts_are_told_only_for_a_partial_table_and_the_head_counts_as_a_row() {
    let _g = serial();
    let whole = show(&Table::new(cols(), rows()).id("tw"));
    assert_eq!(attr(&whole, "tw", "aria-rowcount"), None);
    assert_eq!(attr(&whole, "tw", "aria-colcount"), None);
    let part = show(&Table::new(cols(), rows()).id("tp").of(100, 5));
    assert_eq!(attr(&part, "tp", "aria-rowcount").as_deref(), Some("101"));
    assert_eq!(attr(&part, "tp", "aria-colcount").as_deref(), Some("5"));
}

#[test]
fn head_cells_are_column_headers_cells_are_cells_and_row_headers_name_their_row() {
    let _g = serial();
    let plain = show(&Table::new(cols(), rows()).id("tr"));
    assert_eq!(attr(&plain, "tr-head-0", "role").as_deref(), Some("columnheader"));
    assert_eq!(attr(&plain, "tr-head-1", "role").as_deref(), Some("columnheader"));
    assert_eq!(attr(&plain, "tr-cell-0-0", "role").as_deref(), Some("cell"));
    assert_eq!(attr(&plain, "tr-cell-0-1", "role").as_deref(), Some("cell"));
    let named = show(&Table::new(cols(), rows()).id("th").row_headers());
    assert_eq!(attr(&named, "th-cell-0-0", "role").as_deref(), Some("rowheader"));
    assert_eq!(attr(&named, "th-cell-1-0", "role").as_deref(), Some("rowheader"));
    assert_eq!(attr(&named, "th-cell-0-1", "role").as_deref(), Some("cell"), "only the first cell names the row");
    assert_eq!(attr(&named, "th-cell-0-0", "aria-colindex"), None, "a whole table has no column indexes");
    let part = show(&Table::new(cols(), rows()).id("ti").of(9, 4));
    assert_eq!(attr(&part, "ti-cell-0-1", "aria-colindex").as_deref(), Some("2"));
    assert_eq!(attr(&part, "ti-head-0", "aria-colindex").as_deref(), Some("1"));
}
