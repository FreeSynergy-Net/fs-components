//! `range`: two thumbs with their own names and limits that follow each other, the keys on each thumb (window keys AND raw), the pointer
//! (the nearer thumb is grabbed, a drag moves it, a thumb never passes the other), the callback.
//!
//! Tests: attribute reads on the built element; an in-process keyboard and pointer.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::prelude::RenderTree;
use common::{Input, attr, frame, layout_with, serial};
use fs_components_widgets::keys::{Key, KeyEvent};
use fs_components_widgets::range::{Range, Which};
use fs_components_widgets::slider::{STYLES, Scale, THUMB};

type Log = Arc<Mutex<Vec<(f64, f64)>>>;

fn range(log: &Log) -> Range {
    let l = Arc::clone(log);
    Range::new(
        Scale::new(0.0, 100.0, 5.0).page(20.0),
        "Lowest price",
        "Highest price",
    )
    .id("rg")
    .values(20.0, 80.0)
    .on_change(move |a, b| l.lock().unwrap().push((a, b)))
}

fn show(r: &Range) -> RenderTree {
    layout_with(r.build(400.0), 600.0, 80.0, STYLES)
}

#[test]
fn which_thumb_is_held_back_by_the_other() {
    assert!(
        (Which::Low.clamp(90.0, (20.0, 80.0)) - 80.0).abs() < 1e-9,
        "the low thumb stops at the high one"
    );
    assert!((Which::Low.clamp(10.0, (20.0, 80.0)) - 10.0).abs() < 1e-9);
    assert!(
        (Which::High.clamp(5.0, (20.0, 80.0)) - 20.0).abs() < 1e-9,
        "the high thumb stops at the low one"
    );
    assert!((Which::High.clamp(95.0, (20.0, 80.0)) - 95.0).abs() < 1e-9);
}

#[test]
fn each_thumb_is_a_named_slider_whose_limit_is_the_other_value() {
    let _g = serial();
    let tree = show(&range(&Log::default()));
    assert_eq!(attr(&tree, "rg-low", "role").as_deref(), Some("slider"));
    assert_eq!(attr(&tree, "rg-high", "role").as_deref(), Some("slider"));
    assert_eq!(
        attr(&tree, "rg-low", "aria-label").as_deref(),
        Some("Lowest price")
    );
    assert_eq!(
        attr(&tree, "rg-high", "aria-label").as_deref(),
        Some("Highest price")
    );
    assert_eq!(
        attr(&tree, "rg-low", "aria-valuenow").as_deref(),
        Some("20")
    );
    assert_eq!(
        attr(&tree, "rg-high", "aria-valuenow").as_deref(),
        Some("80")
    );
    assert_eq!(attr(&tree, "rg-low", "aria-valuemin").as_deref(), Some("0"));
    assert_eq!(
        attr(&tree, "rg-low", "aria-valuemax").as_deref(),
        Some("80"),
        "the low thumb's maximum is the high value"
    );
    assert_eq!(
        attr(&tree, "rg-high", "aria-valuemin").as_deref(),
        Some("20"),
        "the high thumb's minimum is the low value"
    );
    assert_eq!(
        attr(&tree, "rg-high", "aria-valuemax").as_deref(),
        Some("100")
    );
    assert_eq!(
        attr(&tree, "rg-low", "tabindex").as_deref(),
        Some("0"),
        "each thumb is a tab stop"
    );
    assert_eq!(attr(&tree, "rg-high", "tabindex").as_deref(), Some("0"));
}

#[test]
fn the_values_start_ordered_and_snapped() {
    let _g = serial();
    let r = Range::new(Scale::new(0.0, 100.0, 5.0), "a", "b").values(90.0, 12.0);
    assert_eq!(
        r.get(),
        (12.0 - 2.0, 90.0),
        "given the wrong way round and off the step: ordered and snapped"
    );
}

#[test]
fn the_window_keys_move_the_focused_thumb_and_stop_at_the_other_thumb() {
    let _g = serial();
    let log = Log::default();
    let r = range(&log);
    let mut tree = show(&r);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "rg-low"));
    input.key(&mut tree, 39);
    assert_eq!(r.get(), (25.0, 80.0));
    input.key(&mut tree, 35);
    assert_eq!(r.get(), (80.0, 80.0), "End stops at the high thumb");
    input.key(&mut tree, 39);
    assert_eq!(r.get(), (80.0, 80.0), "no change, nothing heard");
    frame(&mut tree, 600.0, 80.0);
    assert!(input.tab_to(&mut tree, "rg-high"));
    input.key(&mut tree, 37);
    assert_eq!(
        r.get(),
        (80.0, 80.0),
        "the high thumb cannot go below the low one"
    );
    input.key(&mut tree, 36);
    assert_eq!(
        r.get(),
        (80.0, 80.0),
        "Home on the high thumb stops at the low one"
    );
    input.key(&mut tree, 35);
    assert_eq!(r.get(), (80.0, 100.0));
    assert_eq!(
        *log.lock().unwrap(),
        [(25.0, 80.0), (80.0, 80.0), (80.0, 100.0)]
    );
    frame(&mut tree, 600.0, 80.0);
    assert_eq!(
        attr(&tree, "rg-high", "aria-valuenow").as_deref(),
        Some("100")
    );
    assert_eq!(
        attr(&tree, "rg-low", "aria-valuemax").as_deref(),
        Some("100"),
        "the limit follows"
    );
}

