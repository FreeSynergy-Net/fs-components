//! `MeasuredTargets` (P4): the places come from measured rectangles, with the P0 conditions: a scroll without a layout moves the
//! rectangles by the way scrolled (test "scroll without layout"), a layout that has run is not shifted twice, a zoom is the layout's
//! size (the rule works on any size), what is not laid out is left out, the container's rule is asked, the line of the place.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]
mod card_support;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use card_support::{at, card};
use fs_components_widgets::answer::Reason;
use fs_components_widgets::card::{Bounds, DropTargets, MeasuredTargets, Rect, Scrolled};

/// Fixed rectangles by element id, as the laid-out tree would say them.
#[derive(Default)]
struct Fixed(Mutex<HashMap<String, Rect>>);
impl Fixed {
    fn put(&self, id: &str, r: Rect) {
        self.0.lock().unwrap().insert(id.to_owned(), r);
    }
}
impl Bounds for Fixed {
    fn bounds(&self, id: &str) -> Option<Rect> {
        self.0.lock().unwrap().get(id).copied()
    }
}

struct Offset(Mutex<(f32, f32)>);
impl Scrolled for Offset {
    fn offset(&self) -> (f32, f32) {
        *self.0.lock().unwrap()
    }
}

/// Two columns, `todo` with cards a, b, c (80 high, 10 gap, from y 40 = under a head), `doing` empty; `zoom` scales the LAYOUT sizes.
fn layout(zoom: f32, dy: f32) -> Arc<Fixed> {
    let b = Arc::new(Fixed::default());
    b.put("cell-todo", Rect::new(0.0, dy, 300.0 * zoom, 600.0 * zoom));
    b.put(
        "cell-doing",
        Rect::new(320.0 * zoom, dy, 300.0 * zoom, 600.0 * zoom),
    );
    for (i, n) in ["a", "b", "c"].iter().enumerate() {
        b.put(
            &format!("card-{n}"),
            Rect::new(
                0.0,
                dy + (40.0 + i as f32 * 90.0) * zoom,
                300.0 * zoom,
                80.0 * zoom,
            ),
        );
    }
    b
}

fn targets(b: Arc<Fixed>) -> MeasuredTargets {
    let mut t = MeasuredTargets::new(b)
        .zone(None, "todo", "cell-todo")
        .zone(None, "doing", "cell-doing");
    for n in ["a", "b", "c"] {
        t = t.card(card(n), None, "todo", &format!("card-{n}"));
    }
    t
}

#[test]
fn the_place_comes_from_the_rectangles() {
    let t = targets(layout(1.0, 0.0));
    // y 140 is the middle of b (130..210)? b: 130..210, middle 170; a: 40..120, middle 80
    assert_eq!(
        t.target_at((50.0, 100.0), &card("x")).unwrap(),
        at("todo", Some("a"), Some("b"))
    );
    assert_eq!(
        t.target_at((50.0, 10.0), &card("x")).unwrap(),
        at("todo", None, Some("a"))
    );
    assert_eq!(
        t.target_at((50.0, 590.0), &card("x")).unwrap(),
        at("todo", Some("c"), None)
    );
    assert_eq!(
        t.target_at((400.0, 300.0), &card("x")).unwrap(),
        at("doing", None, None)
    );
    assert!(t.target_at((310.0, 300.0), &card("x")).is_none(), "the gap");
    // the carried card is not among its own neighbours
    assert_eq!(
        t.target_at((50.0, 100.0), &card("a")).unwrap(),
        at("todo", None, Some("b"))
    );
}

#[test]
fn scroll_without_layout_moves_the_rectangles_by_the_way_scrolled() {
    // P0 B1: the cache is stale after a scroll by exactly the way scrolled. The scroll is -300; the bounds still say the old layout.
    let bounds = layout(1.0, 0.0);
    let scroll = Arc::new(Offset(Mutex::new((0.0, 0.0))));
    let t = targets(bounds.clone()).scrolled(scroll.clone());
    assert_eq!(
        t.target_at((50.0, 300.0), &card("x")).unwrap().after,
        Some(card("c")),
        "before the scroll"
    );
    *scroll.0.lock().unwrap() = (0.0, -150.0);
    // the pointer is at y 100 of the window: with the content scrolled up by 150 that is y 250 of the content = after b, before c
    assert_eq!(
        t.target_at((50.0, 100.0), &card("x")).unwrap(),
        at("todo", Some("b"), Some("c"))
    );
    // without the correction the answer would have been the old one
    let plain = targets(bounds.clone());
    assert_eq!(
        plain.target_at((50.0, 100.0), &card("x")).unwrap(),
        at("todo", Some("a"), Some("b"))
    );
    assert!(
        t.target_at((50.0, 500.0), &card("x")).is_none(),
        "below the scrolled cell"
    );
    assert!(t.target_at((50.0, 400.0), &card("x")).is_some());
}

