//! The blocks of contract version 0.2 in every style: pictures, the theme guard and "two styles".
//!
//! Tests: draws every scene in every style (light and dark) with the theme guard of `styled.rs` ([THEME-WAECHTER]: the picture
//! carries the requested theme, the swatch of the primary token and the ground must match the style), then compares pairs of
//! pictures: a block whose picture does not change between two styles is red ("zwei-stile"). Pictures are kept when
//! `FS_COMP_SHOTS` names a directory.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;
mod scenes;
#[path = "../../fs-components-widgets/tests/common/styled.rs"]
mod styled;

use std::collections::BTreeMap;

use common::serial;
use fs_components_styles::StyleId;
use image::RgbaImage;
use scenes::{SCENES, THEMES};
use styled::{STRIP, SWATCH, draw_styled, ground_of};

fn tag(id: StyleId, dark: bool) -> String {
    format!(
        "{}-{}",
        id.name(),
        if dark || id == StyleId::Craft {
            "dark"
        } else {
            "light"
        }
    )
}

fn dist(a: [u8; 4], b: [u8; 4]) -> i32 {
    (0..3)
        .map(|i| (i32::from(a[i]) - i32::from(b[i])).abs())
        .sum()
}

/// The share of the pixels that hold ink in either picture and differ clearly between them (the plan's `r`).
fn changed_share(a: &RgbaImage, b: &RgbaImage) -> f32 {
    let (ga, gb) = (ground_of(a), ground_of(b));
    let (mut union, mut changed) = (0u32, 0u32);
    for y in 0..a.height() {
        for x in 0..a.width() {
            if x < STRIP + 4 && y < SWATCH + 4 {
                continue; // the swatch of the guard
            }
            let (pa, pb) = (a.get_pixel(x, y).0, b.get_pixel(x, y).0);
            if dist(pa, ga) > 40 || dist(pb, gb) > 40 {
                union += 1;
                if dist(pa, pb) > 48 {
                    changed += 1;
                }
            }
        }
    }
    if union == 0 {
        0.0
    } else {
        changed as f32 / union as f32
    }
}

fn ink(img: &RgbaImage) -> u32 {
    let g = ground_of(img);
    img.pixels().filter(|p| dist(p.0, g) > 40).count() as u32
}

/// All pictures: (scene, theme tag) to picture.
fn pictures() -> BTreeMap<(&'static str, String), RgbaImage> {
    let css = scenes::css();
    let mut all = BTreeMap::new();
    for (name, w, h, build) in SCENES {
        for (id, dark) in THEMES {
            let img = draw_styled(build, w, h, id, dark, &css, &format!("{name}.png"));
            all.insert((name, tag(id, dark)), img);
        }
    }
    all
}

