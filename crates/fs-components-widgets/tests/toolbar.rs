//! `toolbar`: role, name, orientation, one tab stop, the arrows (no wrap, disabled skipped, separators skipped), Home and End,
//! Enter and Space as the window's click AND as raw keys, toggles (`aria-pressed`), the state classes, the callback.
//!
//! Tests: attribute reads on the built element, an in-process keyboard and pointer, the key events a window sends.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::prelude::RenderTree;
use common::{Input, attr, frame, layout_with, serial};
use fs_components_widgets::disabled::Why;
use fs_components_widgets::toolbar::{Item, STYLES, Toolbar};

type Log = Arc<Mutex<Vec<String>>>;

fn bar(log: &Log) -> Toolbar {
    let l = Arc::clone(log);
    Toolbar::new(vec![
        Item::button("cut", "Cut"),
        Item::button("copy", "Copy").disabled(),
        Item::separator(),
        Item::toggle("bold", "Bold", false),
        Item::button("paste", "Paste"),
    ])
    .id("tl")
    .name("Edit")
    .on_press(move |k, on| l.lock().unwrap().push(format!("{k}:{on}")))
}

fn show(t: &Toolbar) -> RenderTree {
    layout_with(t.build(), 600.0, 120.0, STYLES)
}

fn seen(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

fn has(tree: &RenderTree, id: &str, class: &str) -> bool {
    let node = tree.query_by_id(id).expect("the item");
    tree.element_registry().has_class(node, class)
}

#[test]
fn the_bar_is_a_named_toolbar_with_one_tab_stop_and_buttons_inside() {
    let _g = serial();
    let tree = show(&bar(&Log::default()));
    assert_eq!(attr(&tree, "tl", "role").as_deref(), Some("toolbar"));
    assert_eq!(attr(&tree, "tl", "aria-label").as_deref(), Some("Edit"));
    assert_eq!(
        attr(&tree, "tl", "aria-orientation").as_deref(),
        Some("horizontal")
    );
    assert_eq!(attr(&tree, "tl", "tabindex").as_deref(), Some("0"));
    assert_eq!(
        attr(&tree, "tl-item-cut", "role").as_deref(),
        Some("button")
    );
    assert_eq!(
        attr(&tree, "tl-item-cut", "aria-label").as_deref(),
        Some("Cut")
    );
    assert_eq!(
        attr(&tree, "tl-item-cut", "aria-pressed"),
        None,
        "a button is not a toggle"
    );
    assert_eq!(
        attr(&tree, "tl-item-bold", "aria-pressed").as_deref(),
        Some("false")
    );
    assert_eq!(
        attr(&tree, "tl-item-copy", "aria-disabled").as_deref(),
        Some("true")
    );
    assert_eq!(
        attr(&tree, "tl-sep-2", "role").as_deref(),
        Some("separator")
    );
    assert_eq!(
        attr(&tree, "tl-sep-2", "aria-orientation").as_deref(),
        Some("vertical"),
        "a separator in a horizontal bar stands upright"
    );
}

#[test]
fn the_arrows_skip_the_disabled_item_and_the_separator_and_do_not_wrap() {
    let _g = serial();
    let t = bar(&Log::default());
    let mut tree = show(&t);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "tl"));
    assert!(
        has(&tree, "tl-item-cut", "fs-tool-current"),
        "the cursor starts on the first item that can be reached"
    );
    input.key(&mut tree, 39);
    frame(&mut tree, 600.0, 120.0);
    assert!(
        has(&tree, "tl-item-bold", "fs-tool-current"),
        "Copy (disabled) and the separator are skipped"
    );
    assert!(!has(&tree, "tl-item-cut", "fs-tool-current"));
    input.key(&mut tree, 39);
    input.key(&mut tree, 39);
    frame(&mut tree, 600.0, 120.0);
    assert!(
        has(&tree, "tl-item-paste", "fs-tool-current"),
        "no wrap at the end"
    );
    input.key(&mut tree, 37);
    input.key(&mut tree, 37);
    input.key(&mut tree, 37);
    frame(&mut tree, 600.0, 120.0);
    assert!(
        has(&tree, "tl-item-cut", "fs-tool-current"),
        "no wrap at the start"
    );
    input.key(&mut tree, 35);
    frame(&mut tree, 600.0, 120.0);
    assert!(has(&tree, "tl-item-paste", "fs-tool-current"));
    input.key(&mut tree, 36);
    frame(&mut tree, 600.0, 120.0);
    assert!(has(&tree, "tl-item-cut", "fs-tool-current"));
}

