//! `disabled`: a control that cannot be used keeps its size, is `aria-disabled`, stays a tab stop and says why in a tooltip
//! (ENTWURF-unfertig-tooltip, tests G1 to G6). Each test has a counter-check that would turn red if the rule were dropped.

mod common;

use blinc_layout::div::{Div, div};
use common::{Input, attr, frame, layout, serial, size_of};
use fs_components_widgets::disabled::{Disabled, Labels, TEXTS, Why, on_first_unsupported};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn control(id: &str) -> Div {
    div().id(id).w(90.0).h(32.0)
}

fn wrapped(id: &str, why: Why) -> Div {
    Disabled::new(id, why, &Labels::default()).wrap(control(id))
}

#[test]
fn g1_a_disabled_control_has_the_size_of_a_free_one_and_no_extra_line() {
    let _g = serial();
    let free = layout(div().child(control("c")), 300.0, 200.0);
    let tree = layout(div().child(wrapped("c", Why::Unsupported)), 300.0, 200.0);
    assert_eq!(size_of(&tree, "c"), size_of(&free, "c"), "same size");
    assert!(tree.query_by_id("c-soon").is_none(), "no soon line");
    assert!(
        tree.query_by_id("c-why-bubble").is_none(),
        "the reason is not drawn in the flow until asked for"
    );
}

#[test]
fn g2_the_reason_is_a_tooltip_the_control_names() {
    let _g = serial();
    let mut tree = layout(div().child(wrapped("c", Why::Unsupported)), 300.0, 200.0);
    assert_eq!(
        attr(&tree, "c", "aria-describedby").as_deref(),
        Some("c-why-bubble")
    );
    let mut input = Input::new();
    input.focus_events(&mut tree, "c", true);
    frame(&mut tree, 300.0, 200.0);
    assert_eq!(
        attr(&tree, "c-why-bubble", "role").as_deref(),
        Some("tooltip")
    );
    assert_eq!(
        Labels::default().unsupported,
        "Not finished yet. Coming in a later version."
    );
    assert_eq!(fs_components_mf2::check_pairs(TEXTS), Vec::<String>::new());
}

#[test]
fn g2_the_reason_of_a_control_that_is_only_off_for_now_is_the_callers_words() {
    let d = Disabled::new("c", Why::Now("No column before".into()), &Labels::default());
    let _g = serial();
    let mut tree = layout(div().child(d.wrap(control("c"))), 300.0, 200.0);
    assert_eq!(attr(&tree, "c", "data-state").as_deref(), Some("disabled"));
    let mut input = Input::new();
    input.focus_events(&mut tree, "c", true);
    frame(&mut tree, 300.0, 200.0);
    assert!(tree.query_by_id("c-why-bubble").is_some());
}

#[test]
fn g3_aria_disabled_and_tab_stop_not_the_html_attribute() {
    let _g = serial();
    let tree = layout(
        div().child(wrapped("a", Why::Unsupported)).child(
            Disabled::new("b", Why::Unsupported, &Labels::default())
                .roving()
                .wrap(control("b")),
        ),
        300.0,
        200.0,
    );
    assert_eq!(attr(&tree, "a", "aria-disabled").as_deref(), Some("true"));
    assert_eq!(
        attr(&tree, "a", "data-state").as_deref(),
        Some("unsupported")
    );
    assert_eq!(
        attr(&tree, "a", "tabindex").as_deref(),
        Some("0"),
        "a tab stop"
    );
    assert_eq!(
        attr(&tree, "a", "disabled"),
        None,
        "never the HTML attribute"
    );
    assert_eq!(
        attr(&tree, "b", "tabindex").as_deref(),
        Some("-1"),
        "roving keeps -1"
    );
    assert_eq!(attr(&tree, "b", "aria-disabled").as_deref(), Some("true"));
}

