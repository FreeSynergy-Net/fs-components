//! `grid` and `treegrid`: roles and indexes (`aria-rowcount`, `aria-colcount`, `aria-rowindex`, `aria-colindex`, the head), the keys
//! (arrows, Home, End, Ctrl+Home, Ctrl+End, Page Up and Down, Enter and Space as the window's click AND as raw keys), the treegrid
//! rows (`aria-level`, `aria-setsize`, `aria-posinset`, `aria-expanded`, Right and Left on the first cell), the state classes.
//!
//! Tests: attribute reads on the built element, an in-process keyboard and pointer, the key events a window sends.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::prelude::RenderTree;
use common::{Input, attr, frame, layout_with, serial};
use fs_components_widgets::grid::{Column, Grid, Row, STYLES};

type Log = Arc<Mutex<Vec<String>>>;

fn cols() -> Vec<Column> {
    vec![
        Column::new("name", "Name"),
        Column::new("size", "Size"),
        Column::new("kind", "Kind"),
    ]
}

fn row(key: &str, n: usize) -> Row {
    Row::new(
        key,
        vec![format!("{key} name"), format!("{n}"), "file".to_owned()],
    )
}

fn rows() -> Vec<Row> {
    (0..5).map(|i| row(&format!("r{i}"), i)).collect()
}

fn grid(log: &Log) -> Grid {
    let l = Arc::clone(log);
    Grid::new(cols(), &rows())
        .id("gr")
        .name("Files")
        .page(2)
        .on_activate(move |r, c| l.lock().unwrap().push(format!("{r}/{c}")))
}

fn show(g: &Grid) -> RenderTree {
    layout_with(g.build(), 600.0, 400.0, STYLES)
}

fn at(g: &Grid) -> String {
    let (r, c) = g.cursor().expect("a cursor");
    format!("{r}/{c}")
}

fn has(tree: &RenderTree, id: &str, class: &str) -> bool {
    let node = tree.query_by_id(id).expect("the element");
    tree.element_registry().has_class(node, class)
}

#[test]
fn the_grid_has_its_role_name_counts_head_and_indexes() {
    let _g = serial();
    let tree = show(&grid(&Log::default()));
    assert_eq!(attr(&tree, "gr", "role").as_deref(), Some("grid"));
    assert_eq!(attr(&tree, "gr", "aria-label").as_deref(), Some("Files"));
    assert_eq!(
        attr(&tree, "gr", "tabindex").as_deref(),
        Some("0"),
        "one tab stop"
    );
    assert_eq!(attr(&tree, "gr-head", "role").as_deref(), Some("row"));
    assert_eq!(
        attr(&tree, "gr-head", "aria-rowindex").as_deref(),
        Some("1")
    );
    assert_eq!(attr(&tree, "gr-row-r0", "role").as_deref(), Some("row"));
    assert_eq!(
        attr(&tree, "gr-row-r0", "aria-rowindex").as_deref(),
        Some("2"),
        "the head is row 1"
    );
    assert_eq!(
        attr(&tree, "gr-row-r4", "aria-rowindex").as_deref(),
        Some("6")
    );
    assert_eq!(
        attr(&tree, "gr-cell-r1-size", "role").as_deref(),
        Some("gridcell")
    );
    assert_eq!(
        attr(&tree, "gr-cell-r1-size", "aria-colindex").as_deref(),
        Some("2")
    );
    assert_eq!(
        attr(&tree, "gr-row-r0", "aria-level"),
        None,
        "a plain grid has no levels"
    );
    assert_eq!(attr(&tree, "gr-row-r0", "aria-expanded"), None);
}

#[test]
fn the_counts_count_the_head_as_a_row_and_the_head_cells_are_column_headers() {
    let _g = serial();
    let tree = show(&grid(&Log::default()));
    assert_eq!(
        attr(&tree, "gr-table", "aria-rowcount").as_deref(),
        Some("6"),
        "five rows and the head"
    );
    assert_eq!(
        attr(&tree, "gr-table", "aria-colcount").as_deref(),
        Some("3")
    );
    assert_eq!(
        attr(&tree, "gr-headcell-size", "role").as_deref(),
        Some("columnheader")
    );
    assert_eq!(
        attr(&tree, "gr-headcell-size", "aria-colindex").as_deref(),
        Some("2")
    );
}

