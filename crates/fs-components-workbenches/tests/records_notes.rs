//! The notes of a list in the order the docs promise: the state of the source first (`stale`), then what is wrong with the data
//! (the problems of the snapshot). (The notes about the binding, `too-many-*`, belonged to the board and fell away with K3.)
//!
//! Mutants (each makes the named test fail): `stale` is not first in `map::notes` -> `the_notes_come_in_the_order_stale_data`;
//! a part is dropped -> the same.
//! Pattern: none; tests.
mod records_common;

use fs_components_workbenches::records::{RecordsRows, Status};
use records_common::*;

#[test]
fn the_notes_come_in_the_order_stale_data() {
    let s = filled();
    // data problem: a record in a group that does not exist
    script(&s)
        .rows
        .push(row("t9", "gone", &[("title", &["Lost"])]));
    let w = start(&s, binding());
    let b = RecordsRows::new(w.clone(), binding(), judge());
    let codes = |b: &RecordsRows| b.notes().into_iter().map(|n| n.code).collect::<Vec<_>>();
    assert_eq!(codes(&b), ["group-unknown"]);
    // the source stops answering: the state goes first, the rest stays
    let v = w.version();
    script(&s).fail = Some("unavailable".into());
    w.reload();
    until(|| w.version() > v, "stale");
    assert!(matches!(w.status(), Status::Stale(_)));
    assert_eq!(
        codes(&b),
        ["stale", "group-unknown"],
        "stale, then the data"
    );
}
