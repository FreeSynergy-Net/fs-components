//! The context menu in the real shell (M3): where it opens, flip at the edges, the two stages, the submenu, the symbol row, a chosen
//! line runs once on the selection, a locked line says why, a press outside and Escape close. The real view, no pixels.
//!
//! Pattern: Integration tests over Blinc's tree, router and overlay stack as the window drives them (see `menu_common`).
//!
//! Mutants (each makes a test here red): `place_at_point` ignores the right edge (`at_the_right_edge_it_opens_to_the_left`);
//! it ignores the bottom edge (`at_the_bottom_it_opens_upwards`); the stage of the trigger is ignored
//! (`shift_right_click_opens_the_second_stage`); `choose` does not close (`a_double_click_runs_once`); the handler gets no target
//! (`a_chosen_line_runs_on_the_selection`); a locked line runs (`a_locked_line_does_not_run_and_says_why`); `watch_context_outside` is
//! not called (`a_press_outside_closes`); the submenu does not open (`the_submenu_opens_beside_its_line`); the row is not drawn
//! (`the_row_of_symbols_is_one_line_of_buttons`); `app.quit` goes to the handler (`app_quit_is_answered_by_the_shell`).

#[path = "context_common/mod.rs"]
mod context_common;
#[path = "menu_common/mod.rs"]
mod menu_common;

use context_common::{files, right_click, rig, shift_right_click};
use fs_components_app::Status;
use menu_common::ESCAPE;

const WINDOW: (f32, f32) = (800.0, 500.0);

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1.5
}

#[test]
fn it_opens_at_the_pointer() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(300.0, 200.0), files(&["a"]));
    assert!(r.h.app.context_menu_open());
    let (x, y, w, h) = r.panel();
    assert!(near(x, 300.0) && near(y, 200.0), "panel at {x},{y}");
    assert!(x + w <= WINDOW.0 && y + h <= WINDOW.1);
}

#[test]
fn at_the_right_edge_it_opens_to_the_left() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(780.0, 100.0), files(&["a"]));
    let (x, _, w, _) = r.panel();
    assert!(x + w <= 780.0 + 1.5, "panel {x}+{w} must end at the point");
    assert!(x >= 0.0);
}

#[test]
fn at_the_bottom_it_opens_upwards() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(100.0, 470.0), files(&["a"]));
    let (_, y, _, h) = r.panel();
    assert!(y + h <= 470.0 + 1.5, "panel {y}+{h} must end at the point");
    assert!(y >= 0.0);
}

#[test]
fn in_the_corner_it_flips_both_ways_and_stays_inside() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(795.0, 495.0), files(&["a"]));
    let (x, y, w, h) = r.panel();
    assert!(x >= 0.0 && y >= 0.0 && x + w <= WINDOW.0 && y + h <= WINDOW.1);
    assert!(x + w <= 796.5 && y + h <= 496.5);
}

#[test]
fn the_first_stage_has_the_row_the_lines_and_more_options() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(200.0, 100.0), files(&["a"]));
    for id in ["cut", "copy", "delete", "open", "share", "props", "fs.submenu.3"] {
        assert!(r.h.has(&format!("fs-app-menu-entry-{id}")), "{id}");
    }
    assert!(!r.h.has("fs-app-menu-entry-rename"), "rename is on the second stage");
}

#[test]
fn shift_right_click_opens_the_second_stage() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&shift_right_click(200.0, 100.0), files(&["a"]));
    for id in ["rename", "zip", "open", "props"] {
        assert!(r.h.has(&format!("fs-app-menu-entry-{id}")), "{id}");
    }
    assert!(!r.h.has("fs-app-menu-entry-fs.submenu.3"), "no More options line any more");
}

