//! Tests: the command `layout.frontend` and the radio lines of the frontends (data only, nothing drawn).
//!
//! Pattern: Registry tests with fixed expected ids. The texts de/en live in `fs-components-app` (`tests/frontend_texts.rs`).
//!
//! Mutants (drawn, see BERICHT-frontend-c0.md): the choice drops a row (`the_command_offers_every_row`); the chosen line is not
//! switched on (`the_chosen_row_is_on_and_choosing_switches_the_others_off`); `layout.` is not reserved
//! (`a_program_cannot_use_the_layout_prefix`).

use fs_components_commands::{
    Arrangement, Command, Commands, FRONTEND, FrontendRow, ParamType, Problem, Resolved,
    frontend_command, frontend_lines, frontend_menu, resolve, standard_menubar, validate_commands,
    with_standard,
};

fn rows() -> Vec<FrontendRow> {
    ["frontend-list", "frontend-board", "frontend-tree"]
        .map(FrontendRow::new)
        .to_vec()
}

#[test]
fn the_command_offers_every_row() {
    let c = frontend_command(&rows());
    assert_eq!(c.id.as_str(), "layout.frontend");
    assert_eq!(c.params.len(), 1);
    assert_eq!(c.params[0].name, "frontend");
    assert!(c.params[0].required);
    assert_eq!(
        c.params[0].ty,
        Some(ParamType::choice([
            "frontend-list",
            "frontend-board",
            "frontend-tree"
        ]))
    );
    assert_eq!(validate_commands(&Commands::new().with(c)), vec![]);
}

#[test]
fn the_chosen_row_is_on_and_choosing_switches_the_others_off() {
    let mut lines = frontend_lines(&rows(), Some("frontend-board"));
    assert_eq!(validate_commands(&lines), vec![]);
    let on = |l: &Commands| -> Vec<String> {
        l.radio_on("layout.frontend")
            .iter()
            .map(|i| i.as_str().to_owned())
            .collect()
    };
    assert_eq!(on(&lines), ["layout.frontend.frontend-board"]);
    lines
        .choose_radio(&"layout.frontend.frontend-tree".into())
        .unwrap();
    assert_eq!(on(&lines), ["layout.frontend.frontend-tree"]);
    assert_eq!(
        on(&frontend_lines(&rows(), Some("nope"))),
        ["layout.frontend.frontend-list"]
    );
    assert_eq!(
        on(&frontend_lines(&rows(), None)),
        ["layout.frontend.frontend-list"]
    );
    assert!(frontend_lines(&[], None).is_empty());
}

#[test]
fn the_view_menu_lines_resolve_against_the_lines() {
    let lines = frontend_lines(&rows(), None);
    // the lines carry the reserved prefix, so a program cannot bring them (the shell adds them itself)
    assert_eq!(with_standard(&lines).unwrap_err().len(), 3);
    let shell = fs_components_commands::standard_commands()
        .iter()
        .chain(lines.iter())
        .fold(Commands::new(), |a, c| a.with(c.clone()));
    let bar = Arrangement::new("menubar", standard_menubar(frontend_menu(&rows())));
    let out = resolve(&bar, &shell).unwrap();
    let Resolved::Submenu { children, .. } = &out[1] else {
        panic!("{out:?}")
    };
    assert_eq!(children.len(), 3);
}

#[test]
fn a_program_cannot_use_the_layout_prefix() {
    let program = Commands::new()
        .with(Command::new(FRONTEND, "x"))
        .with(Command::new("ok", "ok"));
    assert_eq!(
        with_standard(&program).unwrap_err(),
        vec![Problem::ReservedId(FRONTEND.into())]
    );
}
