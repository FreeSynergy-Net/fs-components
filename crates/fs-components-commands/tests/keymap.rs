//! Tests: the dispatcher (shortcut to command): platforms, conflicts, locked commands, contexts of one key.
//! Each rule has a case that fails without it (the mutants of M2: conflict ignored, locked run).

mod keymap_common;

use fs_components_commands::{
    Command, Commands, FocusContext, KeyPress, Keymap, Modifiers, Outcome, Platform,
};
use keymap_common::{commands, id, run, sc};

const LINUX: Platform = Platform::Linux;
const MAC: Platform = Platform::Mac;

#[test]
fn primary_is_ctrl_on_linux_and_cmd_on_macos() {
    let linux = Keymap::from_commands(&commands(), LINUX);
    let mac = Keymap::from_commands(&commands(), MAC);
    assert_eq!(
        run(&linux, "Control+S", FocusContext::Global),
        Outcome::Run(id("file-save"))
    );
    assert_eq!(
        run(&linux, "Meta+S", FocusContext::Global),
        Outcome::NoMatch,
        "Super is not Primary on Linux"
    );
    assert_eq!(
        run(&mac, "Meta+S", FocusContext::Global),
        Outcome::Run(id("file-save"))
    );
    assert_eq!(
        run(&mac, "Control+S", FocusContext::Global),
        Outcome::NoMatch,
        "Ctrl is not Primary on macOS"
    );
    assert_eq!(
        Keymap::from_commands(&commands(), Platform::Windows).platform(),
        Platform::Windows
    );
}

#[test]
fn modifiers_must_match_exactly_and_all_shortcuts_of_a_command_run_it() {
    let km = Keymap::from_commands(&commands(), LINUX);
    assert_eq!(
        run(&km, "Control+Shift+S", FocusContext::Global),
        Outcome::NoMatch
    );
    assert_eq!(run(&km, "S", FocusContext::Global), Outcome::NoMatch);
    assert_eq!(
        run(&km, "Control+Shift+P", FocusContext::Global),
        Outcome::Run(id("view.palette"))
    );
    assert_eq!(
        run(&km, "F1", FocusContext::Global),
        Outcome::Run(id("app.help"))
    );
    assert_eq!(
        run(&km, "Shift+F1", FocusContext::Global),
        Outcome::Run(id("app.help"))
    );
}

#[test]
fn the_key_is_compared_as_a_key_not_as_a_place() {
    // QWERTZ: the key that makes `Z` is Z, whatever its place; case does not matter either.
    let km = Keymap::from_commands(&commands(), LINUX);
    let ctrl = Modifiers {
        control: true,
        ..Modifiers::default()
    };
    assert_eq!(
        km.dispatch(&KeyPress::new(ctrl, "z").unwrap(), FocusContext::Global),
        Outcome::Run(id("edit-undo"))
    );
    assert_eq!(
        km.dispatch(&KeyPress::new(ctrl, "Z").unwrap(), FocusContext::Global),
        Outcome::Run(id("edit-undo"))
    );
}

#[test]
fn a_locked_command_does_not_run_and_says_why() {
    let km = Keymap::from_commands(&commands(), LINUX);
    assert_eq!(
        run(&km, "Control+E", FocusContext::Global),
        Outcome::Disabled {
            command: id("file-export"),
            reason: "nothing-to-export".into()
        }
    );
    let unsupported = Commands::new().with(
        Command::new("a.b", "b")
            .shortcut(sc("F4"))
            .unsupported("not-here"),
    );
    assert_eq!(
        run(
            &Keymap::from_commands(&unsupported, LINUX),
            "F4",
            FocusContext::Global
        ),
        Outcome::Disabled {
            command: id("a.b"),
            reason: "not-here".into()
        }
    );
}

#[test]
fn two_commands_with_one_key_in_one_context_run_nothing() {
    let doubled = commands().with(Command::new("other.save", "save too").shortcut(sc("Control+S")));
    let km = Keymap::from_commands(&doubled, LINUX);
    assert_eq!(
        run(&km, "Control+S", FocusContext::Global),
        Outcome::Conflict(vec![id("file-save"), id("other.save")]),
        "Primary+S and Control+S are one key on Linux"
    );
    // On macOS the two are different keys: no conflict there.
    let mac = Keymap::from_commands(&doubled, MAC);
    assert_eq!(
        run(&mac, "Meta+S", FocusContext::Global),
        Outcome::Run(id("file-save"))
    );
}

#[test]
fn a_conflict_counts_even_when_one_of_the_two_is_locked() {
    let doubled = commands().with(
        Command::new("other.save", "x")
            .shortcut(sc("Control+S"))
            .disabled("why"),
    );
    let km = Keymap::from_commands(&doubled, LINUX);
    assert!(matches!(
        run(&km, "Control+S", FocusContext::Global),
        Outcome::Conflict(_)
    ));
}

#[test]
fn one_command_that_lists_a_key_twice_is_no_conflict() {
    let twice = Commands::new().with(
        Command::new("x.y", "y")
            .shortcut(sc("Primary+Y"))
            .shortcut(sc("Control+Y")),
    );
    let km = Keymap::from_commands(&twice, LINUX);
    assert_eq!(
        run(&km, "Control+Y", FocusContext::Global),
        Outcome::Run(id("x.y"))
    );
}
