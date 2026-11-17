//! Shortcuts run commands: the global key hook, one handler for click and key, locked commands, conflicts, text field, and
//! the shortcut a line shows is the one that runs it. The real shell and Blinc's broadcast of `KEY_DOWN`; no window.

#[path = "menu_common/mod.rs"]
mod menu_common;
#[path = "menu_model_common/mod.rs"]
mod menu_model_common;
#[path = "shortcuts_common/mod.rs"]
mod shortcuts_common;

use fs_components_app::commands::{
    Command, FocusContext, KeyPress, Keymap, Modifiers, Outcome, Platform, Shortcut,
};
use fs_components_app::{AppWidget, Status};
use fs_components_words::Locale;
use menu_model_common::{commands, say};
use shortcuts_common::{C, DELETE, N, S, key, log, program, release, status};

#[test]
fn a_shortcut_runs_its_command_once_through_the_menu_handler() {
    let (mut h, seen) = program(None);
    key(&mut h, N, true);
    assert_eq!(log(&seen), ["document-new"]);
    release(&mut h, N, true);
    key(&mut h, N, false);
    key(&mut h, C, true);
    assert_eq!(
        log(&seen),
        ["document-new", "edit-copy"],
        "no Ctrl, no run; a submenu command runs too"
    );
}

#[test]
fn click_and_shortcut_each_fire_once() {
    let (mut h, seen) = program(None);
    h.click("fs-app-menu-0");
    h.click("fs-app-menu-entry-document-new");
    assert_eq!(
        log(&seen),
        ["document-new"],
        "a click fires once, the shortcut did not add one"
    );
    key(&mut h, N, true);
    assert_eq!(log(&seen), ["document-new", "document-new"]);
}

#[test]
fn a_shortcut_closes_an_open_menu_and_runs() {
    let (mut h, seen) = program(None);
    h.click("fs-app-menu-0");
    assert!(h.has("fs-app-menu-panel"));
    key(&mut h, N, true);
    h.settle();
    assert_eq!(log(&seen), ["document-new"]);
    assert!(!h.has("fs-app-menu-panel"));
}

#[test]
fn a_locked_command_does_not_run_and_the_reason_is_shown() {
    let (mut h, seen) = program(None);
    key(&mut h, S, true);
    assert_eq!(log(&seen), Vec::<String>::new());
    assert_eq!(status(&h), Some(Status::Info("Nothing to save".into())));
    key(&mut h, DELETE, false);
    assert!(log(&seen).is_empty(), "the delete line is locked as well");
    assert_eq!(status(&h), Some(Status::Info("Nothing selected".into())));
}

#[test]
fn a_conflict_runs_nothing_and_is_shown() {
    let twin = Command::new("other.new", "new").shortcut(Shortcut::parse("Control+N").unwrap());
    let (mut h, seen) = program(Some(twin));
    key(&mut h, N, true);
    assert!(log(&seen).is_empty(), "never decided silently");
    let Some(Status::Problem(text)) = status(&h) else {
        panic!("a problem in the status line: {:?}", status(&h));
    };
    assert!(
        text.contains("Control+N") && text.contains("document-new") && text.contains("other.new"),
        "{text}"
    );
}

#[test]
fn the_conflict_message_comes_in_german() {
    let twin = Command::new("other.new", "new").shortcut(Shortcut::parse("Control+N").unwrap());
    let (mut h, _) = program(Some(twin));
    let keymap = Keymap::from_commands(
        &commands()
            .with(Command::new("other.new", "n").shortcut(Shortcut::parse("Control+N").unwrap())),
        Platform::Linux,
    );
    h.app
        .set_shortcuts(keymap, say(false), Locale::parse("de").unwrap());
    key(&mut h, N, true);
    let Some(Status::Problem(text)) = status(&h) else {
        panic!("no problem")
    };
    assert!(text.starts_with("Die Taste Control+N"), "{text}");
}

#[test]
fn in_a_text_field_the_text_keeps_its_keys_and_the_program_keeps_the_rest() {
    let (h, seen) = program(None);
    let ctrl =
        |key: &str| KeyPress::new(Shortcut::parse("Control+N").unwrap().modifiers(), key).unwrap();
    // Ctrl+C is copy in the field, even though the program has a command for it; Ctrl+N is the program's.
    assert_eq!(h.app.press(&ctrl("C"), true), Outcome::NoMatch);
    assert_eq!(
        h.app.press(&ctrl("N"), true),
        Outcome::Run("document-new".into())
    );
    assert_eq!(log(&seen), ["document-new"]);
    assert_eq!(
        h.app.press(&ctrl("C"), false),
        Outcome::Run("edit-copy".into())
    );
}

#[test]
fn a_list_that_has_the_keyboard_gives_its_context() {
    let (h, seen) = program(None);
    let delete = KeyPress::new(Modifiers::default(), "Delete").unwrap();
    h.app.set_focus_context(FocusContext::List);
    assert!(matches!(
        h.app.press(&delete, false),
        Outcome::Disabled { .. }
    ));
    assert!(
        matches!(h.app.press(&delete, true), Outcome::NoMatch),
        "in a field Delete is text"
    );
    assert_eq!(log(&seen), Vec::<String>::new());
}

#[test]
fn no_shortcuts_installed_means_no_match() {
    let app = AppWidget::new("T", vec![]).unwrap();
    let press = KeyPress::new(Shortcut::parse("Control+N").unwrap().modifiers(), "N").unwrap();
    assert_eq!(app.press(&press, false), Outcome::NoMatch);
}

#[test]
fn the_shortcut_a_line_shows_is_the_one_that_runs_it() {
    // Display and execution come from the same resolved items: for every line that shows a shortcut, pressing it
    // leads to that line's command (or, if the line is locked, to its reason).
    let (h, seen) = program(None);
    let menus = h.app.with_shell(|s| s.menus().to_vec());
    let mut checked = 0;
    for menu in &menus {
        let mut lines = Vec::new();
        for entry in &menu.entries {
            lines.push(entry);
            lines.extend(&entry.children);
        }
        for line in lines {
            let Some(aria) = &line.shortcut_aria else {
                continue;
            };
            let shortcut = Shortcut::parse(aria).unwrap();
            let mods = shortcut.modifiers();
            let mods = fs_components_app::commands::Modifiers {
                primary: false,
                control: mods.primary || mods.control,
                ..mods
            };
            let outcome = h
                .app
                .press(&KeyPress::new(mods, shortcut.key()).unwrap(), false);
            if line.disabled {
                assert!(
                    matches!(&outcome, Outcome::Disabled { command, .. } if command.as_str() == line.id),
                    "{} {outcome:?}",
                    line.id
                );
            } else {
                assert_eq!(
                    outcome,
                    Outcome::Run(line.id.as_str().into()),
                    "{}",
                    line.id
                );
            }
            checked += 1;
        }
    }
    assert_eq!(
        checked, 7,
        "new, open, save, hidden, copy, delete, quit have a shortcut and a line"
    );
    assert_eq!(log(&seen).len(), 5, "the two locked lines did not run");
}

#[test]
fn the_ids_of_the_registered_commands_are_a_fixed_contract() {
    // Snapshot (decision b7): a command id is stable from M1 on, later workflows name commands by it. Changing one fails here.
    let ids: Vec<String> = commands()
        .iter()
        .map(|c| c.id.as_str().to_owned())
        .collect();
    assert_eq!(
        ids,
        [
            "document-new",
            "document-open",
            "document-save",
            "view-list",
            "view-tree",
            "files.hidden",
            "edit-copy",
            "edit-delete",
            "application-quit"
        ]
    );
}
