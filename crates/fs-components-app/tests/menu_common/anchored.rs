//! A surface with one dropdown whose trigger stands where the test wants it.
//!
//! Tests: Test fixture.

#![allow(dead_code)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use blinc_core::Color;
use blinc_layout::div::div;
use blinc_layout::text;
use fs_components_app::{Element, Surface};
use fs_components_widgets::dropdown::{Align, dropdown};

/// Where the trigger stands in the surface, and what the last line counts.
pub struct Anchored {
    pub at: Arc<Mutex<(f32, f32)>>,
    pub chosen: Arc<AtomicUsize>,
    pub lines: usize,
}

impl Surface for Anchored {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        let (x, y) = *self.at.lock().unwrap();
        let chosen = Arc::clone(&self.chosen);
        let mut menu = dropdown(|_| trigger()).align(Align::Start).width(220.0);
        for line in 0..self.lines - 1 {
            menu = menu.item(format!("Line {line}"), || {});
        }
        let menu = menu.item("Last line", move || {
            chosen.fetch_add(1, Ordering::SeqCst);
        });
        Box::new(
            div()
                .w(width)
                .h(height)
                .relative()
                .child(div().absolute().left(x).top(y).child(menu.into_element())),
        )
    }
}

/// The visible trigger: a button with a label, 60 x 28 px (the size the position checks count on).
fn trigger() -> blinc_layout::div::Div {
    div()
        .id("anchor")
        .w(60.0)
        .h(28.0)
        .rounded(6.0)
        .border(1.0, Color::rgba(0.35, 0.40, 0.50, 1.0))
        .bg(Color::rgba(0.88, 0.90, 0.94, 1.0))
        .flex_row()
        .items_center()
        .justify_center()
        .child(
            text("Menu")
                .size(13.0)
                .color(Color::rgba(0.10, 0.12, 0.18, 1.0))
                .no_wrap()
                .no_cursor()
                .pointer_events_none(),
        )
}
