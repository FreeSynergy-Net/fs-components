//! The words of the help read in every style, light and dark: the text and the note on the ground of the help reach 4.5:1.
//!
//! Tests: model test over the colours of the theme of each style; the pairs are read from the sheet of the help itself, so a changed
//! sheet is measured, not an assumption (the lesson of the tooltip).

mod common;
#[path = "common/styled.rs"]
mod styled;

use blinc_layout::div::div;
use blinc_theme::{ColorToken, ThemeState};
use common::serial;
use fs_components_styles::{Rgba, StyleId};

const LEAST: f32 = 4.5;

fn rgba(token: ColorToken) -> Rgba {
    let c = ThemeState::get().color(token);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Rgba {
        r: byte(c.r),
        g: byte(c.g),
        b: byte(c.b),
        a: 255,
    }
}

/// The token the rule that starts with `rule` reads for `property`.
fn token_of(rule: &str, property: &str) -> ColorToken {
    let line = fs_components_widgets::help::STYLES
        .lines()
        .find(|l| l.starts_with(rule))
        .unwrap_or_else(|| panic!("no rule {rule}"));
    let key = format!("{property}: var(--");
    let at = line
        .find(&key)
        .unwrap_or_else(|| panic!("{rule} has no {property}"))
        + key.len();
    match &line[at..at + line[at..].find(')').expect("a closing bracket")] {
        "text-primary" => ColorToken::TextPrimary,
        "text-secondary" => ColorToken::TextSecondary,
        "surface" => ColorToken::Surface,
        "background" => ColorToken::Background,
        other => panic!("a token the test does not know: {other}"),
    }
}

#[test]
fn the_text_and_the_note_read_on_the_ground_in_every_style_light_and_dark() {
    let _g = serial();
    let css = fs_components_widgets::styles();
    for id in StyleId::ALL {
        for dark in [false, true] {
            let _tree = styled::layout_styled(div, 10.0, 10.0, id, dark, &css);
            let ground = rgba(token_of(".fs-help {", "background"));
            for rule in [".fs-help-text {", ".fs-help-note {"] {
                let ink = rgba(token_of(rule, "color"));
                let c = ink.contrast(ground);
                assert!(
                    c >= LEAST,
                    "{} dark={dark}: {rule} reads {c:.2}:1",
                    id.name()
                );
            }
        }
    }
}

#[test]
fn a_pale_colour_as_the_text_would_fail_in_some_style() {
    // Counter-test through the same path: the colour of a border as the colour of the words misses 4.5:1 on the ground in at least
    // one style, so the measure can fail.
    let _g = serial();
    let css = fs_components_widgets::styles();
    let mut failed = false;
    for id in StyleId::ALL {
        for dark in [false, true] {
            let _tree = styled::layout_styled(div, 10.0, 10.0, id, dark, &css);
            failed |= rgba(ColorToken::Border).contrast(rgba(token_of(".fs-help {", "background")))
                < LEAST;
        }
    }
    assert!(failed, "the measure cannot fail");
}
