//! `listbox`: role, name, multiselectable, option roles and `aria-selected`, the keys (arrows, Home, End, Space and Enter as the window's
//! click AND as raw keys, Ctrl+A), single pick follows the cursor, several toggle, disabled options, state classes, the callback.
//!
//! Tests: attribute reads on the built element, an in-process keyboard and pointer, the key events a window sends.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::prelude::RenderTree;
use common::{Input, attr, frame, layout_with, serial};
use fs_components_widgets::listbox::{Listbox, Opt, STYLES};

type Log = Arc<Mutex<Vec<String>>>;

fn opts() -> Vec<Opt> {
    vec![
        Opt::new("a", "Alpha"),
        Opt::new("b", "Beta").disabled(),
        Opt::new("c", "Gamma"),
        Opt::new("d", "Delta"),
    ]
}

fn single(log: &Log) -> Listbox {
    let l = Arc::clone(log);
    Listbox::new(opts())
        .id("lb")
        .name("Letters")
        .on_change(move |k| l.lock().unwrap().push(k.join("+")))
}

fn several(log: &Log) -> Listbox {
    let l = Arc::clone(log);
    Listbox::multiple(opts())
        .id("lm")
        .on_change(move |k| l.lock().unwrap().push(k.join("+")))
}

fn show(l: &Listbox) -> RenderTree {
    layout_with(l.build(), 400.0, 240.0, STYLES)
}

fn has(tree: &RenderTree, id: &str, class: &str) -> bool {
    let node = tree.query_by_id(id).expect("the option");
    tree.element_registry().has_class(node, class)
}

fn seen(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

#[test]
fn the_list_is_a_named_listbox_with_one_tab_stop_and_options_that_tell_their_pick() {
    let _g = serial();
    let tree = show(&single(&Log::default()));
    assert_eq!(attr(&tree, "lb", "role").as_deref(), Some("listbox"));
    assert_eq!(attr(&tree, "lb", "aria-label").as_deref(), Some("Letters"));
    assert_eq!(attr(&tree, "lb", "tabindex").as_deref(), Some("0"));
    assert_eq!(
        attr(&tree, "lb", "aria-multiselectable"),
        None,
        "one pick: not multiselectable"
    );
    assert_eq!(attr(&tree, "lb-opt-a", "role").as_deref(), Some("option"));
    assert_eq!(
        attr(&tree, "lb-opt-a", "aria-selected").as_deref(),
        Some("true"),
        "the first option is picked at first"
    );
    assert_eq!(
        attr(&tree, "lb-opt-c", "aria-selected").as_deref(),
        Some("false")
    );
    assert_eq!(
        attr(&tree, "lb-opt-b", "aria-disabled").as_deref(),
        Some("true")
    );
    let tree = show(&several(&Log::default()));
    assert_eq!(
        attr(&tree, "lm", "aria-multiselectable").as_deref(),
        Some("true")
    );
    assert_eq!(
        attr(&tree, "lm-opt-a", "aria-selected").as_deref(),
        Some("false"),
        "several: nothing is picked at first"
    );
}

#[test]
fn in_the_single_way_the_pick_follows_the_cursor_and_skips_the_disabled_option() {
    let _g = serial();
    let log = Log::default();
    let l = single(&log);
    let mut tree = show(&l);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "lb"));
    input.key(&mut tree, 40);
    assert_eq!(l.picked(), ["c"], "Beta is disabled");
    input.key(&mut tree, 40);
    assert_eq!(l.picked(), ["d"]);
    input.key(&mut tree, 40);
    assert_eq!(l.picked(), ["d"], "no wrap");
    input.key(&mut tree, 38);
    assert_eq!(l.picked(), ["c"]);
    input.key(&mut tree, 36);
    assert_eq!(l.picked(), ["a"]);
    input.key(&mut tree, 35);
    assert_eq!(l.picked(), ["d"]);
    input.key(&mut tree, 39);
    assert_eq!(l.picked(), ["d"], "Right is no key of a vertical list");
    assert_eq!(
        seen(&log),
        ["c", "d", "c", "a", "d"],
        "each change is heard once"
    );
    frame(&mut tree, 400.0, 240.0);
    assert_eq!(
        attr(&tree, "lb-opt-d", "aria-selected").as_deref(),
        Some("true")
    );
    assert_eq!(
        attr(&tree, "lb-opt-a", "aria-selected").as_deref(),
        Some("false")
    );
}

#[test]
fn in_the_several_way_space_and_enter_toggle_the_option_under_the_cursor_once() {
    let _g = serial();
    let log = Log::default();
    let l = several(&log);
    let mut tree = show(&l);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "lm"));
    input.key(&mut tree, 32);
    assert_eq!(l.picked(), ["a"], "Space (the window's click) picks");
    input.key(&mut tree, 40);
    assert!(l.picked() == ["a"], "an arrow only moves");
    input.key(&mut tree, 13);
    assert_eq!(l.picked(), ["a", "c"], "Enter (the window's click) picks");
    input.key(&mut tree, 13);
    assert_eq!(l.picked(), ["a"], "again unpicks");
    assert_eq!(seen(&log), ["a", "a+c", "a"]);
}

