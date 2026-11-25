//! Enter and Space told apart: a window turns both into a click on the focused element and announces the key only on its
//! release; `bind_keys` reports `Key::Enter` or `Key::Space` then, never `Key::Activate`.
//!
//! Tests: the order of a window (navigation, then the broadcast of the release) over the task line; fixed data, no module.

#![allow(clippy::pedantic)]

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, layout_with, serial};
use fs_components_widgets::plan::{Events, STYLES, TaskLine};

type Log = Arc<Mutex<Vec<String>>>;

fn events(log: &Log) -> Events {
    let say = |tag: &'static str, log: &Log| {
        let log = Arc::clone(log);
        Arc::new(move |k: &str| log.lock().unwrap().push(format!("{tag} {k}")))
    };
    Events {
        tick: Some(say("toggle", log)),
        open: Some(say("open", log)),
        ..Events::default()
    }
}

#[test]
fn in_the_window_order_space_ticks_and_enter_opens() {
    let _g = serial();
    let log = Log::default();
    let line = TaskLine::new("t1", "Write the letter");
    let mut tree = layout_with(line.build("L", &events(&log)), 600.0, 80.0, STYLES);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "L-t1-head"));
    for code in [32, 13, 32] {
        input.key(&mut tree, code);
    }
    assert_eq!(
        *log.lock().unwrap(),
        ["toggle t1", "open t1", "toggle t1"],
        "one request per key, the right one for each"
    );
}

#[test]
fn a_mouse_click_on_the_line_never_arms_the_keyboard() {
    let _g = serial();
    let log = Log::default();
    let line = TaskLine::new("t1", "Write the letter");
    let mut tree = layout_with(line.build("L", &events(&log)), 600.0, 80.0, STYLES);
    let mut input = Input::new();
    input.click(&mut tree, "L-t1");
    input.key_release_broadcast(&mut tree, 32);
    assert_eq!(*log.lock().unwrap(), ["open t1"], "only the click opened");
}
