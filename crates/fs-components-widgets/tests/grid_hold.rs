//! `grid` variant `cells-hold` (APG interactive cells): the table of who has which key (M3/M4 of the review), as a pure table and
//! through the grid: Enter goes into a body cell, Escape comes back, inside the cell the content has every other key (the grid
//! answers "not mine"), a spanning head is activated by Enter as before, and a grid without `hold_cells` is exactly as it was.
//!
//! Mutants (each turns the named test red): Enter enters in a grid that does not hold -> `without_hold_cells_enter_activates_as_before`;
//! Enter also activates when it enters -> `enter_goes_into_the_cell_and_does_not_activate`; arrows still move inside the cell ->
//! `inside_the_cell_the_content_has_every_key_but_escape`; Escape ignored inside -> `escape_comes_back_out`; a click elsewhere keeps the
//! keys inside -> `leaving_the_cell_by_the_pointer_or_the_content_gives_the_keys_back`; the head row entered -> `a_spanning_head_is_activated_not_entered`.
//!
//! Pattern: none (tests).

mod common;

use std::sync::{Arc, Mutex};

use blinc_layout::div::{Div, div};
use common::{Input, attr, frame, layout_with, serial};
use fs_components_widgets::grid::{CellAt, CellView, Column, Grid, Hold, Row, STYLES, hold};
use fs_components_widgets::{Key, KeyEvent};

type Log = Arc<Mutex<Vec<String>>>;

const KEYS: [Key; 11] = [
    Key::Up,
    Key::Down,
    Key::Left,
    Key::Right,
    Key::Home,
    Key::End,
    Key::PageUp,
    Key::PageDown,
    Key::Space,
    Key::Enter,
    Key::Escape,
];

#[test]
fn the_table_is_the_table() {
    for key in KEYS {
        // a grid that does not hold: always the normal rule
        for (inside, spanning) in [(false, false), (false, true), (true, false), (true, true)] {
            assert_eq!(hold(false, inside, spanning, key), Hold::Normal, "{key:?}");
        }
        // on the cursor: only Enter on a body cell is new
        let on_cursor = hold(true, false, false, key);
        assert_eq!(
            on_cursor,
            if key == Key::Enter {
                Hold::Enter
            } else {
                Hold::Normal
            },
            "{key:?}"
        );
        assert_eq!(
            hold(true, false, true, key),
            Hold::Normal,
            "a head: {key:?}"
        );
        // inside: Escape leaves, the rest is the content's
        let inside = hold(true, true, false, key);
        assert_eq!(
            inside,
            if key == Key::Escape {
                Hold::Leave
            } else {
                Hold::Content
            },
            "{key:?}"
        );
    }
}

struct Sees(Log);

impl CellView for Sees {
    fn cell(&self, at: &CellAt<'_>) -> Option<Div> {
        if at.inside {
            self.0
                .lock()
                .unwrap()
                .push(format!("{}/{}", at.row, at.col));
        }
        (at.col == "b").then(|| div().id(format!("in-{}", at.row)).h(20.0).w(20.0))
    }
}

fn grid(log: &Log, hold_it: bool) -> Grid {
    let rows = vec![
        Row::spanning("lane", "Anna"),
        Row::new("r0", vec!["a".into(), "b".into()]),
        Row::new("r1", vec!["c".into(), "d".into()]),
    ];
    let (e, l, a) = (Arc::clone(log), Arc::clone(log), Arc::clone(log));
    let g = Grid::new(vec![Column::new("a", "A"), Column::new("b", "B")], &rows)
        .id("gr")
        .cells(Arc::new(Sees(Arc::clone(log))))
        .on_enter(move |r, c| e.lock().unwrap().push(format!("enter {r}/{c}")))
        .on_leave(move |r, c| l.lock().unwrap().push(format!("leave {r}/{c}")))
        .on_activate(move |r, c| a.lock().unwrap().push(format!("activate {r}/{c}")));
    if hold_it { g.hold_cells() } else { g }
}

fn key(g: &Grid, k: Key) -> bool {
    g.press(KeyEvent::new(k))
}

fn taken(log: &Log) -> Vec<String> {
    std::mem::take(&mut *log.lock().unwrap())
        .into_iter()
        .filter(|l| l.contains(' '))
        .collect()
}

fn on_r0(g: &Grid) {
    assert!(g.go_to("r0", "a"));
}

#[test]
fn enter_goes_into_the_cell_and_does_not_activate() {
    let log = Log::default();
    let g = grid(&log, true);
    on_r0(&g);
    assert!(!g.inside());
    assert!(key(&g, Key::Enter));
    assert!(g.inside());
    assert_eq!(taken(&log), ["enter r0/a"], "entered once, activated never");
    assert!(!key(&g, Key::Space), "inside, Space is the content's");
}

