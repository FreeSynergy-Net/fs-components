//! The board workbench without a window, part 2: moving by the pointer, the program's answers, undo, the keyboard against the mouse.
//!
//! Pattern: Model tests on a `MemoryBoard` (see `board_model.rs` for the mutants of the first part; the ones named here are `a_drop_between_two_cards_names_the_card_before`, `after_is_the_shown_card_before_never_a_hidden_one`, `a_column_that_refuses_gets_no_request_and_a_reason`, `a_locked_card_is_not_carried`, `over_the_limit_warns_and_never_locks`, `a_refused_move_puts_the_card_back_and_says_why`, `a_question_waits_for_the_reader_and_asks_again_confirmed`, `shift_left_moves_left_like_the_mouse`, `the_keyboard_and_the_mouse_ask_the_same`, `escape_gives_the_held_card_up`).

#![allow(clippy::pedantic)]

mod board_common;

use board_common::{Spot, carry};
use std::sync::Arc;

use fs_components_kanban::{
    Action, BoardBench, BoardState, Card, Column, Labels, MemoryBoard, Reason, wip_mark,
};
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
    BoardBench::new(Arc::new(board.clone()), Labels::default())
        .events(Arc::new(board.clone()))
        .id("B")
}

fn key(key: Key, shift: bool) -> KeyEvent {
    let mut e = KeyEvent::new(key);
    e.shift = shift;
    e
}

#[test]
fn a_drop_between_two_cards_names_the_card_before() {
    let b = board();
    let bench = bench(&b);
    // doing holds only d; drop "a" below d: after = d, index 1
    carry(&bench, "a", "B-col-doing", Spot::End);
    assert_eq!(b.log(), ["moved:a:todo>doing@d"]);
    assert_eq!(
        b.snapshot()[1]
            .cards
            .iter()
            .map(|c| c.key.as_str())
            .collect::<Vec<_>>(),
        ["d", "a"]
    );
}

#[test]
fn after_is_the_shown_card_before_never_a_hidden_one() {
    // b is hidden by a filter: the board shows a and c only; between them the program is told "after a"
    let b = MemoryBoard::new(vec![column("todo", &["a", "c"]), column("doing", &["d"])])
        .with_hidden("todo", 1);
    let bench = bench(&b);
    carry(&bench, "d", "B-col-todo", Spot::After("a"));
    assert_eq!(
        b.log(),
        ["moved:d:doing>todo@a"],
        "after a, not after the hidden one"
    );
}

#[test]
fn a_column_that_refuses_gets_no_request_and_a_reason() {
    let b = board().refusing("done", Reason::new("no-right", "You may not finish cards."));
    let bench = bench(&b);
    carry(&bench, "a", "B-col-done", Spot::End);
    assert!(b.log().is_empty(), "the program was not asked to move");
    assert_eq!(
        bench.read(|s| s.notice().map(|n| (n.text.clone(), n.refused))),
        Some(("You may not finish cards.".into(), true))
    );
    assert_eq!(b.snapshot()[0].cards.len(), 3, "the card stays");
}

#[test]
fn a_locked_card_is_not_carried() {
    let b = board().locking("a", Reason::new("claimed", "Held by an agent."));
    let bench = bench(&b);
    carry(&bench, "a", "B-col-doing", Spot::End);
    assert!(b.log().is_empty());
    assert_eq!(
        bench
            .read(|s| s.notice().map(|n| n.text.clone()))
            .as_deref(),
        Some("Held by an agent.")
    );
}

#[test]
fn over_the_limit_warns_and_never_locks() {
    let mut full = column("doing", &["d"]);
    full.limit = Some(1);
    let b = MemoryBoard::new(vec![column("todo", &["a"]), full]);
    let bench = bench(&b);
    carry(&bench, "a", "B-col-doing", Spot::End);
    assert_eq!(
        b.log(),
        ["moved:a:todo>doing@d"],
        "a full column still takes the card"
    );
    let mark = bench.read(|s| s.wip(1)).unwrap();
    assert_eq!((mark.text.as_str(), mark.over), ("2/1", true));
    assert!(!wip_mark(1, Some(1)).over, "at the limit is not over it");
    assert_eq!(wip_mark(4, None).text, "4");
    assert!(!wip_mark(4, None).over);
}

#[test]
fn the_limit_counts_the_cards_a_filter_hides() {
    let mut col = column("todo", &["a", "b"]);
    col.limit = Some(3);
    let b = MemoryBoard::new(vec![col]).with_hidden("todo", 2);
    let bench = bench(&b);
    let mark = bench.read(|s| s.wip(0)).unwrap();
    assert_eq!((mark.text.as_str(), mark.over), ("4/3", true));
}

#[test]
fn a_refused_move_puts_the_card_back_and_says_why() {
    let b = board().refusing_moves(Reason::new("changed", "Someone else moved it first."));
    let bench = bench(&b);
    carry(&bench, "a", "B-col-doing", Spot::End);
    assert_eq!(b.log().len(), 1, "the program was asked");
    assert_eq!(b.snapshot()[0].cards.len(), 3, "and said no");
    let order: Vec<_> = bench.read(|s| s.lanes()[0].items.iter().map(|i| i.key.clone()).collect());
    assert_eq!(
        order,
        ["a", "b", "c"],
        "the card is back where it was on the screen too"
    );
    assert_eq!(
        bench.read(|s| s
            .notice()
            .map(|n| (n.text.clone(), n.refused, n.undo.is_none()))),
        Some(("Someone else moved it first.".into(), true, true))
    );
}