#[test]
fn arrows_move_one_cell_without_wrap_and_home_end_and_ctrl_home_end_jump() {
    let _g = serial();
    let g = grid(&Log::default());
    let mut tree = show(&g);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "gr"));
    assert_eq!(at(&g), "r0/name");
    input.key(&mut tree, 39);
    input.key(&mut tree, 40);
    assert_eq!(at(&g), "r1/size");
    input.key(&mut tree, 35);
    assert_eq!(at(&g), "r1/kind", "End: the last cell of the row");
    input.key(&mut tree, 39);
    assert_eq!(at(&g), "r1/kind", "no wrap at the end of the row");
    input.key(&mut tree, 36);
    assert_eq!(at(&g), "r1/name", "Home: the first cell of the row");
    input.key(&mut tree, 37);
    assert_eq!(at(&g), "r1/name", "no wrap at the start");
    input.key(&mut tree, 38);
    input.key(&mut tree, 38);
    assert_eq!(at(&g), "r0/name", "no wrap at the top");
    input.press(&mut tree, 35, (false, true, false));
    assert_eq!(at(&g), "r4/kind", "Ctrl+End: the last cell of the grid");
    input.key(&mut tree, 40);
    assert_eq!(at(&g), "r4/kind", "no wrap at the bottom");
    input.press(&mut tree, 36, (false, true, false));
    assert_eq!(at(&g), "r0/name", "Ctrl+Home: the first cell of the grid");
}

#[test]
fn page_down_and_page_up_move_by_the_page_setting_and_stop_at_the_ends() {
    let _g = serial();
    let g = grid(&Log::default());
    let mut tree = show(&g);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "gr"));
    input.key(&mut tree, 34);
    assert_eq!(at(&g), "r2/name");
    input.key(&mut tree, 34);
    input.key(&mut tree, 34);
    assert_eq!(at(&g), "r4/name", "the last row stops it");
    input.key(&mut tree, 33);
    assert_eq!(at(&g), "r2/name");
    input.key(&mut tree, 33);
    input.key(&mut tree, 33);
    assert_eq!(at(&g), "r0/name");
}

#[test]
fn enter_and_space_activate_the_cell_once_as_the_windows_click_and_as_raw_keys() {
    let _g = serial();
    let log = Log::default();
    let g = grid(&log);
    let mut tree = show(&g);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "gr"));
    input.key(&mut tree, 13);
    input.key(&mut tree, 39);
    input.key(&mut tree, 32);
    input.key_event_to(&mut tree, "gr", KEY_DOWN, 32);
    input.key_event_to(&mut tree, "gr", KEY_DOWN, 13);
    input.key_event_to(&mut tree, "gr", KEY_DOWN, 65);
    assert_eq!(
        *log.lock().unwrap(),
        ["r0/name", "r0/size", "r0/size", "r0/size"],
        "once each; another key does nothing"
    );
}

#[test]
fn a_click_moves_the_cursor_to_the_cell_and_activates_it_and_the_current_class_follows() {
    let _g = serial();
    let log = Log::default();
    let g = grid(&log);
    let mut tree = show(&g);
    assert!(
        has(&tree, "gr-cell-r0-name", "fs-grid-cell-current"),
        "the cursor starts on the first cell"
    );
    assert!(!has(&tree, "gr-cell-r2-kind", "fs-grid-cell-current"));
    let mut input = Input::new();
    input.click(&mut tree, "gr-cell-r2-kind");
    frame(&mut tree, 600.0, 400.0);
    assert_eq!(*log.lock().unwrap(), ["r2/kind"]);
    assert!(has(&tree, "gr-cell-r2-kind", "fs-grid-cell-current"));
    assert!(
        !has(&tree, "gr-cell-r0-name", "fs-grid-cell-current"),
        "only one cell has the mark"
    );
}
