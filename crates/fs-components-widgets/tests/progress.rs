//! `progress`: role and ARIA values while the end is known, none while it is not, whole percent in the value text, the done and
//! unknown classes, `set` and `set_unknown` draw again, values off the range are held to it, no tab stop.
//!
//! Tests: attribute reads on the built element.

mod common;

use blinc_layout::prelude::RenderTree;
use common::{attr, frame, layout_with, serial};
use fs_components_widgets::progress::{Progress, STYLES};

fn show(p: &Progress) -> RenderTree {
    layout_with(p.build(200.0), 400.0, 60.0, STYLES)
}

fn has(tree: &RenderTree, id: &str, class: &str) -> bool {
    let node = tree.query_by_id(id).expect("the bar");
    tree.element_registry().has_class(node, class)
}

#[test]
fn a_bar_with_a_known_end_tells_name_value_limits_and_text() {
    let _g = serial();
    let p = Progress::new("Copying").id("pg");
    p.set(45.4);
    let tree = show(&p);
    assert_eq!(attr(&tree, "pg", "role").as_deref(), Some("progressbar"));
    assert_eq!(attr(&tree, "pg", "aria-label").as_deref(), Some("Copying"));
    assert_eq!(attr(&tree, "pg", "aria-valuenow").as_deref(), Some("45"));
    assert_eq!(attr(&tree, "pg", "aria-valuemin").as_deref(), Some("0"));
    assert_eq!(attr(&tree, "pg", "aria-valuemax").as_deref(), Some("100"));
    assert_eq!(attr(&tree, "pg", "aria-valuetext").as_deref(), Some("45 %"));
    assert_eq!(
        attr(&tree, "pg", "tabindex"),
        None,
        "a bar is read, not set"
    );
    assert_eq!(attr(&tree, "pg", "aria-busy"), None);
    assert!(!has(&tree, "pg", "fs-progress-unknown") && !has(&tree, "pg", "fs-progress-done"));
}

#[test]
fn an_unknown_end_has_no_value_attributes_at_all_and_is_busy() {
    let _g = serial();
    let p = Progress::new("Working").id("pu").unknown();
    let tree = show(&p);
    assert_eq!(attr(&tree, "pu", "role").as_deref(), Some("progressbar"));
    for name in [
        "aria-valuenow",
        "aria-valuemin",
        "aria-valuemax",
        "aria-valuetext",
    ] {
        assert_eq!(
            attr(&tree, "pu", name),
            None,
            "{name} is left out while the end is unknown"
        );
    }
    assert_eq!(attr(&tree, "pu", "aria-busy").as_deref(), Some("true"));
    assert!(has(&tree, "pu", "fs-progress-unknown"));
    assert_eq!(p.get(), None);
}

#[test]
fn set_draws_again_and_done_and_unknown_follow() {
    let _g = serial();
    let p = Progress::new("Copying").id("ps");
    let mut tree = show(&p);
    assert_eq!(attr(&tree, "ps", "aria-valuenow").as_deref(), Some("0"));
    p.set(70.0);
    frame(&mut tree, 400.0, 60.0);
    assert_eq!(attr(&tree, "ps", "aria-valuenow").as_deref(), Some("70"));
    p.set(100.0);
    frame(&mut tree, 400.0, 60.0);
    assert_eq!(
        attr(&tree, "ps", "aria-valuetext").as_deref(),
        Some("100 %")
    );
    assert!(has(&tree, "ps", "fs-progress-done"), "100 percent is done");
    p.set_unknown();
    frame(&mut tree, 400.0, 60.0);
    assert_eq!(attr(&tree, "ps", "aria-valuenow"), None);
    assert!(has(&tree, "ps", "fs-progress-unknown"));
    assert!(!has(&tree, "ps", "fs-progress-done"));
    p.set(10.0);
    frame(&mut tree, 400.0, 60.0);
    assert_eq!(
        attr(&tree, "ps", "aria-valuenow").as_deref(),
        Some("10"),
        "a value makes it known again"
    );
}

#[test]
fn values_off_the_range_are_held_and_the_fill_is_the_fraction_of_the_width() {
    let _g = serial();
    let p = Progress::new("x").id("pf");
    p.set(250.0);
    assert_eq!(p.get(), Some(100.0));
    p.set(-3.0);
    assert_eq!(p.get(), Some(0.0));
    p.set(50.0);
    let tree = show(&p);
    let node = tree.query_by_id("pf-fill").expect("the fill");
    let w = tree.get_absolute_bounds(node).expect("bounds").width;
    assert!((w - 100.0).abs() < 1.0, "half of 200 pixels, got {w}");
}

#[test]
fn a_change_within_the_same_whole_percent_does_not_draw_again() {
    let _g = serial();
    let p = Progress::new("x").id("pw");
    p.set(40.2);
    let mut tree = show(&p);
    p.set(40.4);
    frame(&mut tree, 400.0, 60.0);
    assert_eq!(attr(&tree, "pw", "aria-valuenow").as_deref(), Some("40"));
    p.set(40.9);
    frame(&mut tree, 400.0, 60.0);
    assert_eq!(attr(&tree, "pw", "aria-valuenow").as_deref(), Some("41"));
}
