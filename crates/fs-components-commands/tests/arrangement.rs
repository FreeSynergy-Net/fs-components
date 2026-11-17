//! Tests: validation of the arrangement tree (separators, groups, submenus) and its resolution.

use fs_components_commands::{
    Arrangement, Command, CommandId, Commands, MenuItem, Problem, Resolved, resolve,
    validate_arrangement,
};

fn commands() -> Commands {
    ["new", "open", "save", "quit", "a", "b"]
        .into_iter()
        .fold(Commands::new(), |list, name| {
            list.with(Command::new(name, name))
        })
}

fn c(name: &str) -> MenuItem {
    MenuItem::command(name)
}

fn check(items: Vec<MenuItem>) -> Vec<Problem> {
    validate_arrangement(&Arrangement::new("menubar", items), &commands())
}

#[test]
fn a_good_tree_has_no_problems() {
    let items = vec![
        c("new"),
        MenuItem::Separator,
        MenuItem::submenu("more", vec![c("open"), MenuItem::Separator, c("save")]),
        MenuItem::Placeholder("search".into()),
        MenuItem::Separator,
        c("quit"),
    ];
    assert_eq!(check(items), vec![]);
}

#[test]
fn a_separator_at_the_edge_is_a_problem() {
    assert_eq!(
        check(vec![MenuItem::Separator, c("new")]),
        vec![Problem::SeparatorAtEdge { path: vec![0] }]
    );
    assert_eq!(
        check(vec![c("new"), MenuItem::Separator]),
        vec![Problem::SeparatorAtEdge { path: vec![1] }]
    );
    assert_eq!(
        check(vec![MenuItem::Separator]),
        vec![Problem::SeparatorAtEdge { path: vec![0] }]
    );
    assert_eq!(
        check(vec![MenuItem::submenu(
            "m",
            vec![MenuItem::Separator, c("new")]
        )]),
        vec![Problem::SeparatorAtEdge { path: vec![0, 0] }]
    );
}

#[test]
fn two_separators_in_a_row_are_a_problem() {
    assert_eq!(
        check(vec![
            c("new"),
            MenuItem::Separator,
            MenuItem::Separator,
            c("open")
        ]),
        vec![Problem::SeparatorTwice { path: vec![2] }]
    );
    assert_eq!(
        check(vec![
            c("new"),
            MenuItem::Separator,
            c("open"),
            MenuItem::Separator,
            c("save")
        ]),
        vec![]
    );
}

#[test]
fn groups_count_as_part_of_the_level_around_them() {
    let inside = vec![
        c("new"),
        MenuItem::group("g", vec![MenuItem::Separator]),
        MenuItem::Separator,
        c("open"),
    ];
    assert_eq!(
        check(inside),
        vec![Problem::SeparatorTwice { path: vec![2] }]
    );
    let edge = vec![MenuItem::group("g", vec![MenuItem::Separator, c("new")])];
    assert_eq!(
        check(edge),
        vec![Problem::SeparatorAtEdge { path: vec![0, 0] }]
    );
    let fine = vec![
        MenuItem::group("g1", vec![c("new"), c("open")]),
        MenuItem::Separator,
        MenuItem::group("g2", vec![c("save")]),
    ];
    assert_eq!(check(fine), vec![]);
}

#[test]
fn unknown_and_repeated_commands() {
    assert_eq!(
        check(vec![c("new"), c("ghost")]),
        vec![Problem::UnknownCommand {
            id: CommandId::new("ghost"),
            path: vec![1]
        }]
    );
    assert_eq!(
        check(vec![c("new"), MenuItem::submenu("m", vec![c("new")])]),
        vec![Problem::CommandTwice(CommandId::new("new"))]
    );
}

#[test]
fn empty_submenus_and_empty_names() {
    assert_eq!(
        check(vec![c("new"), MenuItem::submenu("m", vec![])]),
        vec![Problem::EmptySubmenu { path: vec![1] }]
    );
    assert_eq!(
        check(vec![
            c("new"),
            MenuItem::submenu("m", vec![MenuItem::group("g", vec![])])
        ]),
        vec![Problem::EmptySubmenu { path: vec![1] }]
    );
    let only_lines = check(vec![MenuItem::submenu("m", vec![MenuItem::Separator])]);
    assert!(only_lines.contains(&Problem::EmptySubmenu { path: vec![0] }));
    assert_eq!(
        check(vec![MenuItem::submenu("", vec![c("new")])]),
        vec![Problem::EmptyName { path: vec![0] }]
    );
    assert_eq!(
        check(vec![MenuItem::group("", vec![c("new")])]),
        vec![Problem::EmptyName { path: vec![0] }]
    );
    assert_eq!(
        check(vec![MenuItem::submenu(
            "m",
            vec![MenuItem::Placeholder("x".into())]
        )]),
        vec![]
    );
}

#[test]
fn resolution_inlines_groups_and_keeps_the_rest() {
    let list = commands();
    let tree = Arrangement::new(
        "context:file",
        vec![
            MenuItem::group("main", vec![c("new"), c("open")]),
            MenuItem::Separator,
            MenuItem::submenu("more", vec![c("save")]),
            MenuItem::Placeholder("search".into()),
        ],
    );
    let out = resolve(&tree, &list).unwrap();
    assert_eq!(out.len(), 5);
    assert!(matches!(&out[0], Resolved::Command(cmd) if cmd.id.as_str() == "new"));
    assert!(matches!(&out[1], Resolved::Command(cmd) if cmd.id.as_str() == "open"));
    assert_eq!(out[2], Resolved::Separator);
    match &out[3] {
        Resolved::Submenu {
            label, children, ..
        } => {
            assert_eq!(label.as_str(), "more");
            assert_eq!(children.len(), 1);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(out[4], Resolved::Placeholder("search".into()));
}

#[test]
fn resolution_carries_the_state_of_the_command() {
    let list = Commands::new().with(Command::new("save", "save").disabled("save-why"));
    let out = resolve(&Arrangement::new("m", vec![c("save")]), &list).unwrap();
    match &out[0] {
        Resolved::Command(cmd) => assert_eq!(cmd.state.reason().unwrap().as_str(), "save-why"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn resolution_reports_the_problems_of_both() {
    let list = Commands::new()
        .with(Command::new("a", "a").disabled(""))
        .with(Command::new("b", "b"));
    let tree = Arrangement::new("m", vec![MenuItem::Separator, c("a"), c("zzz")]);
    let problems = resolve(&tree, &list).unwrap_err();
    assert_eq!(problems.len(), 3);
    assert!(problems.contains(&Problem::LockedWithoutReason(CommandId::new("a"))));
    assert!(problems.contains(&Problem::SeparatorAtEdge { path: vec![0] }));
    assert!(
        problems
            .iter()
            .any(|p| matches!(p, Problem::UnknownCommand { .. }))
    );
    assert!(problems[0].to_string().contains("locked"));
}
