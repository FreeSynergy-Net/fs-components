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

// ---- the commands of the notification module (ADR 0106 point 3) ----
//
// Intended mutants (none drawn; this is intent, not proof): `notification.` is not reserved for a program (`a_program_cannot_use_the_notification_ids`); a command loses its
// result or its required title (`the_notification_commands_are_typed`); the shell menu changes (`with_standard_keeps_the_shell_menu_and_the_notification_commands_join_beside_it`).

use fs_components_commands::{ParamType, ResultType, notification_commands};

#[test]
fn the_notification_commands_are_typed() {
    let list = notification_commands();
    assert_eq!(validate_commands(&list), vec![]);
    let ids: Vec<&str> = list.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "notification.tell",
            "notification.ask",
            "notification.close"
        ]
    );
    let tell = list.get(&"notification.tell".into()).unwrap();
    assert!(
        tell.params
            .iter()
            .any(|p| p.name == "title" && p.required && p.ty == Some(ParamType::Text))
    );
    assert_eq!(tell.result, ResultType::Value(ParamType::Text));
    let close = list.get(&"notification.close".into()).unwrap();
    assert_eq!(close.result, ResultType::Nothing);
}

#[test]
fn a_program_cannot_use_the_notification_ids() {
    let program = Commands::new()
        .with(Command::new("notification.tell", "x"))
        .with(Command::new("list", "list"));
    let problems = with_standard(&program).unwrap_err();
    assert_eq!(
        problems,
        vec![Problem::ReservedId("notification.tell".into())]
    );
}

#[test]
fn with_standard_keeps_the_shell_menu_and_the_notification_commands_join_beside_it() {
    let program = Commands::new().with(Command::new("list", "list"));
    let all = with_standard(&program).unwrap();
    let ids: Vec<&str> = all.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(
        ids,
        ["app.settings", "app.quit", "app.help", "app.about", "list"],
        "the menu is unchanged"
    );
    let both = notification_commands()
        .iter()
        .fold(all, |all, c| all.with(c.clone()));
    assert_eq!(
        validate_commands(&both),
        vec![],
        "no clash between the three groups"
    );
}

#[test]
fn app_settings_has_an_optional_key_and_nothing_else() {
    // Q1: the jump to one setting is a free parameter; without it the command is what it was (Mutant: `key` required -> red)
    let list = standard_commands();
    let c = list.get(&SETTINGS.into()).unwrap();
    assert_eq!(c.params.len(), 1);
    assert_eq!(c.params[0].name, "key");
    assert!(!c.params[0].required, "the key is optional");
    assert_eq!(validate_commands(&list), vec![]);
    for id in [QUIT, HELP, ABOUT] {
        assert_eq!(list.get(&id.into()).unwrap().params, vec![]);
    }
}
