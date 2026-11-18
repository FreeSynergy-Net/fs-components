//! The status line is a WAI-ARIA live region: role `status` for a note, `alert` for a problem (APG alert pattern), and it
//! stays a tab stop only when its text is cut. Through the real harness; no window is opened.
//!
//! Tests: attribute reads on the built tree, in both kinds and when the kind changes.

#[path = "menu_common/mod.rs"]
mod menu_common;

use fs_components_app::Status;
use menu_common::Harness;
use menu_common::fixture::harness;

const LINE: &str = "fs-app-status";

fn attr(h: &Harness, name: &str) -> Option<String> {
    let node = h.tree.query_by_id(LINE)?;
    h.tree.element_registry().attribute(node, name)
}

fn say(h: &mut Harness, status: Status) {
    h.app.set_status(Some(status));
    h.rebuild();
    h.frame();
}

#[test]
fn a_note_is_a_polite_status_and_a_problem_an_assertive_alert() {
    let (mut h, _, _) = harness(&["a"]);
    say(&mut h, Status::Info("Saved.".into()));
    assert_eq!(attr(&h, "role").as_deref(), Some("status"));
    assert_eq!(attr(&h, "aria-live").as_deref(), Some("polite"));
    say(&mut h, Status::Problem("Could not save.".into()));
    assert_eq!(attr(&h, "role").as_deref(), Some("alert"));
    assert_eq!(attr(&h, "aria-live").as_deref(), Some("assertive"));
}

#[test]
fn an_empty_line_is_present_and_still_a_status() {
    let (mut h, _, _) = harness(&["a"]);
    h.app.set_status(None);
    h.rebuild();
    h.frame();
    assert_eq!(attr(&h, "role").as_deref(), Some("status"));
}
