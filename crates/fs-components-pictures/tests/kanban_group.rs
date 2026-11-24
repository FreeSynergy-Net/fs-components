//! Group `kanban`: the one board element with three columns, one card with all its parts, and one column over its limit.
//!
//! Tests: picture comparison against `contract/umbau/bilder/kanban` (drawn on the software GPU; skipped with a message without one, a
//! failure when `CI` is set).
//! Pattern: none (tests). The element has fixed column widths (`COLUMN_W`), so the picture is wide enough for three columns.

use std::sync::Arc;

use fs_components_fields::Look;
use fs_components_kanban::{
    BoardBench, Card, Column, Due, DueState, Labels, MemoryBoard, Operation,
};
use fs_components_pictures::{Group, Tolerance};
use fs_components_styles::StyleId;

fn group() -> Group {
    Group::new("kanban", (960, 420), Tolerance::SYSTEM_FONT_BOARD)
}

fn board(without: &[Operation], menu: bool) -> BoardBench {
    let mut a = Card::new("a", "Write the spec");
    a.subtitle = Some("Part one".into());
    a.due = Some(Due {
        text: "12 Nov".into(),
        state: DueState::Overdue,
    });
    a.assignee = Some("Ada Lovelace".into());
    let mut todo = Column::new("todo", "To do");
    todo.cards = vec![a, Card::new("b", "Review")];
    todo.limit = Some(1);
    let mut host = MemoryBoard::new(vec![
        todo,
        Column::new("doing", "Doing"),
        Column::new("done", "Done"),
    ]);
    for op in without {
        host = host.without(*op);
    }
    if !menu {
        host = host.without_menu();
    }
    let host = Arc::new(host);
    BoardBench::new(host.clone(), Labels::default())
        .events(host)
        .id("B")
        .look(Look::new(StyleId::Craft, false))
}

#[test]
fn a_board_with_moves() {
    // Everything is supported but the quick entry (the foot of a column is drawn locked with "soon").
    let b = board(&[Operation::AddCard], true);
    group().check("board_moves", &fullstyles(), |w, h| b.build(w, h));
}

#[test]
fn a_board_without_operations() {
    let b = board(&Operation::ALL, false);
    group().check("board_plain", &fullstyles(), |w, h| b.build(w, h));
}

fn fullstyles() -> String {
    fs_components_kanban::styles()
}