#[test]
fn space_still_activates_on_the_cursor() {
    let log = Log::default();
    let g = grid(&log, true);
    on_r0(&g);
    assert!(key(&g, Key::Space));
    assert_eq!(taken(&log), ["activate r0/a"]);
    assert!(!g.inside());
}

#[test]
fn inside_the_cell_the_content_has_every_key_but_escape() {
    let log = Log::default();
    let g = grid(&log, true);
    on_r0(&g);
    key(&g, Key::Enter);
    taken(&log);
    for k in KEYS.into_iter().filter(|k| *k != Key::Escape) {
        assert!(!key(&g, k), "{k:?} is not the grid's inside a cell");
        assert_eq!(
            g.cursor(),
            Some(("r0".into(), "a".into())),
            "{k:?} did not move it"
        );
    }
    assert!(taken(&log).is_empty(), "nothing was activated or left");
    assert!(g.inside());
}

#[test]
fn escape_comes_back_out() {
    let log = Log::default();
    let g = grid(&log, true);
    on_r0(&g);
    assert!(
        !key(&g, Key::Escape),
        "on the cursor Escape is the window's"
    );
    key(&g, Key::Enter);
    assert!(key(&g, Key::Escape));
    assert!(!g.inside());
    assert_eq!(taken(&log), ["enter r0/a", "leave r0/a"]);
    assert!(key(&g, Key::Down), "the arrows are the grid's again");
    assert_eq!(g.cursor(), Some(("r1".into(), "a".into())));
}

#[test]
fn a_spanning_head_is_activated_not_entered() {
    let log = Log::default();
    let g = grid(&log, true);
    assert!(g.go_to("lane", "a"));
    assert!(key(&g, Key::Enter));
    assert!(!g.inside());
    assert_eq!(taken(&log), ["activate lane/a"]);
}

#[test]
fn without_hold_cells_enter_activates_as_before() {
    let log = Log::default();
    let g = grid(&log, false);
    on_r0(&g);
    assert!(key(&g, Key::Enter));
    assert!(!g.inside());
    assert!(!key(&g, Key::Escape));
    assert_eq!(taken(&log), ["activate r0/a"]);
}

#[test]
fn leaving_the_cell_by_the_pointer_or_the_content_gives_the_keys_back() {
    let _g = serial();
    let log = Log::default();
    let g = grid(&log, true);
    let mut tree = layout_with(g.build(), 400.0, 300.0, STYLES);
    on_r0(&g);
    key(&g, Key::Enter);
    taken(&log);
    // the content got Escape and says so
    g.leave_cell();
    assert!(!g.inside());
    assert_eq!(taken(&log), ["leave r0/a"]);
    g.leave_cell();
    assert!(taken(&log).is_empty(), "nothing to leave twice");
    // a click on another cell takes the keys out of the cell they were in
    key(&g, Key::Enter);
    taken(&log);
    Input::new().click(&mut tree, "gr-cell-r1-b");
    assert!(!g.inside());
    assert_eq!(g.cursor(), Some(("r1".into(), "b".into())));
}

#[test]
fn the_cell_the_keys_are_in_is_marked_and_told_to_its_strategy() {
    let _g = serial();
    let log = Log::default();
    let g = grid(&log, true);
    let mut tree = layout_with(g.build(), 400.0, 300.0, STYLES);
    let has = |tree: &blinc_layout::prelude::RenderTree, id: &str, class: &str| {
        let node = tree.query_by_id(id).expect("the element");
        tree.element_registry().has_class(node, class)
    };
    assert!(has(&tree, "gr", "fs-grid-cells-hold"), "the root says so");
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "gr"));
    input.key(&mut tree, 40); // Down: r0/a
    input.key(&mut tree, 39); // Right: r0/b, where the strategy draws an element
    input.key(&mut tree, 13);
    frame(&mut tree, 400.0, 300.0);
    assert!(g.inside());
    assert!(has(&tree, "gr-cell-r0-b", "fs-grid-cell-open"));
    assert_eq!(
        attr(&tree, "gr-cell-r0-b", "role").as_deref(),
        Some("gridcell")
    );
    let seen: Vec<String> = log.lock().unwrap().clone();
    assert_eq!(taken(&log), ["enter r0/b"], "one `enter`");
    assert!(
        seen.iter().any(|l| l == "r0/b"),
        "the strategy was told `inside`: {seen:?}"
    );
    assert!(
        !seen.iter().any(|l| l == "r1/b" || l == "r0/a"),
        "only the current cell: {seen:?}"
    );
    input.key(&mut tree, 27);
    frame(&mut tree, 400.0, 300.0);
    assert!(!g.inside());
    assert!(!has(&tree, "gr-cell-r0-b", "fs-grid-cell-open"));
}
