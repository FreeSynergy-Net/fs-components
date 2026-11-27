//! Group `kanban`: the one board element with three columns, one card with all its parts, and one column over its limit.
//!
//! Tests: picture comparison against `contract/umbau/bilder/kanban` (drawn on the software GPU; skipped with a message without one, a
//! failure when `CI` is set).
//! Pattern: none (tests). The element has fixed column widths (`COLUMN_W`), so the picture is wide enough for three columns.

use std::sync::Arc;

use fs_components_fields::Look;
use fs_components_kanban::{
    BoardBench, Card, Column, DetailContent, DetailFrame, Due, DueState, Labels, MemoryBoard,
    Operation, Reason,
};
use fs_components_pictures::{Group, Tolerance};
use fs_components_styles::StyleId;

fn group() -> Group {
    Group::new("kanban", (960, 420), Tolerance::SYSTEM_FONT_BOARD)
}

struct Detail;

impl DetailContent for Detail {
    fn frame(&self, card: &str) -> Option<DetailFrame> {
        Some(DetailFrame::new(format!("Card {card}")).path("Home > To do"))
    }
}

fn board(without: &[Operation], menu: bool, locked: bool) -> BoardBench {
    let mut a = Card::new("a", "Write the spec");
    a.subtitle = Some("Part one".into());
    a.due = Some(Due {
        text: "12 Nov".into(),
        state: DueState::Overdue,
    });
    a.assignees = vec!["Ada Lovelace".into()];
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
    if locked {
        host = host
            .locking("a", Reason::new("claimed", "Held by an agent."))
            .refusing("done", Reason::new("closed", "The column is closed."));
    }
    let host = Arc::new(host);
    let bench = BoardBench::new(host.clone(), Labels::default())
        .events(host)
        .detail(Arc::new(Detail))
        .id("B")
        .look(Look::new(StyleId::Craft, false));
    if locked {
        // What a reader without the right to move sees: the frame of the card with the move buttons locked ("soon"), and after a press
        // on the card the line under the board says "Moving cards: soon" (`say_move_soon`). The moves picture has neither, and the two differ by more
        // than the tolerance of the group (`tests/tolerance_counter.rs`), so the lock is seen in the picture.
        bench.open("a");
        bench.say_move_soon();
    }
    bench
}

#[test]
fn a_board_with_moves() {
    // Everything is supported but the quick entry (the foot of a column is drawn locked with "soon").
    let b = board(&[Operation::AddCard], true, false);
    group().check("board_moves", &fullstyles(), |w, h| b.build(w, h));
}

#[test]
fn a_board_without_operations() {
    // Nothing may be done and the menu is off; the card "a" and the column "done" are locked, which is what the picture must show.
    let b = board(&Operation::ALL, false, true);
    group().check("board_plain", &fullstyles(), |w, h| b.build(w, h));
}

fn fullstyles() -> String {
    fs_components_kanban::styles()
}
