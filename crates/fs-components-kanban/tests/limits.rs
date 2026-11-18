//! Rule 57 (A5): how many facts a card takes is a SETTING of the instance (`fs.kanban.card-facts`, default 4), not a constant:
//! the declaration, its words, the effective value, and a card that cannot be filled past its limit through a public field.
//!
//! Mutants: the default is not 4 -> `the_default_is_four_and_declared`; `facts_limit` ignores the saved value -> `a_saved_value_is_read`;
//! `Card::limited` ignores its limit -> `a_card_takes_what_its_limit_says`; a value of 0 is accepted -> `zero_is_refused_by_the_kind`.
//!
//! Pattern: none; tests.
#![allow(clippy::pedantic)]

use std::sync::Arc;

use fs_components_kanban::{Card, DEFAULT_FACTS, FieldSlot, KanbanSettings, facts_limit};
use fs_components_mf2::{Source, check_pairs};
use fs_settings::{
    Catalog, Declares, Kind, MemoryStore, NoSystem, ProgramName, SettingCode, Settings, Target,
    Value,
};

fn settings(declared: bool) -> Settings {
    let catalog = if declared {
        Catalog::of(KanbanSettings.declarations().unwrap()).unwrap()
    } else {
        Catalog::new()
    };
    Settings::open(
        ProgramName::new("tasks").unwrap(),
        catalog,
        Arc::new(MemoryStore::new()),
        Arc::new(NoSystem),
    )
}

fn slot(i: usize) -> FieldSlot {
    FieldSlot::new("field-email", "label.email", &format!("a{i}@example.org"))
}

#[test]
fn the_default_is_four_and_declared() {
    assert_eq!(DEFAULT_FACTS.get(), 4);
    let d = KanbanSettings.declarations().unwrap();
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].key.as_str(), "fs.kanban.card-facts");
    assert_eq!(d[0].default, Value::Whole(4));
    assert_eq!(
        d[0].kind,
        Kind::Whole {
            min: 1,
            max: i64::from(i32::MAX)
        }
    );
}

#[test]
fn the_words_of_the_setting_are_in_both_languages() {
    let sources = [
        Source {
            language: "en",
            text: fs_components_kanban::limits::TEXT_EN,
        },
        Source {
            language: "de",
            text: fs_components_kanban::limits::TEXT_DE,
        },
    ];
    assert_eq!(check_pairs(&sources), Vec::<String>::new());
    let d = &KanbanSettings.declarations().unwrap()[0];
    for id in [d.label_key(), d.about_key()] {
        for text in [
            fs_components_kanban::limits::TEXT_EN,
            fs_components_kanban::limits::TEXT_DE,
        ] {
            assert!(
                text.lines().any(|l| l.starts_with(&format!("{id} ="))),
                "{id}"
            );
        }
    }
}

#[test]
fn a_saved_value_is_read() {
    let s = settings(true);
    assert_eq!(facts_limit(&s), DEFAULT_FACTS, "nothing saved: the default");
    let key = KanbanSettings::facts_key().unwrap();
    s.save(&key, Some(&Value::Whole(7)), Target::ThisProgram)
        .unwrap();
    assert_eq!(facts_limit(&s).get(), 7);
    s.save(&key, Some(&Value::Whole(1)), Target::ThisProgram)
        .unwrap();
    assert_eq!(facts_limit(&s).get(), 1);
    s.save(&key, None, Target::ThisProgram).unwrap();
    assert_eq!(facts_limit(&s), DEFAULT_FACTS, "reset: back to the default");
    assert_eq!(
        facts_limit(&settings(false)),
        DEFAULT_FACTS,
        "not declared: the default"
    );
}

#[test]
fn zero_is_refused_by_the_kind() {
    let s = settings(true);
    let key = KanbanSettings::facts_key().unwrap();
    let e = s
        .save(&key, Some(&Value::Whole(0)), Target::ThisProgram)
        .unwrap_err();
    assert_eq!(e.code(), SettingCode::OutOfRange);
    assert_eq!(facts_limit(&s), DEFAULT_FACTS);
}

#[test]
fn a_card_takes_what_its_limit_says() {
    let s = settings(true);
    let key = KanbanSettings::facts_key().unwrap();
    s.save(&key, Some(&Value::Whole(6)), Target::ThisProgram)
        .unwrap();
    let mut card = Card::limited("k", "K", facts_limit(&s));
    for i in 0..6 {
        card = card.fact(slot(i)).expect("room");
    }
    assert_eq!(card.facts().len(), 6);
    assert_eq!(card.clone().fact(slot(9)).unwrap_err(), slot(9));
    assert_eq!(card.facts().len(), 6, "nothing dropped, nothing added");
    let mut plain = Card::new("k", "K");
    for i in 0..4 {
        plain = plain.fact(slot(i)).expect("room");
    }
    assert_eq!(plain.max_facts(), 4);
    assert!(plain.fact(slot(9)).is_err());
}
