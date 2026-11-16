//! What the theme says about the writing surface.
//!
//! Pattern: Value object — read once per build from the theme (tokens, no literal colour, size or font), then passed down. The
//! style of the process (Standard, Craft, ...) is a theme, so the surface follows it without knowing it.

use blinc_core::Color;
use blinc_theme::{ColorToken, SpacingToken, ThemeState};
use fs_components_styles::StyleId;

/// The least room (px) above and below a rule: the room of the Standard style, held for every style.
pub const RULE_GAP: f32 = 9.0;

/// The colours and sizes of the current theme, as the surface needs them.
#[derive(Clone, Debug)]
pub struct Look {
    /// The style of the process the surface is drawn in (its icon set draws the check mark).
    pub style: StyleId,
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
    /// The room above and below a rule: half a block gap, but never less than
    /// `RULE_GAP`, so that in a style with small spacing a rule next to a code block is not a second frame of it.
    /// A value of the `Look`, not a theme token: the geometry is made in Rust (`place`) and Blinc reads only its own fixed
    /// spacing tokens, none of ours.
    pub rule_gap: f32,
    /// The room between two list items.
    pub tight: f32,
    /// The room inside a code block (above and below, left and right).
    pub pad: f32,
    /// The width of the column a marker (bullet, number, box) stands in, and the indent of a quote.
    pub marker: f32,
    /// The colour that reads on the accent (the check mark of a ticked box).
    pub on_accent: Color,
    /// The side of a task box.
    pub box_side: f32,
}

impl Look {
    /// Read the theme (initialised by the shell before any widget is built) for the style `Admin`, the one a surface has when its
    /// host did not name another (see [`crate::RichSurface::with_style`]).
    #[must_use]
    pub fn current() -> Self {
        Self::of(StyleId::Admin)
    }

    /// Read the theme for the style the process draws in.
    #[must_use]
    pub fn of(style: StyleId) -> Self {
        let theme = ThemeState::get();
        let t = theme.typography();
        let space = |s| theme.spacing_value(s);
        Self {
            style,
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
            rule_gap: (space(SpacingToken::Space3) * 0.5).max(RULE_GAP),
            tight: space(SpacingToken::Space1),
            pad: space(SpacingToken::Space3),
            // Relative to the text, not to the base unit of the style: about 28 px in both (a column of 42 px left a gap that was too wide).
            marker: t.text_base * 2.2,
            on_accent: theme.color(ColorToken::TextInverse),
            box_side: t.text_base + 2.0,
        }
    }
}
