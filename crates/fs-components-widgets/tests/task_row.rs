//! `task-row`: the roles and state, the parts that are there only when given, the tick and the open by pointer and by key, a picture.
//!
//! Tests: attribute reads on the built element, an in-process keyboard and pointer; fixed data, no module.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use common::{Input, attr, frame, layout_with, serial};
use fs_components_widgets::task_row::{STYLES, TaskRow, Tone};

type Log = Arc<Mutex<Vec<String>>>;

fn full(log: &Log) -> TaskRow {
    let (a, b) = (Arc::clone(log), Arc::clone(log));
    TaskRow::new("t1", "Write the letter")
        .subline("to the office")
        .prio("High", Tone::Danger)
        .due("tomorrow")
        .described("Has a description")
        .on_toggle(move |k| a.lock().unwrap().push(format!("toggle {k}")))
        .on_open(move |k| b.lock().unwrap().push(format!("open {k}")))
}

fn show(row: &TaskRow) -> blinc_layout::prelude::RenderTree {
    layout_with(row.build(), 600.0, 80.0, STYLES)
}

fn seen(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

#[test]
fn the_row_is_a_checkbox_with_its_state_and_parts() {
    let _g = serial();
    let log = Log::default();
    let tree = show(&full(&log));
    let id = "task-row-t1";
    assert_eq!(attr(&tree, id, "role").as_deref(), Some("checkbox"));
    assert_eq!(attr(&tree, id, "aria-checked").as_deref(), Some("false"));
    assert_eq!(
        attr(&tree, id, "aria-label").as_deref(),
        Some("Write the letter")
    );
    for part in ["check", "prio", "due", "more"] {
        assert!(
            tree.query_by_id(&format!("{id}-{part}")).is_some(),
            "{part}"
        );
    }
    assert_eq!(
        attr(&tree, "task-row-t1-more", "aria-label").as_deref(),
        Some("Has a description")
    );
    let done = show(&full(&log).done());
    assert_eq!(attr(&done, id, "aria-checked").as_deref(), Some("true"));
}

#[test]
fn parts_that_are_not_given_are_not_there() {
    let _g = serial();
    let tree = show(&TaskRow::new("t2", "Plain").id("p"));
    assert!(tree.query_by_id("p").is_some());
    for part in ["prio", "due", "more"] {
        assert!(tree.query_by_id(&format!("p-{part}")).is_none(), "{part}");
    }
}

#[test]
fn a_click_on_the_tick_ticks_and_a_click_on_the_title_opens() {
    let _g = serial();
    let log = Log::default();
    let mut tree = show(&full(&log));
    let mut input = Input::new();
    input.click(&mut tree, "task-row-t1-check");
    input.click(&mut tree, "task-row-t1-due");
    assert_eq!(seen(&log), ["toggle t1", "open t1"]);
}

#[test]
fn space_ticks_enter_opens_and_other_keys_do_nothing() {
    let _g = serial();
    let log = Log::default();
    let mut tree = show(&full(&log));
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "task-row-t1"));
    input.key_event_to(&mut tree, "task-row-t1", KEY_DOWN, 32);
    input.key_event_to(&mut tree, "task-row-t1", KEY_DOWN, 13);
    input.key_event_to(&mut tree, "task-row-t1", KEY_DOWN, 65);
    frame(&mut tree, 600.0, 80.0);
    assert_eq!(seen(&log), ["toggle t1", "open t1"]);
}

#[test]
fn a_row_without_handlers_takes_keys_without_harm() {
    let _g = serial();
    let row = TaskRow::new("t3", "Quiet");
    assert!(row.press(fs_components_widgets::KeyEvent::new(
        fs_components_widgets::Key::Space
    )));
    assert!(!row.press(fs_components_widgets::KeyEvent::new(
        fs_components_widgets::Key::Up
    )));
}
