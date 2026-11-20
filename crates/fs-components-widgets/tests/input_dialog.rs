//! The input dialog: the element tree, typing, Enter and Escape, the buttons, blanks, the picture.
//!
//! Tests: an in-process mouse and keyboard over the built dialog, a pixel test.

mod common;

use std::sync::{Arc, Mutex};

use blinc_layout::prelude::RenderTree;
use common::{Input, background, draw, ink, layout, serial};
use fs_components_widgets::dialogs::{InputDialog, Labels};

type Log = Arc<Mutex<Vec<String>>>;

fn dialog(initial: &str, log: &Log) -> InputDialog {
    let (got, left) = (Arc::clone(log), Arc::clone(log));
    InputDialog::new(&Labels::default(), "New column", "Name of the column")
        .id("id")
        .message("Shown at the top of the column.")
        .initial(initial)
        .on_input(move |t| got.lock().unwrap().push(format!("input {t}")))
        .on_cancel(move || left.lock().unwrap().push("cancel".into()))
}

fn open(d: &InputDialog) -> (RenderTree, Input) {
    let tree = layout(d.build(), 480.0, 260.0);
    d.focus();
    blinc_layout::widgets::text_input::process_pending_input_focus();
    (tree, Input::new())
}

fn type_in(tree: &mut RenderTree, text: &str) {
    for ch in text.chars() {
        tree.broadcast_text_input_event(ch, false, false, false, false);
    }
}

#[test]
fn the_tree_has_title_field_and_the_two_buttons_in_a_dialog() {
    let _g = serial();
    let log = Log::default();
    let d = dialog("", &log);
    let (tree, _) = open(&d);
    for id in ["id", "id-title", "id-body", "id-field", "id-ok", "id-cancel"] {
        assert!(tree.query_by_id(id).is_some(), "no {id}");
    }
    assert_eq!(
        common::attr(&tree, "id", "role").as_deref(),
        Some("dialog")
    );
}

#[test]
fn enter_hands_over_the_trimmed_line_and_escape_leaves() {
    let _g = serial();
    let log = Log::default();
    let d = dialog("Start", &log);
    let (mut tree, mut input) = open(&d);
    type_in(&mut tree, "  Doing ");
    input.key(&mut tree, 13);
    assert_eq!(*log.lock().unwrap(), ["input Doing"]);
    input.key(&mut tree, 27);
    assert_eq!(
        log.lock().unwrap().last().map(String::as_str),
        Some("cancel")
    );
}

#[test]
fn a_blank_line_is_no_answer_and_the_buttons_do_what_the_keys_do() {
    let _g = serial();
    let log = Log::default();
    let d = dialog("   ", &log);
    let (mut tree, mut input) = open(&d);
    input.key(&mut tree, 13);
    input.click(&mut tree, "id-ok");
    assert!(!d.confirm());
    assert!(log.lock().unwrap().is_empty(), "nothing was handed over");
    type_in(&mut tree, "Review");
    input.click(&mut tree, "id-ok");
    input.click(&mut tree, "id-cancel");
    assert_eq!(*log.lock().unwrap(), ["input Review", "cancel"]);
}

#[test]
fn the_picture_has_ink() {
    let _g = serial();
    let log = Log::default();
    let d = dialog("Backlog", &log);
    let img = draw(d.build(), 480, 220, "input-dialog.png");
    assert!(ink(&img, (0, 0, 480, 220), background(&img)) > 2500);
}
