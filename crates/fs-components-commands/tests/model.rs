//! Tests: command list checks, radio groups, conflicts of shortcuts.

use fs_components_commands::{
    Command, CommandId, Commands, Platform, Problem, RadioError, Shortcut, State, Toggle,
    conflicts, validate_commands,
};

fn sc(text: &str) -> Shortcut {
    Shortcut::parse(text).unwrap()
}

fn id(name: &str) -> CommandId {
    CommandId::new(name)
}

fn views() -> Commands {
    Commands::new()
        .with(Command::new("view-list", "view-list").radio("view", true))
        .with(Command::new("view-tree", "view-tree").radio("view", false))
        .with(Command::new("view-grid", "view-grid").radio("view", false))
        .with(Command::new("view-hidden", "view-hidden").check(false))
        .with(Command::new("other", "other").radio("sort", true))
}

#[test]
fn a_valid_list_has_no_problems() {
    assert_eq!(validate_commands(&views()), vec![]);
    assert!(Commands::new().is_empty());
    assert_eq!(views().len(), 5);
}

#[test]
fn locked_needs_a_reason() {
    let ok = Commands::new()
        .with(Command::new("a", "a").disabled("a-why"))
        .with(Command::new("b", "b").unsupported("b-why"));
    assert_eq!(validate_commands(&ok), vec![]);
    let bad = Commands::new()
        .with(Command::new("a", "a").disabled(""))
        .with(Command::new("b", "b").unsupported(""))
        .with(Command::new("c", "c"));
    assert_eq!(
        validate_commands(&bad),
        vec![
            Problem::LockedWithoutReason(id("a")),
            Problem::LockedWithoutReason(id("b"))
        ]
    );
    let state = Command::new("a", "a").disabled("why").state;
    assert!(state.is_locked());
    assert_eq!(state.reason().unwrap().as_str(), "why");
    assert!(!State::Enabled.is_locked());
    assert!(State::Enabled.reason().is_none());
}

#[test]
fn empty_and_double_ids() {
    let bad = Commands::new()
        .with(Command::new("", "x"))
        .with(Command::new("a", "a"))
        .with(Command::new("a", "a"))
        .with(Command::new("a", "a"));
    assert_eq!(
        validate_commands(&bad),
        vec![Problem::EmptyCommandId, Problem::DuplicateCommand(id("a"))]
    );
}

#[test]
fn a_radio_group_has_exactly_one_on() {
    let none = Commands::new()
        .with(Command::new("a", "a").radio("g", false))
        .with(Command::new("b", "b").radio("g", false));
    assert_eq!(
        validate_commands(&none),
        vec![Problem::RadioGroupOn {
            group: "g".into(),
            on: 0
        }]
    );
    let two = Commands::new()
        .with(Command::new("a", "a").radio("g", true))
        .with(Command::new("b", "b").radio("g", true))
        .with(Command::new("c", "c").radio("h", true));
    assert_eq!(
        validate_commands(&two),
        vec![Problem::RadioGroupOn {
            group: "g".into(),
            on: 2
        }]
    );
}

#[test]
fn choosing_a_radio_line_switches_the_others_of_its_group_off() {
    let mut list = views();
    list.choose_radio(&id("view-tree")).unwrap();
    assert_eq!(list.radio_on("view"), vec![&id("view-tree")]);
    assert_eq!(
        list.radio_on("sort"),
        vec![&id("other")],
        "another group stays"
    );
    assert_eq!(validate_commands(&list), vec![]);
    list.choose_radio(&id("view-tree")).unwrap();
    assert_eq!(
        list.radio_on("view"),
        vec![&id("view-tree")],
        "again: no change"
    );
    list.choose_radio(&id("view-grid")).unwrap();
    assert_eq!(list.radio_on("view"), vec![&id("view-grid")]);
    assert!(matches!(
        list.get(&id("view-list")).unwrap().toggle,
        Toggle::Radio { on: false, .. }
    ));
}

#[test]
fn a_bad_choice_changes_nothing() {
    let mut list = views().with(
        Command::new("locked", "l")
            .radio("view", false)
            .disabled("why"),
    );
    let before = list.clone();
    assert_eq!(
        list.choose_radio(&id("nope")),
        Err(RadioError::Unknown(id("nope")))
    );
    assert_eq!(
        list.choose_radio(&id("view-hidden")),
        Err(RadioError::NotToggle(id("view-hidden")))
    );
    assert_eq!(
        list.choose_radio(&id("locked")),
        Err(RadioError::Locked(id("locked")))
    );
    assert_eq!(list, before);
}

#[test]
fn check_lines_switch_on_and_off() {
    let mut list = views().with(Command::new("l", "l").check(false).disabled("why"));
    list.set_check(&id("view-hidden"), true).unwrap();
    assert!(matches!(
        list.get(&id("view-hidden")).unwrap().toggle,
        Toggle::Check { on: true }
    ));
    list.set_check(&id("view-hidden"), false).unwrap();
    assert!(matches!(
        list.get(&id("view-hidden")).unwrap().toggle,
        Toggle::Check { on: false }
    ));
    assert_eq!(
        list.set_check(&id("zzz"), true),
        Err(RadioError::Unknown(id("zzz")))
    );
    assert_eq!(
        list.set_check(&id("view-list"), true),
        Err(RadioError::NotToggle(id("view-list")))
    );
    assert_eq!(
        list.set_check(&id("l"), true),
        Err(RadioError::Locked(id("l")))
    );
}

#[test]
fn a_conflict_names_both_commands() {
    let list = Commands::new()
        .with(Command::new("save", "s").shortcut(sc("Primary+S")))
        .with(Command::new("search", "s").shortcut(sc("Control+S")))
        .with(Command::new("open", "o").shortcut(sc("Primary+O")))
        .with(Command::new("third", "t").shortcut(sc("Primary+S")));
    let found = conflicts(&list, Platform::Linux);
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].commands,
        vec![id("save"), id("search"), id("third")]
    );
    assert_eq!(found[0].shortcut.aria(), "Primary+S");
}

#[test]
fn the_same_text_is_no_conflict_where_the_keys_differ() {
    let list = Commands::new()
        .with(Command::new("a", "a").shortcut(sc("Primary+S")))
        .with(Command::new("b", "b").shortcut(sc("Control+S")));
    assert_eq!(conflicts(&list, Platform::Linux).len(), 1);
    assert_eq!(
        conflicts(&list, Platform::Mac),
        vec![],
        "Cmd+S and Ctrl+S are two keys on a Mac"
    );
}

#[test]
fn one_command_twice_and_different_keys_are_no_conflict() {
    let list = Commands::new()
        .with(
            Command::new("a", "a")
                .shortcut(sc("Control+S"))
                .shortcut(sc("Primary+S")),
        )
        .with(Command::new("b", "b").shortcut(sc("Control+Shift+S")))
        .with(Command::new("c", "c").shortcut(sc("Control+T")));
    assert_eq!(conflicts(&list, Platform::Linux), vec![]);
}

#[test]
fn a_locked_command_still_conflicts() {
    let list = Commands::new()
        .with(Command::new("a", "a").shortcut(sc("F5")).disabled("why"))
        .with(Command::new("b", "b").shortcut(sc("F5")));
    assert_eq!(conflicts(&list, Platform::Linux).len(), 1);
}
