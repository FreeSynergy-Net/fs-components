//! `person-face`: the round picture of a person, or a circle with the initials when there is no picture.
//!
//! Pattern: Adapter over Blinc's image and text — the circle is ours (any size, not the few steps of `blinc_cn`'s avatar), the
//! initials come from [`initials`](crate::avatar_group::initials) (safe for any name) and the rule is the one of the avatar group:
//! a picture that cannot be loaded is never handed over (Blinc would draw a grey disc), the initials stand instead. A face is a
//! picture to the eye, so the name is put on it for a screen reader (`aria-label`).

use blinc_layout::div::{Div, div};
use blinc_layout::image::img;
use blinc_layout::text;
use blinc_theme::{ColorToken, ThemeState};

use crate::avatar_group::initials;

/// The style sheet of the face (the colours come from the theme at build time).
pub const STYLES: &str = ".fs-face { flex-shrink: 0; overflow: hidden; }\n";

/// The part of the diameter that the letters are tall.
const LETTER_SHARE: f32 = 0.38;

/// One person's face.
#[derive(Clone, Debug)]
pub struct PersonFace {
    name: String,
    image: Option<String>,
    px: f32,
}

impl PersonFace {
    /// The face of `name`, 40 px wide, without a picture.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            image: None,
            px: 40.0,
        }
    }

    /// With a picture (a `data:` URI, a local file or a URL).
    #[must_use]
    pub fn image(mut self, image: Option<String>) -> Self {
        self.image = image;
        self
    }

    /// The diameter in pixels.
    #[must_use]
    pub fn size(mut self, px: f32) -> Self {
        self.px = px.max(8.0);
        self
    }

    /// The picture that will be drawn: the one given, if it can be loaded.
    pub fn shown_image(&self) -> Option<&str> {
        self.image
            .as_deref()
            .filter(|i| crate::picture::loadable(i))
    }

    /// The letters drawn when there is no picture.
    pub fn letters(&self) -> String {
        initials(&self.name)
    }

    /// The height of the letters in pixels: a fixed share of the diameter, so the letters grow with the circle.
    pub fn letter_size(&self) -> f32 {
        (self.px * LETTER_SHARE).round().max(8.0)
    }

    /// The diameter in pixels.
    pub fn diameter(&self) -> f32 {
        self.px
    }

    /// The element to place in a layout.
    pub fn build(&self) -> Div {
        let theme = ThemeState::get();
        let face = div()
            .class("fs-face")
            .w(self.px)
            .h(self.px)
            .rounded(self.px / 2.0)
            .flex_row()
            .items_center()
            .justify_center()
            .attr("role", "img")
            .attr("aria-label", &self.name);
        match self.shown_image() {
            Some(src) => face.child(img(src.to_owned()).w(self.px).h(self.px).cover()),
            None => face.bg(theme.color(ColorToken::Primary)).child(
                text(self.letters())
                    .size(self.letter_size())
                    .semibold()
                    .color(theme.color(ColorToken::TextInverse))
                    .no_cursor(),
            ),
        }
    }
}
