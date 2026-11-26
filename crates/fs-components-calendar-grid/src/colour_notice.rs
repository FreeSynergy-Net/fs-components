//! The notice above the grid about colours: items whose colour could not be read, and items whose colour is hard to see in the style.
//!
//! Pattern: Adapter (counts and the locale in, one notice element out) – KG7-b and ADR 0074 section 9.4: too little contrast is
//! not overwritten but secured with an outline and reported. The look of a colour is the value object of the styles, not decided here.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use fs_components_styles::{ContentLook, Palette};
use fs_components_words::Locale;

use crate::calendar_core::BarEntry;
use crate::texts::CalendarTexts;

/// How many of the items of `entries` have a colour that is below 3:1 on the ground of the window in `palette`.
#[must_use]
pub fn faint_count(entries: &[BarEntry], palette: &Palette) -> usize {
    entries
        .iter()
        .filter_map(|e| e.color)
        .filter(|c| ContentLook::of(*c, palette, palette.content_tint).faint)
        .count()
}

/// The lines of the notice (empty when there is nothing to say).
#[must_use]
pub fn lines(locale: &Locale, unreadable: usize, faint: usize) -> Vec<String> {
    let say = CalendarTexts::new(locale);
    [
        (unreadable, "calendar-colour-unreadable"),
        (faint, "calendar-colour-faint"),
    ]
    .into_iter()
    .filter(|(n, _)| *n > 0)
    .map(|(n, id)| say.counted(id, n))
    .collect()
}

/// The notice element for `lines`, or `None` when there are none.
#[must_use]
pub fn notice(lines: &[String]) -> Option<Div> {
    if lines.is_empty() {
        return None;
    }
    let mut words = div().class("fs-partial-words");
    for (i, line) in lines.iter().enumerate() {
        words = words.child(
            text(line.clone())
                .id(format!("calendar-colour-notice-{i}"))
                .class("fs-partial-text"),
        );
    }
    Some(
        div()
            .id("calendar-colour-notice")
            .class("fs-partial")
            .attr("role", "status")
            .child(words),
    )
}