#[test]
fn every_block_is_drawn_in_every_style_and_changes_between_styles() {
    let _g = serial();
    let all = pictures(); // the theme guard ran for each picture
    // 1. every picture holds a block (ink beyond the swatch)
    for ((scene, theme), img) in &all {
        assert!(
            ink(img) > 1_200,
            "{scene} in {theme}: nearly empty ({} ink pixels)",
            ink(img)
        );
    }
    // 1b. nothing is cut off: the last rows and columns of a picture are ground (a style with more padding needs a taller scene)
    for ((scene, theme), img) in &all {
        let g = ground_of(img);
        let (w, h) = (img.width(), img.height());
        let edge = (0..w).map(|x| (x, h - 3)).chain((0..h).map(|y| (w - 3, y)));
        let cut = edge
            .filter(|&(x, y)| dist(img.get_pixel(x, y).0, g) > 40)
            .count();
        assert_eq!(
            cut, 0,
            "{scene} in {theme}: the picture cuts the block off ({cut} edge pixels)"
        );
    }
    // 2. two styles. Must differ (red when not): Fiori (admin) and Windows light, the light and the dark appearance of a style,
    //    Craft against Fiori, Fiori against Simple. The other pairs are measured and reported (Windows and Apple are close
    //    cousins in a light dialog: small corners, the same scrim).
    let must = [
        ("admin-light", "windows-light"),
        ("admin-light", "simple-light"),
        ("craft-dark", "admin-light"),
        ("admin-light", "admin-dark"),
        ("windows-light", "windows-dark"),
        ("simple-light", "simple-dark"),
        ("apple-light", "apple-dark"),
    ];
    let report = [
        ("admin-light", "apple-light"),
        ("windows-light", "apple-light"),
        ("simple-light", "apple-light"),
        ("craft-dark", "windows-dark"),
    ];
    let mut worst = (f32::MAX, String::new());
    let mut table = String::from("scene | pair | r | kind\n");
    for (scene, ..) in SCENES {
        for (a, b) in must.into_iter().chain(report) {
            let r = changed_share(&all[&(scene, a.to_owned())], &all[&(scene, b.to_owned())]);
            let kind = if must.contains(&(a, b)) {
                "must  "
            } else {
                "report"
            };
            println!("MEASURE zwei-stile {kind} {scene:13} {a:14} / {b:14} r = {r:.2}");
            table.push_str(&format!("{scene} | {a} / {b} | {r:.2} | {}\n", kind.trim()));
            if must.contains(&(a, b)) {
                if r < worst.0 {
                    worst = (r, format!("{scene} {a} / {b}"));
                }
                assert!(
                    r >= 0.25,
                    "RED zwei-stile: {scene} does not hang on the style ({a} / {b}): r = {r:.2}"
                );
            }
        }
    }
    if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
        std::fs::write(std::path::Path::new(&dir).join("zwei-stile.txt"), &table)
            .expect("zwei-stile.txt");
    }
    println!(
        "MEASURE zwei-stile lowest r of the pairs that must differ = {:.2} ({})",
        worst.0, worst.1
    );
    // 3. counter-check: the same picture twice is "unchanged" (r would be red); a different scene is changed
    let one = &all[&("switch", "admin-light".to_owned())];
    assert!(
        changed_share(one, one) < 0.01,
        "the check does not see an unchanged picture"
    );
    assert!(
        changed_share(one, &all[&("toast", "admin-light".to_owned())]) > 0.25 || one.width() != 420
    );
}

#[test]
fn the_theme_guard_catches_a_wrong_theme() {
    let _g = serial();
    // drawn in Windows light, claimed to be Craft (or Windows dark): the references of the guard are those of the claim
    let css = scenes::css();
    for claimed in [
        (StyleId::Craft, true),
        (StyleId::Windows, true),
        (StyleId::Admin, false),
    ] {
        let tree = styled::layout_styled_as(
            scenes::SCENES[5].3,
            360.0,
            240.0,
            (StyleId::Windows, false),
            claimed,
            &css,
        );
        let img = common::draw_tree(&tree, 360, 240, "guard-probe.png");
        let result = std::panic::catch_unwind(|| styled::guard(&img, claimed.0, claimed.1));
        assert!(
            result.is_err(),
            "the guard let a Windows light picture pass as {claimed:?}"
        );
    }
    let right = draw_styled(
        scenes::SCENES[5].3,
        360,
        240,
        StyleId::Windows,
        false,
        &css,
        "guard-probe.png",
    );
    styled::guard(&right, StyleId::Windows, false);
}

#[test]
fn every_variable_the_new_sheets_read_exists() {
    let _g = serial();
    common::init_runtime();
    let theme = blinc_theme::ThemeState::get().to_css_variable_map();
    let own: Vec<String> = fs_components_styles::styles::block_tokens::NAMES
        .iter()
        .map(|n| (*n).to_owned())
        .collect();
    let others = ["control-border", "code-bg"]; // control_tokens.rs, in every bundle
    let sheet = scenes::css();
    let mut missing = Vec::new();
    for part in sheet.split("var(--").skip(1) {
        let name: String = part
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        if !theme.contains_key(&name) && !own.contains(&name) && !others.contains(&name.as_str()) {
            missing.push(name);
        }
    }
    missing.sort();
    missing.dedup();
    assert!(missing.is_empty(), "variables nobody defines: {missing:?}");
}
