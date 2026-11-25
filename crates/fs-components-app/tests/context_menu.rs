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
//! (`the_row_of_symbols_is_one_line_of_buttons`); `app.quit` goes to the handler (`app_quit_is_answered_by_the_shell`);
//! a text line is back to 41 px (`a_line_is_32_px_high`); the symbol is not drawn before the label (`the_symbol_stands_left_of_the_label`);
//! the row is spread over the whole width again (`the_row_of_symbols_is_grouped_at_the_left`).

#[path = "context_common/mod.rs"]
mod context_common;
#[path = "menu_common/mod.rs"]
mod menu_common;

use context_common::{files, rig, right_click, shift_right_click};
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
    let (left, top, wide, high) = r.panel();
    assert!(
        near(left, 300.0) && near(top, 200.0),
        "panel at {left},{top}"
    );
    assert!(left + wide <= WINDOW.0 && top + high <= WINDOW.1);
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
    let (left, top, wide, high) = r.panel();
    assert!(left >= 0.0 && top >= 0.0 && left + wide <= WINDOW.0 && top + high <= WINDOW.1);
    assert!(left + wide <= 796.5 && top + high <= 496.5);
}

#[test]
fn the_first_stage_has_the_row_the_lines_and_more_options() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(200.0, 100.0), files(&["a"]));
    for id in [
        "cut",
        "copy",
        "delete",
        "open",
        "share",
        "props",
        "fs.submenu.3",
    ] {
        assert!(r.h.has(&format!("fs-app-menu-entry-{id}")), "{id}");
    }
    assert!(
        !r.h.has("fs-app-menu-entry-rename"),
        "rename is on the second stage"
    );
}

#[test]
fn shift_right_click_opens_the_second_stage() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&shift_right_click(200.0, 100.0), files(&["a"]));
    for id in ["rename", "zip", "open", "props"] {
        assert!(r.h.has(&format!("fs-app-menu-entry-{id}")), "{id}");
    }
    assert!(
        !r.h.has("fs-app-menu-entry-fs.submenu.3"),
        "no More options line any more"
    );
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
    assert!(
        open.1 > cut.1 + cut.3 - 1.0,
        "the text lines stand below the row"
    );
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
    assert!(
        near(sx, px + pw),
        "the next level stands at the right edge of the panel"
    );
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
    assert_eq!(r.ran_ids(), Vec::<String>::new());
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
    assert_eq!(r.ran_ids(), Vec::<String>::new());
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
    r.open(&right_click(400.0, 100.0), files(&["b"]));
    r.h.settle();
    let (x, y, ..) = r.panel();
    assert!(near(x, 400.0) && near(y, 100.0));
    assert_eq!(r.h.app.context_target(), Some(files(&["b"])));
}

/// The Explorer's line is about 32 px; the class paddings made it 41. A line with a reason is as high (the reason is a tooltip, no second line).
#[test]
fn a_line_is_32_px_high() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(200.0, 100.0), files(&["a"]));
    let open = r.h.bounds("fs-app-menu-entry-open");
    assert!(near(open.3, 32.0), "a text line is 32 px, was {}", open.3);
    let props = r.h.bounds("fs-app-menu-entry-props");
    assert!(near(props.3, 32.0), "the last line too, was {}", props.3);
}

/// Each line has its symbol in front of the label, in one column.
#[test]
fn the_symbol_stands_left_of_the_label() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(200.0, 100.0), files(&["a"]));
    let (px, _, _, _) = r.panel();
    let open_icon = r.h.bounds("fs-app-menu-icon-open");
    let props_icon = r.h.bounds("fs-app-menu-icon-props");
    assert!(open_icon.2 > 0.0, "the symbol cell is there");
    assert!(near(open_icon.0, props_icon.0), "one column");
    assert!(open_icon.0 < px + 24.0, "at the left edge of the panel");
}

/// The quick actions sit together at the left, with a small gap, not stretched to the panel's edges.
#[test]
fn the_row_of_symbols_is_grouped_at_the_left() {
    let mut r = rig(WINDOW.0, WINDOW.1);
    r.open(&right_click(200.0, 100.0), files(&["a"]));
    let (px, _, pw, _) = r.panel();
    let cut = r.h.bounds("fs-app-menu-entry-cut");
    let copy = r.h.bounds("fs-app-menu-entry-copy");
    let delete = r.h.bounds("fs-app-menu-entry-delete");
    assert!(cut.0 < px + 20.0, "the first button at the left edge");
    assert!(copy.0 - (cut.0 + cut.2) < 8.0, "a small gap");
    assert!(
        delete.0 + delete.2 < px + pw * 0.75,
        "the group does not reach the right half"
    );
}
