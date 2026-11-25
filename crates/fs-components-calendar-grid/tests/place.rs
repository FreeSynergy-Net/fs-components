//! The mode a place asks for, and the English words behind a host without sentences.
//!
//! Pattern: Table test — one place per row, the mode it must give.
#![allow(clippy::pedantic)]

use std::sync::Arc;

use fs_components_calendar_grid::Mode;
use fs_components_calendar_grid::place::{DEFAULT, mode_of};
use fs_components_calendar_grid::words::Readable;
use fs_components_core::form_view::PlaceInfo;
use fs_components_fields::{KeysAsText, Say};

#[test]
fn nothing_asked_and_unknown_words_are_the_contract_default() {
    assert_eq!(DEFAULT, Mode::Month);
    assert_eq!(mode_of(&PlaceInfo::default()), Mode::Month);
    // `agenda` is the view of calendar-view; an unknown word is no error either.
    for word in ["agenda", "", "Week", "year", "week "] {
        assert_eq!(
            mode_of(&PlaceInfo::default().with_view(word)),
            Mode::Month,
            "{word:?}"
        );
    }
    // A variant alone changes no mode.
    assert_eq!(
        mode_of(&PlaceInfo::default().with_variant("compact")),
        Mode::Month
    );
}

#[test]
fn the_three_views_of_the_contract_give_their_mode() {
    for (word, mode) in [
        ("month", Mode::Month),
        ("week", Mode::Week),
        ("day", Mode::Day),
    ] {
        assert_eq!(mode_of(&PlaceInfo::default().with_view(word)), mode);
        assert_eq!(
            mode_of(&PlaceInfo::default().with_view(word).with_variant("color-2")),
            mode,
            "the variant does not decide the mode"
        );
    }
}

#[test]
fn a_key_without_a_sentence_shows_a_readable_word_never_the_key() {
    let say = Readable::new(Arc::new(KeysAsText));
    assert_eq!(say.say("calendar.previous"), "Previous");
    assert_eq!(say.say("calendar.mode.week"), "Week");
    assert_eq!(say.say("calendar.weekday.0"), "Mon");
    assert_eq!(say.say("calendar.weekday.6"), "Sun");
    assert_eq!(say.say("calendar.month.1"), "January");
    assert_eq!(say.say("calendar.month.11"), "November");
    // A key the table does not know stays what the host answered.
    assert_eq!(say.say("calendar.unknown"), "calendar.unknown");
}

#[test]
fn the_sentence_of_the_host_wins() {
    let host = |key: &str| {
        if key == "calendar.today" {
            "Heute".to_owned()
        } else {
            key.to_owned()
        }
    };
    let say = Readable::new(Arc::new(host));
    assert_eq!(say.say("calendar.today"), "Heute");
    assert_eq!(say.say("calendar.next"), "Next");
}
