//! Tests: the dispatcher in a text field, with an input method, and built from resolved items.
//! Each rule has a case that fails without it (the mutant of M2: text precedence removed).

mod keymap_common;

use fs_components_commands::{
    Arrangement, Command, Commands, FocusContext, Keymap, MenuItem, Outcome, Platform, resolve,
};
use keymap_common::{commands, id, press, run, sc};

const LINUX: Platform = Platform::Linux;
const MAC: Platform = Platform::Mac;

#[test]
fn contexts_let_two_commands_share_a_key() {
    let shared = Commands::new()
        .with(Command::new("list.delete", "delete rows").shortcut(sc("Delete")))
        .with(Command::new("tree.delete", "delete node").shortcut(sc("Delete")));
    let km = Keymap::from_commands(&shared, LINUX)
        .only_in(&id("list.delete"), FocusContext::List)
        .only_in(&id("tree.delete"), FocusContext::Tree);
    assert_eq!(
        run(&km, "Delete", FocusContext::List),
        Outcome::Run(id("list.delete"))
    );
    assert_eq!(
        run(&km, "Delete", FocusContext::Tree),
        Outcome::Run(id("tree.delete"))
    );
    assert_eq!(
        run(&km, "Delete", FocusContext::Global),
        Outcome::NoMatch,
        "neither list nor tree has the keyboard"
    );
    let both_global = Keymap::from_commands(&shared, LINUX);
    assert!(matches!(
        run(&both_global, "Delete", FocusContext::Global),
        Outcome::Conflict(_)
    ));
}

#[test]
fn in_a_text_field_the_text_keeps_its_shortcuts() {
    // Undo and copy are program commands here, and still the field's own in a field.
    let km = Keymap::from_commands(&commands(), LINUX);
    for text in ["Control+Z", "Control+C", "Control+Shift+Z"] {
        assert_eq!(
            run(&km, text, FocusContext::TextField),
            Outcome::NoMatch,
            "{text} belongs to the field"
        );
    }
    assert_eq!(
        run(&km, "Control+Z", FocusContext::Global),
        Outcome::Run(id("edit-undo"))
    );
    assert_eq!(
        run(&km, "Control+C", FocusContext::List),
        Outcome::Run(id("edit-copy"))
    );
    let mac = Keymap::from_commands(&commands(), MAC);
    assert_eq!(
        run(&mac, "Meta+Z", FocusContext::TextField),
        Outcome::NoMatch
    );
    assert_eq!(
        run(&mac, "Control+Z", FocusContext::TextField),
        Outcome::NoMatch,
        "no command, no match"
    );
}

#[test]
fn in_a_text_field_program_shortcuts_still_work() {
    let km = Keymap::from_commands(&commands(), LINUX);
    assert_eq!(
        run(&km, "Control+S", FocusContext::TextField),
        Outcome::Run(id("file-save"))
    );
    assert_eq!(
        run(&km, "Control+Shift+P", FocusContext::TextField),
        Outcome::Run(id("view.palette"))
    );
    assert_eq!(
        run(&km, "Control+Enter", FocusContext::TextField),
        Outcome::Run(id("app.send"))
    );
    assert_eq!(
        run(&km, "F2", FocusContext::TextField),
        Outcome::Run(id("file-rename"))
    );
    assert_eq!(
        run(&km, "F1", FocusContext::TextField),
        Outcome::Run(id("app.help"))
    );
}

#[test]
fn in_a_text_field_editing_keys_and_letters_never_run_a_command() {
    let km = Keymap::from_commands(&commands(), LINUX);
    assert_eq!(
        run(&km, "Delete", FocusContext::TextField),
        Outcome::NoMatch,
        "Delete deletes text"
    );
    assert_eq!(
        run(&km, "Delete", FocusContext::List),
        Outcome::Run(id("edit-delete"))
    );
    // A letter without Ctrl is typed, even if a command has it.
    let bare = Commands::new().with(
        Command::new("a.s", "s")
            .shortcut(sc("S"))
            .shortcut(sc("Shift+Q")),
    );
    let km2 = Keymap::from_commands(&bare, LINUX);
    assert_eq!(run(&km2, "S", FocusContext::TextField), Outcome::NoMatch);
    assert_eq!(
        run(&km2, "Shift+Q", FocusContext::TextField),
        Outcome::NoMatch
    );
    assert_eq!(
        run(&km2, "S", FocusContext::Global),
        Outcome::Run(id("a.s"))
    );
}

#[test]
fn alt_letters_belong_to_the_text_in_a_field_and_work_elsewhere() {
    // The spike: an Alt letter is typed into the field, so no mnemonic runs there.
    let km = Keymap::from_commands(&commands(), LINUX);
    assert_eq!(run(&km, "Alt+F", FocusContext::TextField), Outcome::NoMatch);
    assert_eq!(
        run(&km, "Alt+F", FocusContext::Global),
        Outcome::Run(id("app.menu"))
    );
    // AltGr (Ctrl+Alt) is text on Windows and Linux.
    let altgr = Commands::new().with(Command::new("a.q", "q").shortcut(sc("Control+Alt+Q")));
    let km2 = Keymap::from_commands(&altgr, Platform::Windows);
    assert_eq!(
        run(&km2, "Control+Alt+Q", FocusContext::TextField),
        Outcome::NoMatch
    );
    assert_eq!(
        run(&km2, "Control+Alt+Q", FocusContext::Global),
        Outcome::Run(id("a.q"))
    );
}

#[test]
fn an_input_method_that_composes_owns_every_key() {
    let km = Keymap::from_commands(&commands(), LINUX);
    for context in [
        FocusContext::Global,
        FocusContext::List,
        FocusContext::TextField,
    ] {
        assert_eq!(
            km.dispatch(&press("Control+S").composing(true), context),
            Outcome::NoMatch
        );
        assert_eq!(
            km.dispatch(&press("F2").composing(true), context),
            Outcome::NoMatch
        );
    }
    assert!(press("F2").composing(true).is_composing() && !press("F2").is_composing());
}

#[test]
fn the_keymap_of_resolved_items_has_what_the_menu_shows_once() {
    let arrangement = Arrangement::new(
        "menubar",
        vec![
            MenuItem::command("file-save"),
            MenuItem::submenu("more", vec![MenuItem::command("edit-undo")]),
        ],
    );
    let items = resolve(&arrangement, &commands()).unwrap();
    let km = Keymap::from_resolved(&items, LINUX);
    assert_eq!(
        run(&km, "Control+S", FocusContext::Global),
        Outcome::Run(id("file-save"))
    );
    assert_eq!(
        run(&km, "Control+Z", FocusContext::Global),
        Outcome::Run(id("edit-undo"))
    );
    assert_eq!(
        run(&km, "Control+C", FocusContext::Global),
        Outcome::NoMatch,
        "not offered in this place"
    );
}
