//! Craft's capital labels (heading, tabs, buttons, badge, link) in the mono face the machine has. The picture is kept when
//! `FS_LABELS_DIR` names a folder. Each item has a band of its own on the Craft ground (#121316); the test finds the box of
//! the item in the pixels (the row that differs from the ground over the longest run, a border or a fill) and checks that no
//! ink lies outside that box, that the words keep a gap to its edges inside, and that they read on the surface they stand on
//! (WCAG 2.1 1.4.3, 4.5:1). One style per process, hence a file of its own.
//!
//! Tests: pixels of the real picture. Without a graphics adapter or without a mono font it says SKIPPED (red when `CI` is set).

// Pixel arithmetic on a small, bounded picture; one long test that walks the items of the scene.
#![allow(
    clippy::cast_precision_loss,
    clippy::many_single_char_names,
    clippy::too_many_lines
)]

use std::collections::HashMap;
use std::sync::Arc;

use blinc_layout::div::div;
use blinc_theme::{ColorToken, ThemeState};
use fs_components_render::{Canvas, RenderError, draw_to_png, use_style};
use fs_components_styles::StyleId;
use fs_components_widgets::craft::{BadgeTone, Look, badge, button, heading, link, tab, tabs};
use image::{Rgba, RgbaImage};

const W: u32 = 420;
const ROW: u32 = 56;
const ROW_F: f32 = 56.0;
const BANDS: u32 = 6;
const GROUND: [u8; 3] = [0x12, 0x13, 0x16];

fn act() -> Arc<dyn Fn() + Send + Sync> {
    Arc::new(|| {})
}

fn luminance(p: Rgba<u8>) -> f64 {
    let c = |v: u8| {
        let s = f64::from(v) / 255.0;
        if s <= 0.039_28 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * c(p.0[0]) + 0.7152 * c(p.0[1]) + 0.0722 * c(p.0[2])
}

fn contrast(a: Rgba<u8>, b: Rgba<u8>) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

fn differs(a: Rgba<u8>, b: Rgba<u8>) -> bool {
    (0..3).any(|i| a.0[i].abs_diff(b.0[i]) > 6)
}

/// A rectangle of pixels: left, top, right, bottom (all inside).
type Rect = (u32, u32, u32, u32);

/// The box of the one item in `band`: the longest run that differs from the ground gives its left and right edge, the rows
/// that run (nearly) as long give its top and bottom. `None` when nothing is drawn.
fn find_box(img: &RgbaImage, band: u32, ground: Rgba<u8>) -> Option<Rect> {
    let rows = band * ROW..(band + 1) * ROW;
    let count = |y: u32| {
        (0..W)
            .filter(|&x| differs(*img.get_pixel(x, y), ground))
            .count()
    };
    let widest = rows.clone().max_by_key(|&y| count(y))?;
    let max = count(widest);
    if max < 20 {
        return None;
    }
    let xs: Vec<u32> = (0..W)
        .filter(|&x| differs(*img.get_pixel(x, widest), ground))
        .collect();
    let full: Vec<u32> = rows.filter(|&y| count(y) * 10 >= max * 9).collect();
    Some((xs[0], *full.first()?, *xs.last()?, *full.last()?))
}

/// The most frequent colour inside `r`, inset by 2 px: the surface the words stand on.
fn surface(img: &RgbaImage, r: Rect) -> Rgba<u8> {
    let mut seen: HashMap<[u8; 4], u32> = HashMap::new();
    for y in r.1 + 2..=r.3.saturating_sub(2) {
        for x in r.0 + 2..=r.2.saturating_sub(2) {
            *seen.entry(img.get_pixel(x, y).0).or_default() += 1;
        }
    }
    Rgba(
        seen.into_iter()
            .max_by_key(|(_, n)| *n)
            .map_or([0; 4], |(c, _)| c),
    )
}

/// Ink bounding box and the best contrast of the pixels inside `r` (inset 2 px) against `surface`.
fn ink_in(img: &RgbaImage, r: Rect, surface: Rgba<u8>) -> Option<(Rect, f64)> {
    let (mut l, mut t, mut rr, mut b, mut best) = (u32::MAX, u32::MAX, 0, 0, 1.0_f64);
    for y in r.1 + 2..=r.3.saturating_sub(2) {
        for x in r.0 + 2..=r.2.saturating_sub(2) {
            let c = contrast(*img.get_pixel(x, y), surface);
            if c > 1.8 {
                (l, t, rr, b) = (l.min(x), t.min(y), rr.max(x), b.max(y));
                best = best.max(c);
            }
        }
    }
    (rr > 0).then_some(((l, t, rr, b), best))
}

fn scene() -> impl blinc_layout::prelude::ElementBuilder + 'static {
    let band = |child: blinc_layout::div::Div| {
        div().h(ROW_F).p(2.0).flex_row().items_center().child(child)
    };
    div()
        .w(W as f32)
        .h(ROW_F * BANDS as f32)
        .bg(ThemeState::get().color(ColorToken::Background))
        .flex_col()
        .child(band(div().child(heading("01 / Classic apps"))))
        .child(band(tabs(vec![
            tab("t1", "All", Some(12), true, act()),
            tab("t2", "Installed", Some(3), false, act()),
            tab("t3", "Updates", Some(1), false, act()),
        ])))
        .child(band(button("b1", "Install", Look::Fill, true, act())))
        .child(band(button("b2", "Open", Look::Line, true, act())))
        .child(band(badge("Installed", BadgeTone::Ok)))
        .child(band(div().child(link("l1", "Documentation", act()))))
}

