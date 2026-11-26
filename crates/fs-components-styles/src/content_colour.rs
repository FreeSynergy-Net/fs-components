//! How a colour that belongs to the CONTENT (the `color` of an event, RFC 8984 section 4.2.11, RFC 7986 section 5.9) shows in a style.
//!
//! Pattern: Adapter + Value Object – [`Rgba::from_css`] adapts CSS colour text (crate `csscolorparser`) to [`Rgba`]; [`ContentLook`]
//! is an immutable value, valid when built. The style decides HOW a content colour appears (tint of the ground, ink, outline), never
//! WHICH colour it is (ADR 0074 section 2). Too little contrast is not overwritten but secured with an outline (`faint`, section 9.4).

use crate::color::Rgba;
use crate::tokens::Palette;

/// Least contrast of text (WCAG 1.4.3).
const TEXT_CONTRAST: f32 = 4.5;
/// Least contrast of a graphic (WCAG 1.4.11).
const MARK_CONTRAST: f32 = 3.0;

impl Rgba {
    /// Reads a CSS colour value (names, `#rgb`, `rgb()`, `hsl()` ...). `None` for what is not a colour, for hex digits without `#`
    /// (CSS has none; no colour name is made of hex digits only) and for a fully transparent colour (there is nothing to show).
    pub fn from_css(value: &str) -> Option<Self> {
        let value = value.trim();
        if value.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let [r, g, b, a] = csscolorparser::parse(value).ok()?.to_rgba8();
        (a != 0).then_some(Self { r, g, b, a })
    }
}

/// What a content colour looks like in one style and appearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentLook {
    /// The mark (edge, stripe, dot): the colour as given, unchanged.
    pub mark: Rgba,
    /// The ground of the item: the colour tinted into the panel.
    pub ground: Rgba,
    /// The ink on `ground`, at least 4.5:1.
    pub ink: Rgba,
    /// The securing outline, set when the mark is `faint`.
    pub outline: Option<Rgba>,
    /// The mark is below 3:1 on the ground of the window and needs the outline (and a notice).
    pub faint: bool,
}

impl ContentLook {
    /// The look of `colour` (as given, never changed) in `palette`; `tint` = per mille of the colour in the ground (`content_tint`).
    #[must_use]
    pub fn of(colour: Rgba, palette: &Palette, tint: u16) -> Self {
        let ground = palette.panel.mix(colour, tint);
        let seen = palette
            .ground
            .mix(colour, (u32::from(colour.a) * 1000 / 255) as u16);
        let faint = seen.contrast(palette.ground) < MARK_CONTRAST;
        Self {
            mark: colour,
            ground,
            ink: ink_on(ground, palette),
            outline: faint.then_some(palette.text_weak),
            faint,
        }
    }

    /// The quiet look of an item without colour: ground `hover`, mark `text_weak`, ink `text` (theme-own, no accent, no status).
    #[must_use]
    pub fn quiet(palette: &Palette, _tint: u16) -> Self {
        Self {
            mark: palette.text_weak,
            ground: palette.hover,
            ink: ink_on(palette.hover, palette),
            outline: None,
            faint: false,
        }
    }
}

/// `palette.text` when it reads on `ground` (4.5:1), else the black or white that reads better.
fn ink_on(ground: Rgba, palette: &Palette) -> Rgba {
    if palette.text.contrast(ground) >= TEXT_CONTRAST {
        palette.text
    } else {
        crate::styles::colour_roles::readable_on(&[ground])
    }
}
