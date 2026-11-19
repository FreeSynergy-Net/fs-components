//! `disclosure` and `accordion`: the header's ARIA state, the panel role, Enter and Space as the window's click AND as raw keys, the
//! single way, the heading level, the state classes, the callbacks, the keyboard staying on a header after it redraws.
//!
//! Tests: attribute reads on the built element, an in-process keyboard and pointer, the key events a window sends.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::div::div;
use blinc_layout::prelude::{RenderTree, text};
use common::{Input, attr, frame, layout_with, serial};
use fs_components_widgets::disclosure::{Accordion, Disclosure, STYLES, Section};

type Log = Arc<Mutex<Vec<String>>>;

fn has(tree: &RenderTree, id: &str, class: &str) -> bool {
    let node = tree.query_by_id(id).expect("the element");
    tree.element_registry().has_class(node, class)
}

fn single_disclosure(log: &Log) -> Disclosure {
    let l = Arc::clone(log);
    Disclosure::new("Details")
        .id("dc")
        .on_toggle(move |o| l.lock().unwrap().push(format!("{o}")))
}

fn show_one(d: &Disclosure) -> RenderTree {
    layout_with(d.build(|| div().child(text("More"))), 400.0, 200.0, STYLES)
}

#[test]
fn a_closed_disclosure_has_a_header_button_and_no_panel() {
    let _g = serial();
    let tree = show_one(&single_disclosure(&Log::default()));
    assert_eq!(attr(&tree, "dc", "role").as_deref(), Some("button"));
    assert_eq!(attr(&tree, "dc", "aria-expanded").as_deref(), Some("false"));
    assert_eq!(
        attr(&tree, "dc", "aria-controls").as_deref(),
        Some("dc-panel")
    );
    assert_eq!(attr(&tree, "dc", "tabindex").as_deref(), Some("0"));
    assert!(tree.query_by_id("dc-panel").is_none());
}

#[test]
fn enter_and_space_toggle_it_once_each_as_the_windows_click_and_the_state_follows() {
    let _g = serial();
    let log = Log::default();
    let d = single_disclosure(&log);
    let mut tree = show_one(&d);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "dc"));
    input.key(&mut tree, 13);
    frame(&mut tree, 400.0, 200.0);
    assert!(d.is_open());
    assert_eq!(attr(&tree, "dc", "aria-expanded").as_deref(), Some("true"));
    assert!(tree.query_by_id("dc-panel").is_some());
    assert!(
        has(&tree, "dc-host", "fs-disclosure-open"),
        "open: the host carries the class"
    );
    input.key(&mut tree, 32);
    frame(&mut tree, 400.0, 200.0);
    assert!(
        !d.is_open(),
        "the keyboard stayed on the header after it was drawn again, so Space closes it"
    );
    assert_eq!(attr(&tree, "dc", "aria-expanded").as_deref(), Some("false"));
    assert!(
        !has(&tree, "dc-host", "fs-disclosure-open"),
        "closed: no open class"
    );
    assert_eq!(*log.lock().unwrap(), ["true", "false"]);
}

#[test]
fn enter_and_space_as_raw_key_events_toggle_it() {
    let _g = serial();
    let log = Log::default();
    let d = single_disclosure(&log);
    let mut tree = show_one(&d);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "dc"));
    input.key_event_to(&mut tree, "dc", KEY_DOWN, 32);
    assert!(d.is_open(), "a raw Space opens it");
    frame(&mut tree, 400.0, 200.0);
    assert!(input.tab_to(&mut tree, "dc"));
    input.key_event_to(&mut tree, "dc", KEY_DOWN, 13);
    assert!(!d.is_open(), "a raw Enter closes it");
    input.key_event_to(&mut tree, "dc", KEY_DOWN, 65);
    assert!(!d.is_open(), "another key does nothing");
}

#[test]
fn a_click_toggles_once_the_program_can_set_it_and_the_open_class_follows() {
    let _g = serial();
    let log = Log::default();
    let d = single_disclosure(&log);
    let mut tree = show_one(&d);
    let mut input = Input::new();
    input.click(&mut tree, "dc");
    frame(&mut tree, 400.0, 200.0);
    assert_eq!(*log.lock().unwrap(), ["true"], "one click, one toggle");
    assert!(d.is_open());
    d.set_open(false);
    frame(&mut tree, 400.0, 200.0);
    assert_eq!(attr(&tree, "dc", "aria-expanded").as_deref(), Some("false"));
    assert_eq!(
        *log.lock().unwrap(),
        ["true"],
        "the program's own change is not a press"
    );
    let opened = Disclosure::new("A").id("op").opened();
    let tree = layout_with(
        opened.build(|| div().child(text("x"))),
        300.0,
        100.0,
        STYLES,
    );
    assert!(
        tree.query_by_id("op-panel").is_some(),
        "`opened` starts open"
    );
    assert!(has(&tree, "op-panel", "fs-disc-panel"));
}

fn accordion(log: &Log, single: bool) -> Accordion {
    let l = Arc::clone(log);
    let a = Accordion::new(vec![
        Section::new("a", "Alpha", || div().child(text("alpha body"))),
        Section::new("b", "Beta", || div().child(text("beta body"))),
        Section::new("c", "Gamma", || div().child(text("gamma body"))),
    ])
    .id("ac")
    .on_toggle(move |k, o| l.lock().unwrap().push(format!("{k}:{o}")));
    if single { a.single() } else { a }
}

