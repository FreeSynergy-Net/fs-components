//! The pointer machine of the card and the rule of the place under a point (moved from the board's `drag.rs` and `hit.rs`, the cases
//! of `board_model.rs` kept): a click stays a click below 4 px, a carry past it, a release over no place gives the card back, the
//! middle of a card is the border between "before" and "after", the gap between two cells belongs to none, both edges belong to the cell.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]
mod card_support;

use card_support::{at, card, stack, zones};
use fs_components_widgets::card::gesture::{Gesture, Moved, Over, Release, THRESHOLD};
use fs_components_widgets::card::{CardRef, Rect, Target, line_at, target_at};

/// A container of one place: the left half is place 1.
struct Halves;
impl Over<String, u32> for Halves {
    fn place(&self, x: f32, _: f32, _: Option<&String>) -> Option<u32> {
        (x >= 0.0).then_some(u32::from(x > 100.0))
    }
}

type G = Gesture<String, String, u32>;

#[test]
fn a_press_below_four_pixels_is_a_click() {
    let mut g = G::default();
    g.press("a", "todo", (10.0, 10.0));
    assert_eq!(
        g.pointer((12.0, 12.0), (12.0, 12.0), &Halves),
        Moved::Nothing
    );
    const { assert!(THRESHOLD > 2.9 && THRESHOLD < 4.1) };
    assert_eq!(g.release(), Release::Click("a".into()));
    assert_eq!(
        g.release(),
        Release::Nothing,
        "a second release finds nothing going on"
    );
}

#[test]
fn a_press_moved_past_the_threshold_carries_and_drops() {
    let mut g = G::default();
    g.press("a", "todo", (10.0, 10.0));
    assert_eq!(
        g.pointer((10.0, 20.0), (10.0, 20.0), &Halves),
        Moved::Started
    );
    assert_eq!(
        g.pointer((150.0, 20.0), (150.0, 20.0), &Halves),
        Moved::Carried
    );
    assert_eq!(g.carried(), Some(&"a".to_owned()));
    assert_eq!(
        g.release(),
        Release::Drop {
            card: "a".into(),
            from: "todo".into(),
            spot: 1
        }
    );
}

#[test]
fn a_release_over_no_place_gives_the_card_back_and_escape_gives_up() {
    let mut g = G::default();
    g.press("a", "todo", (10.0, 10.0));
    g.pointer((-50.0, 20.0), (-50.0, 20.0), &Halves);
    assert_eq!(g.release(), Release::Back);
    g.press("a", "todo", (10.0, 10.0));
    g.pointer((50.0, 50.0), (50.0, 50.0), &Halves);
    assert!(g.cancel());
    assert!(!g.cancel(), "nothing left to give up");
    assert_eq!(g.release(), Release::Nothing);
}

#[test]
fn the_cell_under_the_point_and_the_edges() {
    let z = zones(None);
    let c = stack(None, "doing", 120.0, &["a"]);
    let t = |x: f32, y: f32| target_at(&z, &c, (x, y), None).map(|t| t.column);
    assert_eq!(t(10.0, 10.0).as_deref(), Some("todo"));
    assert_eq!(
        t(100.0, 10.0).as_deref(),
        Some("todo"),
        "the right edge belongs to the cell"
    );
    assert_eq!(t(110.0, 10.0), None, "the gap belongs to none");
    assert_eq!(
        t(120.0, 400.0).as_deref(),
        Some("doing"),
        "the bottom edge too"
    );
    assert_eq!(t(130.0, 401.0), None);
}

#[test]
fn between_cards_the_middle_is_the_border_and_a_card_at_its_middle_counts_as_above() {
    let z = zones(None);
    let c = stack(None, "todo", 0.0, &["a", "b"]);
    let mid_a = 10.0 + 20.0;
    let at_y = |y: f32, skip: Option<&CardRef>| target_at(&z, &c, (30.0, y), skip).unwrap();
    assert_eq!(at_y(0.0, None), at("todo", None, Some("a")));
    assert_eq!(
        at_y(mid_a, None),
        at("todo", None, Some("a")),
        "exactly at the middle: still before a"
    );
    assert_eq!(at_y(mid_a + 0.1, None), at("todo", Some("a"), Some("b")));
    assert_eq!(at_y(399.0, None).after, Some(card("b")));
    assert_eq!(
        at_y(399.0, None).before,
        None,
        "below the last card: the end of the cell"
    );
    // the carried card is not counted among the cards of its own cell
    assert_eq!(at_y(300.0, Some(&card("b"))), at("todo", Some("a"), None));
}

#[test]
fn an_empty_cell_takes_a_card_and_lanes_are_told_apart() {
    let mut z = zones(Some("anna"));
    z.extend(zones(Some("ben")).into_iter().map(|mut z| {
        z.rect = Rect::new(z.rect.x, 500.0, z.rect.w, 100.0);
        z
    }));
    let c = stack(Some("anna"), "todo", 0.0, &["a"]);
    let hit = |x: f32, y: f32| target_at(&z, &c, (x, y), None).unwrap();
    assert_eq!(hit(250.0, 50.0).lane.as_deref(), Some("anna"));
    let ben = hit(250.0, 550.0);
    assert_eq!(
        (
            ben.lane.as_deref(),
            ben.column.as_str(),
            ben.after,
            ben.before
        ),
        (Some("ben"), "done", None, None)
    );
    assert!(
        target_at(&z, &c, (250.0, 450.0), None).is_none(),
        "between the lanes"
    );
    // a card of another lane does not count in the cell of this one
    let mixed = [c.clone(), stack(Some("ben"), "todo", 0.0, &["x"])].concat();
    assert_eq!(
        target_at(&z, &mixed, (30.0, 300.0), None).unwrap().after,
        Some(card("a"))
    );
}

#[test]
fn the_line_stands_between_the_neighbours() {
    let z = zones(None);
    let c = stack(None, "todo", 0.0, &["a", "b"]); // a: 10..50, b: 60..100
    let line = |t: Target| line_at(&z, &c, &t).unwrap();
    assert_eq!(line(at("todo", Some("a"), Some("b"))).y, 55.0);
    assert_eq!(line(at("todo", None, Some("a"))).y, 10.0);
    assert_eq!(line(at("todo", Some("b"), None)).y, 100.0);
    assert_eq!(
        line(at("done", None, None)).y,
        0.0,
        "an empty cell: its top"
    );
    assert_eq!(
        (
            line(at("doing", None, None)).x,
            line(at("doing", None, None)).w
        ),
        (120.0, 100.0)
    );
    assert!(line_at(&z, &c, &at("nowhere", None, None)).is_none());
}
