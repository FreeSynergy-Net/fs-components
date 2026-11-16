//! Craft's capital labels (heading, tabs, buttons, badge, link) in the mono face the machine has: the picture is kept when
//! `FS_LABELS_DIR` names a folder, and the ink of every row is measured against the frame it must stay in (no label runs out
//! of its box) and against the ground (WCAG 2.1 1.4.3, 4.5:1). One style per process, hence a file of its own.
//!
//! Tests: pixels of the real picture; skipped (with a message) where there is no graphics adapter.

use std::sync::Arc;

use blinc_layout::div::div;
use fs_components_render::{Canvas, RenderError, draw_to_png, use_style};
use fs_components_styles::StyleId;
use fs_components_widgets::craft::{BadgeTone, Look, badge, button, heading, link, tab, tabs};
use image::{Rgba, RgbaImage};

const W: u32 = 420;
const ROW: u32 = 56;
const ROW_F: f32 = 56.0;

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

/// (left, right, brightest pixel) of what differs from the ground in one row of the picture.
fn ink(img: &RgbaImage, row: u32, ground: Rgba<u8>) -> Option<(u32, u32, Rgba<u8>)> {
    let (mut l, mut r, mut best) = (u32::MAX, 0, ground);
    for y in row * ROW..(row + 1) * ROW {
        for x in 0..img.width() {
            let p = *img.get_pixel(x, y);
            if contrast(p, ground) > 1.5 {
                l = l.min(x);
                r = r.max(x);
                if contrast(p, ground) > contrast(best, ground) {
                    best = p;
                }
            }
        }
    }
    (r > 0).then_some((l, r, best))
}

#[test]
fn the_capital_labels_stay_in_their_room_and_read_on_the_ground() {
    use_style(Some(StyleId::Craft));
    let dir = std::env::var_os("FS_LABELS_DIR").map(std::path::PathBuf::from);
    let path = dir.clone().unwrap_or_else(std::env::temp_dir).join(format!(
        "craft-labels-{}.png",
        if dir.is_some() {
            String::new()
        } else {
            std::process::id().to_string()
        }
    ));
    if let Some(d) = &dir {
        std::fs::create_dir_all(d).expect("folder");
    }
    let canvas = Canvas::new(W, ROW * 4, "");
    let drawn = draw_to_png(
        &canvas,
        |_, _| {
            div()
                .flex_col()
                .child(div().h(ROW_F).p(2.0).child(heading("01 / Classic apps")))
                .child(div().h(ROW_F).p(2.0).child(tabs(vec![
                    tab("t1", "All", Some(12), true, act()),
                    tab("t2", "Installed", Some(3), false, act()),
                    tab("t3", "Updates", Some(1), false, act()),
                ])))
                .child(
                    div()
                        .h(ROW_F)
                        .p(2.0)
                        .flex_row()
                        .gap(8.0)
                        .child(button("b1", "Install", Look::Fill, true, act()))
                        .child(button("b2", "Open", Look::Line, true, act()))
                        .child(badge("Installed", BadgeTone::Ok)),
                )
                .child(
                    div()
                        .h(ROW_F)
                        .p(2.0)
                        .child(link("l1", "Documentation", act())),
                )
        },
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
    for (name, row) in [
        ("heading", 0),
        ("tabs", 1),
        ("buttons+badge", 2),
        ("link", 3),
    ] {
        let (l, r, best) =
            ink(&img, row, ground).unwrap_or_else(|| panic!("{name}: nothing drawn"));
        let c = contrast(best, ground);
        eprintln!(
            "labels {name}: ink x {l}..{r} of {W} (width {}), brightest contrast {c:.2}:1",
            r - l + 1
        );
        assert!(
            r < W - 8,
            "{name}: the labels run to {r} of {W}: they leave their room"
        );
        assert!(
            c >= 4.5,
            "{name}: the brightest ink reads {c:.2}:1 on the ground"
        );
    }
}
