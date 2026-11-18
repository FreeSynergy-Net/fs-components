//! The records adapter: board (K4), facts (K5), frame slots (K6), list (K8), the thread boundary, and every way the data can be
//! wrong or missing (nothing is left out without a word).
//!
//! Mutants (each makes the named test fail): M1 columns ignore the group of a row -> `columns_follow_groups`; M2 a fact for a hidden
//! field shows the value -> `hidden_shows_reason_never_value`; M3 more than four facts -> `at_most_four_facts`; M10 `can_drop` allows -> `reading_only_refuses_with_reason`;
//! M11 the menu offers what `may` does not list -> `menu_offers_only_what_may_lists`; M12 a lock is not reported -> `lock_is_reported`.
#![allow(clippy::many_single_char_names)] // short names for the board, worker and script of each case
mod records_common;

use fs_components_widgets::list_view::RowSource;
use fs_components_workbenches::board::{
    Answer, BoardEvents, BoardSource, ContextTarget, DetailContent, MoveRequest, NewCard, SlotValue,
};
use fs_components_workbenches::records::{
    HiddenReason, LockKind, RecordLock, RecordsBoard, RecordsRows, RecordsTexts, TEXTS,
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
    let unpaired = fs_components_mf2::check_pairs(TEXTS);
    assert_eq!(unpaired.len(), 0, "{unpaired:?}");
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
    assert_eq!(facts(1, 0).len(), 0);
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
    assert_eq!(b.context(&ContextTarget::Board, (0.0, 0.0)).len(), 0);
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
