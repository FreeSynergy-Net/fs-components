//! Tests: the pure rules of the context menu model (M3): trigger and place, target, the two stages, icon rows.
//!
//! Pattern: Strategy tests (pure functions) with hand-written expected values; each rule has a case that fails without it.
//!
//! Mutants (each makes a test here red):
//! - Shift no longer opens the second stage: `shift_and_right_click_opens_the_second_stage`;
//! - the keyboard opens at the pointer / the element's origin: `the_keys_open_at_the_focused_element`;
//! - a row outside the selection keeps the old selection: `a_row_outside_the_selection_becomes_the_selection`;
//! - a row inside the selection acts on that row alone: `a_row_inside_the_selection_acts_on_all`;
//! - `stages` loses the separator or the order: `the_second_stage_dissolves_more_options`;
//! - an icon row without a symbol passes validation: `an_icon_row_needs_commands_with_symbols`.

#[path = "context_common/mod.rs"]
mod context_common;

use context_common::{at, context, file, file_commands, id};
use fs_components_commands::{
    Area, Arrangement, MORE_KEY, MenuItem, Problem, Resolved, Stage, Trigger, resolve, stages,
    target_for,
};

#[test]
fn the_pointer_opens_at_the_pointer() {
    let t = Trigger::Pointer {
        at: at(120.0, 80.0),
        shift: false,
    };
    assert_eq!(t.point(), at(120.0, 80.0));
    assert_eq!(t.stage(), Stage::First);
}

#[test]
fn shift_and_right_click_opens_the_second_stage() {
    let t = Trigger::Pointer {
        at: at(1.0, 2.0),
        shift: true,
    };
    assert_eq!(t.stage(), Stage::Full);
}

#[test]
fn the_keys_open_at_the_focused_element() {
    let element = Area {
        x: 100.0,
        y: 200.0,
        width: 300.0,
        height: 24.0,
    };
    for t in [Trigger::MenuKey { element }, Trigger::ShiftF10 { element }] {
        assert_eq!(t.point(), at(108.0, 224.0));
        assert_eq!(t.stage(), Stage::First, "the key's Shift is not the stage Shift");
    }
    // A tall element (a whole list has the keyboard) does not push the menu down; a narrow one keeps it inside.
    let list = Area {
        x: 10.0,
        y: 50.0,
        width: 5.0,
        height: 900.0,
    };
    assert_eq!(Trigger::MenuKey { element: list }.point(), at(15.0, 82.0));
}

#[test]
fn a_row_inside_the_selection_acts_on_all() {
    let selection = ["a".to_owned(), "b".to_owned(), "c".to_owned()];
    assert_eq!(target_for("file", &selection, Some("b")), file(&["a", "b", "c"]));
}

#[test]
fn a_row_outside_the_selection_becomes_the_selection() {
    let selection = ["a".to_owned(), "b".to_owned()];
    assert_eq!(target_for("file", &selection, Some("z")), file(&["z"]));
}

#[test]
fn empty_space_acts_on_nothing() {
    let selection = ["a".to_owned()];
    assert_eq!(target_for("file", &selection, None), file(&[]));
}

#[test]
fn the_second_stage_dissolves_more_options() {
    let items = context();
    let s = stages(&items);
    assert!(s.is_two_stage());
    assert_eq!(s.first, items);
    let names = |list: &[Resolved]| -> Vec<String> {
        list.iter()
            .map(|r| match r {
                Resolved::Command(c) => c.id.as_str().to_owned(),
                Resolved::Submenu { label, .. } => format!("sub:{}", label.as_str()),
                Resolved::Separator => "-".into(),
                Resolved::IconRow(row) => format!("icons:{}", row.len()),
                other => format!("{other:?}"),
            })
            .collect()
    };
    assert_eq!(
        names(&s.first),
        ["icons:3", "open", "share", "sub:shell-context-more", "-", "props"]
    );
    // The separator before the submenu's lines is added once; the one the arrangement had stays.
    assert_eq!(
        names(&s.full),
        ["icons:3", "open", "share", "-", "rename", "zip", "-", "props"]
    );
}

#[test]
fn a_menu_without_more_options_has_one_stage() {
    let only = vec![Resolved::Command(Command::new("a", "a"))];
    let s = stages(&only);
    assert!(!s.is_two_stage());
    assert_eq!(s.full, only);
}

#[test]
fn more_options_first_in_the_menu_adds_no_leading_separator() {
    let tree = Arrangement::new(
        "c",
        vec![MenuItem::submenu(MORE_KEY, vec![MenuItem::command("rename")])],
    );
    let items = resolve(&tree, &file_commands()).unwrap();
    assert_eq!(stages(&items).full.len(), 1);
}

#[test]
fn an_icon_row_needs_commands_with_symbols() {
    let tree = Arrangement::new(
        "c",
        vec![MenuItem::icon_row(
            "quick",
            vec![
                MenuItem::command("copy"),
                MenuItem::command("rename"),
                MenuItem::Separator,
                MenuItem::command("ghost"),
            ],
        )],
    );
    let problems = resolve(&tree, &file_commands()).unwrap_err();
    assert_eq!(
        problems,
        vec![
            Problem::IconRowWithoutIcon(id("rename")),
            Problem::IconRowNotCommand { path: vec![0, 2] },
            Problem::UnknownCommand {
                id: id("ghost"),
                path: vec![0, 3]
            },
        ]
    );
}

#[test]
fn an_icon_row_command_counts_for_the_keymap_and_twice_is_reported() {
    let tree = Arrangement::new(
        "c",
        vec![
            MenuItem::icon_row("quick", vec![MenuItem::command("copy")]),
            MenuItem::command("copy"),
        ],
    );
    assert_eq!(
        resolve(&tree, &file_commands()).unwrap_err(),
        vec![Problem::CommandTwice(id("copy"))]
    );
}
