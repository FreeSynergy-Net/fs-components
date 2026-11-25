//! Test support of the card tests: references, places and a container with fixed rectangles.
//!
//! Pattern: none (tests).
#![allow(dead_code)]

use fs_components_widgets::card::{CardRef, Placed, Rect, Target, Zone};

/// The card `n` as a reference (an AS2 id and a media type).
pub fn card(n: &str) -> CardRef {
    CardRef::parse(
        &format!("https://example.org/tasks/{n}"),
        "application/activity+json",
    )
    .unwrap()
}

/// Three columns of 100 x 400 side by side (gap 20) in the lane `lane`: `todo` at x 0, `doing` at x 120, `done` at x 240, from y 0.
pub fn zones(lane: Option<&str>) -> Vec<Zone> {
    ["todo", "doing", "done"]
        .iter()
        .enumerate()
        .map(|(i, c)| Zone {
            lane: lane.map(str::to_owned),
            column: (*c).to_owned(),
            rect: Rect::new(i as f32 * 120.0, 0.0, 100.0, 400.0),
        })
        .collect()
}

/// Cards of 100 x 40 with a gap of 10, from y 10, in `column`.
pub fn stack(lane: Option<&str>, column: &str, x: f32, names: &[&str]) -> Vec<Placed> {
    names
        .iter()
        .enumerate()
        .map(|(i, n)| Placed {
            card: card(n),
            lane: lane.map(str::to_owned),
            column: column.to_owned(),
            rect: Rect::new(x, 10.0 + i as f32 * 50.0, 100.0, 40.0),
        })
        .collect()
}

/// The place of the cell `column` with these neighbours.
pub fn at(column: &str, after: Option<&str>, before: Option<&str>) -> Target {
    Target {
        lane: None,
        column: column.to_owned(),
        after: after.map(card),
        before: before.map(card),
    }
}
