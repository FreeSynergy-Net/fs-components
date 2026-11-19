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
    vec![Column::new("name", "Name"), Column::new("size", "Size"), Column::new("kind", "Kind")]
}

fn row(key: &str, n: usize) -> Row {
    Row::new(key, vec![format!("{key} name"), format!("{n}"), "file".to_owned()])
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
    assert_eq!(attr(&tree, "gr", "tabindex").as_deref(), Some("0"), "one tab stop");
    assert_eq!(attr(&tree, "gr-head", "role").as_deref(), Some("row"));
    assert_eq!(attr(&tree, "gr-head", "aria-rowindex").as_deref(), Some("1"));
    assert_eq!(attr(&tree, "gr-row-r0", "role").as_deref(), Some("row"));
    assert_eq!(attr(&tree, "gr-row-r0", "aria-rowindex").as_deref(), Some("2"), "the head is row 1");
    assert_eq!(attr(&tree, "gr-row-r4", "aria-rowindex").as_deref(), Some("6"));
    assert_eq!(attr(&tree, "gr-cell-r1-size", "role").as_deref(), Some("gridcell"));
    assert_eq!(attr(&tree, "gr-cell-r1-size", "aria-colindex").as_deref(), Some("2"));
    assert_eq!(attr(&tree, "gr-row-r0", "aria-level"), None, "a plain grid has no levels");
    assert_eq!(attr(&tree, "gr-row-r0", "aria-expanded"), None);
}

#[test]
fn the_counts_count_the_head_as_a_row_and_the_head_cells_are_column_headers() {
    let _g = serial();
    let tree = show(&grid(&Log::default()));
    assert_eq!(attr(&tree, "gr-table", "aria-rowcount").as_deref(), Some("6"), "five rows and the head");
    assert_eq!(attr(&tree, "gr-table", "aria-colcount").as_deref(), Some("3"));
    assert_eq!(attr(&tree, "gr-headcell-size", "role").as_deref(), Some("columnheader"));
    assert_eq!(attr(&tree, "gr-headcell-size", "aria-colindex").as_deref(), Some("2"));
    let g = Grid::treegrid(cols(), &family()).id("tc");
    let tree = show(&g);
    assert_eq!(attr(&tree, "tc-table", "aria-rowcount").as_deref(), Some("3"), "two visible rows and the head");
    g.set_open("p1", true);
    let tree = show(&g);
    assert_eq!(attr(&tree, "tc-table", "aria-rowcount").as_deref(), Some("5"), "open rows count");
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
    assert_eq!(*log.lock().unwrap(), ["r0/name", "r0/size", "r0/size", "r0/size"], "once each; another key does nothing");
}

#[test]
fn a_click_moves_the_cursor_to_the_cell_and_activates_it_and_the_current_class_follows() {
    let _g = serial();
    let log = Log::default();
    let g = grid(&log);
    let mut tree = show(&g);
    assert!(has(&tree, "gr-cell-r0-name", "fs-grid-cell-current"), "the cursor starts on the first cell");
    assert!(!has(&tree, "gr-cell-r2-kind", "fs-grid-cell-current"));
    let mut input = Input::new();
    input.click(&mut tree, "gr-cell-r2-kind");
    frame(&mut tree, 600.0, 400.0);
    assert_eq!(*log.lock().unwrap(), ["r2/kind"]);
    assert!(has(&tree, "gr-cell-r2-kind", "fs-grid-cell-current"));
    assert!(!has(&tree, "gr-cell-r0-name", "fs-grid-cell-current"), "only one cell has the mark");
}

fn family() -> Vec<Row> {
    vec![
        row("p1", 1).with_children(vec![row("c1", 11), row("c2", 12).with_children(vec![row("g1", 121)])]),
        row("p2", 2),
    ]
}

fn tg() -> Grid {
    Grid::treegrid(cols(), &family()).id("tg").name("Tree")
}

