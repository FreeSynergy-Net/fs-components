//! The tooltip bubble reads in every style, light and dark: the text (`--background`) on the ground (`--text-primary`) reaches the
//! WCAG contrast of body text (4.5 : 1) in all styles of the set.
//!
//! Tests: model test over the colours of the theme of each style (`fs_components_styles::Rgba::contrast`), no drawing; the pair is
//! read from the theme the same way the sheet reads it (`var(--text-primary)`, `var(--background)`).

mod common;
#[path = "common/styled.rs"]
mod styled;

use blinc_layout::div::div;
use blinc_theme::{ColorToken, ThemeState};
use common::serial;
use fs_components_styles::{Rgba, StyleId};

/// The least contrast of the text of a bubble on its ground (WCAG AA for normal text).
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

#[test]
fn the_bubble_text_reaches_4_5_to_1_in_every_style_light_and_dark() {
    let _g = serial();
    let css = fs_components_widgets::styles();
    for id in StyleId::ALL {
        for dark in [false, true] {
            let _tree = styled::layout_styled(div, 10.0, 10.0, id, dark, &css);
            let ground = rgba(ColorToken::TextPrimary);
            let text = rgba(ColorToken::Background);
            let c = text.contrast(ground);
            assert!(
                c >= LEAST,
                "{} dark={dark}: the bubble text has contrast {c:.2} on its ground",
                id.name()
            );
        }
    }
}

#[test]
fn the_pair_the_sheet_used_before_would_fail_this_measure() {
    // Counter-test through the same code path: the inverse text token on the same ground (the first version of the sheet) is
    // too pale in at least one dark style, so the choice of the pair is what makes the test pass.
    let _g = serial();
    let css = fs_components_widgets::styles();
    let mut failed = false;
    for id in StyleId::ALL {
        let _tree = styled::layout_styled(div, 10.0, 10.0, id, true, &css);
        let c = rgba(ColorToken::TextInverse).contrast(rgba(ColorToken::TextPrimary));
        failed |= c < LEAST;
    }
    assert!(failed, "the measure cannot fail");
}