#[test]
fn enter_and_space_press_the_item_under_the_cursor_once_and_a_toggle_turns() {
    let _g = serial();
    let log = Log::default();
    let t = bar(&log);
    let mut tree = show(&t);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "tl"));
    input.key(&mut tree, 13);
    input.key(&mut tree, 39);
    input.key(&mut tree, 32);
    frame(&mut tree, 600.0, 120.0);
    assert_eq!(seen(&log), ["cut:false", "bold:true"]);
    assert!(t.is_pressed("bold"));
    assert_eq!(
        attr(&tree, "tl-item-bold", "aria-pressed").as_deref(),
        Some("true")
    );
    assert!(
        has(&tree, "tl-item-bold", "fs-tool-on"),
        "the on state is a class"
    );
    input.key(&mut tree, 32);
    frame(&mut tree, 600.0, 120.0);
    assert_eq!(seen(&log).last().map(String::as_str), Some("bold:false"));
    assert!(!has(&tree, "tl-item-bold", "fs-tool-on"));
    assert_eq!(
        attr(&tree, "tl-item-bold", "aria-pressed").as_deref(),
        Some("false"),
        "turned off again: the attribute follows too"
    );
    input.key(&mut tree, 32);
    frame(&mut tree, 600.0, 120.0);
    assert_eq!(
        attr(&tree, "tl-item-bold", "aria-pressed").as_deref(),
        Some("true"),
        "and on a third time"
    );
}

#[test]
fn enter_and_space_as_raw_key_events_press_too() {
    let _g = serial();
    let log = Log::default();
    let t = bar(&log);
    let mut tree = show(&t);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "tl"));
    input.key_event_to(&mut tree, "tl", KEY_DOWN, 32);
    input.key_event_to(&mut tree, "tl", KEY_DOWN, 13);
    input.key_event_to(&mut tree, "tl", KEY_DOWN, 65);
    assert_eq!(
        seen(&log),
        ["cut:false", "cut:false"],
        "a raw Space and a raw Enter press once each, another key not"
    );
}

#[test]
fn a_click_presses_the_item_and_moves_the_cursor_and_a_disabled_item_ignores_it() {
    let _g = serial();
    let log = Log::default();
    let t = bar(&log);
    let mut tree = show(&t);
    let mut input = Input::new();
    input.click(&mut tree, "tl-item-copy");
    assert_eq!(seen(&log), Vec::<String>::new());
    input.click(&mut tree, "tl-item-paste");
    frame(&mut tree, 600.0, 120.0);
    assert_eq!(seen(&log), ["paste:false"]);
    assert!(has(&tree, "tl-item-paste", "fs-tool-current"));
}

#[test]
fn a_vertical_bar_moves_with_up_and_down_and_its_separator_is_horizontal() {
    let _g = serial();
    let t = Toolbar::new(vec![
        Item::button("a", "A"),
        Item::separator(),
        Item::button("b", "B"),
    ])
    .id("vb")
    .vertical();
    let mut tree = show(&t);
    assert_eq!(
        attr(&tree, "vb", "aria-orientation").as_deref(),
        Some("vertical")
    );
    let node = tree.query_by_id("vb").expect("the bar");
    assert!(
        tree.element_registry()
            .has_class(node, "fs-toolbar-vertical")
    );
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "vb"));
    input.key(&mut tree, 39);
    frame(&mut tree, 600.0, 120.0);
    assert!(
        has(&tree, "vb-item-a", "fs-tool-current"),
        "Right does nothing in a vertical bar"
    );
    input.key(&mut tree, 40);
    frame(&mut tree, 600.0, 120.0);
    assert!(has(&tree, "vb-item-b", "fs-tool-current"));
}

#[test]
fn the_cursor_reaches_a_disabled_item_with_a_reason_without_a_tab_stop_and_enter_does_nothing() {
    let _g = serial();
    let log = Log::default();
    let l = Arc::clone(&log);
    let t = Toolbar::new(vec![
        Item::button("cut", "Cut"),
        Item::button("copy", "Copy").disabled_because(Why::Now("Nothing selected".into())),
        Item::button("paste", "Paste"),
    ])
    .id("tr")
    .on_press(move |k, on| l.lock().unwrap().push(format!("{k}:{on}")));
    let mut tree = show(&t);
    assert_eq!(
        attr(&tree, "tr-item-copy", "tabindex").as_deref(),
        Some("-1"),
        "no tab stop of its own in the bar"
    );
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "tr"));
    input.key(&mut tree, 39);
    frame(&mut tree, 600.0, 120.0);
    assert!(
        has(&tree, "tr-item-copy", "fs-tool-current"),
        "the cursor stands on the locked item, its reason can be read"
    );
    input.key(&mut tree, 13);
    assert_eq!(seen(&log), Vec::<String>::new(), "Enter presses nothing");
    input.key(&mut tree, 39);
    frame(&mut tree, 600.0, 120.0);
    assert!(has(&tree, "tr-item-paste", "fs-tool-current"));
}
