//! The mode a place asks for: `ctx.place.view` of the form description, read once when the view is mounted.
//!
//! Pattern: Strategy (the choice) — the place, not the component, says which look is wanted; nothing asked or an
//! unknown word is the contract default of `calendar-grid` (the month), a normal case with no special branch. `agenda` is
//! the view of `calendar-view`, so here it is "unknown" and the default stands.

use fs_components_core::form_view::PlaceInfo;

use crate::model::Mode;

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
