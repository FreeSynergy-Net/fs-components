//! Test support for the crates that play a described input on the one board element (replay, app): a host that keeps the cards
//! (`MemoryBoard`, the same double the tests of this crate use) and builds the element over it.
//!
//! Pattern: none (tests). The element has a mouse drag (4 px rule): a card moves by pressing it, moving the pointer over another
//! column and releasing there.
#![allow(clippy::pedantic, dead_code)]

use std::sync::Arc;

use fs_components_fields::Look;
use fs_components_kanban::{BoardBench, Card, Column, Labels, MemoryBoard};
use fs_components_styles::StyleId;

/// Size of the window the tests play in.
pub const W: f32 = 900.0;
/// Size of the window the tests play in.
pub const H: f32 = 600.0;

/// The id prefix of the element: cards are `B-card-<key>`, columns `B-col-<key>`.
pub const ID: &str = "B";

fn column(key: &str, title: &str, cards: &[(&str, &str)]) -> Column {
    let mut c = Column::new(key, title);
    c.cards = cards.iter().map(|(k, t)| Card::new(k, t)).collect();
    c
}

/// A host with three columns and two cards in the first.
pub fn host() -> Arc<MemoryBoard> {
    Arc::new(MemoryBoard::new(vec![
        column("todo", "To do", &[("a", "Write the spec"), ("b", "Review")]),
        column("doing", "Doing", &[]),
        column("done", "Done", &[]),
    ]))
}

/// The element over the host, in the look of `style`.
pub fn board(h: &Arc<MemoryBoard>, style: StyleId) -> BoardBench {
    BoardBench::new(h.clone(), Labels::default())
        .events(h.clone())
        .id(ID)
        .look(Look::new(style, false))
}

/// The last thing the host heard (`moved:<card>:<from>><to>@<after>`), or nothing.
pub fn last(h: &MemoryBoard) -> String {
    h.log().last().cloned().unwrap_or_default()
}

/// The cards of a column, by key.
pub fn cards_in(h: &MemoryBoard, col: &str) -> Vec<String> {
    h.snapshot()
        .iter()
        .find(|c| c.key == col)
        .unwrap()
        .cards
        .iter()
        .map(|c| c.key.clone())
        .collect()
}

/// The pointer actions of a mouse drag from the centre of `from` to the centre of `to`: press, two moves (over the 4 px rule, then
/// onto the target), release, let the clock run.
pub fn drag(from: &str, to: &str) -> String {
    const ELEMENT: &str = "element-6066-11e4-a52e-4f735466cecf";
    format!(
        r#"{{"type":"pointerMove","origin":{{"{ELEMENT}":"{from}"}},"x":0,"y":0}},{{"type":"pointerDown","button":0}},{{"type":"pointerMove","origin":{{"{ELEMENT}":"{from}"}},"x":0,"y":20}},{{"type":"pointerMove","origin":{{"{ELEMENT}":"{to}"}},"x":0,"y":0}},{{"type":"pointerUp","button":0}},{{"type":"pause","duration":100}}"#
    )
}
