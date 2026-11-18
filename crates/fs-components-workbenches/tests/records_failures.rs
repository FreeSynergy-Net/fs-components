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
use fs_components_workbenches::board::BoardSource;
use fs_components_workbenches::records::{
    HiddenReason, RecordsBoard, RecordsRows, RecordsTexts, Status,
};
use records_common::*;

fn board(s: &Shared) -> RecordsBoard {
    RecordsBoard::new(start(s, binding()), binding(), RecordsTexts::default())
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
    let b = board(&s);
    assert!(
        b.columns().iter().all(|c| c.cards.is_empty()),
        "the whole page is dropped"
    );
    let notes = b.notes();
    assert_eq!(notes.len(), 1);
    assert_eq!(
        (notes[0].code.as_str(), notes[0].detail.as_deref()),
        ("page-refused", Some("note"))
    );
}

#[test]
fn unknown_group_not_shown_but_said() {
    let s = filled();
    script(&s)
        .rows
        .push(row("t9", "gone", &[("title", &["Lost"])]));
    let b = board(&s);
    let all: Vec<_> = b
        .columns()
        .iter()
        .flat_map(|c| c.cards.iter().map(|k| k.key.clone()))
        .collect();
    assert!(!all.contains(&"t9".to_owned()));
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
    let b = RecordsBoard::new(w, bind, RecordsTexts::default());
    let n: usize = b.columns().iter().map(|c| c.cards.len()).sum();
    assert_eq!(n, 2);
    assert!(b.notes().iter().any(|x| x.code == "truncated"));
    assert_eq!(script(&s).finds, 2);
}

#[test]
fn failure_keeps_last_state_and_says_so() {
    let s = filled();
    let w = start(&s, binding());
    let b = RecordsBoard::new(w.clone(), binding(), RecordsTexts::default());
    assert_eq!(b.notes().len(), 0);
    let v = w.version();
    script(&s).fail = Some("unavailable".into());
    w.reload();
    until(|| w.version() > v, "stale");
    assert!(matches!(b.status(), Status::Stale(f) if f.code == "unavailable"));
    assert_eq!(b.columns()[0].cards.len(), 2, "the last state stays");
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
    let rows = RecordsRows::new(w, binding(), RecordsTexts::default());
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
    let b = RecordsBoard::new(w.clone(), binding(), RecordsTexts::default());
    let (gen0, v) = (b.generation(), w.version());
    {
        let mut g = script(&s);
        g.generation = 8;
        g.rows.push(row("t4", "doing", &[("title", &["New"])]));
    }
    w.poll();
    until(|| w.version() > v, "poll");
    assert_ne!(
        b.generation(),
        gen0,
        "the board sees that something changed"
    );
    assert_eq!(b.columns()[1].cards.len(), 2);
    assert_eq!(w.snapshot().generation, 8);
}
