//! What the theme says about the writing surface.
//!
//! Pattern: Value object — read once per build from the theme (tokens, no literal colour, size or font), then passed down. The
//! style of the process (Standard, Craft, ...) is a theme, so the surface follows it without knowing it.

use blinc_core::Color;
use blinc_theme::{ColorToken, SpacingToken, ThemeState};

/// The colours and sizes of the current theme, as the surface needs them.
#[derive(Clone, Debug)]
pub struct Look {
    /// Text.
    pub text: Color,
    /// Secondary text (markers, the language of a code block).
    pub weak: Color,
    /// The size of body text.
    pub base: f32,
    /// The size of code.
    pub small: f32,
    /// The size of a heading of level 1 to 6.
    pub heading: [f32; 6],
    /// Line height as a multiplier of the font size.
    pub line: f32,
    /// The room between two blocks.
    pub gap: f32,
    /// The room between two list items.
    pub tight: f32,
    /// The room inside a code block (above and below, left and right).
    pub pad: f32,
    /// The width of the column a marker (bullet, number, box) stands in, and the indent of a quote.
    pub marker: f32,
    /// The side of a task box.
    pub box_side: f32,
}

impl Look {
    /// Read the theme (initialised by the shell before any widget is built).
    #[must_use]
    pub fn current() -> Self {
        let theme = ThemeState::get();
        let t = theme.typography();
        let space = |s| theme.spacing_value(s);
        Self {
            text: theme.color(ColorToken::TextPrimary),
            weak: theme.color(ColorToken::TextSecondary),
            base: t.text_base,
            small: t.text_sm,
            heading: [
                t.text_3xl,
                t.text_2xl,
                t.text_xl,
                t.text_lg,
                t.text_base,
                t.text_base,
            ],
            line: 1.5,
            gap: space(SpacingToken::Space3),
            tight: space(SpacingToken::Space1),
            pad: space(SpacingToken::Space3),
            marker: 2.0 * space(SpacingToken::Space6),
            box_side: t.text_base + 2.0,
        }
    }
}
