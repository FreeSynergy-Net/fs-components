//! What the shortcut tests have in common: a program with the File menu of the model and its shortcuts, and key helpers.
//!
//! Tests: Test fixture.

#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use super::menu_common::Harness;
use super::menu_model_common::{arrangement, commands, say};
use blinc_core::events::event_types::{KEY_DOWN, KEY_UP};
use fs_components_app::commands::{Command, KeyNames, Keymap, MenuItem, Platform, resolve};
use fs_components_app::{AppWidget, Menu, ModelTexts, Status};
use fs_components_words::Locale;

pub type Log = Arc<Mutex<Vec<String>>>;

pub const N: u32 = 78;
pub const S: u32 = 83;
pub const C: u32 = 67;
pub const DELETE: u32 = 127;

/// A program with the File menu of the model, its shortcuts installed from the same resolved items.
pub fn program(extra: Option<Command>) -> (Harness, Log) {
    let mut list = commands();
    let mut tree = arrangement();
    if let Some(command) = extra {
        tree.items.push(MenuItem::command(command.id.clone()));
        list = list.with(command);
    }
    let items = resolve(&tree, &list).unwrap();
    let say = say(false);
    let names = KeyNames::english();
    let texts = ModelTexts {
        say: &say,
        platform: Platform::Linux,
        names: &names,
    };
    let app = AppWidget::new("T", vec![]).unwrap();
    let log: Log = Log::default();
    let sink = Arc::clone(&log);
    app.set_menus(vec![Menu::from_model("File", &items, &texts)], move |id| {
        sink.lock().unwrap().push(id.to_owned());
    })
    .unwrap();
    app.set_shortcuts(
        Keymap::from_resolved(&items, Platform::Linux),
        say(false),
        Locale::english(),
    );
    (Harness::new(app, 800.0, 500.0), log)
}

/// What a window does for a pressed key: Blinc broadcasts `KEY_DOWN` to every handler.
pub fn key(h: &mut Harness, code: u32, ctrl: bool) {
    h.tree
        .broadcast_key_event(KEY_DOWN, code, false, ctrl, false, false);
}

pub fn log(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

pub fn status(h: &Harness) -> Option<Status> {
    h.app.with_shell(|s| s.status().cloned())
}

/// The window broadcasts the release of a key with its code too.
pub fn release(h: &mut Harness, code: u32, ctrl: bool) {
    h.tree
        .broadcast_key_event(KEY_UP, code, false, ctrl, false, false);
}
