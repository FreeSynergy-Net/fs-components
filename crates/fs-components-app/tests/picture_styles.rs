//! `AppWidget::picture` draws with the styles a window has (Blinc's default look of its widgets ahead of the rules of the theme),
//! so the picture of a program shows what its window shows (finding of the 0.0.13 follow-ups, 2026-11-14: the pictures of the
//! programs were drawn with other styles than the real window, "open edge").
//!
//! Test: a surface with a real checkbox of the theme library; the box has its frame only with the styles of a window.

use std::sync::Arc;

use blinc_cn::components::checkbox::{CheckboxBuilder, CheckboxSize};
use blinc_core::reactive::signal;
use blinc_layout::div::div;
use fs_components_app::{AppError, AppWidget, Element, Screen, Surface};
use fs_components_render::RenderError;

struct Boxed;

impl Surface for Boxed {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        let state = signal(false);
        Box::new(div().w(width).h(height).p(20.0).child(
            CheckboxBuilder::with_key("picture-styles-box", state).size(CheckboxSize::Large),
        ))
    }
}

/// Pixels that are not the colour of the page in the body (around the box).
fn ink(img: &image::RgbaImage) -> usize {
    let page = *img.get_pixel(400, 250);
    (100..170)
        .flat_map(|y| (60..160).map(move |x| (x, y)))
        .filter(|&(x, y)| img.get_pixel(x, y)[0].abs_diff(page[0]) > 25)
        .count()
}

#[test]
fn the_picture_has_the_look_of_the_widgets_a_window_has() {
    let app = AppWidget::new("Own", vec![Screen::new("own", "Own", Arc::new(Boxed))]).unwrap();
    let dir = std::env::temp_dir().join(format!("fs-pic-styles-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("boxed.png");
    match app.picture(600, 400, &file) {
        Err(AppError::Picture(RenderError::NoGraphics(why))) => {
            eprintln!("skipped: {why}");
            return;
        }
        other => other.unwrap(),
    }
    let img = image::open(&file).unwrap().to_rgba8();
    if let Some(shots) = std::env::var_os("FS_COMP_SHOTS") {
        std::fs::create_dir_all(&shots).unwrap();
        std::fs::copy(
            &file,
            std::path::Path::new(&shots).join("app-picture-styles.png"),
        )
        .unwrap();
    }
    let _ = std::fs::remove_dir_all(&dir);
    let found = ink(&img);
    assert!(
        found > 110,
        "the box of the checkbox is drawn with its frame and its fill ({found} px)"
    );
}
