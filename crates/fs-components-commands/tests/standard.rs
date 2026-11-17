//! Tests: the standard menu as fixed commands `app.*` (ADR 0065) in the model.
//!
//! Pattern: Registry / Template tests with fixed expected ids and keys.
//!
//! Mutants: Quit loses `Primary+Q` (`quit_and_help_have_their_keys`); `with_standard` accepts a program id `app.x`
//! (`a_program_cannot_use_an_id_of_the_shell`); the menu bar is not resolvable (`the_menubar_resolves_against_the_commands`);
//! View appears without lines (`view_exists_only_with_lines`); a standard command loses its symbol.

use fs_components_commands::standard::{ABOUT, HELP, QUIT, SETTINGS};
use fs_components_commands::{
    Arrangement, Command, Commands, KeyNames, MenuItem, Platform, Problem, Resolved, Shortcut,
    resolve, standard_commands, standard_menubar, validate_commands, with_standard,
};

#[test]
fn the_standard_commands_are_valid_and_fixed() {
    let list = standard_commands();
    assert_eq!(validate_commands(&list), vec![]);
    let ids: Vec<&str> = list.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, ["app.settings", "app.quit", "app.help", "app.about"]);
    assert!(list.iter().all(|c| c.icon.is_some()), "each has a symbol");
}

#[test]
fn quit_and_help_have_their_keys() {
    let list = standard_commands();
    let key = |id: &str| {
        list.get(&id.into())
            .and_then(|c| c.shortcuts.first())
            .map(|s| s.display(Platform::Linux, &KeyNames::english()))
    };
    assert_eq!(key(QUIT).as_deref(), Some("Ctrl+Q"));
    assert_eq!(key(HELP).as_deref(), Some("F1"));
    assert_eq!(key(SETTINGS), None);
    assert_eq!(key(ABOUT), None);
    assert_eq!(
        list.get(&QUIT.into()).unwrap().shortcuts[0],
        Shortcut::parse("Primary+Q").unwrap()
    );
}

#[test]
fn the_menubar_resolves_against_the_commands() {
    let bar = Arrangement::new("menubar", standard_menubar(vec![]));
    let out = resolve(&bar, &standard_commands()).unwrap();
    let labels: Vec<&str> = out
        .iter()
        .map(|r| match r {
            Resolved::Submenu { label, .. } => label.as_str(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(labels, ["shell-menu-program", "shell-menu-help"]);
}

#[test]
fn view_exists_only_with_lines() {
    let with = standard_menubar(vec![MenuItem::command("list")]);
    assert_eq!(with.len(), 3);
    assert!(
        matches!(&with[1], MenuItem::Submenu { label, .. } if label.as_str() == "shell-menu-view")
    );
    assert_eq!(standard_menubar(vec![]).len(), 2);
}

#[test]
fn the_program_commands_come_after_the_fixed_ones() {
    let all = with_standard(&Commands::new().with(Command::new("list", "list"))).unwrap();
    let ids: Vec<&str> = all.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        ids,
        ["app.settings", "app.quit", "app.help", "app.about", "list"]
    );
}

#[test]
fn a_program_cannot_use_an_id_of_the_shell() {
    let program = Commands::new()
        .with(Command::new("app.quit", "mine"))
        .with(Command::new("ok", "ok"))
        .with(Command::new("app.other", "x"));
    assert_eq!(
        with_standard(&program).unwrap_err(),
        vec![
            Problem::ReservedId("app.quit".into()),
            Problem::ReservedId("app.other".into())
        ]
    );
}