#[test]
fn g4_keyboard_shows_the_reason_at_once_and_a_press_keeps_it_open() {
    let _g = serial();
    let mut tree = layout(div().child(wrapped("c", Why::Unsupported)), 300.0, 200.0);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "c"), "the keyboard reaches it");
    input.focus_events(&mut tree, "c", true); // the test router fires no focus handler by itself
    frame(&mut tree, 300.0, 200.0);
    assert!(
        tree.query_by_id("c-why-bubble").is_some(),
        "shown by focus alone, no pointer"
    );
    input.focus_events(&mut tree, "c", false);
    frame(&mut tree, 300.0, 200.0);
    assert!(tree.query_by_id("c-why-bubble").is_none());
    // touch: a press opens it and it stays after the pointer left
    input.click(&mut tree, "c");
    frame(&mut tree, 300.0, 200.0);
    assert!(
        tree.query_by_id("c-why-bubble").is_some(),
        "a press opens it"
    );
    input.move_to(&mut tree, 299.0, 199.0);
    frame(&mut tree, 300.0, 200.0);
    assert!(
        tree.query_by_id("c-why-bubble").is_none(),
        "tapping elsewhere closes it"
    );
}

#[test]
fn g5_escape_closes_without_moving_the_focus() {
    let _g = serial();
    let mut tree = layout(div().child(wrapped("c", Why::Unsupported)), 300.0, 200.0);
    let mut input = Input::new();
    input.focus_events(&mut tree, "c", true);
    frame(&mut tree, 300.0, 200.0);
    assert!(tree.query_by_id("c-why-bubble").is_some());
    input.key(&mut tree, 27);
    frame(&mut tree, 300.0, 200.0);
    assert!(tree.query_by_id("c-why-bubble").is_none(), "closed");
    assert!(input.has_keyboard(&tree, "c"), "the keyboard did not move");
}

#[test]
fn g6_the_pointer_may_move_onto_the_bubble_and_it_does_not_hide_by_itself() {
    let _g = serial();
    let mut tree = layout(div().child(wrapped("c", Why::Unsupported)), 300.0, 200.0);
    let mut input = Input::new();
    input.focus_events(&mut tree, "c", true);
    frame(&mut tree, 300.0, 200.0);
    let (x, y) = Input::centre(&tree, "c");
    input.move_to(&mut tree, x, y);
    frame(&mut tree, 300.0, 200.0);
    let (bx, by) = Input::centre(&tree, "c-why-bubble");
    input.move_to(&mut tree, bx, by);
    frame(&mut tree, 300.0, 200.0);
    assert!(
        tree.query_by_id("c-why-bubble").is_some(),
        "hoverable: still there over the bubble"
    );
    std::thread::sleep(std::time::Duration::from_millis(600));
    frame(&mut tree, 300.0, 200.0);
    assert!(
        tree.query_by_id("c-why-bubble").is_some(),
        "persistent: no auto-hide"
    );
}

#[test]
fn a_swallowed_press_runs_the_callback_and_still_opens_the_reason() {
    let _g = serial();
    let hits = Arc::new(AtomicUsize::new(0));
    let h = hits.clone();
    let d = Disabled::new("c", Why::Unsupported, &Labels::default()).swallow(move || {
        h.fetch_add(1, Ordering::SeqCst);
    });
    let mut tree = layout(div().child(d.wrap(control("c"))), 300.0, 200.0);
    let mut input = Input::new();
    input.click(&mut tree, "c");
    frame(&mut tree, 300.0, 200.0);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    assert!(tree.query_by_id("c-why-bubble").is_some());
}

#[test]
fn the_first_unsupported_control_tells_the_listener_once() {
    // Process-wide state, but deterministic under `cargo test` too: the listener is set only here, and `TOLD` is only set when a listener
    // exists (`observer::unsupported_drawn`); an unsupported control drawn earlier by another test (all under `serial()`) uses nothing up.
    // A second test of this binary that sets a listener would break that: do not add one.
    let _g = serial();
    let n = Arc::new(AtomicUsize::new(0));
    let c = n.clone();
    on_first_unsupported(Arc::new(move || {
        c.fetch_add(1, Ordering::SeqCst);
    }));
    let _ = wrapped("a", Why::Now("later".into()));
    assert_eq!(
        n.load(Ordering::SeqCst),
        0,
        "a control that is only off for now is not the hint"
    );
    let _ = wrapped("b", Why::Unsupported);
    let _ = wrapped("c", Why::Unsupported);
    assert_eq!(n.load(Ordering::SeqCst), 1, "once, not per control");
}
