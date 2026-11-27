//! The board workbench without a window: the locks, the work-in-progress
//! mark, the program's answers, the keyboard against the mouse, undo, the foot of a column, auto-scroll, the commands and the words.
//!
//! Pattern: Model tests over `BoardState` and a `BoardBench` on a `MemoryBoard` (the program as a Stub that
//! keeps its columns behind a Mutex and records every call).
//!
//! Mutants (each makes the test named red): the 4 px threshold is 0 (now `card_gesture` in `fs-components-widgets`, with the rule of the places: edges, middle,
//! the carried card left out of the count); `after` is the card at the index of the unfiltered list
//! (`after_is_the_shown_card_before_never_a_hidden_one`); `can_drop` is ignored (`a_column_that_refuses_gets_no_request_and_a_reason`);
//! the lock of a card is ignored (`a_locked_card_is_not_carried`); a full column refuses (`over_the_limit_warns_and_never_locks`);
//! `Refused` is ignored (`a_refused_move_puts_the_card_back_and_says_why`); `Ask` is treated as `Accepted`
//! (`a_question_waits_for_the_reader_and_asks_again_confirmed`); Left and Right swapped in `step` (`shift_left_moves_left_like_the_mouse`);
//! the keyboard and the mouse build different requests (`the_keyboard_and_the_mouse_ask_the_same`); Escape does not give up
//! (`escape_gives_the_held_card_up`); `top` is not reversed for many lines (`several_lines_at_the_top_keep_their_order`); the edge
//! distance is 0 (`near_the_edge_the_area_moves`); a command that cannot run is offered (`a_card_at_the_top_has_no_up`).

#![allow(clippy::pedantic)]

use std::sync::Arc;

use fs_components_kanban::commands;
use fs_components_kanban::quick::{Quick, submit};
use fs_components_kanban::scroll;
use fs_components_kanban::{
    Action, BoardBench, BoardState, Card, Column, Labels, MemoryBoard, TEXTS,
};
use fs_components_mf2::check_pairs;
use fs_components_words::{Locale, Localized};

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

#[test]
fn the_quick_entry_makes_one_card_per_line_and_asks_before_many() {
    assert_eq!(submit("todo", "   ", false), Quick::Nothing);
    assert_eq!(submit("todo", "\n\n", true), Quick::Nothing);
    let Quick::Create(one) = submit("todo", "  Write letter ", false) else {
        panic!("one line creates")
    };
    assert_eq!((one[0].title.as_str(), one[0].top), ("Write letter", false));
    let Quick::Confirm(many) = submit("todo", "x\n\ny\nz", false) else {
        panic!("several ask")
    };
    assert_eq!(
        many.iter().map(|c| c.title.as_str()).collect::<Vec<_>>(),
        ["x", "y", "z"]
    );
}

#[test]
fn several_lines_at_the_top_keep_their_order() {
    // each one lands above the one before, so the program is asked for the last line first
    let Quick::Confirm(many) = submit("todo", "first\nsecond", true) else {
        panic!()
    };
    assert_eq!(
        many.iter().map(|c| c.title.as_str()).collect::<Vec<_>>(),
        ["second", "first"]
    );
    assert!(many.iter().all(|c| c.top));
}

#[test]
fn near_the_edge_the_area_moves() {
    assert!(
        scroll::step(500.0, 0.0, 1000.0).abs() < f32::EPSILON,
        "the middle does not"
    );
    assert!(
        scroll::step(990.0, 0.0, 1000.0) > 0.0,
        "near the end it goes forward"
    );
    assert!(
        scroll::step(10.0, 0.0, 1000.0) < 0.0,
        "near the start it goes back"
    );
    let (a, b) = (
        scroll::step(980.0, 0.0, 1000.0),
        scroll::step(999.0, 0.0, 1000.0),
    );
    assert!(b > a, "the nearer, the faster");
    assert!(
        (scroll::step(2000.0, 0.0, 1000.0) - scroll::MAX_STEP).abs() < f32::EPSILON,
        "beyond the edge: the most"
    );
}

#[test]
fn a_card_at_the_top_has_no_up() {
    let lines = commands::card_lines(0, 3, &["doing".into()]);
    let enabled = |id: &str| lines.iter().find(|l| l.id == id).map(|l| l.enabled);
    assert_eq!(enabled(commands::UP), Some(false));
    assert_eq!(enabled(commands::TOP), Some(false));
    assert_eq!(enabled(commands::DOWN), Some(true));
    assert_eq!(enabled("board.to:doing"), Some(true));
    let last = commands::card_lines(2, 3, &[]);
    assert!(last.iter().any(|l| l.id == commands::DOWN && !l.enabled));
    assert_eq!(commands::column_of("board.to:doing"), Some("doing"));
    assert_eq!(commands::column_of("board.up"), None);
}

#[test]
fn a_command_of_the_menu_runs_the_same_move() {
    let b = board();
    let bench = bench(&b);
    assert!(bench.run("board.to:done", "a"));
    assert_eq!(b.log(), ["moved:a:todo>done@^"]);
    assert!(bench.run(commands::DOWN, "b"));
    assert!(!bench.run("board.nonsense", "b"));
    assert!(!bench.run(commands::UP, "nobody"));
    let menu = bench.card_menu("c", (0.0, 0.0));
    assert!(menu.iter().any(|i| i == "board.to:done"));
    assert!(menu.contains(&"board.open".to_owned()));
}

#[test]
fn another_writer_makes_the_board_stale_and_sync_reloads() {
    let b = board();
    let bench = bench(&b);
    assert!(!bench.sync());
    b.put(vec![column("todo", &["z"])]);
    assert!(bench.sync());
    assert_eq!(bench.read(|s| s.lanes().len()), 1);
    assert!(!bench.sync(), "once");
}

#[test]
fn the_words_are_paired_and_german_is_german() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    assert_eq!(de.add, "Karte hinzufügen");
    assert_eq!(de.hidden(3), "3 ausgeblendet");
    assert_eq!(Labels::default().hidden(3), "3 hidden");
    assert_eq!(Labels::default().create_many(2), "Create 2 cards?");
}

#[test]
fn a_card_that_is_gone_gives_no_request() {
    let b = board();
    let mut state = BoardState::new(&b);
    assert_eq!(state.place("nobody", "todo", 0, &b, false), Action::None);
}