#[test]
fn a_layout_that_has_run_is_not_shifted_twice() {
    let scroll = Arc::new(Offset(Mutex::new((0.0, 0.0))));
    let t = targets(layout(1.0, 0.0)).scrolled(scroll.clone());
    *scroll.0.lock().unwrap() = (0.0, -150.0);
    let stale = t.target_at((50.0, 100.0), &card("x")).unwrap();
    // the layout runs: the bounds now say the true places (dy = -150) and the container says so
    let fresh = targets(layout(1.0, -150.0)).scrolled(scroll.clone());
    fresh.layout_done();
    assert_eq!(
        fresh.target_at((50.0, 100.0), &card("x")).unwrap(),
        stale,
        "same answer before and after the layout"
    );
    t.layout_done();
    // the old bounds with layout_done would now be taken as true: the answer differs (the container must not call it before the layout)
    assert_ne!(t.target_at((50.0, 100.0), &card("x")).unwrap(), stale);
}

#[test]
fn zoom_is_the_layout_size_and_the_rule_is_the_same() {
    for zoom in [0.5_f32, 1.0, 1.5, 2.0] {
        let t = targets(layout(zoom, 0.0));
        // the point between a and b scales with the layout
        let y = (40.0 + 85.0) * zoom;
        assert_eq!(
            t.target_at((50.0 * zoom, y), &card("x")).unwrap(),
            at("todo", Some("a"), Some("b")),
            "zoom {zoom}"
        );
        assert_eq!(
            t.line(&at("todo", Some("a"), Some("b"))).unwrap().y,
            y,
            "the line is in the gap at zoom {zoom}"
        );
    }
}

#[test]
fn what_is_not_laid_out_is_left_out_and_the_rule_of_the_container_is_asked() {
    let b = layout(1.0, 0.0);
    let t = targets(b.clone()).rule(Arc::new(|c, to| {
        if to.column == "doing" && c.iri_text().ends_with("/a") {
            Err(Reason::new("wip", "Doing is full."))
        } else {
            Ok(())
        }
    }));
    assert_eq!(
        t.can_drop(&card("a"), &at("doing", None, None))
            .unwrap_err()
            .code,
        "wip"
    );
    assert!(t.can_drop(&card("b"), &at("doing", None, None)).is_ok());
    let lonely = MeasuredTargets::new(Arc::new(Fixed::default())).zone(None, "todo", "cell-todo");
    assert!(
        lonely.target_at((10.0, 10.0), &card("x")).is_none(),
        "no layout, no place"
    );
    assert!(
        lonely.can_drop(&card("x"), &at("todo", None, None)).is_ok(),
        "no rule, every cell takes every card"
    );
    let (zones, cards) = targets(b).read();
    assert_eq!((zones.len(), cards.len()), (2, 3));
}

/// The container reads the scroll of the layout from the tree and says it (`laid_out_at`), whenever the scroll moved after it: the old
/// bounds (laid out at dy 0) are corrected by the way scrolled since, never guessed from "now".
#[test]
fn the_scroll_of_the_layout_is_told_not_guessed() {
    let scroll = Arc::new(Offset(Mutex::new((0.0, -150.0))));
    // first look after a scroll without layout: the bounds are those of dy 0, the container reads that from the tree
    let t = targets(layout(1.0, 0.0)).scrolled(scroll.clone());
    t.laid_out_at((0.0, 0.0));
    assert_eq!(
        t.target_at((50.0, 100.0), &card("x")).unwrap(),
        at("todo", Some("b"), Some("c")),
        "shifted by the 150 px scrolled since the layout"
    );
    // a layout ran at dy -150: the tree says so
    let fresh = targets(layout(1.0, -150.0)).scrolled(scroll);
    fresh.laid_out_at((0.0, -150.0));
    assert_eq!(
        fresh.target_at((50.0, 100.0), &card("x")).unwrap(),
        at("todo", Some("b"), Some("c")),
        "no shift twice"
    );
}