#[test]
fn the_capital_labels_stay_in_their_box_and_read_on_the_surface() {
    // MONO-GUARD-BEGIN
    if fs_components_render::installed_mono_face().is_none() {
        assert!(
            std::env::var_os("CI").is_none(),
            "no monospace font installed on a CI machine: install one (for example fonts-dejavu-core)"
        );
        eprintln!(
            "SKIPPED: no monospace font installed (render/craft_labels); this run is no proof"
        );
        return;
    }
    // MONO-GUARD-END
    use_style(Some(StyleId::Craft));
    let dir = std::env::var_os("FS_LABELS_DIR").map(std::path::PathBuf::from);
    let path = match &dir {
        Some(d) => {
            std::fs::create_dir_all(d).expect("folder");
            d.join("craft-labels.png")
        }
        None => std::env::temp_dir().join(format!("craft-labels-{}.png", std::process::id())),
    };
    let drawn = draw_to_png(
        &Canvas::new(W, ROW * BANDS, ""),
        |_, _| scene(),
        || {},
        &path,
    );
    if let Err(RenderError::NoGraphics(why)) = &drawn {
        assert!(
            std::env::var_os("CI").is_none(),
            "cannot be drawn in CI: {why}"
        );
        eprintln!("SKIPPED render/craft_labels: {why}");
        return;
    }
    drawn.expect("drawn");
    let img = image::open(&path).expect("picture").to_rgba8();
    if dir.is_none() {
        let _ = std::fs::remove_file(&path);
    }
    let ground = *img.get_pixel(W - 2, 2);
    assert_eq!(
        ground.0[..3],
        GROUND,
        "the picture ground is the Craft surface, not the edge of the canvas"
    );

    // (name, band, has a box of its own, the box is joined to its words (tabs: a filled tab touches the box))
    let items = [
        ("heading", 0, false, false),
        ("tabs", 1, true, true),
        ("button fill", 2, true, false),
        ("button line", 3, true, false),
        ("badge", 4, true, false),
        ("link", 5, false, false),
    ];
    for (name, band, boxed, joined) in items {
        let rows = band * ROW..(band + 1) * ROW;
        if !boxed {
            let ink: Vec<u32> = (0..W)
                .filter(|&x| rows.clone().any(|y| differs(*img.get_pixel(x, y), ground)))
                .collect();
            let (l, r) = (*ink.first().expect(name), *ink.last().expect(name));
            eprintln!("labels {name}: ink x {l}..{r} of {W}");
            assert!(
                l >= 4 && r < W - 8,
                "{name}: the words run from {l} to {r} of {W}: they leave the room"
            );
            // they read on the ground they stand on (WCAG 1.4.3)
            let room = (l, rows.start, r, rows.end - 1);
            let (_, best) =
                ink_in(&img, room, ground).unwrap_or_else(|| panic!("{name}: no words"));
            eprintln!("labels {name}: best contrast {best:.2}:1 on the ground");
            assert!(
                best >= 4.5,
                "{name}: the words read {best:.2}:1 on the ground"
            );
            continue;
        }
        let b = find_box(&img, band, ground).unwrap_or_else(|| panic!("{name}: no box found"));
        // nothing outside the box (one pixel of slack for the edge)
        let outside = rows
            .flat_map(|y| (0..W).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                !(b.0.saturating_sub(1)..=b.2 + 1).contains(&x)
                    || !(b.1.saturating_sub(1)..=b.3 + 1).contains(&y)
            })
            .filter(|&(x, y)| differs(*img.get_pixel(x, y), ground))
            .count();
        assert_eq!(
            outside, 0,
            "{name}: {outside} ink pixels outside the box {b:?}"
        );
        // tabs: the first tab is the filled one (light fill, dark words); the words of the inactive tabs, on the ground, are what
        // is measured, so the filled tab does not stand in for them
        let measured = if name == "tabs" {
            (b.0 + (b.2 - b.0) / 3, b.1, b.2, b.3)
        } else {
            b
        };
        let s = surface(&img, measured);
        let (ink, best) =
            ink_in(&img, measured, s).unwrap_or_else(|| panic!("{name}: no words in the box"));
        let ink = if name == "tabs" {
            ink_in(&img, b, surface(&img, b)).map_or(ink, |(i, _)| i)
        } else {
            ink
        };
        eprintln!(
            "labels {name}: box x {}..{} (width {}), words x {}..{} (width {}), best contrast {best:.2}:1 on {:?}",
            b.0,
            b.2,
            b.2 - b.0 + 1,
            ink.0,
            ink.2,
            ink.2 - ink.0 + 1,
            &s.0[..3]
        );
        assert!(
            best >= 4.5,
            "{name}: the words read {best:.2}:1 on their surface"
        );
        let (left, right) = (ink.0 - b.0, b.2 - ink.2);
        if joined {
            assert!(
                right >= 3,
                "{name}: the words end {right} px from the right edge"
            );
        } else {
            assert!(
                left >= 3 && right >= 3,
                "{name}: the words are {left} px from the left and {right} px from the right edge"
            );
            assert!(
                ink.1 - b.1 >= 2 && b.3 - ink.3 >= 2,
                "{name}: the words touch the top or the bottom edge"
            );
        }
    }
}
