//! The records adapter when the data is wrong, missing or changing: a refused page, rows in no group, a cut-off list, a source that
//! fails (the last state stays and is said to be old), a list that never got an answer, the generation (one call when unchanged,
//! a reload when changed). Nothing is left out without a word.
//!
//! Mutants (each makes the named test fail): M4 a leaking page is accepted -> `leaking_page_is_refused_and_said`; M5 a row of an
//! unknown group is dropped without a note -> `unknown_group_not_shown_but_said`; M6 the cut-off at `max_pages` is silent ->
//! `cut_off_is_said`; M7 a failed call clears the snapshot or says nothing -> `failure_keeps_last_state_and_says_so`; M8 an unchanged
//! generation reads again -> `unchanged_generation_costs_one_call`; M9 a changed generation is ignored -> `changed_generation_reloads`.
#![allow(clippy::many_single_char_names)] // short names for the board, worker and script of each case
mod records_common;

use fs_components_widgets::list_view::RowSource;
use fs_components_workbenches::records::{HiddenReason, RecordsRows, Status};
use records_common::*;

fn list(s: &Shared) -> RecordsRows {
    RecordsRows::new(start(s, binding()), binding(), judge())
}

#[test]
fn leaking_page_is_refused_and_said() {
    let s = filled();
    {
        let mut g = script(&s);
        let mut r = hide(g.rows[2].clone(), "note", HiddenReason::NoRight);
        r.values
            .push(fs_components_workbenches::records::FieldValue {
                field: "note".into(),
                values: vec!["leak".into()],
            });
        g.rows[2] = r;
    }
    let b = list(&s);
    assert_eq!(b.count(), 0, "the whole page is dropped");
    let notes = b.notes();
    assert_eq!(notes.len(), 1);
    assert_eq!(
        (notes[0].code.as_str(), notes[0].detail.as_deref()),
        ("bind-error-page-refused", Some("hidden-has-value note"))
    );
}

#[test]
fn unknown_group_not_shown_but_said() {
    let s = filled();
    script(&s)
        .rows
        .push(row("t9", "gone", &[("title", &["Lost"])]));
    let b = list(&s);
    assert_eq!(b.count(), 3, "the row of the unknown group is not shown");
    let notes = b.notes();
    assert_eq!(notes[0].code, "group-unknown");
    assert_eq!(notes[0].detail.as_deref(), Some("1"));
}

#[test]
fn cut_off_is_said() {
    let s = filled();
    script(&s).page_len = 1;
    let mut bind = binding();
    bind.max_pages = 2;
    let w = start(&s, bind.clone());
    let b = RecordsRows::new(w, bind, judge());
    assert_eq!(b.count(), 2);
    assert!(b.notes().iter().any(|x| x.code == "truncated"));
    assert_eq!(script(&s).finds, 2);
}

#[test]
fn failure_keeps_last_state_and_says_so() {
    let s = filled();
    let w = start(&s, binding());
    let b = RecordsRows::new(w.clone(), binding(), judge());
    assert_eq!(b.notes().len(), 0);
    let v = w.version();
    script(&s).fail = Some("unavailable".into());
    w.reload();
    until(|| w.version() > v, "stale");
    assert!(matches!(w.status(), Status::Stale(f) if f.code == "unavailable"));
    assert_eq!(b.count(), 3, "the last state stays");
    let n = &b.notes()[0];
    assert_eq!(
        (n.code.as_str(), n.detail.as_deref()),
        ("stale", Some("unavailable"))
    );
    script(&s).fail = None;
    w.reload();
    until(|| matches!(w.status(), Status::Ready), "ready again");
    assert_eq!(b.notes().len(), 0);
}

#[test]
fn never_answered_list_says_why() {
    let s = filled();
    script(&s).fail = Some("unavailable".into());
    let w = start(&s, binding());
    let rows = RecordsRows::new(w, binding(), judge());
    let e = rows.rows(0, 10).unwrap_err();
    assert!(e.contains("unavailable"), "{e}");
}

#[test]
fn unchanged_generation_costs_one_call() {
    let s = filled();
    let w = start(&s, binding());
    let finds = script(&s).finds;
    let v = w.version();
    w.poll();
    until(|| w.version() > v, "poll");
    assert_eq!(script(&s).finds, finds, "no second read");
    assert_eq!(script(&s).generations, 2);
}

#[test]
fn changed_generation_reloads() {
    let s = filled();
    let w = start(&s, binding());
    let (gen0, v) = (w.snapshot().generation, w.version());
    {
        let mut g = script(&s);
        g.generation = 8;
        g.rows.push(row("t4", "doing", &[("title", &["New"])]));
    }
    w.poll();
    until(|| w.version() > v, "poll");
    assert_ne!(
        w.snapshot().generation,
        gen0,
        "the reader sees that something changed"
    );
    assert_eq!(w.snapshot().rows.len(), 4);
    assert_eq!(w.snapshot().generation, 8);
}
