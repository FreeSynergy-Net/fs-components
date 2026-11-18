//! Rule 57 (A5): how many facts a card takes is a LIMIT handed in as a parameter (default 4); the block reads no settings. The
//! declaration of the setting is data the host registers. A card cannot be filled past its limit through a public field.
//!
//! Mutants: the default is not 4 -> `the_default_is_four_and_declared`; `Card::limited` ignores its limit ->
//! `a_card_takes_what_its_limit_says`; the key or the words are wrong -> `the_setting_is_data_with_words_in_both_languages`.
//!
//! Pattern: none; tests.
#![allow(clippy::pedantic)]

use fs_components_fields::Limit;
use fs_components_kanban::{Card, DEFAULT_FACTS, FACTS_SETTING, FieldSlot};
use fs_components_mf2::{Source, check_pairs};

use fs_components_kanban::limits::{TEXT_DE, TEXT_EN};

fn slot(i: usize) -> FieldSlot {
    FieldSlot::new("field-email", "label.email", &format!("a{i}@example.org"))
}

#[test]
fn the_default_is_four_and_declared() {
    assert_eq!(DEFAULT_FACTS.get(), 4);
    assert_eq!(FACTS_SETTING.key, "fs.kanban.card-facts");
    assert_eq!(FACTS_SETTING.default, DEFAULT_FACTS);
    assert_eq!(
        FACTS_SETTING.group, "board",
        "the group the settings window shows it in"
    );
    assert_eq!(Card::new("k", "K").max_facts(), 4);
}

#[test]
fn the_setting_is_data_with_words_in_both_languages() {
    let sources = [
        Source {
            language: "en",
            text: TEXT_EN,
        },
        Source {
            language: "de",
            text: TEXT_DE,
        },
    ];
    assert_eq!(check_pairs(&sources), Vec::<String>::new());
    let id = FACTS_SETTING.key.replace('.', "-");
    assert_eq!(FACTS_SETTING.label_key, format!("setting-label-{id}"));
    assert_eq!(FACTS_SETTING.about_key, format!("setting-about-{id}"));
    for id in [FACTS_SETTING.label_key, FACTS_SETTING.about_key] {
        for text in [TEXT_EN, TEXT_DE] {
            assert!(
                text.lines().any(|l| l.starts_with(&format!("{id} ="))),
                "{id}"
            );
        }
    }
}

#[test]
fn a_card_takes_what_its_limit_says() {
    let mut card = Card::limited("k", "K", Limit::new(6).unwrap());
    for i in 0..6 {
        card = card.fact(slot(i)).expect("room");
    }
    assert_eq!((card.facts().len(), card.max_facts()), (6, 6));
    assert_eq!(card.clone().fact(slot(9)).unwrap_err(), slot(9));
    assert_eq!(card.facts().len(), 6, "nothing dropped, nothing added");
    let mut one = Card::limited("k", "K", Limit::of(1));
    one = one.fact(slot(0)).unwrap();
    assert!(one.fact(slot(1)).is_err());
    let mut plain = Card::new("k", "K");
    for i in 0..4 {
        plain = plain.fact(slot(i)).expect("room");
    }
    assert!(plain.fact(slot(9)).is_err());
}