#[test]
fn a_treegrid_tells_level_size_position_and_expanded_on_its_rows() {
    let _g = serial();
    let g = tg();
    g.set_open("p1", true);
    g.set_open("c2", true);
    let tree = show(&g);
    assert_eq!(attr(&tree, "tg", "role").as_deref(), Some("treegrid"));
    assert!(has(&tree, "tg", "fs-treegrid"));
    let n = |id: &str, a: &str| attr(&tree, id, a);
    assert_eq!(n("tg-row-p1", "aria-level").as_deref(), Some("1"));
    assert_eq!(n("tg-row-p1", "aria-setsize").as_deref(), Some("2"));
    assert_eq!(n("tg-row-p1", "aria-posinset").as_deref(), Some("1"));
    assert_eq!(n("tg-row-p1", "aria-expanded").as_deref(), Some("true"));
    assert_eq!(n("tg-row-c2", "aria-level").as_deref(), Some("2"));
    assert_eq!(n("tg-row-c2", "aria-posinset").as_deref(), Some("2"));
    assert_eq!(n("tg-row-g1", "aria-level").as_deref(), Some("3"));
    assert_eq!(n("tg-row-g1", "aria-expanded"), None, "a leaf row has no aria-expanded");
    assert_eq!(n("tg-row-p2", "aria-expanded"), None);
    assert!(has(&tree, "tg-row-p1", "fs-grid-row-open"));
    assert!(!has(&tree, "tg-row-p2", "fs-grid-row-open"));
}

#[test]
fn a_plain_grid_ignores_child_rows() {
    let _g = serial();
    let g = Grid::new(cols(), &family()).id("pg");
    let tree = show(&g);
    assert_eq!(attr(&tree, "pg", "role").as_deref(), Some("grid"));
    assert!(tree.query_by_id("pg-row-c1").is_none());
    assert!(!has(&tree, "pg", "fs-treegrid"));
}

#[test]
fn right_and_left_on_the_first_cell_open_and_close_a_parent_row() {
    let _g = serial();
    let g = tg();
    let mut tree = show(&g);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "tg"));
    input.key(&mut tree, 39);
    frame(&mut tree, 600.0, 400.0);
    assert!(g.is_open("p1"), "Right on the first cell of a closed parent opens it");
    assert_eq!(g.cursor().unwrap().1, "name", "and the cursor stays");
    assert_eq!(attr(&tree, "tg-row-p1", "aria-expanded").as_deref(), Some("true"));
    assert!(tree.query_by_id("tg-row-c1").is_some());
    input.key(&mut tree, 39);
    assert_eq!(g.cursor().unwrap().1, "size", "Right on an open parent moves one cell");
    input.key(&mut tree, 37);
    input.key(&mut tree, 37);
    frame(&mut tree, 600.0, 400.0);
    assert!(!g.is_open("p1"), "Left on the first cell of an open parent closes it");
    assert!(tree.query_by_id("tg-row-c1").is_none());
    input.key(&mut tree, 40);
    input.key(&mut tree, 39);
    assert_eq!(g.cursor().unwrap(), ("p2".to_owned(), "size".to_owned()), "a leaf row: Right moves a cell");
}

#[test]
fn closing_a_parent_keeps_the_cursor_on_a_shown_row() {
    let _g = serial();
    let g = tg();
    g.set_open("p1", true);
    g.set_open("c2", true);
    let mut tree = show(&g);
    let mut input = Input::new();
    input.click(&mut tree, "tg-cell-g1-name");
    assert_eq!(g.cursor().unwrap().0, "g1");
    g.set_open("p1", false);
    frame(&mut tree, 600.0, 400.0);
    assert_eq!(g.cursor().unwrap().0, "p1", "the cursor goes to the closed ancestor");
    g.set_open("p2", true);
    g.set_open("nothing", true);
    assert!(!g.is_open("p2") && !g.is_open("nothing"));
}
