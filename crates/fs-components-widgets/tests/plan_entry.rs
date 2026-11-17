//! The entry line gives text back (`Entry::set`) and has a place for the host's notice (`Entry::set_notice`): the v0.0.13
//! limit of Fix A ("a create that failed loses what was typed, and there is nowhere to say why").
//!
//! Tests: model tests over the handle, one build for the notice place. Mutants: `set` leaves the old text -> `set_puts_the_text_back`
//! red; `set` does not clear the selection/cursor -> `set_puts_the_text_back` (cursor assertion is the widget's, see `blinc_limits`);
//! the notice is not drawn -> `the_notice_stands_under_the_line` red. Not tested here: the order inside Blinc's submit callback (text, notice cleared, host called): there is no key injection in `common`; it is a Not-verified item in the CHANGELOG.
//! Pattern: none; tests of one handle (Stub: the host is a recording closure behind a Mutex).

mod common;

use std::sync::{Arc, Mutex};

use blinc_layout::div::div;
use common::{layout, serial, size_of};
use fs_components_widgets::plan::Entry;

#[test]
fn set_puts_the_text_back() {
    let _g = serial();
    let entry = Entry::new("Title");
    assert_eq!(entry.text(), "");
    entry.set("Reply to the landlord");
    assert_eq!(entry.text(), "Reply to the landlord");
    entry.set("");
    assert_eq!(entry.text(), "", "an empty text empties the line");
    entry.set("Umlaut: grüße");
    assert_eq!(entry.text(), "Umlaut: grüße");
}

#[test]
fn the_notice_stands_under_the_line() {
    let _g = serial();
    let entry = Entry::new("Title");
    let seen: Arc<Mutex<Vec<(String, bool)>>> = Arc::default();
    let sink = Arc::clone(&seen);
    let on_enter =
        Arc::new(move |t: &str, shift: bool| sink.lock().unwrap().push((t.to_owned(), shift)));
    let plain = layout(
        div()
            .w(300.0)
            .child(entry.build_with("E", on_enter.clone())),
        300.0,
        200.0,
    );
    assert!(size_of(&plain, "E").1 > 0.0);
    assert!(seen.lock().unwrap().is_empty());
    entry.set_notice(Some("Not in the filter - show"));
    assert_eq!(entry.notice().as_deref(), Some("Not in the filter - show"));
    let with = layout(
        div().w(300.0).child(entry.build_with("E", on_enter)),
        300.0,
        200.0,
    );
    let (_, notice_h) = size_of(&with, "E-notice");
    assert!(notice_h > 0.0, "the notice is drawn");
    assert!(
        size_of(&with, "E").1 > size_of(&plain, "E").1,
        "and the entry grew by it"
    );
    entry.set_notice(None);
    assert_eq!(entry.notice(), None);
}
