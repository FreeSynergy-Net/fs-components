//! `slider`: the scale (snap, fraction, page, keys), role and ARIA values on the thumb, the keys (arrows, Page Up and Down, Home, End,
//! as the window's keys AND raw), the pointer (press on the track, drag), the callback, the words, the active class.
//!
//! Tests: model tests over `Scale`; attribute reads on the built element; an in-process keyboard and pointer.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::prelude::RenderTree;
use common::{Input, attr, frame, layout_with, serial};
use fs_components_widgets::keys::Key;
use fs_components_widgets::slider::{STYLES, Scale, Slider, THUMB, number};

type Log = Arc<Mutex<Vec<String>>>;

fn slider(log: &Log) -> Slider {
    let l = Arc::clone(log);
    Slider::new(Scale::new(0.0, 100.0, 5.0).page(20.0), "Volume")
        .id("sl")
        .value(50.0)
        .on_change(move |v| l.lock().unwrap().push(number(v)))
}

fn show(s: &Slider) -> RenderTree {
    layout_with(s.build(300.0), 400.0, 80.0, STYLES)
}

fn seen(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

#[test]
fn a_scale_snaps_clamps_and_maps_a_fraction_both_ways() {
    let s = Scale::new(0.0, 100.0, 5.0);
    assert!((s.snap(52.0) - 50.0).abs() < 1e-9);
    assert!((s.snap(53.0) - 55.0).abs() < 1e-9);
    assert!((s.snap(-9.0) - 0.0).abs() < 1e-9 && (s.snap(900.0) - 100.0).abs() < 1e-9);
    assert!((s.fraction(25.0) - 0.25).abs() < 1e-9);
    assert!((s.at(0.5) - 50.0).abs() < 1e-9);
    assert!(
        (s.at(2.0) - 100.0).abs() < 1e-9 && (s.at(-1.0)).abs() < 1e-9,
        "a place off the track is held to it"
    );
    let odd = Scale::new(0.0, 10.0, 4.0);
    assert!(
        (odd.snap(10.0) - 10.0).abs() < 1e-9,
        "the greatest value is reachable though 10 is no multiple of 4"
    );
    let swapped = Scale::new(9.0, 1.0, 1.0);
    assert!((swapped.min() - 1.0).abs() < 1e-9 && (swapped.max() - 9.0).abs() < 1e-9);
    let zero_step = Scale::new(0.0, 50.0, 0.0);
    assert!(
        (zero_step.key(10.0, Key::Right).unwrap() - 10.5).abs() < 1e-9,
        "no step: a hundredth of the span"
    );
    assert!(
        (Scale::new(3.0, 3.0, 1.0).fraction(3.0)).abs() < 1e-9,
        "an empty span has no fraction"
    );
}

#[test]
fn the_keys_of_a_scale_are_the_apg_keys_and_nothing_else() {
    let s = Scale::new(0.0, 100.0, 5.0).page(20.0);
    let k = |v: f64, key: Key| s.key(v, key).map(f64::round);
    assert_eq!(k(50.0, Key::Right), Some(55.0));
    assert_eq!(k(50.0, Key::Up), Some(55.0));
    assert_eq!(k(50.0, Key::Left), Some(45.0));
    assert_eq!(k(50.0, Key::Down), Some(45.0));
    assert_eq!(k(50.0, Key::PageUp), Some(70.0));
    assert_eq!(k(50.0, Key::PageDown), Some(30.0));
    assert_eq!(k(50.0, Key::Home), Some(0.0));
    assert_eq!(k(50.0, Key::End), Some(100.0));
    assert_eq!(k(98.0, Key::PageUp), Some(100.0), "held to the end");
    assert_eq!(k(2.0, Key::Left), Some(0.0));
    assert_eq!(k(50.0, Key::Enter), None);
    assert_eq!(k(50.0, Key::Other(65)), None);
    assert_eq!(
        Scale::new(0.0, 100.0, 5.0)
            .page(1.0)
            .key(50.0, Key::PageUp)
            .map(f64::round),
        Some(55.0),
        "a page is never below a step"
    );
}

#[test]
fn the_thumb_is_a_named_slider_with_value_limits_and_words() {
    let _g = serial();
    let s = slider(&Log::default()).words(|v| format!("{v:.0} percent"));
    let tree = show(&s);
    assert_eq!(attr(&tree, "sl", "role").as_deref(), Some("slider"));
    assert_eq!(attr(&tree, "sl", "aria-label").as_deref(), Some("Volume"));
    assert_eq!(
        attr(&tree, "sl", "aria-orientation").as_deref(),
        Some("horizontal")
    );
    assert_eq!(attr(&tree, "sl", "aria-valuenow").as_deref(), Some("50"));
    assert_eq!(attr(&tree, "sl", "aria-valuemin").as_deref(), Some("0"));
    assert_eq!(attr(&tree, "sl", "aria-valuemax").as_deref(), Some("100"));
    assert_eq!(
        attr(&tree, "sl", "aria-valuetext").as_deref(),
        Some("50 percent")
    );
    assert_eq!(attr(&tree, "sl", "tabindex").as_deref(), Some("0"));
    assert_eq!(number(0.5), "0.5");
    assert_eq!(number(3.0), "3");
}

#[test]
fn the_window_keys_move_the_value_and_each_change_is_heard_once() {
    let _g = serial();
    let log = Log::default();
    let s = slider(&log);
    let mut tree = show(&s);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "sl"));
    for code in [39, 39, 37, 38, 40] {
        input.key(&mut tree, code);
    }
    assert!(
        (s.get() - 55.0).abs() < 1e-9,
        "right, right, left, up, down: 50 +5 +5 -5 +5 -5"
    );
    input.key(&mut tree, 33);
    input.key(&mut tree, 34);
    input.key(&mut tree, 34);
    assert!((s.get() - 35.0).abs() < 1e-9);
    input.key(&mut tree, 36);
    assert!(s.get().abs() < 1e-9);
    input.key(&mut tree, 36);
    input.key(&mut tree, 37);
    input.key(&mut tree, 35);
    assert!((s.get() - 100.0).abs() < 1e-9);
    input.key(&mut tree, 39);
    assert_eq!(
        seen(&log),
        ["55", "60", "55", "60", "55", "75", "55", "35", "0", "100"],
        "held keys at the end say nothing"
    );
    frame(&mut tree, 400.0, 80.0);
    assert_eq!(
        attr(&tree, "sl", "aria-valuenow").as_deref(),
        Some("100"),
        "the attribute follows"
    );
}

