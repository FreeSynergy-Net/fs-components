//! A built-in symbol of a menu with a line at least `least` wide is more visible on a dark ground than the style's own thin
//! line (the scissors of the row of symbols in the dark context menu were pale).
//!
//! Tests: pixel test on the dark scheme (`set_dark`), counting ink. Mutants: `icon_named_stroked` ignores `least` -> the two
//! counts are equal and the test is red; `least` taken as the stroke itself (not scaled to the 24 grid) -> a smeared blob,
//! the upper bound is red.
//! Pattern: none; test of one function.

mod common;

use blinc_layout::div::div;
use common::{background, draw, ink, serial, set_dark};
use fs_components_widgets::{Tone, icon_named, icon_named_stroked};

#[test]
fn a_least_stroke_gives_the_dark_scissors_more_ink() {
    let _g = serial();
    set_dark(true);
    let thin = draw(
        div()
            .w(48.0)
            .h(48.0)
            .p(8.0)
            .child(icon_named("edit-cut", 16.0, Tone::Normal)),
        48,
        48,
        "icon-cut-thin-dark.png",
    );
    let strong = draw(
        div().w(48.0).h(48.0).p(8.0).child(icon_named_stroked(
            "edit-cut",
            16.0,
            Tone::Normal,
            Some(1.6),
        )),
        48,
        48,
        "icon-cut-strong-dark.png",
    );
    set_dark(false);
    let (a, b) = (
        ink(&thin, (0, 0, 48, 48), background(&thin)),
        ink(&strong, (0, 0, 48, 48), background(&strong)),
    );
    assert!(b > a, "the least stroke draws more ink: {b} against {a}");
    assert!(b < a * 4 + 300, "but not a blob: {b} against {a}");
}
