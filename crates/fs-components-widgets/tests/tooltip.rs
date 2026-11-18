//! `tooltip`: role and `aria-describedby`, hover after a delay, focus at once, Escape, the keyboard stays on the trigger.
//!
//! Tests: model tests with explicit milliseconds, attribute reads and an in-process pointer and keyboard over the built host.

mod common;

use blinc_layout::div::div;
use common::{Input, attr, frame, layout, serial};
use fs_components_widgets::tooltip::{DEFAULT_DELAY_MS, Tooltip};

fn host(tip: &Tooltip) -> blinc_layout::div::Div {
    tip.attach(
        div()
            .id("tip-trigger")
            .attr("tabindex", "0")
            .w(80.0)
            .h(30.0),
    )
}

#[test]
fn a_hover_shows_after_the_delay_and_leaving_hides() {
    let tip = Tooltip::new("Save").id("tip");
    assert_eq!(DEFAULT_DELAY_MS, 400);
    tip.hover_at(true, 1_000);
    assert!(!tip.is_visible_at(1_399), "not before the delay");
    assert!(tip.is_visible_at(1_400));
    tip.hover_at(false, 2_000);
    assert!(!tip.is_visible_at(2_000));
    let quick = Tooltip::new("x").delay(0);
    quick.hover_at(true, 5);
    assert!(quick.is_visible_at(5), "a delay of 0 shows at once");
}

#[test]
fn focus_shows_at_once_and_blur_hides() {
    let tip = Tooltip::new("Save");
    tip.focus(true);
    assert!(tip.is_visible_at(0));
    tip.focus(false);
    assert!(!tip.is_visible_at(10_000));
}

#[test]
fn escape_closes_until_pointer_and_keyboard_have_left() {
    let tip = Tooltip::new("Save").delay(0);
    tip.hover_at(true, 0);
    tip.focus(true);
    tip.escape();
    assert!(!tip.is_visible_at(10), "closed by Escape");
    tip.hover_at(false, 20);
    assert!(
        !tip.is_visible_at(20),
        "the keyboard is still there: still closed"
    );
    tip.focus(false);
    tip.focus(true);
    assert!(tip.is_visible_at(30), "a new visit shows it again");
}

#[test]
fn the_bubble_has_role_tooltip_and_the_trigger_names_it() {
    let _g = serial();
    let tip = Tooltip::new("Save the page").id("tip").delay(0);
    let mut tree = layout(host(&tip), 300.0, 200.0);
    assert_eq!(
        attr(&tree, "tip-trigger", "aria-describedby").as_deref(),
        Some("tip-bubble")
    );
    assert!(tree.query_by_id("tip-bubble").is_none(), "hidden at first");
    let mut input = Input::new();
    input.focus_events(&mut tree, "tip-trigger", true);
    frame(&mut tree, 300.0, 200.0);
    assert!(tip.is_visible());
    assert_eq!(
        attr(&tree, "tip-bubble", "role").as_deref(),
        Some("tooltip")
    );
    assert!(
        input.has_keyboard(&tree, "tip-trigger"),
        "the keyboard stays on the trigger"
    );
    input.focus_events(&mut tree, "tip-trigger", false);
    frame(&mut tree, 300.0, 200.0);
    assert!(
        !tip.is_visible(),
        "a focus tooltip closes when the keyboard leaves"
    );
}

#[test]
fn escape_on_the_trigger_closes_the_bubble() {
    let _g = serial();
    let tip = Tooltip::new("Save the page").id("tip").delay(0);
    let mut tree = layout(host(&tip), 300.0, 200.0);
    let mut input = Input::new();
    input.focus_events(&mut tree, "tip-trigger", true);
    frame(&mut tree, 300.0, 200.0);
    assert!(tree.query_by_id("tip-bubble").is_some());
    input.key(&mut tree, 27);
    frame(&mut tree, 300.0, 200.0);
    assert!(!tip.is_visible());
    assert!(tree.query_by_id("tip-bubble").is_none());
}

#[test]
fn the_pointer_over_the_host_shows_the_bubble_and_leaving_hides() {
    let _g = serial();
    let tip = Tooltip::new("Save the page").id("tip").delay(0);
    let mut tree = layout(host(&tip), 300.0, 200.0);
    let mut input = Input::new();
    let (x, y) = Input::centre(&tree, "tip-trigger");
    input.move_to(&mut tree, x, y);
    frame(&mut tree, 300.0, 200.0);
    assert!(tip.is_visible(), "the pointer is over the trigger");
    input.move_to(&mut tree, 299.0, 199.0);
    frame(&mut tree, 300.0, 200.0);
    assert!(!tip.is_visible());
}

#[test]
fn a_hover_draws_the_bubble_once_the_delay_is_over() {
    // Mutant O2 of the review (the timer thread of `hover` never starts) survived: nothing drew the bubble after the delay.
    let _g = serial();
    let tip = Tooltip::new("Save the page").id("tip").delay(80);
    let mut tree = layout(host(&tip), 300.0, 200.0);
    tip.hover(true);
    frame(&mut tree, 300.0, 200.0);
    assert!(!tip.is_visible_at(0), "not yet: the delay is not over");
    std::thread::sleep(std::time::Duration::from_millis(600));
    frame(&mut tree, 300.0, 200.0);
    assert!(tip.is_visible());
    assert!(
        tree.query_by_id("tip-bubble").is_some(),
        "the timer must have asked for a new draw after the delay"
    );
}
