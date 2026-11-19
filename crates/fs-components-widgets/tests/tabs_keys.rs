//! `tabs` (keys and states): the vertical row, the manual way (Enter and Space as the window's click AND as raw key events), a click, the
//! state classes (chosen, cursor, disabled, vertical), and a row of only disabled tabs.
//!
//! Tests: attribute reads and classes on the built element, an in-process keyboard and pointer, the key events a window sends.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::div::div;
use blinc_layout::prelude::text;
use common::{Input, attr, frame, layout_with, serial};
use fs_components_widgets::tabs::{STYLES, Tab, Tabs};

type Log = Arc<Mutex<Vec<String>>>;

fn tabs(log: &Log) -> Tabs {
    let l = Arc::clone(log);
    Tabs::new(vec![
        Tab::new("one", "One"),
        Tab::new("two", "Two").disabled(),
        Tab::new("three", "Three"),
        Tab::new("four", "Four"),
    ])
    .id("tb")
    .name("Sections")
    .on_select(move |k| l.lock().unwrap().push(k.to_owned()))
}

fn page(t: &Tabs) -> impl blinc_layout::prelude::ElementBuilder + 'static {
    div()
        .child(t.build())
        .child(t.panel(|key| div().child(text(format!("body {key}")))))
}

fn show(t: &Tabs) -> blinc_layout::prelude::RenderTree {
    layout_with(page(t), 500.0, 200.0, STYLES)
}

fn seen(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

#[test]
fn a_vertical_row_moves_with_up_and_down_and_says_so() {
    let _g = serial();
    let t = Tabs::new(vec![Tab::new("a", "A"), Tab::new("b", "B")])
        .id("vt")
        .vertical();
    let mut tree = show(&t);
    assert_eq!(
        attr(&tree, "vt", "aria-orientation").as_deref(),
        Some("vertical")
    );
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "vt"));
    input.key(&mut tree, 39);
    assert_eq!(
        t.chosen().as_deref(),
        Some("a"),
        "Right does nothing in a vertical row"
    );
    input.key(&mut tree, 40);
    assert_eq!(t.chosen().as_deref(), Some("b"));
    input.key(&mut tree, 38);
    assert_eq!(t.chosen().as_deref(), Some("a"));
}

#[test]
fn the_manual_way_moves_the_cursor_and_enter_or_space_chooses() {
    let _g = serial();
    let log = Log::default();
    let t = tabs(&log).manual();
    let mut tree = show(&t);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "tb"));
    input.key(&mut tree, 39);
    frame(&mut tree, 500.0, 200.0);
    assert_eq!(
        t.chosen().as_deref(),
        Some("one"),
        "the arrow only moves the cursor"
    );
    assert_eq!(seen(&log), Vec::<String>::new());
    assert_eq!(
        attr(&tree, "tb-tab-three", "aria-selected").as_deref(),
        Some("false")
    );
    // the cursor mark: only on the tab the cursor is on while it is not the chosen one
    input.key(&mut tree, 13);
    assert_eq!(
        t.chosen().as_deref(),
        Some("three"),
        "Enter (the window's click) chooses"
    );
    input.key(&mut tree, 39);
    input.key(&mut tree, 32);
    assert_eq!(
        t.chosen().as_deref(),
        Some("four"),
        "Space (the window's click) chooses"
    );
    assert_eq!(seen(&log), ["three", "four"]);
}

#[test]
fn enter_and_space_as_raw_key_events_choose_in_the_manual_way_too() {
    let _g = serial();
    let log = Log::default();
    let t = tabs(&log).manual();
    let mut tree = show(&t);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "tb"));
    input.key(&mut tree, 39);
    input.key_event_to(&mut tree, "tb", KEY_DOWN, 32);
    assert_eq!(t.chosen().as_deref(), Some("three"), "a raw Space chooses");
    input.key(&mut tree, 39);
    input.key_event_to(&mut tree, "tb", KEY_DOWN, 13);
    assert_eq!(t.chosen().as_deref(), Some("four"), "a raw Enter chooses");
    input.key(&mut tree, 37);
    input.key_event_to(&mut tree, "tb", KEY_DOWN, 65);
    assert_eq!(
        t.chosen().as_deref(),
        Some("four"),
        "another key does nothing"
    );
    assert_eq!(seen(&log), ["three", "four"]);
}

