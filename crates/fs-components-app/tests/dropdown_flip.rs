//! A dropdown menu stands inside the window wherever its trigger is, and its lines can be pressed: under the trigger when there is
//! room, above it when there is not, and held at the window's edge when there is room in neither direction. The trigger sits at the
//! bottom, top, left and right edge of windows of 800x500 and 640x400.
//!
//! Tests: layout and click path of Blinc's overlay stack, as the window drives it (see `menu_common`).

#[path = "menu_common/mod.rs"]
mod menu_common;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use blinc_layout::overlay_state::overlay_stack;
use fs_components_app::{AppWidget, Screen};
use menu_common::Harness;
use menu_common::anchored::Anchored;

struct Case {
    h: Harness,
    chosen: Arc<AtomicUsize>,
    window: (f32, f32),
}

fn case(window: (f32, f32), at: (f32, f32), lines: usize) -> Case {
    let at = Arc::new(Mutex::new(at));
    let chosen = Arc::new(AtomicUsize::new(0));
    let surface = Anchored {
        at,
        chosen: Arc::clone(&chosen),
        lines,
    };
    let app = AppWidget::new("Title", vec![Screen::new("a", "A", Arc::new(surface))]).unwrap();
    Case {
        h: Harness::new(app, window.0, window.1),
        chosen,
        window,
    }
}

/// The node of the open panel: the one with an id like `fs-dropdown-panel-<n>`.
fn panel(c: &Case) -> Option<(f32, f32, f32, f32)> {
    let registry = c.h.tree.element_registry();
    c.h.tree.iter_nodes().find_map(|(node, _)| {
        let id = registry.get_id(node)?;
        id.starts_with("fs-dropdown-panel-").then_some(())?;
        let b = c.h.tree.get_absolute_bounds(node)?;
        Some((b.x, b.y, b.width, b.height))
    })
}

fn close(c: &mut Case) {
    overlay_stack().lock().unwrap().close_all();
    c.h.settle();
}

/// Open the menu at `at` (in the surface) and check: whole panel in the window, last line pressable.
fn check(window: (f32, f32), at: (f32, f32), lines: usize) -> bool {
    let mut c = case(window, at, lines);
    c.h.click("anchor");
    let shown = panel(&c).unwrap_or_else(|| panic!("no panel at {at:?} in {window:?}"));
    let (ww, wh) = c.window;
    let anchor = c.h.bounds("anchor");
    assert!(
        shown.0 >= 0.0 && shown.1 >= 0.0 && shown.0 + shown.2 <= ww && shown.1 + shown.3 <= wh,
        "panel {shown:?} not inside {window:?} (anchor {anchor:?}, {lines} lines)"
    );
    let flipped = shown.1 < anchor.1;
    if lines > 8 {
        // too many lines for the window: the panel is held inside it and scrolls; the line at its end is not in view
        close(&mut c);
        return flipped;
    }
    c.h.press_at((shown.0 + shown.2 / 2.0, shown.1 + shown.3 - 14.0));
    c.h.frame();
    assert_eq!(
        c.chosen.load(Ordering::SeqCst),
        1,
        "last line not pressable: panel {shown:?}, anchor {anchor:?} in {window:?}"
    );
    close(&mut c);
    flipped
}

const WINDOWS: [(f32, f32); 2] = [(800.0, 500.0), (640.0, 400.0)];

#[test]
fn a_menu_at_the_bottom_edge_stands_inside_the_window_and_can_be_pressed() {
    for window in WINDOWS {
        // The surface is the window less title bar and status line: sweep the trigger down to its last pixel.
        let room = window.1 - 90.0;
        for below in [
            200.0, 160.0, 130.0, 110.0, 100.0, 96.0, 90.0, 60.0, 30.0, 0.0,
        ] {
            let flipped = check(window, (20.0, (room - 28.0 - below).max(0.0)), 5);
            // (the surface ends above a status line, so the room below the trigger is a little more than `below`)
            assert!(
                flipped || below > 30.0,
                "{below} px below, in {window:?}: stays below"
            );
            assert!(
                !flipped || below < 200.0,
                "{below} px below, in {window:?}: flips"
            );
        }
    }
}

#[test]
fn a_menu_at_the_top_left_and_right_edge_stands_inside_the_window() {
    for window in WINDOWS {
        let room = window.1 - 90.0;
        check(window, (0.0, 0.0), 5);
        check(window, (window.0 - 60.0, 0.0), 5);
        check(window, (window.0 - 60.0, room - 28.0), 5);
        check(window, (window.0 - 60.0, room / 2.0), 5);
    }
}

#[test]
fn a_menu_taller_than_the_window_is_held_inside_it() {
    for window in WINDOWS {
        for at in [(20.0, 0.0), (20.0, 150.0), (20.0, window.1 - 120.0)] {
            check(window, at, 30);
        }
    }
}
