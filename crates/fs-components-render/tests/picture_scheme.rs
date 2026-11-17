//! Which colour scheme a picture is drawn with: `FS_RENDER_SCHEME`, else `FS_COLOR_SCHEME` (the one a program's window
//! reads, so `--picture` and the window agree), else light.
//!
//! Tests: pure model tests over the two environment values, no graphics. Mutants: `FS_COLOR_SCHEME` ignored -> the dark
//! cases without a render value go red (this was the bug: dark pictures came out light); the order swapped -> the
//! "render wins" case goes red; an unknown word taken as dark -> the unknown-word case goes red.
//! Pattern: none, pure function.

use blinc_theme::ColorScheme::{Dark, Light};
use fs_components_render::picture_scheme;

#[test]
fn the_colour_scheme_of_a_program_reaches_its_pictures() {
    assert_eq!(picture_scheme(None, Some("dark")), Dark);
    assert_eq!(picture_scheme(None, Some("DARK")), Dark);
    assert_eq!(picture_scheme(None, Some("light")), Light);
}

#[test]
fn the_render_value_wins_over_the_colour_value() {
    assert_eq!(picture_scheme(Some("dark"), Some("light")), Dark);
    assert_eq!(picture_scheme(Some("light"), Some("dark")), Light);
}

#[test]
fn nothing_or_an_unknown_word_is_light_and_never_the_desktop() {
    assert_eq!(picture_scheme(None, None), Light);
    assert_eq!(picture_scheme(Some("auto"), Some("blue")), Light);
    // an unknown render value does not hide a valid colour value
    assert_eq!(picture_scheme(Some("auto"), Some("dark")), Dark);
}
