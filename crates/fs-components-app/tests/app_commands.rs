//! The standard menu as the fixed commands `app.*` (M3, ADR 0065): the menu bar built from the model, Ctrl+Q and the line Quit both
//! close the window through the shell, Settings, Help and About go to the program, the words come from the shell's own texts, and
//! a program cannot take an id of the shell.
//!
//! Pattern: Integration tests of the real shell (see `menu_common`) with fixed expected texts.
//!
//! Mutants: the shell does not answer `app.quit` (`quit_is_answered_by_the_shell_from_the_menu`, `ctrl_q_closes_the_window`); `app.quit`
//! also reaches the handler (`the_program_never_sees_app_quit`); a shell key is missing in one language
//! (`the_menu_bar_speaks_the_language_of_the_shell`, `the_shell_words_have_a_german_twin_for_the_context_menu`).

#[path = "menu_common/mod.rs"]
mod menu_common;
#[path = "shortcuts_common/mod.rs"]
mod shortcuts_common;
#[path = "menu_model_common/mod.rs"]
mod menu_model_common;

use std::sync::{Arc, Mutex};

use fs_components_app::commands::{
    Arrangement, Command, Commands, KeyNames, Keymap, MenuItem, Platform, TextKey, resolve,
    standard_menubar, with_standard,
};
use fs_components_app::{AppWidget, Menu, ModelTexts, shell_words};
use fs_components_words::Locale;
use menu_common::Harness;
use shortcuts_common::{key, log};

const Q: u32 = 81;

struct Program {
    h: Harness,
    seen: shortcuts_common::Log,
    closed: Arc<Mutex<u32>>,
    menus: Vec<Menu>,
}

fn program(locale: &Locale) -> Program {
    let all = with_standard(Commands::new().with(Command::new("list", "list"))).unwrap();
    let bar = Arrangement::new(
        "menubar",
        standard_menubar(vec![MenuItem::command("list")]),
    );
    let items = resolve(&bar, &all).unwrap();
    let say_shell = shell_words(locale);
    let say = |k: &TextKey| {
        if k.as_str() == "list" {
            "List".to_owned()
        } else {
            say_shell(k)
        }
    };
    let names = KeyNames::english();
    let texts = ModelTexts {
        say: &say,
        platform: Platform::Linux,
        names: &names,
    };
    let menus: Vec<Menu> = items
        .iter()
        .map(|top| match top {
            fs_components_app::commands::Resolved::Submenu {
                label, children, ..
            } => Menu::from_model(say(label), children, &texts),
            other => panic!("{other:?}"),
        })
        .collect();
    let app = AppWidget::new("T", vec![]).unwrap();
    let seen = shortcuts_common::Log::default();
    let sink = Arc::clone(&seen);
    app.set_menus(menus.clone(), move |id| {
        sink.lock().unwrap().push(id.to_owned());
    })
    .unwrap();
    app.set_shortcuts(
        Keymap::from_resolved(&items, Platform::Linux),
        say,
        locale.clone(),
    );
    let closed = Arc::new(Mutex::new(0u32));
    let counter = Arc::clone(&closed);
    app.set_quit(move || *counter.lock().unwrap() += 1);
    Program {
        h: Harness::new(app, 800.0, 500.0),
        seen,
        closed,
        menus,
    }
}

#[test]
fn the_menu_bar_speaks_the_language_of_the_shell() {
    let en = program(&Locale::english());
    let labels = |p: &Program| -> Vec<String> { p.menus.iter().map(|m| m.label.clone()).collect() };
    assert_eq!(labels(&en), ["Program", "View", "Help"]);
    let de = program(&Locale::parse("de").unwrap());
    assert_eq!(labels(&de), ["Programm", "Ansicht", "Hilfe"]);
    let program_menu: Vec<&str> = en.menus[0].entries.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(program_menu, ["app.settings", "", "app.quit"]);
    assert_eq!(en.menus[0].entries[2].shortcut.as_deref(), Some("Ctrl+Q"));
    assert_eq!(en.menus[0].entries[2].label, "Quit");
    assert_eq!(de.menus[0].entries[2].label, "Beenden");
    let help: Vec<&str> = en.menus[2].entries.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(help, ["app.help", "app.about"]);
}

#[test]
fn quit_is_answered_by_the_shell_from_the_menu() {
    let p = program(&Locale::english());
    assert!(p.h.app.choose("app.quit"));
    assert_eq!(*p.closed.lock().unwrap(), 1);
}

#[test]
fn the_program_never_sees_app_quit() {
    let p = program(&Locale::english());
    p.h.app.choose("app.quit");
    assert!(log(&p.seen).is_empty());
}

#[test]
fn ctrl_q_closes_the_window() {
    let mut p = program(&Locale::english());
    key(&mut p.h, Q, true);
    assert_eq!(*p.closed.lock().unwrap(), 1);
    assert!(log(&p.seen).is_empty());
}

#[test]
fn settings_help_and_about_go_to_the_program() {
    let p = program(&Locale::english());
    for id in ["app.settings", "app.help", "app.about"] {
        assert!(p.h.app.choose(id), "{id}");
    }
    assert_eq!(log(&p.seen), ["app.settings", "app.help", "app.about"]);
    assert_eq!(*p.closed.lock().unwrap(), 0);
}

#[test]
fn the_shell_words_have_a_german_twin_for_the_context_menu() {
    let key = TextKey::new("shell-context-more");
    assert_eq!(shell_words(&Locale::english())(&key), "More options");
    assert_eq!(
        shell_words(&Locale::parse("de").unwrap())(&key),
        "Weitere Optionen"
    );
    let unknown = TextKey::new("not-a-shell-key");
    assert_eq!(shell_words(&Locale::english())(&unknown), "not-a-shell-key");
}
