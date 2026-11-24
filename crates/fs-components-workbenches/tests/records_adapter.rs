//! The records adapter of the list (K8): the thread boundary, the cells of a row in order, a hidden field showing the reason of the
//! plan and never the value. (The board half of this file, `RecordsBoard`, fell away with K3, see CHANGELOG.)
//!
//! Mutants (each makes the named test fail): M2 a cell for a hidden field shows the value -> `hidden_shows_the_reason_of_the_plan_never_the_value`;
//! M13 cells in the wrong order, or a row dropped -> `list_rows_are_cells_in_order`.
#![allow(clippy::many_single_char_names)] // short names for the worker and script of each case
mod records_common;

use fs_components_widgets::list_view::RowSource;
use fs_components_workbenches::records::{HiddenReason, RecordsRows};
use records_common::*;

#[test]
fn adapters_cross_the_thread_boundary() {
    fn is_sync<T: Send + Sync>() {}
    is_sync::<RecordsRows>();
    is_sync::<fs_components_workbenches::records::Worker>();
}

#[test]
fn list_rows_are_cells_in_order() {
    let s = filled();
    let w = start(&s, binding());
    let rows = RecordsRows::new(w, binding(), judge());
    assert_eq!(rows.count(), 3);
    let got = rows.rows(1, 5).unwrap();
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].key, "t2");
    assert_eq!(got[0].cells, ["Read", ""]);
    assert_eq!(rows.rows(0, 1).unwrap()[0].cells, ["Write", "2026-11-20"]);
    assert_eq!(rows.position_of("t3"), Some(2));
    assert_eq!(rows.position_of("zz"), None);
}

#[test]
fn hidden_shows_the_reason_of_the_plan_never_the_value() {
    let s = filled();
    {
        let mut g = script(&s);
        let mut r = hide(g.rows[0].clone(), "due", HiddenReason::Sensitive);
        r.values.retain(|v| v.field != "due"); // a hidden field carries no value
        g.rows[0] = r;
    }
    let rows = RecordsRows::new(start(&s, binding()), binding(), judge());
    // the second cell of the first row is the field `due`: the reason, not the value
    assert_eq!(
        rows.rows(0, 1).unwrap()[0].cells,
        ["Write", "hidden: sensitive"]
    );
}
