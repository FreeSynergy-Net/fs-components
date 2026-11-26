//! The mode a place asks for.
//!
//! Pattern: Table test — one place per row, the mode it must give.
#![allow(clippy::pedantic)]

use fs_components_calendar_grid::Mode;
use fs_components_calendar_grid::place::{DEFAULT, mode_of};
use fs_components_core::form_view::PlaceInfo;

#[test]
fn nothing_asked_and_unknown_words_are_the_contract_default() {
    assert_eq!(DEFAULT, Mode::Month);
    assert_eq!(mode_of(&PlaceInfo::default()), Mode::Month);
    // An unknown word is no error either.
    for word in ["", "Week", "year", "week "] {
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
fn the_four_views_of_the_contract_give_their_mode() {
    for (word, mode) in [
        ("month", Mode::Month),
        ("week", Mode::Week),
        ("day", Mode::Day),
        ("agenda", Mode::Agenda),
    ] {
        assert_eq!(mode_of(&PlaceInfo::default().with_view(word)), mode);
        assert_eq!(
            mode_of(&PlaceInfo::default().with_view(word).with_variant("color-2")),
            mode,
            "the variant does not decide the mode"
        );
    }
}
