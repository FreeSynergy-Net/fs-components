//! The notes of a board in the order the docs promise: the state of the source first (`stale`), then what is wrong with the binding
//! (`too-many-*`), then what is wrong with the data (the problems of the snapshot).
//!
//! Mutants (each makes the named test fail): Y3 the problems of the binding come after the problems of the snapshot in `map::notes`
//! -> `the_notes_come_in_the_order_stale_binding_data`; `stale` is not first -> the same; a part is dropped -> the same.
//! Pattern: none; tests.
mod records_common;

use fs_components_workbenches::board::Limit;
use fs_components_workbenches::records::{RecordsBoard, Status};
use records_common::*;

#[test]
fn the_notes_come_in_the_order_stale_binding_data() {
    let s = filled();
    // data problem: a record in a group that does not exist
    script(&s)
        .rows
        .push(row("t9", "gone", &[("title", &["Lost"])]));
    // binding problem: more slots than the card and the frame take
    let bind = binding().limits(Limit::of(1), Limit::of(1));
    let w = start(&s, bind.clone());
    let b = RecordsBoard::new(w.clone(), bind, judge());
    let codes = |b: &RecordsBoard| b.notes().into_iter().map(|n| n.code).collect::<Vec<_>>();
    assert_eq!(
        codes(&b),
        ["too-many-facts", "too-many-elements", "group-unknown"],
        "the binding before the data"
    );
    // the source stops answering: the state goes first, the rest stays
    let v = w.version();
    script(&s).fail = Some("unavailable".into());
    w.reload();
    until(|| w.version() > v, "stale");
    assert!(matches!(w.status(), Status::Stale(_)));
    assert_eq!(
        codes(&b),
        [
            "stale",
            "too-many-facts",
            "too-many-elements",
            "group-unknown"
        ],
        "stale, then the binding, then the data"
    );
}