#[test]
fn a_click_chooses_a_tab_once_and_a_disabled_tab_ignores_it() {
    let _g = serial();
    let log = Log::default();
    let t = tabs(&log);
    let mut tree = show(&t);
    let mut input = Input::new();
    input.click(&mut tree, "tb-tab-two");
    assert_eq!(t.chosen().as_deref(), Some("one"));
    input.click(&mut tree, "tb-tab-four");
    assert_eq!(t.chosen().as_deref(), Some("four"));
    frame(&mut tree, 500.0, 200.0);
    input.click(&mut tree, "tb-tab-four");
    assert_eq!(
        seen(&log),
        ["four"],
        "choosing the chosen tab again says nothing"
    );
    assert!(!t.choose("two") && !t.choose("nothing"));
}

#[test]
fn the_state_classes_follow_the_choice_the_cursor_and_the_disabled_tabs() {
    let _g = serial();
    let t = tabs(&Log::default()).manual();
    let mut tree = show(&t);
    let has = |tree: &blinc_layout::prelude::RenderTree, id: &str, class: &str| {
        let node = tree.query_by_id(id).expect("the tab");
        tree.element_registry().has_class(node, class)
    };
    assert!(has(&tree, "tb-tab-one", "fs-tab-selected"));
    assert!(
        !has(&tree, "tb-tab-one", "fs-tab-current"),
        "the cursor on the chosen tab has no extra mark"
    );
    assert!(has(&tree, "tb-tab-two", "fs-tab-disabled"));
    assert!(!has(&tree, "tb-tab-three", "fs-tab-selected"));
    assert!(has(&tree, "tb-tab-three", "fs-tab"));
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "tb"));
    input.key(&mut tree, 39);
    frame(&mut tree, 500.0, 200.0);
    assert!(
        has(&tree, "tb-tab-three", "fs-tab-current"),
        "the cursor moved"
    );
    assert!(!has(&tree, "tb-tab-three", "fs-tab-selected"));
    assert!(
        has(&tree, "tb-tab-one", "fs-tab-selected"),
        "the choice stays"
    );
    assert!(!has(&tree, "tb-tab-one", "fs-tab-current"));
    input.key(&mut tree, 13);
    frame(&mut tree, 500.0, 200.0);
    assert!(
        has(&tree, "tb-tab-three", "fs-tab-selected"),
        "Enter chose it"
    );
    assert!(!has(&tree, "tb-tab-three", "fs-tab-current"));
    assert!(!has(&tree, "tb-tab-one", "fs-tab-selected"));
}

#[test]
fn the_vertical_row_has_its_class() {
    let _g = serial();
    let t = Tabs::new(vec![Tab::new("a", "A")]).id("vc").vertical();
    let tree = show(&t);
    let node = tree.query_by_id("vc").expect("the row");
    assert!(tree.element_registry().has_class(node, "fs-tabs-vertical"));
    let h = Tabs::new(vec![Tab::new("a", "A")]).id("hc");
    let tree = show(&h);
    let node = tree.query_by_id("hc").expect("the row");
    assert!(!tree.element_registry().has_class(node, "fs-tabs-vertical"));
}

#[test]
fn no_tab_can_be_chosen_when_all_are_disabled() {
    let _g = serial();
    let t = Tabs::new(vec![Tab::new("a", "A").disabled()]).id("dd");
    let mut tree = show(&t);
    assert!(t.chosen().is_none());
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "dd"));
    input.key(&mut tree, 39);
    assert!(t.chosen().is_none());
}
