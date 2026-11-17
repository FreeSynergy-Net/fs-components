//! The File menu of the command model as pictures: open, with the submenu, separators, tick, radio dot, symbols, shortcuts and a
//! disabled line with its reason. Light and dark (run with `FS_RENDER_SCHEME=dark` for dark), German and English.
//!
//! Tests: Pixel tests - skipped (with a message) where there is no graphics adapter. Pictures stay in `FS_COMP_SHOTS` if that is set.

#[path = "menu_common/mod.rs"]
mod menu_common;
#[path = "menu_model_common/mod.rs"]
mod menu_model_common;

use std::path::PathBuf;

use fs_components_render::{Canvas, RenderError, draw_to_png};
use menu_common::{Harness, composed};
use menu_model_common::app;

const SIZE: (u32, u32) = (760, 560);

fn scheme() -> &'static str {
    if std::env::var("FS_RENDER_SCHEME").is_ok_and(|v| v == "dark") {
        "dark"
    } else {
        "light"
    }
}

fn shoot(german: bool) {
    let mut harness = Harness::new(app(german), 760.0, 560.0);
    harness.click("fs-app-menu-0");
    harness.click("fs-app-menu-entry-fs.submenu.9");
    assert!(harness.has("fs-app-menu-sub-panel"));
    let dir = std::env::var_os("FS_COMP_SHOTS").map_or_else(
        || std::env::temp_dir().join(format!("fs-app-m1-{}", std::process::id())),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir).unwrap();
    let name = format!(
        "menu-model-{}-{}.png",
        scheme(),
        if german { "de" } else { "en" }
    );
    let path = dir.join(&name);
    let canvas = Canvas::new(SIZE.0, SIZE.1, harness.app.styles());
    match draw_to_png(&canvas, |w, h| composed(&harness.app, w, h), || {}, &path) {
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

#[test]
fn the_model_menu_in_english() {
    shoot(false);
}

#[test]
fn the_model_menu_in_german() {
    shoot(true);
}
