//! Rule 57 (A6): how many field values the detail frame takes is a SETTING of the instance
//! (`fs.workbenches.board-detail-elements`, default 32), not a constant. 32 and not 16, because the tasks type already has 17 values
//! (the title and 16 fields) and the frame scrolls: more values cost no layout.
//!
//! Mutants: the default is not 32 -> `the_default_fits_the_tasks_type`; `elements_limit` ignores the saved value ->
//! `a_saved_value_is_read`; `DetailFrame::limited` ignores its limit -> `a_frame_takes_what_its_limit_says`.
//!
//! Pattern: none; tests.
#![allow(clippy::pedantic)]

use std::sync::Arc;

use fs_components_mf2::check_pairs;
use fs_components_workbenches::board::{
    BoardSettings, DEFAULT_ELEMENTS, DetailFrame, FieldSlot, TEXTS, elements_limit,
};
use fs_settings::{
    Catalog, Declares, Kind, MemoryStore, NoSystem, ProgramName, Settings, Target, Value,
};

fn settings() -> Settings {
    Settings::open(
        ProgramName::new("tasks").unwrap(),
        Catalog::of(BoardSettings.declarations().unwrap()).unwrap(),
        Arc::new(MemoryStore::new()),
        Arc::new(NoSystem),
    )
}

fn slot(i: usize) -> FieldSlot {
    FieldSlot::new("field-email", "label.email", &format!("a{i}@example.org"))
}

#[test]
fn the_default_fits_the_tasks_type() {
    assert_eq!(DEFAULT_ELEMENTS.get(), 32);
    let d = BoardSettings.declarations().unwrap();
    assert_eq!(d[0].key.as_str(), "fs.workbenches.board-detail-elements");
    assert_eq!(d[0].default, Value::Whole(32));
    assert_eq!(
        d[0].kind,
        Kind::Whole {
            min: 1,
            max: i64::from(i32::MAX)
        }
    );
    // the tasks type: a title and 16 fields = 17 values, all of them in the frame
    let mut frame = DetailFrame::new("T");
    for i in 0..17 {
        frame = frame
            .element("Field", slot(i))
            .expect("room for the tasks type");
    }
    assert_eq!(frame.elements().len(), 17);
}

#[test]
fn the_words_of_the_setting_are_in_both_languages() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let d = &BoardSettings.declarations().unwrap()[0];
    for id in [d.label_key(), d.about_key()] {
        for s in TEXTS {
            assert!(
                s.text.lines().any(|l| l.starts_with(&format!("{id} ="))),
                "{id} {}",
                s.language
            );
        }
    }
}

#[test]
fn a_saved_value_is_read() {
    let s = settings();
    assert_eq!(elements_limit(&s), DEFAULT_ELEMENTS);
    let key = BoardSettings::elements_key().unwrap();
    s.save(&key, Some(&Value::Whole(5)), Target::ThisProgram)
        .unwrap();
    assert_eq!(elements_limit(&s).get(), 5);
    s.save(&key, Some(&Value::Whole(100)), Target::ThisProgram)
        .unwrap();
    assert_eq!(
        elements_limit(&s).get(),
        100,
        "no ceiling of ours below the kind's"
    );
    s.save(&key, None, Target::ThisProgram).unwrap();
    assert_eq!(elements_limit(&s), DEFAULT_ELEMENTS);
}

#[test]
fn a_frame_takes_what_its_limit_says() {
    let s = settings();
    let key = BoardSettings::elements_key().unwrap();
    s.save(&key, Some(&Value::Whole(3)), Target::ThisProgram)
        .unwrap();
    let mut frame = DetailFrame::limited("T", elements_limit(&s));
    for i in 0..3 {
        frame = frame.element("Mail", slot(i)).expect("room");
    }
    assert_eq!(frame.max_elements(), 3);
    assert_eq!(frame.clone().element("Mail", slot(9)).unwrap_err(), slot(9));
    assert_eq!(frame.elements().len(), 3);
    assert_eq!(DetailFrame::new("T").max_elements(), 32);
}