#[test]
fn raw_key_events_and_other_keys_on_a_thumb() {
    let _g = serial();
    let r = range(&Log::default());
    let mut tree = show(&r);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "rg-low"));
    input.key_event_to(&mut tree, "rg-low", KEY_DOWN, 38);
    assert_eq!(r.get(), (25.0, 80.0), "a raw Up");
    input.key_event_to(&mut tree, "rg-low", KEY_DOWN, 32);
    input.key_event_to(&mut tree, "rg-low", KEY_DOWN, 13);
    assert_eq!(r.get(), (25.0, 80.0), "Space and Enter do nothing");
    assert!(!r.press(Which::Low, KeyEvent::new(Key::Enter)));
    assert!(r.press(Which::High, KeyEvent::new(Key::PageDown)));
    assert_eq!(r.get(), (25.0, 60.0), "a page down on the high thumb");
}

#[test]
fn the_pointer_grabs_the_nearer_thumb_and_a_drag_moves_it_but_not_past_the_other() {
    let _g = serial();
    let log = Log::default();
    let r = range(&log);
    let mut tree = show(&r);
    let mut input = Input::new();
    let node = tree.query_by_id("rg-track").expect("the track");
    let b = tree.get_absolute_bounds(node).expect("bounds");
    let run = 400.0 - THUMB * 2.0;
    let low_at = |f: f32| (b.x + THUMB / 2.0 + f * run, b.y + b.height / 2.0);
    let high_at = |f: f32| (b.x + THUMB * 1.5 + f * run, b.y + b.height / 2.0);
    let (x, y) = low_at(0.1);
    input.press_at(&mut tree, x, y);
    assert_eq!(
        r.get(),
        (10.0, 80.0),
        "pressed nearer the low thumb: it moves"
    );
    let (x, y) = low_at(0.4);
    input.move_to(&mut tree, x, y);
    assert_eq!(r.get(), (40.0, 80.0), "dragged");
    let (x, y) = low_at(0.95);
    input.move_to(&mut tree, x, y);
    assert_eq!(
        r.get(),
        (80.0, 80.0),
        "dragged past the high thumb: it stops there"
    );
    input.release_at(&mut tree, x, y);
    frame(&mut tree, 600.0, 80.0);
    let (x, y) = high_at(0.9);
    input.press_at(&mut tree, x, y);
    assert_eq!(
        r.get(),
        (80.0, 90.0),
        "pressed nearer the high thumb: it moves"
    );
    input.release_at(&mut tree, x, y);
}

#[test]
fn the_keyboard_stays_on_a_thumb_across_redraws_and_the_attributes_follow_each_time() {
    let _g = serial();
    let r = range(&Log::default());
    let mut tree = show(&r);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "rg-low"));
    for want in ["25", "30", "35"] {
        input.key(&mut tree, 39);
        frame(&mut tree, 600.0, 80.0);
        assert!(input.has_keyboard(&tree, "rg-low"));
        assert_eq!(
            attr(&tree, "rg-low", "aria-valuenow").as_deref(),
            Some(want)
        );
    }
}

#[test]
fn the_drawn_thumb_of_the_thumb_that_has_the_keyboard_gets_the_focus_class() {
    let _g = serial();
    let r = range(&Log::default());
    let mut tree = show(&r);
    let has = |tree: &RenderTree, id: &str| {
        let n = tree.query_by_id(id).expect("a drawn thumb");
        tree.element_registry()
            .has_class(n, "fs-slider-thumb-focus")
    };
    assert!(!has(&tree, "rg-low-thumb") && !has(&tree, "rg-high-thumb"));
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "rg-low"));
    input.focus_events(&mut tree, "rg-low", true);
    frame(&mut tree, 600.0, 80.0);
    assert!(has(&tree, "rg-low-thumb"), "the low thumb has the ring");
    assert!(!has(&tree, "rg-high-thumb"));
    input.focus_events(&mut tree, "rg-low", false);
    input.focus_events(&mut tree, "rg-high", true);
    frame(&mut tree, 600.0, 80.0);
    assert!(has(&tree, "rg-high-thumb"), "then the high thumb");
    assert!(!has(&tree, "rg-low-thumb"));
    input.focus_events(&mut tree, "rg-high", false);
    frame(&mut tree, 600.0, 80.0);
    assert!(
        !has(&tree, "rg-low-thumb") && !has(&tree, "rg-high-thumb"),
        "none when the keyboard left"
    );
}
