//! WCAG contrast of the new parts in every style, light and dark: measured on the pictures, not on the tokens.
//!
//! Tests: draws the scenes, finds each part by its element id, reads the pixel of its ground, its edge or its mark, and computes the
//! WCAG 2.x ratio against what it stands on. Text needs 4.5:1 (3:1 on the accent, where Apple's own blue gives 3.5, the rule of
//! `fs-components-styles` tests), edges and marks of controls 3:1 (1.4.11), the placeholder bars of a skeleton are decoration (only
//! reported). The table is written to `kontrast.txt` in the folder of `FS_COMP_SHOTS` when it is set.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;
mod scenes;
#[path = "../../fs-components-widgets/tests/common/styled.rs"]
mod styled;

use blinc_layout::prelude::RenderTree;
use common::serial;
use fs_components_styles::{Rgba, StyleId, style};
use image::RgbaImage;
use scenes::THEMES;
use styled::{draw_styled_tree, ground_of, layout_styled};

fn bounds(tree: &RenderTree, id: &str) -> (f32, f32, f32, f32) {
    let node = tree
        .query_by_id(id)
        .unwrap_or_else(|| panic!("no element {id}"));
    let b = tree.get_absolute_bounds(node).expect("bounds");
    (b.x, b.y, b.width, b.height)
}

fn at(img: &RgbaImage, x: f32, y: f32) -> Rgba {
    let p = img
        .get_pixel(
            (x.max(0.0) as u32).min(img.width() - 1),
            (y.max(0.0) as u32).min(img.height() - 1),
        )
        .0;
    Rgba {
        r: p[0],
        g: p[1],
        b: p[2],
        a: 255,
    }
}

fn px(p: [u8; 4]) -> Rgba {
    Rgba {
        r: p[0],
        g: p[1],
        b: p[2],
        a: 255,
    }
}

struct Row {
    theme: String,
    what: String,
    ratio: f32,
    need: f32,
}

fn shot(scene: &str, id: StyleId, dark: bool) -> (RenderTree, RgbaImage) {
    let (_, w, h, build) = *scenes::SCENES.iter().find(|s| s.0 == scene).expect("scene");
    let tree = layout_styled(build, w as f32, h as f32, id, dark, &scenes::css());
    let img = draw_styled_tree(&tree, w, h, id, dark, &format!("{scene}-contrast.png"));
    (tree, img)
}

