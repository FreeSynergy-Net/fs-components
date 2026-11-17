//! A window with a context menu of files (M3): the real shell, a handler that records what ran and on what.
//!
//! Tests: Test fixture over the harness of `menu_common` (it must be declared by the test as `menu_common` too).

#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use blinc_layout::div::div;
use fs_components_app::commands::{
    Arrangement, Command, Commands, KeyNames, MORE_KEY, MenuItem, Platform, Point, Resolved,
    Shortcut, Target, TextKey, Trigger, resolve,
};
use fs_components_app::{AppWidget, Element, Menu, MenuEntry, ModelTexts, Screen, Surface};

use crate::menu_common::Harness;

struct Plain;

impl Surface for Plain {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        Box::new(div().class("t-plain").w(width).h(height))
    }
    fn styles(&self) -> String {
        ".t-plain { background: var(--surface); }".to_owned()
    }
}

/// What the handler saw: the id and the target the context menu named at that moment.
pub type Ran = Arc<Mutex<Vec<(String, Option<Target>)>>>;

pub const WORDS: &[(&str, &str)] = &[
    ("open", "Open"),
    ("cut", "Cut"),
    ("copy", "Copy"),
    ("delete", "Delete"),
    ("rename", "Rename"),
    ("share", "Share"),
    ("share-why", "Nothing to share with"),
    ("zip", "Compress"),
    ("props", "Properties"),
    ("shell-context-more", "More options"),
];

fn commands() -> Commands {
    let key = |t: &str| Shortcut::parse(t).unwrap();
    Commands::new()
        .with(
            Command::new("open", "open")
                .icon("folder")
                .shortcut(key("Enter")),
        )
        .with(Command::new("cut", "cut").icon("edit-cut"))
        .with(Command::new("copy", "copy").icon("edit-copy"))
        .with(Command::new("delete", "delete").icon("edit-delete"))
        .with(
            Command::new("rename", "rename")
                .icon("freesynergy-edit-rename")
                .shortcut(key("F2")),
        )
        .with(
            Command::new("share", "share")
                .icon("freesynergy-share")
                .disabled("share-why"),
        )
        .with(Command::new("zip", "zip").icon("package-x-generic"))
        .with(
            Command::new("props", "props")
                .icon("dialog-information")
                .shortcut(key("Alt+Enter")),
        )
}

/// The arrangement `context:file`: a row of symbols, two lines, "More options", a separator, one line.
pub fn items() -> Vec<Resolved> {
    let c = MenuItem::command;
    let tree = Arrangement::new(
        "context:file",
        vec![
            MenuItem::icon_row("quick", vec![c("cut"), c("copy"), c("delete")]),
            c("open"),
            c("share"),
            MenuItem::submenu(MORE_KEY, vec![c("rename"), c("zip")]),
            MenuItem::Separator,
            c("props"),
        ],
    );
    resolve(&tree, &commands()).unwrap()
}

pub fn say(key: &TextKey) -> String {
    WORDS
        .iter()
        .find(|(k, _)| *k == key.as_str())
        .map_or("?", |(_, text)| *text)
        .to_owned()
}

/// The target `ids` of files.
pub fn files(ids: &[&str]) -> Target {
    Target {
        kind: "file".into(),
        ids: ids.iter().map(|s| (*s).to_owned()).collect(),
    }
}

pub fn right_click(x: f32, y: f32) -> Trigger {
    Trigger::Pointer {
        at: Point { x, y },
        shift: false,
    }
}

pub fn shift_right_click(x: f32, y: f32) -> Trigger {
    Trigger::Pointer {
        at: Point { x, y },
        shift: true,
    }
}

/// A window of `w` x `h` and the record of what ran.
pub struct Rig {
    pub h: Harness,
    pub ran: Ran,
    pub quit: Arc<Mutex<u32>>,
}

pub fn rig(w: f32, h: f32) -> Rig {
    let ran: Ran = Arc::new(Mutex::new(Vec::new()));
    let quit = Arc::new(Mutex::new(0u32));
    let app = AppWidget::new("Title", vec![Screen::new("a", "A", Arc::new(Plain))]).unwrap();
    let me: Arc<Mutex<Option<AppWidget>>> = Arc::new(Mutex::new(None));
    let (record, reader) = (Arc::clone(&ran), Arc::clone(&me));
    app.set_menus(
        vec![Menu::new("Menu", vec![MenuEntry::new("noop", "Nothing")])],
        move |id| {
            let target = reader
                .lock()
                .unwrap()
                .as_ref()
                .and_then(AppWidget::context_target);
            record.lock().unwrap().push((id.to_owned(), target));
        },
    )
    .unwrap();
    let counter = Arc::clone(&quit);
    app.set_quit(move || *counter.lock().unwrap() += 1);
    *me.lock().unwrap() = Some(app.clone());
    Rig {
        h: Harness::new(app, w, h),
        ran,
        quit,
    }
}

impl Rig {
    pub fn open(&mut self, trigger: &Trigger, target: Target) {
        let names = KeyNames::english();
        let texts = ModelTexts {
            say: &say,
            platform: Platform::Linux,
            names: &names,
        };
        self.h
            .app
            .open_context_menu(trigger, target, &items(), &texts);
        self.h.frame();
    }

    pub fn panel(&self) -> (f32, f32, f32, f32) {
        self.h.bounds("fs-app-context-panel")
    }

    pub fn ran_ids(&self) -> Vec<String> {
        self.ran
            .lock()
            .unwrap()
            .iter()
            .map(|(id, _)| id.clone())
            .collect()
    }
}