#[test]
fn raw_key_events_move_it_too_and_enter_and_space_do_not() {
    let _g = serial();
    let s = slider(&Log::default());
    let mut tree = show(&s);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "sl"));
    input.key_event_to(&mut tree, "sl", KEY_DOWN, 39);
    assert!((s.get() - 55.0).abs() < 1e-9, "a raw Right");
    input.key_event_to(&mut tree, "sl", KEY_DOWN, 35);
    assert!((s.get() - 100.0).abs() < 1e-9, "a raw End");
    input.key_event_to(&mut tree, "sl", KEY_DOWN, 32);
    input.key_event_to(&mut tree, "sl", KEY_DOWN, 13);
    assert!((s.get() - 100.0).abs() < 1e-9, "a slider is not pressed");
}

#[test]
fn a_press_on_the_track_sets_the_value_where_it_is_and_a_drag_follows() {
    let _g = serial();
    let log = Log::default();
    let s = slider(&log);
    let mut tree = show(&s);
    let mut input = Input::new();
    let node = tree.query_by_id("sl-track").expect("the track");
    let b = tree.get_absolute_bounds(node).expect("bounds");
    let run = 300.0 - THUMB;
    let at = |f: f32| (b.x + THUMB / 2.0 + f * run, b.y + b.height / 2.0);
    let (x, y) = at(0.25);
    input.press_at(&mut tree, x, y);
    assert!(
        (s.get() - 25.0).abs() < 1e-9,
        "pressed at a quarter: 25, got {}",
        s.get()
    );
    let (x, y) = at(0.75);
    input.move_to(&mut tree, x, y);
    assert!(
        (s.get() - 75.0).abs() < 1e-9,
        "dragged to three quarters: 75, got {}",
        s.get()
    );
    input.release_at(&mut tree, x, y);
    frame(&mut tree, 400.0, 80.0);
    assert_eq!(attr(&tree, "sl", "aria-valuenow").as_deref(), Some("75"));
    let (x, y) = at(1.0);
    input.click_at(&mut tree, x + THUMB / 2.0 - 1.0, y);
    assert!(
        (s.get() - 100.0).abs() < 1e-9,
        "a press past the end is held to the end, got {} (x {} track {:?})",
        s.get(),
        x,
        b
    );
    assert_ne!(seen(&log), Vec::<String>::new());
}

#[test]
fn the_active_class_is_there_while_the_pointer_holds_the_thumb() {
    let _g = serial();
    let sl = slider(&Log::default());
    let mut tree = show(&sl);
    let mut input = Input::new();
    let node = tree.query_by_id("sl-track").expect("the track");
    let bx = tree.get_absolute_bounds(node).expect("bounds");
    let (x, y) = (bx.x + 100.0, bx.y + bx.height / 2.0);
    let has = |tree: &RenderTree| {
        let n = tree.query_by_id("sl-thumb").expect("the drawn thumb");
        tree.element_registry()
            .has_class(n, "fs-slider-thumb-active")
    };
    assert!(!has(&tree));
    input.press_at(&mut tree, x, y);
    frame(&mut tree, 400.0, 80.0);
    assert!(has(&tree), "held");
    input.release_at(&mut tree, x, y);
    frame(&mut tree, 400.0, 80.0);
    assert!(!has(&tree), "let go");
}

#[test]
fn set_from_the_program_is_snapped_draws_again_and_says_nothing() {
    let _g = serial();
    let log = Log::default();
    let s = slider(&log);
    let mut tree = show(&s);
    s.set(62.0);
    frame(&mut tree, 400.0, 80.0);
    assert!((s.get() - 60.0).abs() < 1e-9);
    assert_eq!(attr(&tree, "sl", "aria-valuenow").as_deref(), Some("60"));
    assert!(
        seen(&log).is_empty(),
        "the program's own change is not heard"
    );
}

#[test]
fn the_keyboard_stays_on_the_thumb_when_the_slider_is_drawn_again_between_presses() {
    let _g = serial();
    let s = slider(&Log::default());
    let mut tree = show(&s);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "sl"));
    for _ in 0..3 {
        input.key(&mut tree, 39);
        frame(&mut tree, 400.0, 80.0);
        assert!(
            input.has_keyboard(&tree, "sl"),
            "the thumb still has the keyboard after a frame"
        );
    }
    assert!(
        (s.get() - 65.0).abs() < 1e-9,
        "three presses, each after a frame"
    );
    assert_eq!(attr(&tree, "sl", "aria-valuenow").as_deref(), Some("65"));
    assert_eq!(attr(&tree, "sl", "aria-valuetext").as_deref(), Some("65"));
}
