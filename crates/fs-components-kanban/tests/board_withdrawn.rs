//! The right to move is taken back, or was never there (moved from `view_withdrawn.rs`, ADR 0096 K2b): the right is asked of the
//! program at every gesture, so a card held or carried when it goes is given up, nothing is sent, and the reason is said. Without
//! the right, a press on a card and the move buttons say "soon" (the controls stay, locked).
//!
//! Mutants (each makes the named test fail): `place` does not look at the right -> `a_drop_is_refused_...` and
//! `a_click_on_a_column_does_not_put_the_card_down_...`; `toggle` ignores the withdrawn right -> `putting_down_by_key_...`; the
//! right is read once at load -> all three; a click on the foot bubbles into the column -> `a_click_on_a_locked_control_...`;
//! `supports` is asked with the lock held -> `the_host_may_call_the_board_from_supports_without_a_deadlock`; the press without the
//! right says nothing -> `a_press_on_a_card_without_the_right_says_soon_and_still_opens_it`; a move button without the right moves ->
//! `the_move_buttons_are_locked_and_say_soon_without_the_right`.
//!
//! Pattern: none; tests of one element.
#![allow(clippy::pedantic)]

mod board_common;

use std::sync::{Arc, OnceLock};

use board_common::{Rig, sample};
use fs_components_kanban::board::{
    BoardBench, BoardSource, Card, Column, DetailContent, DetailFrame, Labels, MemoryBoard,
    Operation,
};
use fs_components_widgets::keys::{Key, KeyEvent};

const SIZE: (f32, f32) = (1100.0, 640.0);

struct Detail;

impl DetailContent for Detail {
    fn frame(&self, card: &str) -> Option<DetailFrame> {
        Some(DetailFrame::new(format!("Card {card}")))
    }
}

fn bench(b: &MemoryBoard) -> BoardBench {
    BoardBench::new(Arc::new(b.clone()), Labels::default())
        .events(Arc::new(b.clone()))
        .detail(Arc::new(Detail))
        .id("B")
}

fn key(key: Key) -> KeyEvent {
    KeyEvent::new(key)
}

fn held(bench: &BoardBench) -> Option<String> {
    bench.read(|s| s.held().map(str::to_owned))
}

fn notice(bench: &BoardBench) -> Option<(String, bool)> {
    bench.read(|s| s.notice().map(|n| (n.text.clone(), n.refused)))
}

#[test]
fn a_drop_is_refused_when_the_host_stopped_supporting_the_move_while_a_card_was_carried() {
    let b = sample();
    let c = bench(&b);
    let (x, y, ..) = c.read(|s| s.geometry().card_rect("a")).unwrap();
    c.press("a", (0.0, 0.0));
    c.drag("a", (60.0, 60.0), (330.0 - x, 400.0 - y));
    assert!(c.read(|s| s.carried().is_some()), "carried");
    b.withdrawing(Operation::Move);
    c.release();
    assert!(b.log().is_empty(), "nothing was moved: {:?}", b.log());
    assert!(c.read(|s| s.carried().is_none()));
    assert!(notice(&c).is_some_and(|n| n.1), "the refusal is said");
}

#[test]
fn a_click_on_a_locked_control_does_not_put_the_held_card_down_in_its_column() {
    let b = sample()
        .without(Operation::AddCard)
        .without(Operation::Rename)
        .without_menu();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.bench.press_key(&key(Key::Space));
    assert_eq!(held(&rig.bench).as_deref(), Some("a"));
    rig.click("B-add-doing");
    rig.click("B-menu-doing");
    assert_eq!(held(&rig.bench).as_deref(), Some("a"), "still held");
    assert!(b.log().is_empty(), "{:?}", b.log());
}

/// A source whose `supports` calls back into the board, as a host that reads the board's state would.
struct Reentrant {
    board: OnceLock<BoardBench>,
}

