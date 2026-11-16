//! Pictures of the writing surface and the pixel checks every picture gets: the theme guard, the cursor (colour, 3:1 on the ground).
//!
//! Tests: test helpers.

#![allow(
    dead_code,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use std::path::PathBuf;

use blinc_theme::{ColorToken, ThemeState};

use super::{PAD, Typist, assert_theme, expect};

/// The file of a picture: in `FS_RICH_DIR` (kept, the name carries the process), else in a folder of this process under
/// the temp dir (removed by `shoot` after the checks).
pub fn out(style: &str, name: &str) -> PathBuf {
    let pid = std::process::id();
    let dir = std::env::var_os("FS_RICH_DIR").map_or_else(
        || std::env::temp_dir().join(format!("fs-rich-{pid}")),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir).expect("folder for the pictures");
    dir.join(format!("{style}-{name}-{pid}.png"))
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
pub fn contrast(a: image::Rgba<u8>, b: image::Rgba<u8>) -> f32 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

pub fn near(p: image::Rgba<u8>, c: blinc_core::Color, tol: i32) -> bool {
    let ch = |v: f32| (v * 255.0).round() as i32;
    (i32::from(p[0]) - ch(c.r)).abs() <= tol
        && (i32::from(p[1]) - ch(c.g)).abs() <= tol
        && (i32::from(p[2]) - ch(c.b)).abs() <= tol
}

/// Draw, and check that the caret stands where the geometry says in the colour of the theme.
pub fn shoot(t: &Typist, style: &str, name: &str) -> image::RgbaImage {
    let file = out(style, name);
    t.picture(&file);
    let img = image::open(&file).expect("the picture").to_rgba8();
    if std::env::var_os("FS_RICH_DIR").is_none() {
        // Own folder of this process: the file goes, the folder goes when no other test of the process still uses it.
        let _ = std::fs::remove_file(&file);
        if let Some(dir) = file.parent() {
            let _ = std::fs::remove_dir(dir);
        }
    }
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
    let seen = *img.get_pixel(x, u32::midpoint(y0, y1));
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
