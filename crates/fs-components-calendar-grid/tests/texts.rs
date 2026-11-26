//! Pattern: Chain of Responsibility (test) — the words of the grid: the host first, the catalog second, the key last; the
//! catalogs pair up; one word has one id.
#![allow(clippy::pedantic)]

use std::sync::Arc;

use fs_components_calendar_grid::texts::{
    CalendarTexts, Chain, KEYS, TEXTS, pairs_missing, view_key,
};
use fs_components_fields::{KeysAsText, Say};
use fs_components_words::Locale;

fn locale(tag: &str) -> Locale {
    Locale::parse(tag).unwrap()
}

#[test]
fn the_two_languages_have_the_same_ids() {
    assert!(pairs_missing().is_empty(), "{:?}", pairs_missing());
    assert_eq!(TEXTS.len(), 2);
}

#[test]
fn every_id_the_grid_asks_for_has_a_word_in_both_languages() {
    for tag in ["en", "de"] {
        let say = CalendarTexts::new(&locale(tag));
        for key in KEYS
            .iter()
            .filter(|k| **k != "calendar-more" && !k.contains("-colour-"))
        {
            assert_ne!(say.say(key), *key, "{tag}: {key} has no word");
        }
    }
    for name in ["month", "week", "day", "agenda"] {
        assert!(KEYS.contains(&view_key(name).as_str()), "{name}");
    }
}

#[test]
fn one_word_has_one_id_whatever_the_view() {
    // The day of the toolbar and the zoom of the timeline are the same word: one id, no `calendar.mode.*` beside it.
    assert_eq!(view_key("day"), "calendar-view-day");
    assert!(KEYS.iter().all(|k| !k.contains('.')));
}

#[test]
fn a_host_without_sentences_gets_the_catalog_in_its_language() {
    let en = Chain::new(Arc::new(KeysAsText), &locale("en-US"));
    let de = Chain::new(Arc::new(KeysAsText), &locale("de"));
    assert_eq!(en.say("calendar-today"), "Today");
    assert_eq!(de.say("calendar-today"), "Heute");
    assert_eq!(de.say("calendar-previous"), "Zurück");
    assert_eq!(de.say("calendar-all-day"), "ganztägig");
}

#[test]
fn the_sentence_of_the_host_wins_and_an_unknown_key_stays() {
    let host = |key: &str| {
        if key == "calendar-today" {
            "Jetzt".to_owned()
        } else {
            key.to_owned()
        }
    };
    let say = Chain::new(Arc::new(host), &locale("de"));
    assert_eq!(say.say("calendar-today"), "Jetzt");
    assert_eq!(say.say("calendar-next"), "Weiter");
    assert_eq!(say.say("calendar-unknown"), "calendar-unknown");
}

#[test]
fn the_mark_of_more_items_is_a_number_in_both_languages() {
    for tag in ["en", "de"] {
        assert_eq!(Chain::new(Arc::new(KeysAsText), &locale(tag)).more(3), "+3");
    }
}
