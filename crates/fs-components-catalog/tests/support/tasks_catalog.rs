//! The type description of `Task` and the shape of its records, as fs-tasks would report them (gap L3: tasks serves no
//! `TypeCatalog` today, although `tasks.capnp` has `types @5`). Target: fs-tasks-core `describe.rs`, built in the `tasks_v2`
//! follow-up (draft §6); this file is the draft of WHAT is described, from the `tasks_v2` fixpoint (`sperren/tasks_v2.sha256`,
//! `fs-tasks-testdata/src/draft/tasks_v2/` at d49e6a8: `task.rs` `Task`, `board.rs` `ColumnSet`, `api.rs` `Tasks`).
//!
//! Names follow JSCalendar (RFC 8984) where it has one (`title`, `description`, `due`, `start`, `progress`, `priority`,
//! `keywords`, `participants`); our own fields keep the `tasks_v2` names (`stage`, `note`, `list`, `parent`).
//! Not described (no basic type yet, draft O8): `estimatedDuration` (duration), `percentComplete` (integer), `alerts`,
//! `recurrence`, `links`, `checklist` (structured lists). The order by hand (`rank`) is no field: it is the order `rank`.
//!
//! Pattern: Factory functions (pure; the same values every call).

use fs_types::{BasicType, CommandDescription, CommandKind, FieldDescription, TypeDescription};

use fs_records::{GroupingDescription, OrderDescription, Shape, path};

/// The name of the data type.
pub const TASK: &str = "Task";

/// The five `progress` keys (JSCalendar), in board order.
pub const PROGRESS: [&str; 5] = [
    "needs-action",
    "in-process",
    "completed",
    "failed",
    "cancelled",
];

/// The priority keys: RFC 8984 `priority` 1 (highest) … 9 (lowest); 0 („not set“) is no value.
pub const PRIORITY: [&str; 9] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];

/// The label key of a field: `task.` and the name in kebab case (`dueDate` → `task.due-date`; label keys are lower case).
fn label(name: &str) -> String {
    let mut out = String::from("task.");
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

fn task_ref(name: &str) -> FieldDescription {
    FieldDescription::new(name, BasicType::Reference)
        .required()
        .target(TASK)
        .label(&label(name))
}

/// The description of `Task`.
#[must_use]
pub fn task_type() -> TypeDescription {
    let f = |name: &str, basic: BasicType| FieldDescription::new(name, basic).label(&label(name));
    TypeDescription::new(TASK, "fs-tasks", "RFC 8984 Task", "task")
        .field(f("uid", BasicType::Text).read_only())
        .field(f("title", BasicType::Text).required())
        .field(f("description", BasicType::MultilineText))
        .field(f("due", BasicType::DateTime))
        .field(f("dueDate", BasicType::Date))
        .field(f("start", BasicType::DateTime))
        .field(f("startDate", BasicType::Date))
        .field(
            f("progress", BasicType::Choice)
                .required()
                .choices(&PROGRESS),
        )
        .field(f("priority", BasicType::Choice).choices(&PRIORITY))
        .field(f("keywords", BasicType::Keywords))
        .field(f("participants", BasicType::Reference).multiple())
        .field(f("stage", BasicType::Text).read_only())
        .field(f("note", BasicType::Text))
        .field(
            f("list", BasicType::Reference)
                .target("TaskList")
                .read_only(),
        )
        .field(f("parent", BasicType::Reference).target(TASK))
        .field(f("created", BasicType::DateTime).read_only())
        .field(f("updated", BasicType::DateTime).read_only())
        .command(CommandDescription::standard_find("task.find", TASK))
        .command(
            CommandDescription::new("create", CommandKind::Create, "task.create")
                .input(
                    FieldDescription::new("title", BasicType::Text)
                        .required()
                        .label("task.title"),
                )
                .input(
                    FieldDescription::new("list", BasicType::Reference)
                        .required()
                        .target("TaskList")
                        .label("task.list"),
                ),
        )
        .command(CommandDescription::new(
            "change",
            CommandKind::Change,
            "task.change",
        ))
        .command(CommandDescription::new(
            "delete",
            CommandKind::Delete,
            "task.delete",
        ))
        .command(
            CommandDescription::new("move", CommandKind::Other, "task.move")
                .input(task_ref("task"))
                .input(
                    FieldDescription::new("column", BasicType::Text)
                        .required()
                        .label("task.column"),
                )
                .input(
                    FieldDescription::new("after", BasicType::Reference)
                        .target(TASK)
                        .label("task.after"),
                ),
        )
        .command(
            CommandDescription::new("claim", CommandKind::Other, "task.claim")
                .input(task_ref("task"))
                .input(
                    FieldDescription::new("until", BasicType::DateTime)
                        .required()
                        .label("task.until"),
                ),
        )
        .command(
            CommandDescription::new("release", CommandKind::Other, "task.release")
                .input(task_ref("task")),
        )
        .command(
            CommandDescription::new("note", CommandKind::Other, "task.note")
                .input(task_ref("task"))
                .input(FieldDescription::new("note", BasicType::Text).label("task.note")),
        )
        // `board.columns` takes a whole column set (key, title, progress list, stage, wip): no basic types for it yet.
        // Described, not supported: the frontend shows it locked as „coming“ (draft O8).
        .command(
            CommandDescription::new("setColumns", CommandKind::Other, "task.set-columns")
                .input(
                    FieldDescription::new("list", BasicType::Reference)
                        .required()
                        .target("TaskList")
                        .label("task.list"),
                )
                .unsupported(),
        )
}

/// The shape of `Task`: the board (columns by rule B7), progress, priority; orders by hand, due, priority, updated.
#[must_use]
pub fn task_shape() -> Shape {
    let grouping = |name: &str, field: Option<&str>, mv: Option<&str>| GroupingDescription {
        name: name.to_owned(),
        label: format!("task.grouping.{name}"),
        field: field.map(path),
        move_command: mv.map(str::to_owned),
    };
    let order = |name: &str, field: Option<&str>, manual: bool| OrderDescription {
        name: name.to_owned(),
        label: format!("task.order.{name}"),
        field: field.map(path),
        manual,
    };
    Shape {
        data_type: TASK.to_owned(),
        groupings: vec![
            grouping("board", None, Some("move")),
            grouping("progress", Some("progress"), None),
            grouping("priority", Some("priority"), None),
        ],
        orders: vec![
            order("rank", None, true),
            order("due", Some("due"), false),
            order("priority", Some("priority"), false),
            order("updated", Some("updated"), false),
        ],
    }
}
