//! What the review of the records adapter found alive, and the rules behind it: a lock mark in the list, the page cursor, a reload
//! that forces, the fields asked once, the notes of a list alone,
//! the reason as one type.
//!
//! Mutants (each makes the named test fail): X8 the list drops the lock mark -> `a_locked_record_is_marked_in_the_list`; X20 the
//! page cursor is ignored -> `every_page_is_read_once_in_order`; X22 a reload does not force -> `a_reload_reads_again_with_the_same_generation`;
//! X19 a field is asked twice -> `every_field_is_asked_once`; `RecordsRows::notes`
//! empty -> the `a_list_alone_says_*` tests.
//! Pattern: none; tests.
#![allow(clippy::many_single_char_names)]
mod records_common;

use fs_components_fields::SlotReason;
use fs_components_widgets::list_view::RowSource;
use fs_components_workbenches::records::{
    FieldValue, HiddenReason, LockKind, RecordLock, RecordsRows, Status,
};
use records_common::*;

fn rows_over(s: &Shared, b: fs_components_workbenches::records::RecordsBinding) -> RecordsRows {
    RecordsRows::new(start(s, b.clone()), b, judge())
}

#[test]
fn a_locked_record_is_marked_in_the_list() {
    let s = filled();
    {
        let mut g = script(&s);
        g.rows[0].lock = Some(RecordLock {
            kind: LockKind::Admin,
            reason: Some("Audit".into()),
        });
        g.rows[2].lock = Some(RecordLock {
            kind: LockKind::Claimed,
            reason: None,
        });
    }
    let rows = rows_over(&s, binding()).rows(0, 5).unwrap();
    let marks: Vec<_> = rows
        .iter()
        .map(|r| (r.key.as_str(), r.mark.as_deref()))
        .collect();
    assert_eq!(
        marks,
        [("t1", Some("Audit")), ("t2", None), ("t3", Some("claimed"))]
    );
}

fn many(n: usize) -> Shared {
    let s = filled();
    {
        let mut g = script(&s);
        g.rows = (1..=n)
            .map(|i| {
                row(
                    &format!("t{i}"),
                    "todo",
                    &[("title", &[&format!("Task {i}")])],
                )
            })
            .collect();
        g.page_len = 2;
    }
    s
}

#[test]
fn every_page_is_read_once_in_order() {
    let s = many(5);
    let w = start(&s, binding());
    let ids: Vec<_> = w.snapshot().rows.iter().map(|r| r.id.clone()).collect();
    assert_eq!(
        ids,
        ["t1", "t2", "t3", "t4", "t5"],
        "five rows, each once, in page order"
    );
    let g = script(&s);
    assert_eq!(g.finds, 3, "three pages of two");
    let after: Vec<_> = g.queries.iter().map(|q| q.after.clone()).collect();
    assert_eq!(
        after,
        [None, Some("2".to_owned()), Some("4".to_owned())],
        "each page asks after the cursor of the page before"
    );
    drop(g);
    let keys: Vec<_> = RecordsRows::new(w, binding(), judge())
        .rows(0, 10)
        .unwrap()
        .into_iter()
        .map(|r| r.key)
        .collect();
    assert_eq!(keys, ["t1", "t2", "t3", "t4", "t5"]);
}

#[test]
fn a_reload_reads_again_with_the_same_generation() {
    let s = filled();
    let w = start(&s, binding());
    let (finds, v) = (script(&s).finds, w.version());
    script(&s)
        .rows
        .push(row("t4", "doing", &[("title", &["New, same generation"])]));
    w.poll();
    until(|| w.version() > v, "poll");
    assert_eq!(
        script(&s).finds,
        finds,
        "a poll with the same generation reads nothing"
    );
    assert_eq!(w.snapshot().rows.len(), 3);
    let v = w.version();
    w.reload();
    until(|| w.version() > v, "reload");
    assert_eq!(
        script(&s).finds,
        finds + 1,
        "a reload reads although the generation is the same"
    );
    assert_eq!(w.snapshot().rows.len(), 4);
}

#[test]
fn every_field_is_asked_once() {
    let b = binding();
    // the binding names `title` and `due` in two places each: each field once, in the order of first use
    let b = b.cell(fs_components_workbenches::records::Slot::new(
        "title",
        "Again",
        "field-integer",
    ));
    assert_eq!(b.fields(), ["title", "due"]);
    let s = filled();
    let _ = start(&s, binding());
    assert_eq!(script(&s).queries[0].fields, binding().fields());
}

#[test]
fn a_list_alone_says_a_source_that_does_not_answer() {
    let s = filled();
    let w = start(&s, binding());
    let rows = RecordsRows::new(w.clone(), binding(), judge());
    assert_eq!(rows.notes(), []);
    let v = w.version();
    script(&s).fail = Some("unavailable".into());
    w.reload();
    until(|| w.version() > v, "stale");
    assert!(matches!(w.status(), Status::Stale(_)));
    assert_eq!(rows.count(), 3, "the old rows stay");
    let n = rows.notes();
    assert_eq!(
        n.iter()
            .map(|x| (x.code.as_str(), x.detail.as_deref()))
            .collect::<Vec<_>>(),
        [("stale", Some("unavailable"))]
    );
}

#[test]
fn a_list_alone_says_a_cut_off_list_a_refused_page_and_rows_in_no_group() {
    let s = many(4);
    let mut b = binding();
    b.max_pages = 1;
    let notes = rows_over(&s, b).notes();
    assert_eq!(notes.len(), 1);
    assert_eq!(
        (notes[0].code.as_str(), notes[0].detail.as_deref()),
        ("truncated", Some("2"))
    );

    let s = filled();
    script(&s)
        .rows
        .push(row("t9", "gone", &[("title", &["Lost"])]));
    let notes = rows_over(&s, binding()).notes();
    assert_eq!(
        (notes[0].code.as_str(), notes[0].detail.as_deref()),
        ("group-unknown", Some("1"))
    );

    let s = filled();
    {
        let mut g = script(&s);
        let mut r = hide(g.rows[2].clone(), "note", HiddenReason::NoRight);
        r.values.push(FieldValue {
            field: "note".into(),
            values: vec!["leak".into()],
        });
        g.rows[2] = r;
    }
    let notes = rows_over(&s, binding()).notes();
    assert_eq!(notes[0].code, "bind-error-page-refused");
}

#[test]
fn a_reason_says_its_sentence_or_else_its_code() {
    let r = SlotReason::new("a", "b");
    assert_eq!((r.code.as_str(), r.said()), ("a", "b"));
    assert_eq!(
        SlotReason::new("only-code", "").said(),
        "only-code",
        "no sentence: the code is said, never an empty line"
    );
}
