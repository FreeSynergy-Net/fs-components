//! Test support: draw an element in a named style and appearance, with the theme guard ([THEME-WAECHTER]).
//!
//! Tests: a helper over `fs-components-render`'s `style_bundle`; a test file includes it with `#[path = ".../common/styled.rs"] mod styled;` next to `mod common;`. Every picture carries a swatch of the theme's primary token in its
//! top left corner, next to a reference swatch painted from the palette of the style that was asked for (not from the theme), and
//! a second reference for the ground. Before a picture counts, the guard reads them back: the theme's token must be the colour
//! of the requested style (the same pipeline draws both, so no gamma guess is needed). A wrong theme (a picture drawn in the
//! style of an earlier test) panics with the style, the appearance and both colours.
#![allow(dead_code)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use blinc_layout::css_parser::Stylesheet;
use blinc_layout::prelude::{ElementBuilder, RenderTree};
use blinc_theme::{ColorScheme, ColorToken, ThemeState};
use fs_components_render::style_bundle;
use fs_components_styles::{Rgba, StyleId, style};
use image::RgbaImage;

/// Size of one swatch in the top left corner, in pixels; the strip is four of them (the theme's primary token, the reference accent,
/// the theme's background token, the reference ground).
pub const SWATCH: u32 = 8;
/// Height of the empty row between the strip and the scene.
pub const HEAD: u32 = 6;
/// Width of the strip.
pub const STRIP: u32 = SWATCH * 4;

/// The scheme asked for; a style that is dark by nature (Craft) is dark in both.
fn dark_of(id: StyleId, dark: bool) -> bool {
    dark || id == StyleId::Craft
}

/// The element made by `build` laid out in `w` x `h` in the style `id`, light or dark, with `css` (the widget sheets) on top.
/// `build` runs after the theme is set: an icon takes its tone from the theme when it is built, so an element made before would carry
/// the colours of the theme of the picture before.
pub fn layout_styled<E: ElementBuilder + 'static>(
    build: impl FnOnce() -> E,
    w: f32,
    h: f32,
    id: StyleId,
    dark: bool,
    css: &str,
) -> RenderTree {
    layout_styled_as(build, w, h, (id, dark), (id, dark), css)
}

/// Like [`layout_styled`], but the references of the guard are those of `claimed`: for the counter-check that a picture drawn in
/// one theme and claimed to be another is red.
pub fn layout_styled_as<E: ElementBuilder + 'static>(
    build: impl FnOnce() -> E,
    w: f32,
    h: f32,
    (id, dark): (StyleId, bool),
    claimed: (StyleId, bool),
    css: &str,
) -> RenderTree {
    crate::common::init_runtime();
    blinc_layout::init_text_measurer();
    let bundle = style_bundle(id, css);
    let sources = bundle.css_sources.clone();
    let scheme = if dark_of(id, dark) {
        ColorScheme::Dark
    } else {
        ColorScheme::Light
    };
    ThemeState::init(bundle, scheme);
    let theme = ThemeState::get();
    let p = style(claimed.0).palette(dark_of(claimed.0, claimed.1));
    let paint = |c: Rgba| {
        blinc_core::Color::rgba(
            f32::from(c.r) / 255.0,
            f32::from(c.g) / 255.0,
            f32::from(c.b) / 255.0,
            1.0,
        )
    };
    let square = |at: u32, c: blinc_core::Color| {
        blinc_layout::div::div()
            .absolute()
            .left((at * SWATCH) as f32)
            .top(0.0)
            .w(SWATCH as f32)
            .h(SWATCH as f32)
            .bg(c)
    };
    let swatch = blinc_layout::div::div()
        .absolute()
        .left(0.0)
        .top(0.0)
        .w(STRIP as f32)
        .h(SWATCH as f32)
        .child(square(0, theme.color(ColorToken::Primary)))
        .child(square(1, paint(p.accent)))
        .child(square(2, theme.color(ColorToken::Background)))
        .child(square(3, paint(p.ground)));
    let page = blinc_layout::div::div()
        .w(w)
        .h(h)
        .bg(theme.color(ColorToken::Background))
        .child(
            blinc_layout::div::div()
                .w(w)
                .h(h)
                .flex_col()
                .p_px(SWATCH as f32 + 4.0)
                // room between the strip and the scene
                .child(blinc_layout::div::div().w(1.0).h(HEAD as f32))
                .child(build()),
        )
        .child(swatch);
    let mut tree = RenderTree::from_element(&page);
    let vars = theme.to_css_variable_map();
    let mut sheet = String::new();
    for s in &sources {
        sheet.push_str(s);
        sheet.push('\n');
    }
    tree.set_stylesheet(Stylesheet::parse_with_variables(&sheet, &vars).expect("stylesheet"));
    tree.apply_stylesheet_layout_overrides();
    tree.apply_stylesheet_base_styles();
    tree.compute_layout(w, h);
    tree
}

fn near(a: [u8; 4], b: [u8; 4]) -> bool {
    (0..3).all(|i| a[i].abs_diff(b[i]) <= 2)
}

/// The colour of the ground of a picture (the theme's background token in the strip; the page itself may lie under a shadow).
pub fn ground_of(img: &RgbaImage) -> [u8; 4] {
    img.get_pixel(2 * SWATCH + SWATCH / 2, SWATCH / 2).0
}

/// The theme guard: the theme's primary token is the accent of the requested style, and the theme's background token is the ground of
/// the requested style (each against a reference painted from the style's palette). Panics when not.
pub fn guard(img: &RgbaImage, id: StyleId, dark: bool) {
    let at = |i: u32| img.get_pixel(i * SWATCH + SWATCH / 2, SWATCH / 2).0;
    let (token, accent, ground, ground_ref) = (at(0), at(1), at(2), at(3));
    assert!(
        near(token, accent),
        "THEME GUARD RED: {} dark={dark}: the theme's primary is {token:?}, the style's accent is {accent:?}",
        id.name()
    );
    assert!(
        near(ground, ground_ref),
        "THEME GUARD RED: {} dark={dark}: the ground is {ground:?}, the style's ground is {ground_ref:?}",
        id.name()
    );
}

/// Draw in a style; `name` is the file name when pictures are kept (the style and appearance are put in front of it).
pub fn draw_styled<E: ElementBuilder + 'static>(
    build: impl FnOnce() -> E,
    w: u32,
    h: u32,
    id: StyleId,
    dark: bool,
    css: &str,
    name: &str,
) -> RgbaImage {
    let tree = layout_styled(build, w as f32, h as f32, id, dark, css);
    draw_styled_tree(&tree, w, h, id, dark, name)
}

/// Draw an already laid-out tree of [`layout_styled`] and run the guard.
pub fn draw_styled_tree(
    tree: &RenderTree,
    w: u32,
    h: u32,
    id: StyleId,
    dark: bool,
    name: &str,
) -> RgbaImage {
    let tag = format!(
        "{}-{}-{name}",
        id.name(),
        if dark_of(id, dark) { "dark" } else { "light" }
    );
    let img = crate::common::draw_tree(tree, w, h, &tag);
    guard(&img, id, dark);
    img
}
