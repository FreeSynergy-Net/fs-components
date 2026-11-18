//! The adapter shows exactly what the plan's rules (the `Judge`) say, and decides nothing itself: hidden, missing, refused, locked,
//! the commands, the words for "not possible yet" and "does not answer" are all the judge's. Judges here say things the test
//! double of the other files never says, so an adapter that decided for itself would show the wrong thing.
//!
//! Mutants (each makes the named test fail): J1 the adapter shows the value although the judge says hidden ->
//! `a_judge_that_hides_everything_shows_no_value_anywhere`; J2 the adapter ignores a refused page -> `a_judge_that_refuses_the_page_shows_no_row`;
//! J3 `Missing` drawn as a value -> `a_judge_that_says_missing_shows_its_reason`; J4 the menu adds commands of its own ->
//! `the_menu_is_what_the_judge_lists`; J5 the adapter words the lock or the refusal itself -> `the_words_are_the_judges`.
#![allow(clippy::many_single_char_names)]
mod records_common;

use std::sync::Arc;

use fs_components_widgets::list_view::RowSource;
use fs_components_workbenches::board::{
    Answer, BoardEvents, BoardSource, ContextTarget, DetailContent, MoveRequest, SlotReason,
    SlotState, card_meta,
};
use fs_components_workbenches::records::{Judge, RecordRow, RecordsBoard, RecordsRows, Shown};
use records_common::*;

/// Says `shown` for every field and refuses/locks/lists as told.
struct Says {
    shown: fn(&str) -> Shown,
    refuse: bool,
}

impl Judge for Says {
    fn admit(&self, _rows: &[RecordRow]) -> Result<(), SlotReason> {
        if self.refuse {
            Err(SlotReason::new("x-refused", "the plan refuses this page"))
        } else {
            Ok(())
        }
    }
    fn shown(&self, _row: &RecordRow, field: &str) -> Shown {
        (self.shown)(field)
    }
    fn locked(&self, _row: &RecordRow) -> Option<SlotReason> {
        Some(SlotReason::new("x-locked", "the plan locks every record"))
    }
    fn commands(&self, _row: &RecordRow) -> Vec<String> {
        vec!["only.this".into()]
    }
    fn write_later(&self) -> SlotReason {
        SlotReason::new("x-later", "the plan says: later")
    }
    fn stale(&self, code: &str) -> SlotReason {
        SlotReason::new("x-stale", &format!("the plan says: no answer ({code})"))
    }
}

fn hidden(_: &str) -> Shown {
    Shown::Hidden(SlotReason::new("x-hidden", "no way"))
}

fn missing(_: &str) -> Shown {
    Shown::Missing(SlotReason::new("x-missing", "block not built"))
}

fn board_with(j: Says) -> RecordsBoard {
    let s = filled();
    let j: Arc<dyn Judge> = Arc::new(j);
    RecordsBoard::new(start_with(&s, binding(), Arc::clone(&j)), binding(), j)
}

#[test]
fn a_judge_that_hides_everything_shows_no_value_anywhere() {
    let b = board_with(Says {
        shown: hidden,
        refuse: false,
    });
    let cols = b.columns();
    let card = &cols[0].cards[0];
    assert_eq!(card.title, "no way");
    assert_eq!(card.subtitle.as_deref(), Some("no way"));
    assert!(card.facts().iter().all(
        |f| matches!(&f.state, SlotState::Hidden(r) if r.code == "x-hidden") && f.value.is_empty()
    ));
    assert!(card_meta(card).iter().all(|p| p == "no way"));
    let frame = b.frame("t1").unwrap();
    assert!(
        frame
            .elements()
            .iter()
            .all(|(_, f)| matches!(f.state, SlotState::Hidden(_)) && f.value.is_empty())
    );
    for text in cols.iter().flat_map(|c| &c.cards).flat_map(card_meta) {
        assert!(
            !text.contains("2026") && !text.contains("Long text"),
            "{text}"
        );
    }
    let w = start_with(
        &filled(),
        binding(),
        Arc::new(Says {
            shown: hidden,
            refuse: false,
        }),
    );
    let rows = RecordsRows::new(
        w,
        binding(),
        Arc::new(Says {
            shown: hidden,
            refuse: false,
        }),
    );
    assert_eq!(rows.rows(0, 1).unwrap()[0].cells, ["no way", "no way"]);
}

#[test]
fn a_judge_that_refuses_the_page_shows_no_row() {
    let b = board_with(Says {
        shown: hidden,
        refuse: true,
    });
    assert!(b.columns().iter().all(|c| c.cards.is_empty()));
    let n = b.notes();
    assert_eq!(
        (n[0].code.as_str(), n[0].detail.as_deref()),
        ("x-refused", Some("the plan refuses this page"))
    );
}

#[test]
fn a_judge_that_says_missing_shows_its_reason() {
    let b = board_with(Says {
        shown: missing,
        refuse: false,
    });
    let f = b.frame("t1").unwrap();
    assert!(f.elements().iter().all(
        |(_, s)| matches!(&s.state, SlotState::Missing(r) if r.code == "x-missing")
            && s.line() == "block not built"
    ));
}

#[test]
fn the_menu_is_what_the_judge_lists() {
    let s = filled();
    script(&s).rows[0].may = vec!["claim".into(), "done".into()]; // rights of the provider; the judge decides what is offered
    let j: Arc<dyn Judge> = Arc::new(Says {
        shown: hidden,
        refuse: false,
    });
    let b = RecordsBoard::new(start_with(&s, binding(), Arc::clone(&j)), binding(), j);
    assert_eq!(
        b.context(&ContextTarget::Card("t1".into()), (0.0, 0.0)),
        ["board.open", "only.this"]
    );
}

#[test]
fn the_words_are_the_judges() {
    let b = board_with(Says {
        shown: hidden,
        refuse: false,
    });
    let r = b.locked("t1").unwrap();
    assert_eq!(
        (r.code.as_str(), r.text.as_str()),
        ("x-locked", "the plan locks every record")
    );
    let d = b.can_drop("t1", "doing").unwrap_err();
    assert_eq!(
        (d.code.as_str(), d.text.as_str()),
        ("x-later", "the plan says: later")
    );
    let mv = MoveRequest::new("t1", "todo", "doing", 0, None);
    assert!(matches!(b.moved(&mv), Answer::Refused(r) if r.text == "the plan says: later"));
    // a source that never answered: the list says it in the judge's words
    let s = filled();
    script(&s).fail = Some("unavailable".into());
    let j: Arc<dyn Judge> = Arc::new(Says {
        shown: hidden,
        refuse: false,
    });
    let rows = RecordsRows::new(start_with(&s, binding(), Arc::clone(&j)), binding(), j);
    assert_eq!(
        rows.rows(0, 5).unwrap_err(),
        "the plan says: no answer (unavailable) (x-stale)"
    );
}