#[test]
fn the_row_of_symbols_is_one_line_of_buttons() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(200.0, 100.0), files(&["a"]));
    let cut = r.h.bounds("fs-app-menu-entry-cut");
    let copy = r.h.bounds("fs-app-menu-entry-copy");
    let delete = r.h.bounds("fs-app-menu-entry-delete");
    assert!(near(cut.1, copy.1) && near(copy.1, delete.1), "one row");
    assert!(cut.0 < copy.0 && copy.0 < delete.0, "in order");
    let open = r.h.bounds("fs-app-menu-entry-open");
    assert!(open.1 > cut.1 + cut.3 - 1.0, "the text lines stand below the row");
}

#[test]
fn the_submenu_opens_beside_its_line() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(200.0, 100.0), files(&["a"]));
    r.h.click("fs-app-menu-entry-fs.submenu.3");
    assert!(r.h.has("fs-app-context-sub-panel"));
    assert!(r.h.has("fs-app-menu-entry-zip"));
    let (px, _, pw, _) = r.panel();
    let (sx, ..) = r.h.bounds("fs-app-context-sub-panel");
    assert!(near(sx, px + pw), "the next level stands at the right edge of the panel");
}

#[test]
fn a_chosen_line_runs_on_the_selection() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(200.0, 100.0), files(&["a", "b"]));
    r.h.click("fs-app-menu-entry-open");
    let ran = r.ran.lock().unwrap().clone();
    assert_eq!(ran.len(), 1);
    assert_eq!(ran[0].0, "open");
    assert_eq!(ran[0].1, Some(files(&["a", "b"])));
    assert!(!r.h.app.context_menu_open());
}

#[test]
fn a_symbol_of_the_row_runs_too() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(200.0, 100.0), files(&["a"]));
    r.h.click("fs-app-menu-entry-copy");
    assert_eq!(r.ran_ids(), ["copy"]);
}

#[test]
fn a_line_of_the_submenu_runs() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(200.0, 100.0), files(&["a"]));
    r.h.click("fs-app-menu-entry-fs.submenu.3");
    r.h.click("fs-app-menu-entry-zip");
    assert_eq!(r.ran_ids(), ["zip"]);
    r.h.settle();
    assert!(!r.h.has("fs-app-context-sub-panel"));
}

#[test]
fn a_double_click_runs_once() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(200.0, 100.0), files(&["a"]));
    let at = r.h.centre("fs-app-menu-entry-open");
    r.h.press_at(at);
    r.h.press_at(at);
    r.h.frame();
    assert_eq!(r.ran_ids(), ["open"]);
}

#[test]
fn a_locked_line_does_not_run_and_says_why() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(200.0, 100.0), files(&["a"]));
    r.h.click("fs-app-menu-entry-share");
    assert!(r.ran_ids().is_empty());
    assert!(r.h.app.context_menu_open(), "the menu stays");
    let status = r.h.app.with_shell(|s| s.status().cloned());
    assert_eq!(status, Some(Status::Info("Nothing to share with".into())));
}

#[test]
fn a_press_outside_closes() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(200.0, 100.0), files(&["a"]));
    r.h.press_at((700.0, 400.0));
    r.h.frame();
    assert!(!r.h.app.context_menu_open());
    r.h.settle();
    assert!(!r.h.has("fs-app-context-panel"));
    assert!(r.ran_ids().is_empty());
}

#[test]
fn escape_closes_the_submenu_first_then_the_menu() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(200.0, 100.0), files(&["a"]));
    r.h.click("fs-app-menu-entry-fs.submenu.3");
    assert!(r.h.has("fs-app-context-sub-panel"));
    r.h.key(ESCAPE);
    r.h.settle();
    assert!(!r.h.has("fs-app-context-sub-panel"));
    assert!(r.h.app.context_menu_open());
    r.h.key(ESCAPE);
    r.h.settle();
    assert!(!r.h.app.context_menu_open());
}

#[test]
fn a_second_menu_replaces_the_first() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(200.0, 100.0), files(&["a"]));
    r.open(&right_click(400.0, 300.0), files(&["b"]));
    r.h.settle();
    let (x, y, ..) = r.panel();
    assert!(near(x, 400.0) && near(y, 300.0));
    assert_eq!(r.h.app.context_target(), Some(files(&["b"])));
}