impl BoardSource for Reentrant {
    fn columns(&self) -> Vec<Column> {
        let mut todo = Column::new("todo", "To do");
        todo.cards = vec![Card::new("a", "A")];
        vec![todo, Column::new("doing", "Doing")]
    }
    fn supports(&self, _: Operation) -> bool {
        if let Some(b) = self.board.get() {
            let _ = b.snapshot();
        }
        true
    }
}

#[test]
fn the_host_may_call_the_board_from_supports_without_a_deadlock() {
    let src = Arc::new(Reentrant {
        board: OnceLock::new(),
    });
    let c = BoardBench::new(src.clone(), Labels::default());
    src.board.set(c.clone()).ok();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        c.press_key(&key(Key::Space));
        c.click_cell("todo");
        c.press_key(&key(Key::Space));
        c.press_key(&key(Key::Escape));
        c.press("a", (0.0, 0.0));
        c.release();
        tx.send(()).ok();
    });
    assert!(
        rx.recv_timeout(std::time::Duration::from_secs(5)).is_ok(),
        "the board held its lock while asking the host"
    );
}

#[test]
fn a_click_on_a_column_does_not_put_the_card_down_after_the_right_was_withdrawn() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.bench.press_key(&key(Key::Space));
    assert_eq!(held(&rig.bench).as_deref(), Some("a"));
    b.withdrawing(Operation::Move);
    rig.click("B-cards-doing");
    assert!(b.log().is_empty(), "nothing was moved: {:?}", b.log());
    assert_eq!(held(&rig.bench), None, "given up");
    rig.rebuild();
    assert!(rig.has("B-notice"), "and the reason is shown");
}

#[test]
fn putting_down_by_key_after_the_right_was_withdrawn_is_refused() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    assert!(rig.focus_board("B"));
    assert!(rig.bench.press_key(&key(Key::Space))); // pick up
    rig.key(39, false); // over the next column
    assert!(held(&rig.bench).is_some());
    b.withdrawing(Operation::Move);
    rig.key(32, false); // put down (Activate)
    assert!(b.log().is_empty(), "nothing was moved: {:?}", b.log());
    assert_eq!(held(&rig.bench), None);
    rig.rebuild();
    assert!(rig.has("B-notice"), "and the reason is shown");
}

#[test]
fn a_press_on_a_card_without_the_right_says_soon_and_still_opens_it() {
    let b = sample().without(Operation::Move);
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.click("B-card-a");
    assert_eq!(
        notice(&rig.bench).map(|n| n.0),
        Some(Labels::default().move_soon),
        "it says why nothing can be moved"
    );
    assert_eq!(
        rig.bench
            .read(|s| s.open_card().map(str::to_owned))
            .as_deref(),
        Some("a")
    );
    rig.rebuild();
    assert!(rig.has("B-notice"));
    rig.drag_to("B-card-a", rig.centre("B-head-done"));
    assert!(
        b.log().iter().all(|l| !l.starts_with("moved")),
        "{:?}",
        b.log()
    );
}

#[test]
fn the_move_buttons_are_locked_and_say_soon_without_the_right() {
    let b = sample().without(Operation::Move);
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.bench.open("a");
    rig.rebuild();
    for id in ["B-detail-next", "B-detail-moveto"] {
        let flag = rig
            .tree
            .element_registry()
            .attribute(rig.tree.query_by_id(id).expect("drawn"), "data-locked");
        assert_eq!(flag.as_deref(), Some("true"), "{id} is locked");
    }
    assert!(rig.has("B-detail-move-soon"), "and says soon");
    rig.click("B-detail-next");
    rig.click("B-detail-moveto");
    assert!(
        b.log().iter().all(|l| !l.starts_with("moved")),
        "{:?}",
        b.log()
    );
    assert_eq!(
        notice(&rig.bench).map(|n| n.0),
        Some(Labels::default().move_soon)
    );
    b.supporting(Operation::Move);
    rig.rebuild();
    rig.click("B-detail-next");
    assert_eq!(
        b.log().last().map(String::as_str),
        Some("moved:a:todo>doing@d"),
        "it unlocks by itself"
    );
}