fn show_acc(a: &Accordion) -> RenderTree {
    layout_with(a.build(), 400.0, 300.0, STYLES)
}

#[test]
fn each_header_sits_in_a_heading_of_the_set_level_and_names_its_region() {
    let _g = serial();
    let a = accordion(&Log::default(), false);
    let mut tree = show_acc(&a);
    assert_eq!(
        attr(&tree, "ac-head-a", "aria-expanded").as_deref(),
        Some("false")
    );
    assert_eq!(
        attr(&tree, "ac-head-a", "aria-controls").as_deref(),
        Some("ac-panel-a")
    );
    let mut input = Input::new();
    assert!(
        input.tab_to(&mut tree, "ac-head-b"),
        "every header is a tab stop"
    );
    input.key(&mut tree, 13);
    frame(&mut tree, 400.0, 300.0);
    assert_eq!(attr(&tree, "ac-panel-b", "role").as_deref(), Some("region"));
    assert_eq!(
        attr(&tree, "ac-panel-b", "aria-labelledby").as_deref(),
        Some("ac-head-b")
    );
    assert!(tree.query_by_id("ac-panel-a").is_none());
    assert!(has(&tree, "ac-panel-b", "fs-disc-panel"));
    assert!(
        has(&tree, "ac-section-b", "fs-disclosure-open"),
        "the open section carries the class"
    );
    assert!(!has(&tree, "ac-section-a", "fs-disclosure-open"));
}

#[test]
fn the_heading_level_is_a_setting_with_three_as_default_and_one_to_six() {
    let _g = serial();
    let level = |a: &Accordion| {
        let tree = show_acc(a);
        assert_eq!(
            attr(&tree, "ac-heading-a", "role").as_deref(),
            Some("heading")
        );
        attr(&tree, "ac-heading-a", "aria-level")
    };
    assert_eq!(
        level(&accordion(&Log::default(), false)).as_deref(),
        Some("3")
    );
    assert_eq!(
        level(&accordion(&Log::default(), false).level(2)).as_deref(),
        Some("2")
    );
    assert_eq!(
        level(&accordion(&Log::default(), false).level(0)).as_deref(),
        Some("1")
    );
    assert_eq!(
        level(&accordion(&Log::default(), false).level(9)).as_deref(),
        Some("6")
    );
}

#[test]
fn several_sections_may_be_open_together_unless_single() {
    let _g = serial();
    let log = Log::default();
    let a = accordion(&log, false);
    let mut tree = show_acc(&a);
    let mut input = Input::new();
    input.click(&mut tree, "ac-head-a");
    frame(&mut tree, 400.0, 300.0);
    input.click(&mut tree, "ac-head-c");
    frame(&mut tree, 400.0, 300.0);
    assert!(a.is_open("a") && a.is_open("c") && !a.is_open("b"));
    assert_eq!(*log.lock().unwrap(), ["a:true", "c:true"]);
    input.click(&mut tree, "ac-head-a");
    frame(&mut tree, 400.0, 300.0);
    assert!(!a.is_open("a"), "a header closes its own panel");
    assert_eq!(attr(&tree, "ac-head-a", "aria-disabled"), None);
}

#[test]
fn in_the_single_way_one_opens_the_others_close_and_the_open_header_is_inert() {
    let _g = serial();
    let log = Log::default();
    let a = accordion(&log, true);
    let mut tree = show_acc(&a);
    let mut input = Input::new();
    input.click(&mut tree, "ac-head-a");
    frame(&mut tree, 400.0, 300.0);
    input.click(&mut tree, "ac-head-b");
    frame(&mut tree, 400.0, 300.0);
    assert!(
        !a.is_open("a") && a.is_open("b"),
        "the first closed when the second opened"
    );
    assert_eq!(
        attr(&tree, "ac-head-b", "aria-disabled").as_deref(),
        Some("true"),
        "APG: the open one cannot close itself"
    );
    assert_eq!(attr(&tree, "ac-head-a", "aria-disabled"), None);
    input.click(&mut tree, "ac-head-b");
    frame(&mut tree, 400.0, 300.0);
    assert!(a.is_open("b"), "pressing the inert header does nothing");
    assert!(has(&tree, "ac-head-b", "fs-disc-head-off"));
    assert_eq!(*log.lock().unwrap(), ["a:true", "b:true"]);
}

#[test]
fn raw_keys_work_on_a_section_header_and_the_open_class_follows() {
    let _g = serial();
    let a = accordion(&Log::default(), false);
    let mut tree = show_acc(&a);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "ac-head-c"));
    input.key_event_to(&mut tree, "ac-head-c", KEY_DOWN, 32);
    frame(&mut tree, 400.0, 300.0);
    assert!(a.is_open("c"), "a raw Space opens it");
    assert!(
        has(&tree, "ac-head-c", "fs-disc-head-open"),
        "the open header carries its class"
    );
    a.set_open("c", false);
    frame(&mut tree, 400.0, 300.0);
    assert!(!has(&tree, "ac-head-c", "fs-disc-head-open"));
    a.set_open("nothing", true);
    assert!(!a.is_open("nothing"));
}