#[test]
fn space_and_enter_as_raw_key_events_toggle_too() {
    let _g = serial();
    let log = Log::default();
    let l = several(&log);
    let mut tree = show(&l);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "lm"));
    input.key_event_to(&mut tree, "lm", KEY_DOWN, 32);
    assert_eq!(l.picked(), ["a"], "a raw Space picks");
    input.key_event_to(&mut tree, "lm", KEY_DOWN, 13);
    assert_eq!(l.picked(), Vec::<String>::new(), "a raw Enter unpicks");
    input.key_event_to(&mut tree, "lm", KEY_DOWN, 65);
    assert_eq!(
        l.picked(),
        Vec::<String>::new(),
        "A without Ctrl does nothing"
    );
    assert_eq!(seen(&log), ["a", ""]);
}

#[test]
fn ctrl_a_picks_all_options_that_can_be_picked_in_the_several_way_only() {
    let _g = serial();
    let l = several(&Log::default());
    let mut tree = show(&l);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "lm"));
    input.press(&mut tree, 65, (false, true, false));
    assert_eq!(l.picked(), ["a", "c", "d"], "the disabled option stays out");
    let s = single(&Log::default());
    let mut tree = show(&s);
    assert!(input.tab_to(&mut tree, "lb"));
    input.press(&mut tree, 65, (false, true, false));
    assert_eq!(s.picked(), ["a"], "a single pick has no select-all");
}

#[test]
fn a_click_picks_and_moves_the_cursor_and_a_disabled_option_ignores_it() {
    let _g = serial();
    let log = Log::default();
    let l = single(&log);
    let mut tree = show(&l);
    let mut input = Input::new();
    input.click(&mut tree, "lb-opt-b");
    assert_eq!(seen(&log), Vec::<String>::new());
    input.click(&mut tree, "lb-opt-d");
    frame(&mut tree, 400.0, 240.0);
    assert_eq!(seen(&log), ["d"]);
    assert!(has(&tree, "lb-opt-d", "fs-option-current"));
    assert!(has(&tree, "lb-opt-d", "fs-option-picked"));
}

#[test]
fn the_state_classes_follow_the_pick_the_cursor_and_the_disabled_options() {
    let _g = serial();
    let l = several(&Log::default());
    let mut tree = show(&l);
    assert!(
        has(&tree, "lm-opt-a", "fs-option-current"),
        "the cursor starts on the first option that can be picked"
    );
    assert!(!has(&tree, "lm-opt-a", "fs-option-picked"));
    assert!(has(&tree, "lm-opt-b", "fs-option-disabled"));
    assert!(!has(&tree, "lm-opt-c", "fs-option-current"));
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "lm"));
    input.key(&mut tree, 40);
    frame(&mut tree, 400.0, 240.0);
    assert!(
        has(&tree, "lm-opt-c", "fs-option-current"),
        "the cursor moved"
    );
    assert!(!has(&tree, "lm-opt-a", "fs-option-current"));
    input.key(&mut tree, 32);
    frame(&mut tree, 400.0, 240.0);
    assert!(has(&tree, "lm-opt-c", "fs-option-picked"));
    assert!(!has(&tree, "lm-opt-a", "fs-option-picked"));
}

#[test]
fn a_list_of_only_disabled_options_picks_nothing() {
    let _g = serial();
    let l = Listbox::new(vec![Opt::new("x", "X").disabled()]).id("dd");
    let mut tree = show(&l);
    assert_eq!(l.picked(), Vec::<String>::new());
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "dd"));
    input.key(&mut tree, 40);
    input.key(&mut tree, 32);
    assert_eq!(l.picked(), Vec::<String>::new());
}

#[test]
fn in_the_single_way_space_and_enter_say_nothing_new_and_ctrl_a_does_nothing() {
    let _g = serial();
    let log = Log::default();
    let l = single(&log);
    let mut tree = show(&l);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "lb"));
    input.key(&mut tree, 40);
    assert_eq!(seen(&log), ["c"], "the arrow moved the pick");
    input.key(&mut tree, 32);
    input.key(&mut tree, 13);
    input.key_event_to(&mut tree, "lb", KEY_DOWN, 32);
    input.key_event_to(&mut tree, "lb", KEY_DOWN, 13);
    assert_eq!(
        seen(&log),
        ["c"],
        "the option is picked already: Space and Enter say nothing"
    );
    input.press(&mut tree, 65, (false, true, false));
    assert_eq!(l.picked(), ["c"], "a single pick has no select-all");
    assert_eq!(seen(&log), ["c"], "and no change was heard");
}
