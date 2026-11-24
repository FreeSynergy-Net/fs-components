//! The adapter shows exactly what the plan's rules (the `Judge`) say, and decides nothing itself: hidden, missing, refused, locked,
//! the words for "does not answer" are all the judge's. Judges here say things the test
//! double of the other files never says, so an adapter that decided for itself would show the wrong thing.
//!
//! Mutants (each makes the named test fail): J1 the adapter shows the value although the judge says hidden ->
//! `a_judge_that_hides_everything_shows_no_value_anywhere`; J2 the adapter ignores a refused page -> `a_judge_that_refuses_the_page_shows_no_row`;
//! J3 `Missing` drawn as a value -> `a_judge_that_says_missing_shows_its_reason`;
//! J5 the adapter words the lock itself -> `the_words_are_the_judges`.
#![allow(clippy::many_single_char_names)]
mod records_common;

use std::sync::Arc;

use fs_components_fields::SlotReason;
use fs_components_widgets::list_view::RowSource;
use fs_components_workbenches::records::{Judge, RecordRow, RecordsRows, Shown};
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
    fn stale(&self, code: &str) -> SlotReason {
        SlotReason::new("x-stale", format!("the plan says: no answer ({code})"))
    }
}

fn hidden(_: &str) -> Shown {
    Shown::Hidden(SlotReason::new("x-hidden", "no way"))
}

fn missing(_: &str) -> Shown {
    Shown::Missing(SlotReason::new("x-missing", "block not built"))
}

fn list_with(j: Says) -> RecordsRows {
    let s = filled();
    let j: Arc<dyn Judge> = Arc::new(j);
    RecordsRows::new(start_with(&s, binding(), Arc::clone(&j)), binding(), j)
}

#[test]
fn a_judge_that_hides_everything_shows_no_value_anywhere() {
    let rows = list_with(Says {
        shown: hidden,
        refuse: false,
    });
    assert_eq!(rows.rows(0, 1).unwrap()[0].cells, ["no way", "no way"]);
    for r in rows.rows(0, 5).unwrap() {
        for text in &r.cells {
            assert!(
                !text.contains("2026") && !text.contains("Long text"),
                "{text}"
            );
        }
    }
}

#[test]
fn a_judge_that_refuses_the_page_shows_no_row() {
    let rows = list_with(Says {
        shown: hidden,
        refuse: true,
    });
    assert_eq!(rows.count(), 0);
    let n = rows.notes();
    assert_eq!(
        (n[0].code.as_str(), n[0].detail.as_deref()),
        ("x-refused", Some("the plan refuses this page"))
    );
}

#[test]
fn a_judge_that_says_missing_shows_its_reason() {
    let rows = list_with(Says {
        shown: missing,
        refuse: false,
    });
    assert!(
        rows.rows(0, 5)
            .unwrap()
            .iter()
            .all(|r| r.cells.iter().all(|c| c == "block not built"))
    );
}

#[test]
fn the_words_are_the_judges() {
    // a locked record carries the judge's sentence as the mark
    let rows = list_with(Says {
        shown: hidden,
        refuse: false,
    });
    assert_eq!(
        rows.rows(0, 1).unwrap()[0].mark.as_deref(),
        Some("the plan locks every record")
    );
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
