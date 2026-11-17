//! What the tests of the context menu model have in common: a file command list and its context arrangement.
//!
//! Tests: Test fixture.

#![allow(dead_code)]

use fs_components_commands::{
    Arrangement, Command, CommandId, Commands, MORE_KEY, MenuItem, Point, Resolved, Target, resolve,
};

pub fn id(name: &str) -> CommandId {
    CommandId::new(name)
}

pub fn file_commands() -> Commands {
    Commands::new()
        .with(Command::new("open", "open").icon("document-open"))
        .with(Command::new("cut", "cut").icon("edit-cut"))
        .with(Command::new("copy", "copy").icon("edit-copy"))
        .with(Command::new("delete", "delete").icon("edit-delete"))
        .with(Command::new("rename", "rename"))
        .with(Command::new("share", "share").disabled("share-why"))
        .with(Command::new("zip", "zip"))
        .with(Command::new("props", "props"))
}

pub fn context() -> Vec<Resolved> {
    let tree = Arrangement::new(
        "context:file",
        vec![
            MenuItem::icon_row(
                "quick",
                vec![
                    MenuItem::command("cut"),
                    MenuItem::command("copy"),
                    MenuItem::command("delete"),
                ],
            ),
            MenuItem::command("open"),
            MenuItem::command("share"),
            MenuItem::submenu(
                MORE_KEY,
                vec![MenuItem::command("rename"), MenuItem::command("zip")],
            ),
            MenuItem::Separator,
            MenuItem::command("props"),
        ],
    );
    resolve(&tree, &file_commands()).unwrap()
}

pub fn at(x: f32, y: f32) -> Point {
    Point { x, y }
}

pub fn file(ids: &[&str]) -> Target {
    Target {
        kind: "file".into(),
        ids: ids.iter().map(|s| (*s).to_owned()).collect(),
    }
}
