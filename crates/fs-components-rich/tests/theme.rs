//! The theme of a test: each test has its own (no style leaks from one test to another), and the guard that reads it from the
//! picture fails for a picture drawn in the wrong theme. Also what depends on the theme state: the width of the marker column and
//! the focus counter of the window.
//!
//! Tests: picture tests (a software graphics adapter and fonts are needed).

mod common;

use common::{Typist, assert_theme, own_theme, warm_up};
use fs_components_editor::Doc;

fn picture(style: &str) -> image::RgbaImage {
    let t = Typist::new(style, &Doc::new(vec![String::new()]), 200.0, 100.0);
    let file =
        std::env::temp_dir().join(format!("fs-rich-theme-{}-{style}.png", std::process::id()));
    t.picture(&file);
    let img = image::open(&file).expect("the picture").to_rgba8();
    let _ = std::fs::remove_file(&file);
    img
}

#[test]
fn the_guard_accepts_each_style_in_its_own_theme() {
    assert_theme(&picture("standard"), "standard");
    assert_theme(&picture("craft"), "craft");
}

#[test]
#[should_panic(expected = "another theme")]
fn the_guard_fails_a_craft_picture_asked_for_as_standard() {
    assert_theme(&picture("craft"), "standard");
}

#[test]
#[should_panic(expected = "another theme")]
fn the_guard_fails_a_standard_picture_asked_for_as_craft() {
    assert_theme(&picture("standard"), "craft");
}

#[test]
fn the_marker_column_is_about_28_px_in_both_styles() {
    let _own = own_theme();
    for style in ["standard", "craft"] {
        warm_up(style);
        let id = fs_components_styles::StyleId::from_name(style)
            .unwrap_or(fs_components_styles::StyleId::Admin);
        let m = fs_components_rich::Look::of(id).marker;
        assert!(
            (26.0..=30.0).contains(&m),
            "{style}: the marker column is {m}"
        );
    }
}

#[test]
fn a_surface_that_is_dropped_with_the_keyboard_gives_the_focus_back() {
    let _own = own_theme();
    assert!(!blinc_layout::widgets::text_input::has_focused_text_input());
    let surface = fs_components_rich::RichSurface::new(&Doc::new(vec![String::new()]));
    surface.set_focus(true);
    assert!(blinc_layout::widgets::text_input::has_focused_text_input());
    drop(surface);
    assert!(
        !blinc_layout::widgets::text_input::has_focused_text_input(),
        "the focus count was left standing"
    );
}
