//! K18-K20 of ENTWURF section 5: fs-tasks as a described provider, and the tasks board as a description.
mod support;

use fs_components_catalog::{Binding, Role, Source, check};
use fs_types::{BasicType, TypeRegistry};
use support::tasks_board::{board_bindings, tasks_provider, tasks_range};
use support::tasks_catalog::{PRIORITY, TASK, task_shape, task_type};
use support::util::*;

/// K18: the description of `Task` is valid, its shape names real fields, the commands are there, `setColumns` is unsupported.
#[test]
fn k18_task_description() {
    let mut registry = TypeRegistry::new();
    assert!(registry.add(task_type()).is_ok());
    let t = task_type();
    let shape = task_shape();
    let names = shape
        .groupings
        .iter()
        .filter_map(|g| g.field.as_ref())
        .chain(shape.orders.iter().filter_map(|o| o.field.as_ref()));
    assert!(names.into_iter().all(|p| t.find_field(&p[0]).is_some()));
    let rc: Vec<&str> = t
        .record_commands()
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    for want in ["change", "delete", "move", "claim", "release", "note"] {
        assert!(rc.contains(&want), "{want} in {rc:?}");
    }
    assert!(!t.find_command("setColumns").unwrap().supported);
    assert_eq!(t.find_field("priority").unwrap().choices, PRIORITY);
    assert_eq!(t.find_field("dueDate").unwrap().label, "task.due-date");
    assert_eq!(t.find_field("dueDate").unwrap().basic, BasicType::Date);
}

/// K19: the tasks board, with and without the priority lanes, is drawable from the description alone (no problem).
#[test]
fn k19_board_bindings() {
    let c = catalog();
    let board = c.need("board").unwrap();
    for lanes in [false, true] {
        let b = board_bindings(lanes).unwrap();
        assert_eq!(
            show(&check(board, &b, &[tasks_provider()])),
            Vec::<String>::new(),
            "lanes={lanes}"
        );
    }
}

/// K20: the detail view of a task (record, title, 7 properties, lock, 3 actions) has no problem.
#[test]
fn k20_detail_bindings() {
    let c = catalog();
    let r = tasks_range().unwrap();
    let b = |slot: &str, source| Binding::new(slot, &r, TASK, source);
    let prop = |n: &str| {
        b(
            "properties",
            Source::Field {
                path: vec![n.into()],
                role: Role::Display,
            },
        )
    };
    let mut list = vec![
        b("record", Source::Records),
        b(
            "title",
            Source::Field {
                path: vec!["title".into()],
                role: Role::Display,
            },
        ),
    ];
    list.extend(
        [
            "description",
            "due",
            "progress",
            "priority",
            "keywords",
            "participants",
            "note",
        ]
        .map(prop),
    );
    list.push(b("lock", Source::Lock));
    list.extend(["claim", "release", "note"].map(|n| b("actions", Source::Command(n.into()))));
    assert_eq!(
        show(&check(
            c.need("board-detail").unwrap(),
            &list,
            &[tasks_provider()]
        )),
        Vec::<String>::new()
    );
}
