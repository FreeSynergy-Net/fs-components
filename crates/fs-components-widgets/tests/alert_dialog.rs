//! `alertdialog`: role, name and description, the keys of the APG pattern (Enter, Escape, Tab), the first stop and the request
//! to the host for the keyboard.
//!
//! Tests: attribute reads on the built tree, an in-process keyboard through the window's own router.

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, attr, layout, serial};
use fs_components_widgets::dialogs::{AlertDialog, ConfirmDialog, Labels};

type Log = Arc<Mutex<Vec<String>>>;

fn say(log: &Log, what: &str) {
    log.lock().unwrap().push(what.to_owned());
}

fn seen(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

fn dialog(log: &Log) -> AlertDialog {
    let (a, b) = (Arc::clone(log), Arc::clone(log));
    AlertDialog::new(
        &Labels::default(),
        "Connection lost",
        "Your changes are kept here.",
    )
    .id("ad")
    .on_ok(move || say(&a, "ok"))
    .on_cancel(move || say(&b, "cancel"))
}

#[test]
fn it_is_an_alertdialog_with_a_name_and_a_description_and_the_others_stay_dialogs() {
    let _g = serial();
    let log = Log::default();
    let tree = layout(dialog(&log).build(), 480.0, 260.0);
    assert_eq!(attr(&tree, "ad", "role").as_deref(), Some("alertdialog"));
    assert_eq!(attr(&tree, "ad", "aria-modal").as_deref(), Some("true"));
    assert_eq!(
        attr(&tree, "ad", "aria-labelledby").as_deref(),
        Some("ad-title")
    );
    assert_eq!(
        attr(&tree, "ad", "aria-describedby").as_deref(),
        Some("ad-body")
    );
    assert!(tree.query_by_id("ad-title").is_some() && tree.query_by_id("ad-body").is_some());
    let confirm = ConfirmDialog::new(&Labels::default(), "Remove?", "Sure.").id("cd");
    let tree = layout(confirm.build(), 480.0, 260.0);
    assert_eq!(attr(&tree, "cd", "role").as_deref(), Some("dialog"));
}

#[test]
fn with_one_button_enter_and_escape_both_close_it() {
    let _g = serial();
    let log = Log::default();
    let mut tree = layout(dialog(&log).build(), 480.0, 260.0);
    let mut input = Input::new();
    input.key(&mut tree, 13);
    input.key(&mut tree, 27);
    assert_eq!(seen(&log), ["ok", "ok"]);
    assert!(tree.query_by_id("ad-cancel").is_none(), "one button only");
    input.click(&mut tree, "ad-ok");
    assert_eq!(seen(&log).len(), 3);
}

#[test]
fn with_a_cancel_button_escape_cancels_and_enter_alone_does_nothing() {
    let _g = serial();
    let log = Log::default();
    let d = dialog(&log).with_cancel(None).danger(true);
    let mut tree = layout(d.build(), 480.0, 260.0);
    let mut input = Input::new();
    input.key(&mut tree, 13);
    assert!(
        seen(&log).is_empty(),
        "Enter with no button focused did something"
    );
    input.key(&mut tree, 27);
    assert_eq!(seen(&log), ["cancel"]);
    input.click(&mut tree, "ad-ok");
    assert_eq!(seen(&log), ["cancel", "ok"]);
}

#[test]
fn the_keyboard_starts_on_the_least_destructive_button() {
    let _g = serial();
    let log = Log::default();
    let d = dialog(&log).with_cancel(Some("Keep"));
    assert_eq!(d.first_focus_id(), "ad-cancel");
    let mut tree = layout(d.build(), 480.0, 260.0);
    let mut input = Input::new();
    input.tab(&mut tree);
    assert!(
        input.has_keyboard(&tree, "ad-cancel"),
        "the first stop is Cancel"
    );
    input.key(&mut tree, 13);
    assert_eq!(seen(&log), ["cancel"], "Enter on Cancel cancels");
    let single = dialog(&log);
    assert_eq!(single.first_focus_id(), "ad-ok");
    let mut tree = layout(single.build(), 480.0, 260.0);
    let mut input = Input::new();
    input.tab(&mut tree);
    assert!(input.has_keyboard(&tree, "ad-ok"));
}

#[test]
fn the_dialog_asks_the_host_for_the_keyboard() {
    let asked: Log = Log::default();
    let sink = Arc::clone(&asked);
    let d = dialog(&Log::default())
        .with_cancel(None)
        .focus_to(move |id| say(&sink, id));
    d.focus_first();
    assert_eq!(seen(&asked), ["ad-cancel"]);
    dialog(&Log::default()).focus_first();
    assert_eq!(seen(&asked).len(), 1, "no host, no request");
}
