//! Group `kanban`, the board in lanes: a band per lane (head line with name, count and fold; an empty lane; a lane only the cards name;
//! the cards without a lane), and the plain board with the line "list of lanes missing".
//!
//! Tests: picture comparison against `contract/umbau/bilder/kanban` (`board_lanes`, `board_lanes_missing`; drawn on the software GPU,
//! skipped with a message without one, a failure when `CI` is set).
//! Pattern: none (tests).

use std::sync::Arc;

use fs_components_fields::Look;
use fs_components_kanban::{BoardBench, Card, Column, Labels, LaneHead, LanePlan, MemoryBoard};
use fs_components_pictures::{Group, Tolerance};
use fs_components_styles::StyleId;

fn card(key: &str, title: &str, who: &[&str]) -> Card {
    let mut c = Card::new(key, title);
    c.assignees = who.iter().map(|w| (*w).to_owned()).collect();
    c
}

fn board(plan: LanePlan) -> BoardBench {
    let mut todo = Column::new("todo", "To do");
    todo.cards = vec![
        card("a", "Write the spec", &["Ada Lovelace"]),
        card("b", "Review", &["Ada Lovelace", "Carl"]),
        card("n", "Unplanned", &[]),
    ];
    let mut done = Column::new("done", "Done");
    done.cards = vec![card("c", "Ship it", &["Carl"])];
    let host = Arc::new(
        MemoryBoard::new(vec![todo, Column::new("doing", "Doing"), done]).with_lanes(plan),
    );
    BoardBench::new(host.clone(), Labels::default())
        .events(host)
        .id("B")
        .look(Look::new(StyleId::Craft, false))
}

fn group() -> Group {
    Group::new("kanban", (960, 760), Tolerance::SYSTEM_FONT_BOARD)
}

#[test]
fn a_board_in_lanes() {
    let b = board(LanePlan::On {
        grouping: "assignee".into(),
        heads: vec![
            LaneHead::new("Ada Lovelace", "Ada Lovelace"),
            LaneHead::new("Bob", "Bob (nothing yet)"),
        ],
    });
    group().check("board_lanes", &fs_components_kanban::styles(), |w, h| {
        b.build(w, h)
    });
}

#[test]
fn a_board_whose_list_of_lanes_is_missing() {
    let b = board(LanePlan::Missing);
    group().check(
        "board_lanes_missing",
        &fs_components_kanban::styles(),
        |w, h| b.build(w, h),
    );
}
