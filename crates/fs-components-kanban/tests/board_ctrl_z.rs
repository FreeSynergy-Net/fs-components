//! The board gives Ctrl+Z (and Ctrl+Shift+Z, Ctrl+Y) to the host: undo by key is `app.undo` of the host, the board writes no counter-move
//! of its own (one undo path, else one key press is two changes). The Undo button of the notice stays and writes exactly one.
//!
//! Mutant (makes the test fail): `state_keys.rs` maps Ctrl+Z to `Action::Undo` again.
//!
//! Pattern: none; tests of one element.
#![allow(clippy::pedantic)]

use std::sync::Arc;

use fs_components_kanban::{Action, BoardBench, BoardState, Card, Column, Labels, MemoryBoard};
use fs_components_widgets::keys::{Key, KeyEvent};

fn column(key: &str, cards: &[&str]) -> Column {
    let mut c = Column::new(key, key);
    c.cards = cards
        .iter()
        .map(|k| Card::new(k, &k.to_uppercase()))
        .collect();
    c
}

fn board() -> MemoryBoard {
    MemoryBoard::new(vec![
        column("todo", &["a", "b", "c"]),
        column("doing", &["d"]),
        column("done", &[]),
    ])
}

fn bench(board: &MemoryBoard) -> BoardBench {
    BoardBench::new(Arc::new(board.clone()), Labels::default()).events(Arc::new(board.clone()))
}

fn key(key: Key, shift: bool) -> KeyEvent {
    let mut e = KeyEvent::new(key);
    e.shift = shift;
    e
}

/// Carry `card` to the point `to` and let go: what the pointer does, in one call.
fn carry(bench: &BoardBench, card: &str, to: (f32, f32)) {
    let (x, y, ..) = bench.read(|s| s.geometry().card_rect(card)).expect("shown");
    bench.press(card, (0.0, 0.0));
    bench.drag(card, (60.0, 60.0), (to.0 - x, to.1 - y));
    bench.release();
}

const COL: [f32; 3] = [16.0 + 10.0, 16.0 + 304.0 + 10.0, 16.0 + 608.0 + 10.0];

#[test]
fn ctrl_z_on_the_board_writes_nothing_undo_by_the_button_writes_once() {
    let b = board();
    let bench = bench(&b);
    carry(&bench, "b", (COL[1], 400.0));
    let written = b.log().len();
    assert_eq!(written, 1, "the move is one change");
    for (shift, code) in [(false, 90), (true, 90), (false, 89)] {
        let mut ctrl = key(Key::Other(code), shift);
        ctrl.ctrl = true;
        let mut state = BoardState::new(&b);
        assert_eq!(state.key(&ctrl, &b), Action::None);
        assert!(!bench.press_key(&ctrl), "the key stays free for the host");
        assert_eq!(b.log().len(), written, "a key writes no counter-move");
    }
    assert_eq!(b.snapshot()[1].cards.len(), 2, "b stays where it was put");
    bench.undo();
    assert_eq!(b.log().len(), written + 1, "the button writes exactly one");
    assert_eq!(b.snapshot()[0].cards.len(), 3);
}
