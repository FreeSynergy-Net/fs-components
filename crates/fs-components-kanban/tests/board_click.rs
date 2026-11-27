//! The click-to-move of the one board element (moved from `view_moves.rs`, ADR 0096 K2b): a held card and a click on a column put
//! it down there; the frame at the side has the buttons "previous", "next" and "Move to...", which run `board.to:<column>`.
//!
//! Mutants (each makes the named test fail): `click_cell` does not place -> `a_click_on_a_column_puts_the_held_card_down_there`,
//! `the_click_and_the_mouse_ask_the_same`; a click on a card of another column opens it instead of putting down ->
//! `a_click_on_a_card_puts_the_held_card_down_in_its_column`; `click_cell` places from the carried spot, not from the true origin ->
//! `a_card_carried_by_arrows_and_put_down_by_a_click_names_its_true_origin`; the "next" button runs the wrong column or none ->
//! `the_detail_buttons_move_a_card_and_a_refusal_is_shown`; the list "Move to..." is not drawn -> `move_to_lists_the_other_columns`;
//! an edge button moves -> `the_first_column_has_no_previous`; a refusal is swallowed -> `a_click_on_a_column_that_refuses_says_why`.
//!
//! Pattern: none; tests of one element.
#![allow(clippy::pedantic)]

mod board_common;

use std::sync::Arc;

use board_common::{Rig, Spot, carry, sample};
use fs_components_kanban::{BoardBench, DetailContent, DetailFrame, Labels, MemoryBoard, Reason};
use fs_components_widgets::keys::{Key, KeyEvent};

const SIZE: (f32, f32) = (1100.0, 640.0);

struct Detail;

impl DetailContent for Detail {
    fn frame(&self, card: &str) -> Option<DetailFrame> {
        Some(DetailFrame::new(format!("Card {card}")).path("Home"))
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

fn shown(b: &MemoryBoard, column: &str) -> Vec<String> {
    b.snapshot()
        .iter()
        .find(|c| c.key == column)
        .map(|c| c.cards.iter().map(|k| k.key.clone()).collect())
        .unwrap_or_default()
}

#[test]
fn a_click_on_a_column_puts_the_held_card_down_there() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    assert!(rig.bench.press_key(&key(Key::Space)), "pick up a");
    assert_eq!(held(&rig.bench).as_deref(), Some("a"));
    rig.click("B-cards-doing");
    assert_eq!(b.log(), ["moved:a:todo>doing@d"]);
    assert_eq!(shown(&b, "doing"), ["d", "a"]);
    assert_eq!(held(&rig.bench), None);
}

#[test]
fn a_click_on_an_empty_column_puts_the_held_card_down_there() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.bench.press_key(&key(Key::Space));
    rig.click("B-cards-done");
    assert_eq!(b.log(), ["moved:a:todo>done@^"]);
}

#[test]
fn a_click_on_a_card_puts_the_held_card_down_in_its_column() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.bench.press_key(&key(Key::Space));
    rig.click("B-card-d");
    assert_eq!(b.log(), ["moved:a:todo>doing@d"]);
    assert!(
        rig.bench.read(|s| s.open_card().is_none()),
        "it did not open the card"
    );
}

#[test]
fn a_click_on_the_column_the_card_stands_in_puts_it_down_again() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.bench.press_key(&key(Key::Space));
    rig.click("B-cards-todo");
    assert!(b.log().is_empty(), "nothing moved: {:?}", b.log());
    assert_eq!(held(&rig.bench), None);
}

#[test]
fn a_click_without_a_held_card_moves_nothing() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.click("B-cards-done");
    assert!(b.log().is_empty(), "{:?}", b.log());
    assert_eq!(shown(&b, "done"), Vec::<String>::new());
}

#[test]
fn the_click_and_the_mouse_ask_the_same() {
    let by_mouse = sample();
    let m = bench(&by_mouse);
    carry(&m, "a", "B-col-doing", Spot::End);
    let by_click = sample();
    let c = bench(&by_click);
    c.press_key(&key(Key::Space));
    c.click_cell("doing");
    assert_eq!(by_mouse.log(), ["moved:a:todo>doing@d"]);
    assert_eq!(by_mouse.log(), by_click.log());
}

