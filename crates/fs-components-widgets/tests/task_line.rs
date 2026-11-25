//! `task-row` (the plan `TaskLine`): the roles and state, the parts that are there only when given, the tick and the open by
//! pointer and by key.
//!
//! Tests: attribute reads on the built element, an in-process keyboard and pointer; fixed data, no module.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use common::{Input, attr, frame, layout_with, serial};
use fs_components_widgets::plan::{Chip, Events, STYLES, TaskLine, Tone};
use fs_components_widgets::{Key, KeyEvent};

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

fn full() -> TaskLine {
    let mut line = TaskLine::new("t1", "Write the letter");
    line.sub = Some("to the office".into());
    line.code = Some("T-1".into());
    line.facts = vec![
        Chip::new("High", Tone::Danger),
        Chip::new("tomorrow", Tone::Neutral),
    ];
    line.described = Some("Has a description".into());
    line
}

fn show(line: &TaskLine, ev: &Events) -> blinc_layout::prelude::RenderTree {
    layout_with(line.build("L", ev), 600.0, 80.0, STYLES)
}

#[test]
fn the_head_is_a_checkbox_with_its_state_and_parts() {
    let _g = serial();
    let ev = events(&Log::default());
    let tree = show(&full(), &ev);
    let id = "L-t1-head";
    assert_eq!(attr(&tree, id, "role").as_deref(), Some("checkbox"));
    assert_eq!(attr(&tree, id, "aria-checked").as_deref(), Some("false"));
    assert_eq!(
        attr(&tree, id, "aria-label").as_deref(),
        Some("Write the letter")
    );
    assert_eq!(
        attr(&tree, "t1-more", "aria-label").as_deref(),
        Some("Has a description")
    );
    let mut done = full();
    done.checked = true;
    let tree = show(&done, &ev);
    assert_eq!(attr(&tree, id, "aria-checked").as_deref(), Some("true"));
}

#[test]
fn parts_that_are_not_given_are_not_there() {
    let _g = serial();
    let tree = show(&TaskLine::new("t2", "Plain"), &Events::default());
    assert!(tree.query_by_id("L-t2-head").is_some());
    assert!(tree.query_by_id("t2-more").is_none());
}

#[test]
fn a_click_on_the_tick_ticks_and_a_click_on_the_title_opens() {
    let _g = serial();
    let log = Log::default();
    let mut tree = show(&full(), &events(&log));
    let mut input = Input::new();
    input.click(&mut tree, "L-t1-check");
    input.click(&mut tree, "L-t1");
    assert_eq!(*log.lock().unwrap(), ["toggle t1", "open t1"]);
}

#[test]
fn space_ticks_enter_opens_and_other_keys_do_nothing() {
    let _g = serial();
    let log = Log::default();
    let mut tree = show(&full(), &events(&log));
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "L-t1-head"));
    input.key_event_to(&mut tree, "L-t1-head", KEY_DOWN, 32);
    input.key_event_to(&mut tree, "L-t1-head", KEY_DOWN, 13);
    input.key_event_to(&mut tree, "L-t1-head", KEY_DOWN, 65);
    frame(&mut tree, 600.0, 80.0);
    assert_eq!(*log.lock().unwrap(), ["toggle t1", "open t1"]);
}

#[test]
fn a_line_without_handlers_takes_keys_without_harm() {
    let line = TaskLine::new("t3", "Quiet");
    let none = Events::default();
    assert!(line.press(KeyEvent::new(Key::Space), &none));
    assert!(!line.press(KeyEvent::new(Key::Up), &none));
}
