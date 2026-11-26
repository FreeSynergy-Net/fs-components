//! The mode a place asks for: `ctx.place.view` of the form description, read once when the view is mounted.
//!
//! Pattern: Strategy (the choice) — the place, not the component, says which look is wanted; nothing asked or an
//! unknown word is the contract default of `calendar-grid` (the month), a normal case with no special branch. `agenda` is
//! a view of the grid like the others.

use fs_components_core::form_view::PlaceInfo;

use fs_components_words::DateNames;

use crate::model::{Mode, Variant};
use crate::names;

/// The mode of the contract when the place asks for nothing (or for a view this block does not have).
pub const DEFAULT: Mode = Mode::Month;

/// The mode the place asks for: the `view` word `month`, `week` or `day`; anything else is [`DEFAULT`].
pub fn mode_of(place: &PlaceInfo) -> Mode {
    place
        .view
        .as_deref()
        .and_then(|word| Mode::ALL.into_iter().find(|m| m.name() == word))
        .unwrap_or(DEFAULT)
}

/// The name of the switch in `PlaceInfo::options`.
pub const WEEK_NUMBERS: &str = "view.week-numbers";

/// The contract default of the week numbers: off.
pub const WEEK_NUMBERS_DEFAULT: bool = false;

/// Whether the place wants the week numbers: `true` or `false`; nothing set or any other word is the default.
pub fn week_numbers_of(place: &PlaceInfo) -> bool {
    match place.option(WEEK_NUMBERS) {
        Some("true") => true,
        Some("false") => false,
        _ => WEEK_NUMBERS_DEFAULT,
    }
}

/// The variant the place asks for: the word `compact`; nothing asked or any other word (`color-N` colours items, it does not
/// change the grid) is the contract default, the full grid.
pub fn variant_of(place: &PlaceInfo) -> Variant {
    match place.variant.as_deref() {
        Some("compact") => Variant::Compact,
        _ => Variant::Full,
    }
}

/// The name of the first-weekday choice in `PlaceInfo::options`: `mon` .. `sun`.
pub const WEEK_START: &str = "view.week-start";

/// The first weekday of the grid, 0 Monday .. 6 Sunday: the word of the place (`mon` .. `sun`) when it names one, else the
/// first weekday of the reader's language and setting (`DrawCtx.dates`, CLDR of the region, `-u-fw` of the locale).
pub fn week_start_of(place: &PlaceInfo, dates: &dyn DateNames) -> u8 {
    const WORDS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
    place
        .option(WEEK_START)
        .and_then(|w| WORDS.iter().position(|x| *x == w))
        .map_or_else(|| names::week_start(dates), |i| i as u8)
}
