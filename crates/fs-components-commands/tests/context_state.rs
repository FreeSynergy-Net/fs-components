//! Tests: the state of one open context menu (M3): opening with stage and target, choosing once, a locked line, the submenu,
//! Escape, a press outside.
//!
//! Pattern: State machine tests with hand-written expected values; each rule has a case that fails without it.
//!
//! Mutants (each makes a test here red): `choose` does not close, so a double click runs twice (`a_double_click_runs_once`); a locked
//! line closes or runs (`a_locked_line_stays_and_says_why`); Escape closes the menu before the submenu
//! (`escape_closes_the_submenu_first`); `toggle_sub` opens a command line (`the_submenu_opens_and_closes_and_only_on_a_submenu_line`);
//! a second `open` keeps the first (`opening_stores_stage_target_place_and_lines`).

#[path = "context_common/mod.rs"]
mod context_common;

use context_common::{at, context, file, id};
use fs_components_commands::{Choice, ContextState, Stage, Trigger};

#[test]
fn opening_stores_stage_target_place_and_lines() {
    let mut menu = ContextState::new();
    assert!(!menu.is_open());
    assert_eq!(menu.items(), &[]);
    let first = Trigger::Pointer {
        at: at(10.0, 20.0),
        shift: false,
    };
    menu.open(&first, file(&["a"]), &context());
    assert!(menu.is_open());
    assert_eq!(menu.at(), Some(at(10.0, 20.0)));
    assert_eq!(menu.stage(), Some(Stage::First));
    assert_eq!(menu.target(), Some(&file(&["a"])));
    assert_eq!(menu.items().len(), 6);
    let full = Trigger::Pointer {
        at: at(10.0, 20.0),
        shift: true,
    };
    menu.open(&full, file(&["b"]), &context());
    assert_eq!(menu.stage(), Some(Stage::Full));
    assert_eq!(menu.items().len(), 8, "a second open replaces the first");
}

#[test]
fn a_chosen_line_runs_on_the_target_and_closes() {
    let mut menu = ContextState::new();
    let t = Trigger::Pointer {
        at: at(0.0, 0.0),
        shift: false,
    };
    menu.open(&t, file(&["a", "b"]), &context());
    assert_eq!(
        menu.choose(&id("open")),
        Choice::Run {
            id: id("open"),
            target: file(&["a", "b"])
        }
    );
    assert!(!menu.is_open());
}

#[test]
fn a_symbol_in_the_row_is_a_line_too() {
    let mut menu = ContextState::new();
    let t = Trigger::Pointer {
        at: at(0.0, 0.0),
        shift: false,
    };
    menu.open(&t, file(&["a"]), &context());
    assert!(matches!(menu.choose(&id("copy")), Choice::Run { .. }));
}

#[test]
fn a_double_click_runs_once() {
    let mut menu = ContextState::new();
    let t = Trigger::Pointer {
        at: at(0.0, 0.0),
        shift: false,
    };
    menu.open(&t, file(&["a"]), &context());
    assert!(matches!(menu.choose(&id("open")), Choice::Run { .. }));
    assert_eq!(menu.choose(&id("open")), Choice::Nothing);
}

#[test]
fn a_locked_line_stays_and_says_why() {
    let mut menu = ContextState::new();
    let t = Trigger::Pointer {
        at: at(0.0, 0.0),
        shift: false,
    };
    menu.open(&t, file(&["a"]), &context());
    match menu.choose(&id("share")) {
        Choice::Locked { id: who, reason } => {
            assert_eq!(who, id("share"));
            assert_eq!(reason.as_str(), "share-why");
        }
        other => panic!("{other:?}"),
    }
    assert!(menu.is_open(), "the menu stays open");
}

#[test]
fn an_unknown_line_does_nothing() {
    let mut menu = ContextState::new();
    assert_eq!(menu.choose(&id("open")), Choice::Nothing);
    let t = Trigger::Pointer {
        at: at(0.0, 0.0),
        shift: false,
    };
    menu.open(&t, file(&["a"]), &context());
    assert_eq!(menu.choose(&id("nonexistent")), Choice::Nothing);
    assert!(menu.is_open());
}

#[test]
fn a_line_of_the_submenu_can_be_chosen() {
    let mut menu = ContextState::new();
    let t = Trigger::Pointer {
        at: at(0.0, 0.0),
        shift: false,
    };
    menu.open(&t, file(&["a"]), &context());
    assert!(matches!(menu.choose(&id("zip")), Choice::Run { .. }));
}

#[test]
fn the_submenu_opens_and_closes_and_only_on_a_submenu_line() {
    let mut menu = ContextState::new();
    assert!(!menu.toggle_sub(3), "nothing is open");
    let t = Trigger::Pointer {
        at: at(0.0, 0.0),
        shift: false,
    };
    menu.open(&t, file(&["a"]), &context());
    assert!(!menu.toggle_sub(1), "a command line has no submenu");
    assert!(menu.toggle_sub(3));
    assert_eq!(menu.open_sub(), Some(3));
    assert!(menu.toggle_sub(3));
    assert_eq!(menu.open_sub(), None);
}

#[test]
fn escape_closes_the_submenu_first() {
    let mut menu = ContextState::new();
    assert!(!menu.escape());
    let t = Trigger::Pointer {
        at: at(0.0, 0.0),
        shift: false,
    };
    menu.open(&t, file(&["a"]), &context());
    menu.toggle_sub(3);
    assert!(menu.escape());
    assert!(menu.is_open());
    assert_eq!(menu.open_sub(), None);
    assert!(menu.escape());
    assert!(!menu.is_open());
}

#[test]
fn a_press_outside_closes() {
    let mut menu = ContextState::new();
    assert!(!menu.press_outside());
    let t = Trigger::Pointer {
        at: at(0.0, 0.0),
        shift: false,
    };
    menu.open(&t, file(&["a"]), &context());
    assert!(menu.press_outside());
    assert!(!menu.is_open());
}
