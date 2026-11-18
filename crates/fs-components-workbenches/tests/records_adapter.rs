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
    Answer, BoardEvents, BoardSource, ContextTarget, DEFAULT_FACTS, DetailContent, Limit,
    MoveRequest, NewCard, SlotState, card_meta,
};
use fs_components_workbenches::records::{
    FieldValue, HiddenReason, LockKind, RecordLock, RecordsBoard, RecordsRows, Role, Slot,
};
use records_common::*;

fn board(s: &Shared) -> RecordsBoard {
    RecordsBoard::new(start(s, binding()), binding(), judge())
}

#[test]
fn adapters_cross_the_thread_boundary() {
    fn is_sync<T: Send + Sync>() {}
    is_sync::<RecordsBoard>();
    is_sync::<RecordsRows>();
    is_sync::<fs_components_workbenches::records::Worker>();
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
fn facts_are_slots_of_the_card_and_empty_ones_take_no_room() {
    let b = board(&filled());
    let cols = b.columns();
    let facts = |c: usize, i: usize| {
        cols[c].cards[i]
            .facts()
            .iter()
            .map(|f| (f.block.clone(), f.label.clone(), f.line()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        facts(0, 0),
        [
            ("field-integer".into(), "Due".into(), "2026-11-20".into()),
            ("field-integer".into(), "Priority".into(), "2".into())
        ]
    );
    // no value: the slot is `Unset`, and the line of the card leaves it out (no empty piece between the dots)
    assert!(
        cols[1].cards[0]
            .facts()
            .iter()
            .all(|f| f.state == SlotState::Unset)
    );
    assert_eq!(card_meta(&cols[1].cards[0]), Vec::<String>::new());
    assert_eq!(card_meta(&cols[0].cards[0]), ["soon", "2026-11-20", "2"]);
}

#[test]
fn a_binding_with_more_facts_than_the_card_takes_is_said_not_cut_silently() {
    let s = filled();
    let mut bind = binding();
    for f in ["a", "b", "c", "d", "e"] {
        bind = bind.slot(Slot::new(f, f, Role::Compact, "field-integer"));
        script(&s).rows[0].values.push(FieldValue {
            field: f.into(),
            values: vec![f.into()],
        });
    }
    let b = RecordsBoard::new(start(&s, bind.clone()), bind, judge());
    assert_eq!(b.columns()[0].cards[0].facts().len(), DEFAULT_FACTS.get());
    let n = b.notes();
    assert_eq!(n.len(), 1);
    assert_eq!(
        (n[0].code.as_str(), n[0].detail.as_deref()),
        ("too-many-facts", Some("7"))
    );
}

#[test]
fn the_limits_come_from_the_binding() {
    let s = filled();
    let bind = binding().limits(Limit::of(1), Limit::of(1));
    let b = RecordsBoard::new(start(&s, bind.clone()), bind, judge());
    assert_eq!(b.columns()[0].cards[0].facts().len(), 1);
    assert_eq!(b.frame("t1").unwrap().elements().len(), 1);
    let codes: Vec<_> = b.notes().into_iter().map(|n| n.code).collect();
    assert_eq!(codes, ["too-many-facts", "too-many-elements"]);
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
    let b = board(&s);
    let cols = b.columns();
    let due = &cols[0].cards[0].facts()[0];
    assert!(matches!(&due.state, SlotState::Hidden(r) if r.code == "bind-error-field-hidden"));
    assert_eq!(due.value, "", "the slot carries no value");
    assert_eq!(due.line(), "hidden: sensitive");
}

#[test]
fn frame_elements_show_value_unset_or_the_reason_of_the_plan() {
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
    let by = |fr: &fs_components_workbenches::board::DetailFrame, name: &str| {
        fr.elements()
            .iter()
            .find(|(n, _)| n == name)
            .unwrap()
            .1
            .clone()
    };
    assert_eq!(by(&f, "Description").line(), "Long text");
    assert_eq!(by(&f, "Description").state, SlotState::Value);
    assert!(matches!(by(&f, "Secret").state, SlotState::Hidden(r) if r.text == "hidden: no-right"));
    let g = b.frame("t3").unwrap();
    assert_eq!(by(&g, "Description").state, SlotState::Unset);
    assert!(b.frame("nobody").is_none());
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
fn reading_only_refuses_with_reason() {
    let b = board(&filled());
    let reason = b.can_drop("t1", "doing").unwrap_err();
    assert_eq!(reason.code, "bind-error-write-later");
    let mv = MoveRequest::new("t1", "todo", "doing", 0, None);
    assert!(matches!(b.moved(&mv), Answer::Refused(r) if r.code == "bind-error-write-later"));
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
    assert_eq!(r.text, "claimed");
    assert!(b.locked("t2").is_none());
}
