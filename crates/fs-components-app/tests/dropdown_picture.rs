//! The dropdown at the bottom of the window as a picture: the panel stands above its trigger, inside the window.
//!
//! Tests: Pixel tests — skipped (with a message) where there is no graphics adapter. Pictures stay in `FS_COMP_SHOTS` if that is
//! set (scene `dropdown-unten`, one per style).

#[path = "menu_common/mod.rs"]
mod menu_common;

use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use fs_components_app::{AppWidget, Screen};
use fs_components_render::{Canvas, RenderError, draw_to_png};
use fs_components_styles::StyleId;
use menu_common::anchored::Anchored;
use menu_common::{Harness, composed};

const SIZE: (u32, u32) = (800, 500);

#[test]
fn a_menu_at_the_bottom_of_the_window_is_drawn_above_its_trigger() {
    for style in [StyleId::Craft, StyleId::Apple] {
        fs_components_render::use_style(Some(style));
        let surface = Anchored {
            at: Arc::new(Mutex::new((40.0, 380.0))),
            chosen: Arc::new(AtomicUsize::new(0)),
            lines: 4,
        };
        let app = AppWidget::new("Title", vec![Screen::new("a", "A", Arc::new(surface))]).unwrap();
        let mut h = Harness::new(app, 800.0, 500.0);
        h.click("anchor");
        let dir = std::env::var_os("FS_COMP_SHOTS").map_or_else(
            || std::env::temp_dir().join(format!("fs-dropdown-{}", std::process::id())),
            PathBuf::from,
        );
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("dropdown-unten-{style:?}.png").to_lowercase());
        let canvas = Canvas::new(SIZE.0, SIZE.1, h.app.styles());
        match draw_to_png(&canvas, |w, h2| composed(&h.app, w, h2), || {}, &path) {
            Err(RenderError::NoGraphics(why)) => eprintln!("skipped: {why}"),
            other => other.unwrap(),
        }
        if std::env::var_os("FS_COMP_SHOTS").is_none() {
            let _ = std::fs::remove_file(&path);
        }
    }
}
