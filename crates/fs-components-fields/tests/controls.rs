//! The controls `switch` and `segmented`: states, keys (Space, Enter, arrows, Home, End, Tab), the pointer, disabled.
//!
//! Tests: Model tests over the values, an in-process mouse and keyboard over the built controls (the window's own router).

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use std::sync::{Arc, Mutex};

use blinc_layout::prelude::{ElementBuilder, RenderTree};
use common::{Input, layout_with, serial};
use fs_components_fields::controls::{STYLES, Segmented, Switch, SwitchState};
use fs_components_styles::styles::block_tokens;
use fs_components_styles::{StyleId, style};

/// The element laid out with the sheet of the controls and the tokens of the Admin style.
fn layout(ui: impl ElementBuilder + 'static, w: f32, h: f32) -> RenderTree {
    let tokens = block_tokens::sheet(style(StyleId::Admin));
    layout_with(ui, w, h, &format!("{STYLES}\n{tokens}"))
}

type Log = Arc<Mutex<Vec<String>>>;

fn switch(log: &Log) -> Switch {
    let l = Arc::clone(log);
    Switch::new("Daily timer")
        .id("sw")
        .on_change(move |s| l.lock().unwrap().push(format!("{s:?}")))
}

#[test]
fn a_switch_flips_off_on_off_and_mixed_goes_to_on() {
    let _g = serial();
    let log = Log::default();
    let s = switch(&log);
    assert_eq!(s.state(), SwitchState::Off);
    assert!(s.flip() && s.state() == SwitchState::On);
    assert!(s.flip() && s.state() == SwitchState::Off);
    s.set(SwitchState::Mixed);
    assert_eq!(
        log.lock().unwrap().len(),
        2,
        "set from the program calls nobody"
    );
    assert!(
        s.flip() && s.state() == SwitchState::On,
        "undetermined becomes on"
    );
    assert_eq!(*log.lock().unwrap(), ["On", "Off", "On"]);
}

#[test]
fn a_switch_takes_the_pointer_and_the_keys_once_each() {
    let _g = serial();
    let log = Log::default();
    let s = switch(&log);
    let mut tree = layout(s.build(), 320.0, 80.0);
    let mut input = Input::new();
    input.click(&mut tree, "sw-track");
    assert_eq!(s.state(), SwitchState::On);
    input.click(&mut tree, "sw-label");
    assert_eq!(s.state(), SwitchState::Off);
    assert!(
        input.tab_to(&mut tree, "sw-hit"),
        "the switch is a tab stop"
    );
    input.key(&mut tree, 13); // Enter: the window makes it a click on the focused element
    assert_eq!(s.state(), SwitchState::On, "Enter flips once, not twice");
    input.key(&mut tree, 32);
    assert_eq!(s.state(), SwitchState::Off, "Space flips once");
    assert_eq!(log.lock().unwrap().len(), 4);
}

#[test]
fn a_disabled_switch_is_no_tab_stop_and_ignores_everything() {
    let _g = serial();
    let log = Log::default();
    let s = switch(&log).disabled(true);
    let mut tree = layout(s.build(), 320.0, 80.0);
    let mut input = Input::new();
    input.click(&mut tree, "sw-track");
    assert!(!input.tab_to(&mut tree, "sw-hit"));
    assert!(!s.flip());
    assert_eq!(s.state(), SwitchState::Off);
    assert!(log.lock().unwrap().is_empty());
}

#[test]
fn the_thumb_sits_left_centre_or_right_by_state() {
    let _g = serial();
    let s = switch(&Log::default());
    let mut tree = layout(s.build(), 320.0, 80.0);
    let centre = |tree: &blinc_layout::prelude::RenderTree| Input::centre(tree, "sw-thumb").0;
    let (track_x, _) = Input::centre(&tree, "sw-track");
    let off = centre(&tree);
    assert!(off < track_x, "off: the thumb is left of the middle");
    s.set(SwitchState::On);
    common::frame(&mut tree, 320.0, 80.0);
    assert!(centre(&tree) > track_x, "on: right of the middle");
    s.set(SwitchState::Mixed);
    common::frame(&mut tree, 320.0, 80.0);
    assert!(
        (centre(&tree) - track_x).abs() < 2.0,
        "mixed: in the middle"
    );
}

fn group(log: &Log) -> Segmented {
    let l = Arc::clone(log);
    Segmented::new()
        .id("sg")
        .segment("user", "User", false)
        .segment("system", "System", true)
        .segment("both", "Both", false)
        .on_change(move |k| l.lock().unwrap().push(k.to_owned()))
}

#[test]
fn segmented_starts_on_the_first_segment_that_can_be_chosen() {
    let _g = serial();
    assert_eq!(group(&Log::default()).chosen().as_deref(), Some("user"));
    let none = Segmented::new().segment("a", "A", true);
    assert!(none.chosen().is_none() && !none.set("a") && !none.choose("a"));
}

#[test]
fn segmented_arrows_move_and_skip_disabled_home_end_jump_and_it_is_one_tab_stop() {
    let _g = serial();
    let log = Log::default();
    let g = group(&log);
    let mut tree = layout(g.build(), 360.0, 80.0);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "sg"), "the group is one tab stop");
    input.key(&mut tree, 39);
    assert_eq!(
        g.chosen().as_deref(),
        Some("both"),
        "the disabled segment is skipped"
    );
    input.key(&mut tree, 39);
    assert_eq!(g.chosen().as_deref(), Some("both"), "no wrap");
    input.key(&mut tree, 37);
    assert_eq!(g.chosen().as_deref(), Some("user"));
    input.key(&mut tree, 35);
    assert_eq!(g.chosen().as_deref(), Some("both"));
    input.key(&mut tree, 36);
    assert_eq!(g.chosen().as_deref(), Some("user"));
    assert_eq!(
        *log.lock().unwrap(),
        ["both", "user", "both", "user"],
        "the callback hears each change once"
    );
}

#[test]
fn segmented_pointer_picks_a_segment_but_not_a_disabled_one_or_the_same_again() {
    let _g = serial();
    let log = Log::default();
    let g = group(&log);
    let mut tree = layout(g.build(), 360.0, 80.0);
    let mut input = Input::new();
    input.click(&mut tree, "sg-system");
    assert_eq!(g.chosen().as_deref(), Some("user"));
    input.click(&mut tree, "sg-both");
    input.click(&mut tree, "sg-both");
    assert_eq!(g.chosen().as_deref(), Some("both"));
    assert_eq!(*log.lock().unwrap(), ["both"]);
}

#[test]
fn a_disabled_group_ignores_everything() {
    let _g = serial();
    let log = Log::default();
    let g = group(&log).disabled(true);
    let mut tree = layout(g.build(), 360.0, 80.0);
    let mut input = Input::new();
    input.click(&mut tree, "sg-both");
    assert!(!input.tab_to(&mut tree, "sg"));
    assert_eq!(g.chosen().as_deref(), Some("user"));
    assert!(log.lock().unwrap().is_empty());
}
