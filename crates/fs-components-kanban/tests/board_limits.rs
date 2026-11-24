//! Rule 57 (A6): how many field values the detail frame takes is a LIMIT handed in as a parameter (default 32); the workbench reads
//! no settings. 32 and not 16, because the tasks type already has 17 values (the title and 16 fields) and the frame scrolls.
//!
//! Mutants: the default is not 32 -> `the_default_fits_the_tasks_type`; `DetailFrame::limited` ignores its limit ->
//! `a_frame_takes_what_its_limit_says`; the key or the words are wrong -> `the_setting_is_data_with_words_in_both_languages`.
//!
//! Pattern: none; tests.
#![allow(clippy::pedantic)]

use fs_components_kanban::board::{
    DEFAULT_ELEMENTS, DetailFrame, ELEMENTS_SETTING, FieldSlot, Limit, TEXTS,
};
use fs_components_mf2::check_pairs;

fn slot(i: usize) -> FieldSlot {
    FieldSlot::new("field-email", "label.email", &format!("a{i}@example.org"))
}

#[test]
fn the_default_fits_the_tasks_type() {
    assert_eq!(DEFAULT_ELEMENTS.get(), 32);
    assert_eq!(ELEMENTS_SETTING.key, "fs.workbenches.board-detail-elements");
    assert_eq!(ELEMENTS_SETTING.default, DEFAULT_ELEMENTS);
    assert_eq!(
        ELEMENTS_SETTING.group, "board",
        "the group the settings window shows it in"
    );
    // the tasks type: a title and 16 fields = 17 values, all of them in the frame
    let mut frame = DetailFrame::new("T");
    for i in 0..17 {
        frame = frame
            .element("Field", slot(i))
            .expect("room for the tasks type");
    }
    assert_eq!(frame.elements().len(), 17);
    assert_eq!(frame.max_elements(), 32);
}

#[test]
fn the_setting_is_data_with_words_in_both_languages() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let id = ELEMENTS_SETTING.key.replace('.', "-");
    assert_eq!(ELEMENTS_SETTING.label_key, format!("setting-label-{id}"));
    assert_eq!(ELEMENTS_SETTING.about_key, format!("setting-about-{id}"));
    for id in [ELEMENTS_SETTING.label_key, ELEMENTS_SETTING.about_key] {
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
fn a_frame_takes_what_its_limit_says() {
    let mut frame = DetailFrame::limited("T", Limit::new(3).unwrap());
    for i in 0..3 {
        frame = frame.element("Mail", slot(i)).expect("room");
    }
    assert_eq!(frame.max_elements(), 3);
    assert_eq!(frame.clone().element("Mail", slot(9)).unwrap_err(), slot(9));
    assert_eq!(frame.elements().len(), 3);
    let big = DetailFrame::limited("T", Limit::new(100).unwrap());
    assert_eq!(
        big.max_elements(),
        100,
        "raised by the host: no limit of ours below the ceiling"
    );
}
