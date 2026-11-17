//! The context menu as pictures (M3): in the middle, at the edge below right (flipped), the second stage, light and dark.
//! Run with `FS_RENDER_SCHEME=dark` for dark and `FS_COMP_SHOTS=<dir>` to keep the pictures.
//!
//! Tests: Pixel tests - skipped (with a message) where there is no graphics adapter. Without `FS_COMP_SHOTS` the pictures are removed.

#[path = "context_common/mod.rs"]
mod context_common;
#[path = "menu_common/mod.rs"]
mod menu_common;

use std::path::PathBuf;

use context_common::{Rig, files, rig, right_click, shift_right_click};
use fs_components_commands::Trigger;
use fs_components_render::{Canvas, RenderError, draw_to_png};
use menu_common::composed;

const SIZE: (u32, u32) = (800, 500);

fn scheme() -> &'static str {
    if std::env::var("FS_RENDER_SCHEME").is_ok_and(|v| v == "dark") {
        "dark"
    } else {
        "light"
    }
}

fn shoot(r: &Rig, scene: &str) {
    let dir = std::env::var_os("FS_COMP_SHOTS").map_or_else(
        || std::env::temp_dir().join(format!("fs-app-m3-{}", std::process::id())),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("context-{scene}-{}.png", scheme()));
    let canvas = Canvas::new(SIZE.0, SIZE.1, r.h.app.styles());
    match draw_to_png(&canvas, |w, h| composed(&r.h.app, w, h), || {}, &path) {
        Err(RenderError::NoGraphics(why)) => eprintln!("skipped: {why}"),
        other => {
            other.unwrap();
            let picture = image::open(&path).unwrap().to_rgba8();
            let colours: std::collections::HashSet<_> = picture.pixels().map(|p| p.0).collect();
            assert!(colours.len() > 20, "something was drawn");
            if std::env::var_os("FS_COMP_SHOTS").is_none() {
                std::fs::remove_file(&path).unwrap();
            }
        }
    }
}

fn scene(trigger: &Trigger, name: &str, sub: bool) {
    let mut r = rig(800.0, 500.0);
    r.open(trigger, files(&["a"]));
    if sub {
        r.h.click("fs-app-menu-entry-fs.submenu.3");
    }
    shoot(&r, name);
}

#[test]
fn in_the_middle() {
    scene(&right_click(320.0, 180.0), "middle", false);
}

#[test]
fn at_the_edge_below_right_it_flips() {
    scene(&right_click(770.0, 470.0), "corner-flipped", false);
}

#[test]
fn the_second_stage_by_shift_right_click() {
    scene(&shift_right_click(320.0, 120.0), "second-stage", false);
}

#[test]
fn the_submenu_more_options_open() {
    scene(&right_click(320.0, 120.0), "more-options", true);
}
