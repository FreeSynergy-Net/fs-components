//! Contrast of what the views of `dateTime` and `duration` draw (the generic segmented input) with the colours of the style (not a sheet): the caption and the hint
//! (`text_weak`) on the ground, the sentence of a problem or the empty notice (`error`) on the ground and in the field, the words in the
//! field (`text`) on the field, the edge of a field against the ground, in EVERY style, light and dark. A counter-test shows the
//! measure can fail.
//!
//! Tests: model tests over `fs_components_styles::Rgba::contrast` of the palette each view reads; no drawing.
#![allow(clippy::pedantic)]

use fs_components_styles::{StyleId, style};

const TEXT: f32 = 4.5;
const MARK: f32 = 3.0;

#[test]
fn the_colours_the_two_views_read_reach_the_wcag_levels_in_every_style() {
    let mut bad = Vec::new();
    for id in StyleId::ALL {
        for dark in [false, true] {
            let p = style(id).palette(dark);
            let pairs = [
                (
                    "caption (text_weak) on the ground",
                    p.text_weak,
                    p.ground,
                    TEXT,
                ),
                (
                    "problem or notice (error) on the ground",
                    p.error,
                    p.ground,
                    TEXT,
                ),
                ("problem (error) on the panel", p.error, p.panel, TEXT),
                ("words (text) in the field", p.text, p.field, TEXT),
                ("hint (text_weak) in the field", p.text_weak, p.field, TEXT),
                (
                    "edge of the field (border) on the ground",
                    p.border,
                    p.ground,
                    1.2,
                ),
                (
                    "keyboard ring (focus) on the ground",
                    p.focus,
                    p.ground,
                    MARK,
                ),
            ];
            for (what, fg, bg, least) in pairs {
                let c = fg.contrast(bg);
                if c < least {
                    bad.push(format!(
                        "{} dark={dark}: {what} has {c:.2}, needs {least}",
                        id.name()
                    ));
                }
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn the_measure_can_fail() {
    let p = style(StyleId::Admin).palette(false);
    assert!(
        p.text_weak.contrast(p.ground) < 20.0,
        "a demand of 20:1 is never met"
    );
}