#[test]
fn a_card_carried_by_arrows_and_put_down_by_a_click_names_its_true_origin() {
    let b = sample();
    let c = bench(&b);
    c.press_key(&key(Key::Space));
    c.press_key(&key(Key::Right));
    c.click_cell("done");
    assert_eq!(b.log(), ["moved:a:todo>done@^"]);
    let undo = c.read(|s| s.notice().and_then(|n| n.undo.clone()));
    assert_eq!(
        undo.map(|u| (u.from, u.to, u.index)),
        Some(("done".to_owned(), "todo".to_owned(), 0))
    );
}

#[test]
fn a_click_on_a_column_that_refuses_says_why() {
    let b = sample().refusing("done", Reason::new("no", "Not done yet."));
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.bench.press_key(&key(Key::Space));
    rig.click("B-cards-done");
    rig.rebuild();
    assert!(b.log().is_empty(), "nothing was sent: {:?}", b.log());
    assert!(rig.has("B-notice"), "the reason is shown");
    assert_eq!(shown(&b, "todo"), ["a", "b", "c"]);
    assert_eq!(held(&rig.bench), None);
}

#[test]
fn the_keyboard_picks_up_moves_and_puts_down() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    assert!(rig.focus_board("B-grid"));
    rig.key(13, false); // Enter: the keys go into the cell, the cursor stands on its first card
    // (a Space through the window is the key "Activate" and opens the card, as it always did; the hold is pressed on the board)
    assert!(rig.bench.press_key(&key(Key::Space)));
    assert_eq!(held(&rig.bench).as_deref(), Some("a"));
    rig.key(39, false); // right
    rig.key(32, false); // space: put down
    assert_eq!(b.log(), ["moved:a:todo>doing@^"]);
    rig.bench.press_key(&key(Key::Space));
    rig.key(27, false); // escape
    assert_eq!(held(&rig.bench), None);
}

#[test]
fn the_detail_buttons_move_a_card_and_a_refusal_is_shown() {
    let b = sample().refusing("done", Reason::new("no", "Not done yet."));
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.bench.open("a");
    rig.rebuild();
    assert!(rig.has("B-detail-prev") && rig.has("B-detail-next") && rig.has("B-detail-moveto"));
    rig.click("B-detail-next");
    assert_eq!(b.log(), ["opened:a", "moved:a:todo>doing@d"]);
    assert_eq!(shown(&b, "doing"), ["d", "a"]);
    rig.rebuild();
    rig.click("B-detail-next");
    assert_eq!(
        shown(&b, "done"),
        Vec::<String>::new(),
        "refused by the host"
    );
    rig.rebuild();
    assert!(rig.has("B-notice"), "the reason is shown");
}

#[test]
fn the_first_column_has_no_previous() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.bench.open("a");
    rig.rebuild();
    rig.click("B-detail-prev");
    assert_eq!(b.log(), ["opened:a"], "nothing moved");
    rig.bench.open("d");
    rig.rebuild();
    rig.click("B-detail-prev");
    assert_eq!(
        b.log().last().map(String::as_str),
        Some("moved:d:doing>todo@c")
    );
}

#[test]
fn move_to_lists_the_other_columns() {
    let b = sample();
    let mut rig = Rig::new(bench(&b), SIZE.0, SIZE.1);
    rig.bench.open("a");
    rig.rebuild();
    assert!(!rig.has("B-detail-to"), "closed until asked");
    rig.click("B-detail-moveto");
    rig.rebuild();
    assert!(rig.has("B-detail-to-doing") && rig.has("B-detail-to-done"));
    assert!(!rig.has("B-detail-to-todo"), "not the column it is in");
    rig.click("B-detail-to-done");
    assert_eq!(
        b.log().last().map(String::as_str),
        Some("moved:a:todo>done@^")
    );
    rig.rebuild();
    assert!(!rig.has("B-detail-to"), "closed after the choice");
}
