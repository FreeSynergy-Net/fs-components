//! Typed scenes drawn with the visible cursor, in the standard style and in Craft: the keys go through the handlers of the built
//! element (see `common`), then the state is drawn; the pictures are kept when `FS_RICH_DIR` names a folder. The pixels are
//! checked too: the caret is where the geometry says, in the colour of the theme; the code ground and the boxes are drawn.
//!
//! Tests: picture tests (a software graphics adapter and fonts are needed; without them the test fails, it does not skip).

// Pixel arithmetic: coordinates are small positive numbers, the casts cannot lose anything that matters.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

mod common;

use std::path::PathBuf;

use blinc_theme::{ColorToken, ThemeState};
use common::{DOWN, END, HOME, LEFT, PAD, Typist, UP, assert_theme, expect};
use fs_components_editor::Doc;

const STYLES: [&str; 2] = ["standard", "craft"];

fn out(style: &str, name: &str) -> PathBuf {
    let dir = std::env::var_os("FS_RICH_DIR").map_or_else(std::env::temp_dir, PathBuf::from);
    std::fs::create_dir_all(&dir).expect("folder for the pictures");
    dir.join(format!("{style}-{name}.png"))
}

fn lin(v: u8) -> f32 {
    let c = f32::from(v) / 255.0;
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn luminance(p: image::Rgba<u8>) -> f32 {
    0.2126 * lin(p[0]) + 0.7152 * lin(p[1]) + 0.0722 * lin(p[2])
}

/// The contrast ratio of two pixels (WCAG).
fn contrast(a: image::Rgba<u8>, b: image::Rgba<u8>) -> f32 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

fn near(p: image::Rgba<u8>, c: blinc_core::Color, tol: i32) -> bool {
    let ch = |v: f32| (v * 255.0).round() as i32;
    (i32::from(p[0]) - ch(c.r)).abs() <= tol
        && (i32::from(p[1]) - ch(c.g)).abs() <= tol
        && (i32::from(p[2]) - ch(c.b)).abs() <= tol
}

/// Draw, and check that the caret stands where the geometry says in the colour of the theme.
fn shoot(t: &Typist, style: &str, name: &str) -> image::RgbaImage {
    let file = out(style, name);
    t.picture(&file);
    let img = image::open(&file).expect("the picture").to_rgba8();
    assert_theme(&img, style);
    let c = t.surface.cursor();
    let g = t.surface.geometry();
    let caret = g.caret(&c.path, c.at).expect("a caret for the cursor");
    // The colour of the cursor: the token of the style when it has one (Craft), else the primary colour of the theme.
    let primary = expect(style)
        .cursor
        .unwrap_or_else(|| ThemeState::get().color(ColorToken::Primary));
    let (x, y0, y1) = (
        (PAD + caret.x) as u32,
        (PAD + caret.y + 2.0) as u32,
        (PAD + caret.y + caret.h - 2.0) as u32,
    );
    let hit = (y0..y1)
        .filter(|&y| near(*img.get_pixel(x, y), primary, 6))
        .count();
    assert!(
        hit as f32 >= (y1 - y0) as f32 * 0.6,
        "{style}/{name}: the caret column at x={x} is not the colour of the theme ({hit} of {} rows)",
        y1 - y0
    );
    // The cursor shows on the ground: 3:1 at least, and it is not the colour of the text (it would read as a letter).
    let ground = *img.get_pixel(3, 3);
    let seen = *img.get_pixel(x, (y0 + y1) / 2);
    let ratio = contrast(seen, ground);
    assert!(
        ratio >= 3.0,
        "{style}/{name}: the cursor has {ratio:.2}:1 on the ground"
    );
    let text = ThemeState::get().color(ColorToken::TextPrimary);
    if style == "craft" {
        assert!(
            !near(seen, text, 30),
            "{style}/{name}: the cursor is the colour of the text"
        );
    }
    eprintln!("{style}/{name}: cursor {ratio:.2}:1");
    // Counter-case: a column well away from the caret and the text has no such colour.
    let away = (PAD + 700.0) as u32;
    assert!(
        (y0..y1).all(|y| !near(*img.get_pixel(away, y), primary, 40)),
        "{style}/{name}: the colour of the caret shows where there is none"
    );
    img
}

fn scene_a(style: &str) {
    let mut t = Typist::new(style, &Doc::new(vec![String::new()]), 780.0, 600.0);
    t.click(10.0, 10.0);
    t.type_text("## Plan for Monday\nThings to do before the meeting:\n[] write the summary\nwrite the agenda\nx");
    t.key(common::BACKSPACE);
    t.type_text("\n1. First point\nSecond point\n\n- a bullet\nanother one\n\nThat is all");
    assert_eq!(
        t.markdown(),
        "## Plan for Monday\n\nThings to do before the meeting:\n\n- [ ] write the summary\n- [ ] write the agenda\n\n1. First point\n2. Second point\n\n- a bullet\n- another one\n\nThat is all\n"
    );
    let a1 = shoot(&t, style, "a1-getippt-ende");
    // A task box is drawn (its frame is not the ground), and no box is ticked yet.
    let g = t.surface.geometry();
    let boxes: Vec<_> = g
        .markers
        .iter()
        .filter(|m| matches!(m, fs_components_rich::Marker::Task { .. }))
        .collect();
    assert_eq!(boxes.len(), 2);
    let ground = ThemeState::get().color(ColorToken::Background);
    if let fs_components_rich::Marker::Task { x, y, side, .. } = boxes[0] {
        let p = a1.get_pixel((PAD + x + side / 2.0) as u32, (PAD + y + 1.0) as u32);
        assert!(
            !near(*p, ground, 4),
            "{style}: the frame of the box is drawn"
        );
    }
    // The click on the box, then the arrow keys.
    t.click_box(&[2, 0, 0]);
    assert!(t.markdown().contains("- [x] write the summary"));
    t.keys(UP, 4);
    t.keys(LEFT, 3);
    let a2 = shoot(&t, style, "a2-kaestchen-und-pfeile");
    // The ticked box shows the accent (frame and check mark), the open box next to it does not.
    let primary = ThemeState::get().color(ColorToken::Primary);
    let g = t.surface.geometry();
    let accent_pixels = |at: usize| {
        let Some(fs_components_rich::Marker::Task { x, y, side, .. }) = g.markers.get(at) else {
            panic!("a box");
        };
        let mut n = 0;
        for py in (PAD + y) as u32..(PAD + y + side) as u32 {
            for px in (PAD + x) as u32..(PAD + x + side) as u32 {
                n += usize::from(near(*a2.get_pixel(px, py), primary, 40));
            }
        }
        n
    };
    assert!(
        accent_pixels(0) > 20,
        "{style}: the ticked box shows the accent ({})",
        accent_pixels(0)
    );
    assert_eq!(accent_pixels(1), 0, "{style}: the open box does not");
    t.keys(UP, 6);
    t.key(END);
    let a3 = shoot(&t, style, "a3-pfeile-in-der-ueberschrift");
    assert!(a3.width() > 0);
}

fn scene_b(style: &str) {
    let mut t = Typist::new(style, &Doc::new(vec![String::new()]), 780.0, 560.0);
    t.click(10.0, 10.0);
    t.type_text("> A quoted thought\n\n```rust\nfn main() {\n    println!(\"hello\");\n}\n\n---\nThe last paragraph");
    assert_eq!(
        t.markdown(),
        "> A quoted thought\n\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```\n\n---\n\nThe last paragraph\n"
    );
    let b1 = shoot(&t, style, "b1-zitat-code-linie");
    // The ground of the code block is drawn in the token of the theme.
    let g = t.surface.geometry();
    let code = g
        .shapes
        .iter()
        .find_map(|s| match s {
            fs_components_rich::Shape::Ground { x, y, w, h, .. } => Some((*x, *y, *w, *h)),
            _ => None,
        })
        .expect("a ground for the code block");
    // The ground is a tint over the ground of the page: the token of the theme (Standard), or the `--code-ground` of the style
    // (Craft: 12 percent of the text colour, so that it shows).
    let back = ThemeState::get().color(ColorToken::Background);
    let (tint, alpha) = if style == "craft" {
        (
            ThemeState::get().color(ColorToken::TextPrimary),
            31.0 / 255.0,
        )
    } else {
        let t = ThemeState::get().color(ColorToken::AccentSubtle);
        (t, t.a)
    };
    let mix = |s: f32, b: f32| s * alpha + b * (1.0 - alpha);
    let expected = blinc_core::Color::rgb(
        mix(tint.r, back.r),
        mix(tint.g, back.g),
        mix(tint.b, back.b),
    );
    let p = b1.get_pixel(
        (PAD + code.0 + code.2 - 12.0) as u32,
        (PAD + code.1 + code.3 / 2.0) as u32,
    );
    assert!(
        near(*p, expected, 10),
        "{style}: the code block has the ground of the theme, got {p:?}"
    );
    // The page itself is not that ground (a counter-case for the check above), and the ground shows against the page.
    let page = *b1.get_pixel(PAD as u32 + 400, (PAD + code.1 + code.3 + 3.0) as u32);
    assert!(
        !near(page, expected, 3),
        "{style}: the page is not the code ground"
    );
    let shows = contrast(*p, page);
    eprintln!("{style}: code ground on the page {shows:.2}:1");
    let need = if style == "craft" { 1.3 } else { 1.05 };
    assert!(
        shows >= need,
        "{style}: the code ground shows with {shows:.2}:1, needs {need}"
    );
    // The room around the rule is that of a line, not of a block: the tight room above and below.
    let look = fs_components_rich::Look::of(t.surface.style());
    let rule = g
        .shapes
        .iter()
        .find_map(|s| match s {
            fs_components_rich::Shape::Rule { y, .. } => Some(*y),
            _ => None,
        })
        .expect("a rule");
    let above = g
        .shapes
        .iter()
        .filter_map(|s| match s {
            fs_components_rich::Shape::Ground { y, h, .. } if y + h <= rule => Some(y + h),
            _ => None,
        })
        .fold(0.0_f32, f32::max);
    let below = g
        .leaves
        .iter()
        .filter(|l| l.y > rule + 1.0)
        .map(|l| l.y)
        .fold(f32::MAX, f32::min);
    assert!(
        (rule - above - look.tight).abs() < 0.6,
        "{style}: above the rule {} not {}",
        rule - above,
        look.tight
    );
    assert!(
        (below - rule - 1.0 - look.tight).abs() < 0.6,
        "{style}: below the rule {}",
        below - rule - 1.0
    );
    // The cursor goes from the closing brace of the code to the paragraph below, over the rule, and not onto the rule.
    t.keys(UP, 1);
    t.key(HOME);
    shoot(&t, style, "b2-cursor-im-code");
    let (code_text, c) = t.surface.with_session(|s| {
        (
            s.editor().text(&s.cursor().path).to_owned(),
            s.cursor().clone(),
        )
    });
    assert_eq!(
        c.at + 1,
        code_text.len(),
        "{style}: the cursor stands before the closing brace"
    );
    t.key(END);
    shoot(&t, style, "b3-ende-der-codezeile");
    let c = t.surface.cursor();
    assert_eq!(
        c.at,
        code_text.len(),
        "{style}: the cursor stands at the end of the line of code"
    );
    t.keys(DOWN, 1);
    shoot(&t, style, "b4-runter-ueber-die-linie");
    assert_eq!(
        t.surface
            .with_session(|s| s.editor().text(&s.cursor().path).to_owned()),
        "The last paragraph",
        "{style}: Down walks over the rule"
    );
    t.keys(UP, 1);
    assert_eq!(
        t.surface.cursor().path,
        c.path,
        "{style}: Up walks back over the rule"
    );
}

fn scene_c(style: &str) {
    let mut t = Typist::new(style, &Doc::new(vec![String::new()]), 780.0, 260.0);
    t.click(10.0, 10.0);
    t.type_text("##no space stays text\n[]neither does this\n-nor this\n# but this is a heading");
    assert_eq!(
        t.markdown(),
        "##no space stays text\n\n[]neither does this\n\n-nor this\n\n# but this is a heading\n"
    );
    shoot(&t, style, "c1-ohne-leerzeichen-bleibt-text");
}

#[test]
fn scene_a_standard() {
    scene_a(STYLES[0]);
}
#[test]
fn scene_a_craft() {
    scene_a(STYLES[1]);
}
#[test]
fn scene_b_standard() {
    scene_b(STYLES[0]);
}
#[test]
fn scene_b_craft() {
    scene_b(STYLES[1]);
}
#[test]
fn scene_c_standard() {
    scene_c(STYLES[0]);
}
#[test]
fn scene_c_craft() {
    scene_c(STYLES[1]);
}