#[test]
fn a_question_waits_for_the_reader_and_asks_again_confirmed() {
    let b = board().asking_about("a");
    let bench = bench(&b);
    carry(&bench, "a", "B-col-doing", Spot::End);
    assert_eq!(b.log(), ["moved:a:todo>doing@d"]);
    let asked = bench.read(|s| s.question().map(|(_, q)| q.text.clone()));
    assert!(asked.is_some(), "the board shows the question");
    assert_eq!(b.snapshot()[0].cards.len(), 3, "nothing moved yet");
    bench.answer_question(true);
    assert_eq!(
        b.log(),
        ["moved:a:todo>doing@d", "moved:a:todo>doing@d:yes"]
    );
    assert_eq!(b.snapshot()[1].cards.len(), 2);
    assert!(bench.read(|s| s.question().is_none()));
}

#[test]
fn saying_no_to_the_question_moves_nothing() {
    let b = board().asking_about("a");
    let bench = bench(&b);
    carry(&bench, "a", "B-col-doing", Spot::End);
    bench.answer_question(false);
    assert_eq!(b.log().len(), 1);
    assert_eq!(b.snapshot()[0].cards.len(), 3);
}

#[test]
fn undo_asks_the_program_for_the_counter_move() {
    let b = board();
    let bench = bench(&b);
    carry(&bench, "b", "B-col-doing", Spot::End);
    assert_eq!(b.snapshot()[1].cards.len(), 2);
    assert!(bench.read(|s| s.notice().is_some_and(|n| n.undo.is_some() && !n.refused)));
    bench.undo();
    let names: Vec<_> = b.snapshot()[0]
        .cards
        .iter()
        .map(|c| c.key.clone())
        .collect();
    assert_eq!(names, ["a", "b", "c"], "b is back between a and c");
    assert_eq!(b.log().len(), 2);
}

#[test]
fn shift_left_moves_left_like_the_mouse() {
    let b = board();
    let bench = bench(&b);
    // the cursor starts on a; Right walks to d (first card of the next column with cards)
    assert!(bench.press_key(&key(Key::Right, false)));
    assert_eq!(
        bench.read(|s| s.cursor().map(str::to_owned)).as_deref(),
        Some("d")
    );
    assert!(bench.press_key(&key(Key::Left, true)));
    assert_eq!(
        b.log(),
        ["moved:d:doing>todo@^"],
        "left, at the same index (the top)"
    );
    assert!(bench.press_key(&key(Key::Right, true)));
    assert_eq!(b.log()[1], "moved:d:todo>doing@^");
}

#[test]
fn the_keyboard_and_the_mouse_ask_the_same() {
    let by_mouse = board();
    let m = bench(&by_mouse);
    carry(&m, "d", "B-col-todo", Spot::Top);
    let by_key = board();
    let k = bench(&by_key);
    k.press_key(&key(Key::Right, false));
    k.press_key(&key(Key::Left, true));
    assert_eq!(by_mouse.log(), by_key.log());
}

#[test]
fn shift_down_and_up_reorder_inside_the_column() {
    let b = board();
    let bench = bench(&b);
    assert!(bench.press_key(&key(Key::Down, true)));
    assert_eq!(b.log(), ["moved:a:todo>todo@b"]);
    assert!(bench.press_key(&key(Key::Up, true)));
    assert_eq!(b.log()[1], "moved:a:todo>todo@^");
    assert!(
        !bench.press_key(&key(Key::Up, true)),
        "at the top there is nowhere to go"
    );
}

#[test]
fn space_holds_the_card_and_arrows_carry_it_and_space_puts_it_down() {
    let b = board();
    let bench = bench(&b);
    assert!(bench.press_key(&key(Key::Space, false)));
    assert_eq!(
        bench.read(|s| s.held().map(str::to_owned)).as_deref(),
        Some("a")
    );
    assert!(bench.press_key(&key(Key::Right, false)));
    assert!(b.log().is_empty(), "nothing is asked while it is carried");
    assert!(bench.press_key(&key(Key::Space, false)));
    assert_eq!(b.log(), ["moved:a:todo>doing@^"]);
}

#[test]
fn escape_gives_the_held_card_up() {
    let b = board();
    let bench = bench(&b);
    bench.press_key(&key(Key::Space, false));
    bench.press_key(&key(Key::Right, false));
    assert!(bench.press_key(&key(Key::Escape, false)));
    assert!(bench.read(|s| s.held().is_none()));
    assert!(b.log().is_empty());
    let order: Vec<_> = bench.read(|s| s.lanes()[0].items.iter().map(|i| i.key.clone()).collect());
    assert_eq!(order, ["a", "b", "c"], "back where it was");
}

#[test]
fn enter_opens_the_card_under_the_cursor() {
    let b = board();
    let bench = bench(&b);
    let mut state = BoardState::new(&b);
    assert_eq!(
        state.key(&key(Key::Enter, false), &b),
        Action::Open("a".into())
    );
    assert_eq!(
        state.key(&key(Key::Activate, false), &b),
        Action::Open("a".into())
    );
    assert!(bench.press_key(&key(Key::Enter, false)));
    assert_eq!(b.log(), ["opened:a"]);
}
