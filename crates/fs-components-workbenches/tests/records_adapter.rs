//! The records adapter: board (K4), facts (K5), frame slots (K6), list (K8), the thread boundary, and every way the data can be
//! wrong or missing (nothing is left out without a word).
//!
//! Mutants (each makes the named test fail): M1 columns ignore the group of a row -> `columns_follow_groups`; M2 a fact for a hidden
//! field shows the value -> `hidden_shows_reason_never_value`; M3 more than four facts -> `at_most_four_facts`; M4 a leaking page is
//! accepted -> `leaking_page_is_refused_and_said`; M5 a row of an unknown group is dropped without a note -> `unknown_group_not_shown_but_said`;
//! M6 the cut-off at `max_pages` is silent -> `cut_off_is_said`; M7 a failed call clears the snapshot or says nothing ->
//! `failure_keeps_last_state_and_says_so`; M8 an unchanged generation reads again -> `unchanged_generation_costs_one_call`;
//! M9 a changed generation is ignored -> `changed_generation_reloads`; M10 `can_drop` allows -> `reading_only_refuses_with_reason`;
//! M11 the menu offers what `may` does not list -> `menu_offers_only_what_may_lists`; M12 a lock is not reported -> `lock_is_reported`.
mod records_common;

use fs_components_widgets::list_view::RowSource;
use fs_components_workbenches::board::{
    Answer, BoardEvents, BoardSource, ContextTarget, DetailContent, MoveRequest, NewCard, SlotValue,
};
use fs_components_workbenches::records::{
    HiddenReason, LockKind, RecordLock, RecordsBoard, RecordsRows, RecordsTexts, Status, TEXTS,
};
use records_common::*;

fn board(s: &Shared) -> RecordsBoard {
    RecordsBoard::new(start(s, binding()), binding(), RecordsTexts::default())
}

#[test]
fn adapters_cross_the_thread_boundary() {
    fn is_sync<T: Send + Sync>() {}
    is_sync::<RecordsBoard>();
    is_sync::<RecordsRows>();
    is_sync::<fs_components_workbenches::records::Worker>();
}

#[test]
fn texts_have_both_languages() {
    assert!(fs_components_mf2::check_pairs(TEXTS).is_empty());
}

#[test]
fn columns_follow_groups() {
    let b = board(&filled());
    let cols = b.columns();
    assert_eq!(cols.len(), 2);
    assert_eq!(
        (cols[0].key.as_str(), cols[0].title.as_str()),
        ("todo", "To do")
    );
    assert_eq!(cols[1].title, "doing", "no title: the key is shown");
    assert_eq!(cols[1].limit, Some(2));
    let keys = |i: usize| {
        cols[i]
            .cards
            .iter()
            .map(|c| c.key.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(keys(0), ["t1", "t3"]);
    assert_eq!(keys(1), ["t2"]);
    assert_eq!(cols[0].cards[0].title, "Write");
    assert_eq!(cols[0].cards[0].subtitle.as_deref(), Some("soon"));
}

#[test]
fn facts_are_compact_slots_and_empty_ones_take_no_room() {
    let b = board(&filled());
    let cols = b.columns();
    let facts = |c: usize, i: usize| {
        cols[c].cards[i]
            .facts
            .iter()
            .map(|f| (f.field.clone(), f.text.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        facts(0, 0),
        [
            ("due".into(), "2026-11-20".into()),
            ("priority".into(), "2".into())
        ]
    );
    assert!(facts(1, 0).is_empty());
}

#[test]
fn at_most_four_facts() {
    let s = filled();
    let mut bind = binding();
    for f in ["a", "b", "c", "d", "e"] {
        bind = bind.slot(fs_components_workbenches::records::Slot::new(
            f,
            f,
            fs_components_workbenches::records::Role::Compact,
        ));
        script(&s).rows[0]
            .values
            .push(fs_components_workbenches::records::FieldValue {
                field: f.into(),
                values: vec![f.into()],
            });
    }
    let b = RecordsBoard::new(start(&s, bind.clone()), bind, RecordsTexts::default());
    assert_eq!(b.columns()[0].cards[0].facts.len(), 4);
}

#[test]
fn hidden_shows_reason_never_value() {
    let s = filled();
    {
        let mut g = script(&s);
        let r = hide(g.rows[0].clone(), "due", HiddenReason::Sensitive);
        let mut r = r;
        r.values.retain(|v| v.field != "due"); // a hidden field carries no value
        g.rows[0] = r;
    }
    let b = board(&s);
    let due = &b.columns()[0].cards[0].facts[0];
    assert_eq!(due.field, "due");
    assert_eq!(due.text, RecordsTexts::default().hidden_sensitive);
    assert!(!due.text.contains("2026"));
}

#[test]
fn frame_slots_show_value_unset_or_reason() {
    let s = filled();
    {
        let mut g = script(&s);
        let r = hide(g.rows[0].clone(), "secret", HiddenReason::NoRight);
        g.rows[0] = r;
    }
    let b = board(&s);
    let f = b.frame("t1").expect("frame");
    assert_eq!(f.path, "To do");
    assert_eq!(f.title, "Write");
    let by = |name: &str| {
        f.slots
            .iter()
            .find(|x| x.field == name)
            .unwrap()
            .value
            .clone()
    };
    assert_eq!(by("description"), SlotValue::Text("Long text".into()));
    assert!(matches!(by("secret"), SlotValue::Hidden(r) if r.code == "hidden-no-right"));
    let g = b.frame("t3").unwrap();
    assert_eq!(
        g.slots
            .iter()
            .find(|x| x.field == "description")
            .unwrap()
            .value,
        SlotValue::Unset
    );
    assert!(b.frame("nobody").is_none());
}

#[test]
fn list_rows_are_cells_in_order() {
    let s = filled();
    let w = start(&s, binding());
    let rows = RecordsRows::new(w, binding(), RecordsTexts::default());
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
    assert!(b.notes().is_empty());
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
    assert!(b.notes().is_empty());
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

#[test]
fn reading_only_refuses_with_reason() {
    let b = board(&filled());
    let reason = b.can_drop("t1", "doing").unwrap_err();
    assert_eq!(reason.code, "write-later");
    let mv = MoveRequest::new("t1", "todo", "doing", 0, None);
    assert!(matches!(b.moved(&mv), Answer::Refused(r) if r.code == "write-later"));
    assert!(matches!(
        b.new_card(&NewCard::new("todo", "x", false)),
        Answer::Refused(_)
    ));
    assert!(!b.supports().columns_edit);
}

#[test]
fn menu_offers_only_what_may_lists() {
    let s = filled();
    script(&s).rows[0].may = vec!["claim".into(), "done".into()];
    let b = board(&s);
    assert_eq!(
        b.context(&ContextTarget::Card("t1".into()), (0.0, 0.0)),
        ["board.open", "tasks.claim", "tasks.done"]
    );
    assert_eq!(
        b.context(&ContextTarget::Card("t3".into()), (0.0, 0.0)),
        ["board.open"]
    );
    assert!(b.context(&ContextTarget::Board, (0.0, 0.0)).is_empty());
}

#[test]
fn lock_is_reported() {
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
    let b = board(&s);
    let r = b.locked("t1").unwrap();
    assert_eq!(
        (r.code.as_str(), r.text.as_str()),
        ("locked-admin", "Audit")
    );
    let r = b.locked("t3").unwrap();
    assert_eq!(r.text, RecordsTexts::default().lock_claimed);
    assert!(b.locked("t2").is_none());
}
