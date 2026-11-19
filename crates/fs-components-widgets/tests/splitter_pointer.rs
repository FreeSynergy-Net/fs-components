//! `splitter` (pointer): a drag of the divider by the pixels dragged, held to the limits, and the active class while it is held.
//!
//! Tests: an in-process pointer (press, move, release) and attribute reads on the built element.

mod common;

use std::sync::{Arc, Mutex};

use blinc_layout::div::div;
use blinc_layout::prelude::{RenderTree, text};
use common::{Input, attr, frame, layout_with, serial};
use fs_components_widgets::splitter::{STYLES, Splitter};

type Log = Arc<Mutex<Vec<String>>>;

fn splitter(log: &Log) -> Splitter {
    let l = Arc::clone(log);
    Splitter::new("Sidebar width", 40.0, 10.0, 80.0, 5.0)
        .id("sp")
        .on_move(move |p| l.lock().unwrap().push(format!("{p:.0}")))
}

fn show(s: &Splitter) -> RenderTree {
    layout_with(
        s.build(
            508.0,
            100.0,
            || div().child(text("first")),
            || div().child(text("second")),
        ),
        600.0,
        140.0,
        STYLES,
    )
}

fn seen(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

#[test]
fn a_drag_of_the_divider_moves_it_by_the_pixels_dragged_and_holds_it_to_the_limits() {
    let _g = serial();
    let log = Log::default();
    let s = splitter(&log);
    let mut tree = show(&s);
    let mut input = Input::new();
    let (x, y) = Input::centre(&tree, "sp-bar");
    input.press_at(&mut tree, x, y);
    frame(&mut tree, 600.0, 140.0);
    input.move_to(&mut tree, x + 50.0, y);
    frame(&mut tree, 600.0, 140.0);
    assert!(
        (s.position() - 50.0).abs() < 0.5,
        "50 of 500 pixels is 10 percent: {}",
        s.position()
    );
    assert_eq!(
        attr(&tree, "sp-bar", "aria-valuenow").as_deref(),
        Some("50"),
        "the attribute follows each frame"
    );
    input.move_to(&mut tree, x + 1000.0, y);
    frame(&mut tree, 600.0, 140.0);
    assert!((s.position() - 80.0).abs() < 1e-9, "held to the greatest");
    input.move_to(&mut tree, x - 1000.0, y);
    frame(&mut tree, 600.0, 140.0);
    assert!((s.position() - 10.0).abs() < 1e-9, "held to the least");
    input.release_at(&mut tree, x - 1000.0, y);
    assert_ne!(seen(&log), Vec::<String>::new());
    frame(&mut tree, 600.0, 140.0);
    assert_eq!(
        attr(&tree, "sp-bar", "aria-valuenow").as_deref(),
        Some("10")
    );
}

#[test]
fn the_active_class_is_there_while_the_divider_is_held() {
    let _g = serial();
    let s = splitter(&Log::default());
    let mut tree = show(&s);
    let mut input = Input::new();
    let has = |tree: &RenderTree| {
        let n = tree.query_by_id("sp-bar").expect("the divider");
        tree.element_registry()
            .has_class(n, "fs-splitter-bar-active")
    };
    assert!(!has(&tree));
    let (x, y) = Input::centre(&tree, "sp-bar");
    input.press_at(&mut tree, x, y);
    frame(&mut tree, 600.0, 140.0);
    assert!(has(&tree), "held");
    input.release_at(&mut tree, x, y);
    frame(&mut tree, 600.0, 140.0);
    assert!(!has(&tree), "let go");
}