fn measure(id: StyleId, dark: bool, rows: &mut Vec<Row>) {
    let theme = format!(
        "{}-{}",
        id.name(),
        if dark || id == StyleId::Craft {
            "dark"
        } else {
            "light"
        }
    );
    let p = *style(id).palette(dark || id == StyleId::Craft);
    let mut add = |what: &str, ratio: f32, need: f32| {
        rows.push(Row {
            theme: theme.clone(),
            what: what.to_owned(),
            ratio,
            need,
        })
    };
    // messages: text on the tinted ground, the coloured edge against the page
    let (tree, img) = shot("toast", id, dark);
    let page = px(ground_of(&img));
    for (n, kind) in ["info", "ok", "warning", "error"].iter().enumerate() {
        let (x, y, w, h) = bounds(&tree, &format!("ts-{}", n + 1));
        let ground = at(&img, x + w / 2.0, y + 3.0);
        add(
            &format!("toast/{kind} text on its ground"),
            p.text.contrast(ground),
            4.5,
        );
        if scenes_edge(id) >= 2.0 {
            add(
                &format!("toast/{kind} edge against the page"),
                at(&img, x + 1.0, y + h / 2.0).contrast(page),
                3.0,
            );
        }
    }
    let (tree, img) = shot("states", id, dark);
    for part in ["a-failed", "a-info", "a-ok", "a-warn", "pn"] {
        let (x, y, w, _) = bounds(&tree, part);
        add(
            &format!("alert {part} text on its ground"),
            p.text.contrast(at(&img, x + w / 2.0, y + 3.0)),
            4.5,
        );
    }
    let (x, y, w, _) = bounds(&tree, "pn-text");
    add(
        "partial-notice item text on its ground",
        p.text.contrast(at(&img, x + w + 4.0, y)),
        4.5,
    );
    // switch: the track edge and the thumb
    let (tree, img) = shot("switch", id, dark);
    let page = px(ground_of(&img));
    for (sw, state) in [("s1", "off"), ("s2", "on"), ("s3", "mixed")] {
        let (x, y, _, h) = bounds(&tree, &format!("{sw}-track"));
        let edge = at(&img, x, y + h / 2.0);
        add(
            &format!("switch {state}: track edge against the page"),
            edge.contrast(page),
            3.0,
        );
        let (tx, ty, tw, th) = bounds(&tree, &format!("{sw}-thumb"));
        let thumb = at(&img, tx + tw / 2.0, ty + th / 2.0);
        let track = if state == "on" { p.accent } else { p.field };
        add(
            &format!("switch {state}: thumb against the track"),
            thumb.contrast(track),
            3.0,
        );
    }
    // segmented: the chosen segment's text on the accent, the others' on the surface, the outer edge
    let (tree, img) = shot("segmented", id, dark);
    let page = px(ground_of(&img));
    let (x, y, _, h) = bounds(&tree, "g1-user");
    // 3:1 on the accent only for Apple (its own blue, the rule of the style tests); every other style keeps 4.5:1
    let need = if id == StyleId::Apple { 3.0 } else { 4.5 };
    add(
        "segmented chosen: text on the accent",
        p.on_accent.contrast(at(&img, x + 4.0, y + h / 2.0)),
        need,
    );
    let (x, y, _, h) = bounds(&tree, "g1-system");
    add(
        "segmented other: text on the ground",
        p.text.contrast(at(&img, x + 4.0, y + h / 2.0)),
        4.5,
    );
    let (x, y, _, h2) = bounds(&tree, "g1");
    add(
        "segmented: outer edge against the page",
        at(&img, x, y + h2 / 2.0).contrast(page),
        3.0,
    );
    // choice: the ring of the mark, the chosen option's edge
    let (tree, img) = shot("choice", id, dark);
    let (x, y, _, h) = bounds(&tree, "ch-opt-mono");
    add(
        "choice: chosen option edge against the dialog",
        at(&img, x, y + h / 2.0).contrast(p.panel),
        3.0,
    );
    let (x, y, _, h) = bounds(&tree, "ch-mark-dejavu");
    add(
        "choice: ring of an unchosen mark against the dialog",
        at(&img, x, y + h / 2.0).contrast(p.panel),
        3.0,
    );
    // skeleton: reported only
    let (tree, img) = shot("skeleton", id, dark);
    let page = px(ground_of(&img));
    let (x, y, w, h) = bounds(&tree, "sk1-bar-0-0");
    add(
        "skeleton bar against the page (decoration, reported)",
        at(&img, x + w / 2.0, y + h / 2.0).contrast(page),
        1.1,
    );
}

fn scenes_edge(id: StyleId) -> f32 {
    fs_components_styles::styles::block_tokens::look(id).edge
}

#[test]
fn the_new_parts_are_readable_in_every_style() {
    let _g = serial();
    let mut rows = Vec::new();
    for (id, dark) in THEMES {
        measure(id, dark, &mut rows);
    }
    let mut table = String::from("theme | part | ratio | needs | result\n");
    for r in &rows {
        let ok = r.ratio >= r.need;
        table.push_str(&format!(
            "{} | {} | {:.2} | {:.1} | {}\n",
            r.theme,
            r.what,
            r.ratio,
            r.need,
            if ok { "ok" } else { "RED" }
        ));
    }
    println!("{table}");
    if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
        std::fs::write(std::path::Path::new(&dir).join("kontrast.txt"), &table)
            .expect("kontrast.txt");
    }
    let red: Vec<String> = rows
        .iter()
        .filter(|r| r.ratio < r.need)
        .map(|r| format!("{} {} {:.2} < {:.1}", r.theme, r.what, r.ratio, r.need))
        .collect();
    assert!(
        red.is_empty(),
        "contrast below the line:\n{}",
        red.join("\n")
    );
}
