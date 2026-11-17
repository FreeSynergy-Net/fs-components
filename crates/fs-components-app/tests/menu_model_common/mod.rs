//! A File menu built from the command model, as the tests and the pictures of M1 use it.

#![allow(dead_code)]

use std::sync::Arc;

use blinc_layout::div::div;
use fs_components_app::commands::{
    Arrangement, Command, Commands, KeyNames, MenuItem, Platform, Shortcut, TextKey, resolve,
};
use fs_components_app::{AppWidget, Element, Menu, ModelTexts, Screen, Surface};

struct Plain;

impl Surface for Plain {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        Box::new(div().class("t-plain").w(width).h(height))
    }
    fn styles(&self) -> String {
        ".t-plain { background: var(--surface); }".to_owned()
    }
}

fn sc(text: &str) -> Shortcut {
    Shortcut::parse(text).unwrap()
}

/// The two languages of the example, key by key: (key, English, German).
const WORDS: &[(&str, &str, &str)] = &[
    ("file", "File", "Datei"),
    ("new", "New", "Neu"),
    ("open", "Open", "Öffnen"),
    ("save", "Save", "Speichern"),
    ("save-why", "Nothing to save", "Nichts zu speichern"),
    ("view-list", "As list", "Als Liste"),
    ("view-tree", "As tree", "Als Baum"),
    ("hidden", "Show hidden files", "Versteckte Dateien zeigen"),
    ("more", "More", "Weitere"),
    ("copy", "Copy", "Kopieren"),
    ("delete", "Delete", "Löschen"),
    ("delete-why", "Nothing selected", "Nichts ausgewählt"),
    ("quit", "Quit", "Beenden"),
];

pub fn commands() -> Commands {
    Commands::new()
        .with(
            Command::new("document-new", "new")
                .icon("document-new")
                .shortcut(sc("Primary+N")),
        )
        .with(Command::new("document-open", "open").shortcut(sc("Primary+O")))
        .with(
            Command::new("document-save", "save")
                .shortcut(sc("Primary+S"))
                .disabled("save-why"),
        )
        .with(Command::new("view-list", "view-list").radio("view", true))
        .with(Command::new("view-tree", "view-tree").radio("view", false))
        .with(
            Command::new("files.hidden", "hidden")
                .check(true)
                .shortcut(sc("Primary+Shift+H")),
        )
        .with(
            Command::new("edit-copy", "copy")
                .icon("edit-copy")
                .shortcut(sc("Primary+C")),
        )
        .with(
            Command::new("edit-delete", "delete")
                .icon("edit-delete")
                .shortcut(sc("Delete"))
                .disabled("delete-why"),
        )
        .with(Command::new("application-quit", "quit").shortcut(sc("Primary+Q")))
}

pub fn arrangement() -> Arrangement {
    let c = MenuItem::command;
    Arrangement::new(
        "menubar",
        vec![
            c("document-new"),
            c("document-open"),
            c("document-save"),
            MenuItem::Separator,
            MenuItem::group("view", vec![c("view-list"), c("view-tree")]),
            MenuItem::Separator,
            c("files.hidden"),
            MenuItem::Separator,
            MenuItem::submenu(
                "more",
                vec![c("edit-copy"), MenuItem::Separator, c("edit-delete")],
            ),
            c("application-quit"),
        ],
    )
}

/// The word of a key in `german` or English.
pub fn say(german: bool) -> impl Fn(&TextKey) -> String {
    move |key| {
        WORDS
            .iter()
            .find(|(k, ..)| *k == key.as_str())
            .map_or("?", |(_, en, de)| if german { *de } else { *en })
            .to_owned()
    }
}

/// The File menu on `platform`, in German or English.
pub fn file_menu(german: bool, platform: Platform) -> Menu {
    let say = say(german);
    let names = if german {
        KeyNames::german()
    } else {
        KeyNames::english()
    };
    let items = resolve(&arrangement(), &commands()).unwrap();
    let texts = ModelTexts {
        say: &say,
        platform,
        names: &names,
    };
    Menu::from_model(say(&TextKey::new("file")), &items, &texts)
}

/// An app with the File menu and a Help menu.
pub fn app(german: bool) -> AppWidget {
    let screens = vec![Screen::new("one", "First", Arc::new(Plain))];
    let app = AppWidget::new("Title", screens).unwrap();
    let help = Menu::new(
        "Help",
        vec![fs_components_app::MenuEntry::new("about", "About")],
    );
    app.set_menus(vec![file_menu(german, Platform::Linux), help], |_| {})
        .unwrap();
    app
}
