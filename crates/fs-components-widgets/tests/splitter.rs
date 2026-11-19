//! `splitter`: role `separator` with value, limits, orientation, name and the pane it controls; the keys of both directions (window
//! keys AND raw), Home, End and Enter (collapse and give back), the pointer drag, the pane sizes, the active class, the callback.
//!
//! Tests: model tests over `State`; attribute reads on the built element; an in-process keyboard and pointer.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::div::div;
use blinc_layout::prelude::{RenderTree, text};
use common::{Input, attr, frame, layout_with, serial};
use fs_components_widgets::keys::Key;
use fs_components_widgets::splitter::{Outcome, STYLES, Splitter, State};

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

fn width_of(tree: &RenderTree, id: &str) -> f32 {
    let node = tree.query_by_id(id).expect("the pane");
    tree.get_absolute_bounds(node).expect("bounds").width
}

#[test]
fn the_state_keeps_to_its_limits_and_collapses_and_gives_back() {
    let mut s = State::new(40.0, 10.0, 80.0, 5.0, true);
    assert_eq!(s.press(Key::Right), Outcome::Moved(45.0));
    assert_eq!(s.press(Key::Left), Outcome::Moved(40.0));
    assert_eq!(
        s.press(Key::Up),
        Outcome::Ignored,
        "Up is not a key of a divider between left and right panes"
    );
    assert_eq!(s.press(Key::Home), Outcome::Moved(10.0));
    assert_eq!(
        s.press(Key::Left),
        Outcome::Moved(10.0),
        "held to the least"
    );
    assert_eq!(s.press(Key::End), Outcome::Moved(80.0));
    assert_eq!(
        s.press(Key::Right),
        Outcome::Moved(80.0),
        "held to the greatest"
    );
    s.set(30.0);
    assert_eq!(s.press(Key::Enter), Outcome::Moved(0.0));
    assert!(s.collapsed());
    assert_eq!(
        s.press(Key::Enter),
        Outcome::Moved(30.0),
        "Enter again gives the size back"
    );
    assert!(!s.collapsed());
    s.press(Key::Enter);
    assert_eq!(
        s.press(Key::Right),
        Outcome::Moved(10.0),
        "a move after a collapse ends it, from the least size"
    );
    assert!(!s.collapsed());
}

#[test]
fn stacked_panes_use_up_and_down_and_the_limits_and_step_are_put_right() {
    let mut v = State::new(40.0, 10.0, 80.0, 5.0, false);
    assert_eq!(
        v.press(Key::Down),
        Outcome::Moved(45.0),
        "stacked panes: Down moves"
    );
    assert_eq!(v.press(Key::Up), Outcome::Moved(40.0));
    assert_eq!(v.press(Key::Right), Outcome::Ignored);
    assert!(!v.vertical());
    let swapped = State::new(300.0, 90.0, 20.0, 5.0, true);
    assert!(
        (swapped.min() - 20.0).abs() < 1e-9 && (swapped.max() - 90.0).abs() < 1e-9,
        "limits swapped"
    );
    assert!(
        (swapped.position() - 90.0).abs() < 1e-9,
        "held to the limits"
    );
    let big = State::new(40.0, -50.0, 500.0, 5.0, true);
    assert!(
        big.min().abs() < 1e-9 && (big.max() - 100.0).abs() < 1e-9,
        "limits held to 0 and 100"
    );
    let mut tiny = State::new(40.0, 10.0, 80.0, 0.0, true);
    assert_eq!(
        tiny.press(Key::Right),
        Outcome::Moved(40.1),
        "a step is at least a tenth"
    );
}

