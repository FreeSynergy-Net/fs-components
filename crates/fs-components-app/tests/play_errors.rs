//! `AppWidget::play` without a graphics adapter: the errors that come before any drawing.
//!
//! Pattern: none (tests).
#![cfg(feature = "play")]

use std::sync::Arc;

use fs_components_app::replay::parse;
use fs_components_app::{AppError, AppWidget, Element, Screen, Surface};
use fs_components_render::RenderError;

struct Empty;

impl Surface for Empty {
    fn build(&self, _w: f32, _h: f32) -> Box<dyn Element> {
        Box::new(blinc_layout::div::div())
    }
}

#[test]
fn a_size_of_zero_is_an_error_not_a_panic() {
    let app = AppWidget::new("t", vec![Screen::new("a", "A", Arc::new(Empty))]).unwrap();
    let plan = parse(r#"{"actions":[]}"#).unwrap();
    let path = std::env::temp_dir().join("fs-app-play-zero.png");
    let e = app.play(&plan, 0, 100, &path).unwrap_err();
    assert_eq!(e, AppError::Picture(RenderError::EmptyCanvas));
    assert!(!path.exists());
}
