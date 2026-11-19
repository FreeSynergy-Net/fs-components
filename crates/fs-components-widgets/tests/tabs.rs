//! `tabs`: roles and ARIA state, the keys (arrows by the window's order, Home and End, Enter and Space as the window's click AND as
//! raw keys), the automatic and the manual way, disabled tabs, the state classes, the panel, the callback.
//!
//! Tests: attribute reads on the built element, an in-process keyboard and pointer, the key events a window sends.

mod common;

use std::sync::{Arc, Mutex};

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
fn the_row_is_a_named_tablist_and_each_tab_tells_its_state_and_its_panel() {
    let _g = serial();
    let t = tabs(&Log::default());
    let tree = show(&t);
    assert_eq!(attr(&tree, "tb", "role").as_deref(), Some("tablist"));
    assert_eq!(attr(&tree, "tb", "aria-label").as_deref(), Some("Sections"));
    assert_eq!(
        attr(&tree, "tb", "aria-orientation").as_deref(),
        Some("horizontal")
    );
    assert_eq!(
        attr(&tree, "tb", "tabindex").as_deref(),
        Some("0"),
        "one tab stop"
    );
    for (key, selected) in [("one", "true"), ("two", "false"), ("three", "false")] {
        let id = format!("tb-tab-{key}");
        assert_eq!(attr(&tree, &id, "role").as_deref(), Some("tab"));
        assert_eq!(attr(&tree, &id, "aria-selected").as_deref(), Some(selected));
        assert_eq!(
            attr(&tree, &id, "aria-controls"),
            Some(format!("tb-panel-{key}"))
        );
    }
    assert_eq!(
        attr(&tree, "tb-tab-two", "aria-disabled").as_deref(),
        Some("true")
    );
    assert_eq!(attr(&tree, "tb-tab-one", "aria-disabled"), None);
}

#[test]
fn the_panel_is_a_tabpanel_named_by_its_tab_and_follows_the_choice() {
    let _g = serial();
    let t = tabs(&Log::default());
    let mut tree = show(&t);
    assert_eq!(
        attr(&tree, "tb-panel-one", "role").as_deref(),
        Some("tabpanel")
    );
    assert_eq!(
        attr(&tree, "tb-panel-one", "aria-labelledby").as_deref(),
        Some("tb-tab-one")
    );
    assert!(tree.query_by_id("tb-panel-three").is_none());
    assert!(t.choose("three"));
    frame(&mut tree, 500.0, 200.0);
    assert_eq!(
        attr(&tree, "tb-panel-three", "role").as_deref(),
        Some("tabpanel")
    );
    assert_eq!(
        attr(&tree, "tb-panel-three", "aria-labelledby").as_deref(),
        Some("tb-tab-three")
    );
    assert!(
        tree.query_by_id("tb-panel-one").is_none(),
        "only the chosen panel is there"
    );
    assert_eq!(
        attr(&tree, "tb-tab-three", "aria-selected").as_deref(),
        Some("true")
    );
    assert_eq!(
        attr(&tree, "tb-tab-one", "aria-selected").as_deref(),
        Some("false")
    );
    for (to, was) in [("four", "three"), ("one", "four"), ("three", "one")] {
        assert!(t.choose(to));
        frame(&mut tree, 500.0, 200.0);
        assert_eq!(
            attr(&tree, &format!("tb-tab-{to}"), "aria-selected").as_deref(),
            Some("true"),
            "{to} after {was}"
        );
        assert_eq!(
            attr(&tree, &format!("tb-tab-{was}"), "aria-selected").as_deref(),
            Some("false"),
            "{was} after {to}"
        );
    }
}

#[test]
fn arrows_choose_in_the_automatic_way_skip_the_disabled_tab_and_wrap() {
    let _g = serial();
    let log = Log::default();
    let t = tabs(&log);
    let mut tree = show(&t);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "tb"), "the row is one tab stop");
    input.key(&mut tree, 39);
    assert_eq!(t.chosen().as_deref(), Some("three"), "Two is disabled");
    input.key(&mut tree, 39);
    assert_eq!(t.chosen().as_deref(), Some("four"));
    input.key(&mut tree, 39);
    assert_eq!(t.chosen().as_deref(), Some("one"), "the move wraps");
    input.key(&mut tree, 37);
    assert_eq!(t.chosen().as_deref(), Some("four"), "and wraps back");
    input.key(&mut tree, 36);
    assert_eq!(t.chosen().as_deref(), Some("one"));
    input.key(&mut tree, 35);
    assert_eq!(t.chosen().as_deref(), Some("four"));
    input.key(&mut tree, 40);
    assert_eq!(
        t.chosen().as_deref(),
        Some("four"),
        "Down does nothing in a horizontal row"
    );
    assert_eq!(
        seen(&log),
        ["three", "four", "one", "four", "one", "four"],
        "each change is heard once"
    );
}
