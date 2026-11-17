//! What the dispatcher tests have in common: a command list with many shortcuts, and short helpers.
//!
//! Tests: Test fixture.

#![allow(dead_code)]

use fs_components_commands::{
    Command, CommandId, Commands, FocusContext, KeyPress, Keymap, Outcome, Shortcut,
};

pub fn sc(text: &str) -> Shortcut {
    Shortcut::parse(text).unwrap()
}

pub fn press(text: &str) -> KeyPress {
    let s = sc(text);
    KeyPress::new(s.modifiers(), s.key()).unwrap()
}

pub fn id(name: &str) -> CommandId {
    CommandId::new(name)
}

pub fn commands() -> Commands {
    Commands::new()
        .with(Command::new("file-save", "save").shortcut(sc("Primary+S")))
        .with(Command::new("edit-undo", "undo").shortcut(sc("Primary+Z")))
        .with(Command::new("edit-copy", "copy").shortcut(sc("Primary+C")))
        .with(Command::new("view.palette", "palette").shortcut(sc("Primary+Shift+P")))
        .with(Command::new("app.menu", "menu").shortcut(sc("Alt+F")))
        .with(Command::new("file-rename", "rename").shortcut(sc("F2")))
        .with(Command::new("edit-delete", "delete").shortcut(sc("Delete")))
        .with(Command::new("app.send", "send").shortcut(sc("Primary+Enter")))
        .with(
            Command::new("file-export", "export")
                .shortcut(sc("Primary+E"))
                .disabled("nothing-to-export"),
        )
        .with(
            Command::new("app.help", "help")
                .shortcut(sc("F1"))
                .shortcut(sc("Shift+F1")),
        )
}

pub fn run(keymap: &Keymap, text: &str, context: FocusContext) -> Outcome {
    keymap.dispatch(&press(text), context)
}