#[test]
fn the_divider_is_a_named_separator_with_value_limits_orientation_and_its_pane() {
    let _g = serial();
    let tree = show(&splitter(&Log::default()));
    assert_eq!(attr(&tree, "sp-bar", "role").as_deref(), Some("separator"));
    assert_eq!(
        attr(&tree, "sp-bar", "aria-label").as_deref(),
        Some("Sidebar width")
    );
    assert_eq!(
        attr(&tree, "sp-bar", "aria-orientation").as_deref(),
        Some("vertical"),
        "between left and right panes"
    );
    assert_eq!(
        attr(&tree, "sp-bar", "aria-valuenow").as_deref(),
        Some("40")
    );
    assert_eq!(
        attr(&tree, "sp-bar", "aria-valuemin").as_deref(),
        Some("10")
    );
    assert_eq!(
        attr(&tree, "sp-bar", "aria-valuemax").as_deref(),
        Some("80")
    );
    assert_eq!(
        attr(&tree, "sp-bar", "aria-controls").as_deref(),
        Some("sp-first")
    );
    assert_eq!(attr(&tree, "sp-bar", "tabindex").as_deref(), Some("0"));
    assert!(tree.query_by_id("sp-first").is_some() && tree.query_by_id("sp-second").is_some());
    let stacked = Splitter::stacked("Height", 50.0, 10.0, 90.0, 5.0).id("st");
    assert_eq!(
        attr(&show(&stacked), "st-bar", "aria-orientation").as_deref(),
        Some("horizontal")
    );
}

#[test]
fn the_panes_share_the_room_by_the_position() {
    let _g = serial();
    let s = splitter(&Log::default());
    let mut tree = show(&s);
    let run = 500.0;
    assert!(
        (width_of(&tree, "sp-first") - run * 0.4).abs() < 1.5,
        "first {}",
        width_of(&tree, "sp-first")
    );
    assert!((width_of(&tree, "sp-second") - run * 0.6).abs() < 1.5);
    s.set(75.0);
    frame(&mut tree, 600.0, 140.0);
    assert!((width_of(&tree, "sp-first") - run * 0.75).abs() < 1.5);
    assert_eq!(
        attr(&tree, "sp-bar", "aria-valuenow").as_deref(),
        Some("75")
    );
    s.set(500.0);
    assert!((s.position() - 80.0).abs() < 1e-9, "held to the limits");
}

#[test]
fn the_window_keys_move_the_divider_home_end_and_enter_collapse_and_each_move_is_heard_once() {
    let _g = serial();
    let log = Log::default();
    let s = splitter(&log);
    let mut tree = show(&s);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "sp-bar"));
    for code in [39, 37, 37] {
        input.key(&mut tree, code);
        frame(&mut tree, 600.0, 140.0);
        assert!(
            input.has_keyboard(&tree, "sp-bar"),
            "the divider keeps the keyboard after it is drawn again"
        );
    }
    assert!((s.position() - 35.0).abs() < 1e-9);
    assert_eq!(
        attr(&tree, "sp-bar", "aria-valuenow").as_deref(),
        Some("35"),
        "the attribute follows"
    );
    input.key(&mut tree, 38);
    assert!((s.position() - 35.0).abs() < 1e-9, "Up is not a key here");
    input.key(&mut tree, 35);
    assert!((s.position() - 80.0).abs() < 1e-9);
    input.key(&mut tree, 36);
    assert!((s.position() - 10.0).abs() < 1e-9);
    input.key(&mut tree, 13);
    assert!(
        s.position().abs() < 1e-9 && s.collapsed(),
        "Enter (the window's click) collapses"
    );
    frame(&mut tree, 600.0, 140.0);
    assert!(width_of(&tree, "sp-first") < 1.0, "the first pane is gone");
    input.key(&mut tree, 13);
    assert!(
        (s.position() - 10.0).abs() < 1e-9 && !s.collapsed(),
        "Enter again gives it back"
    );
    assert_eq!(seen(&log), ["45", "40", "35", "80", "10", "0", "10"]);
}

#[test]
fn raw_key_events_move_it_too_and_other_keys_do_nothing() {
    let _g = serial();
    let log = Log::default();
    let s = splitter(&log);
    let mut tree = show(&s);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "sp-bar"));
    input.key_event_to(&mut tree, "sp-bar", KEY_DOWN, 39);
    assert!((s.position() - 45.0).abs() < 1e-9, "a raw Right");
    input.key_event_to(&mut tree, "sp-bar", KEY_DOWN, 13);
    assert!(s.position().abs() < 1e-9, "a raw Enter collapses");
    input.key_event_to(&mut tree, "sp-bar", KEY_DOWN, 13);
    assert!((s.position() - 45.0).abs() < 1e-9, "and gives back");
    input.key_event_to(&mut tree, "sp-bar", KEY_DOWN, 32);
    input.key_event_to(&mut tree, "sp-bar", KEY_DOWN, 65);
    assert!((s.position() - 45.0).abs() < 1e-9, "Space and A do nothing");
    assert_eq!(seen(&log), ["45", "0", "45"]);
}
