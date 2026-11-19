//! `treegrid`: the rows that open and close: `aria-level`, `aria-setsize`, `aria-posinset`, `aria-expanded`, Right and Left on the first cell
//! (and only there), the open class, a plain grid that ignores child rows and is never opened by the keys, the cursor kept on a shown
//! row when a parent closes.
//!
//! Tests: attribute reads on the built element, an in-process keyboard, `frame` between the changes as a window does.

mod common;

use blinc_layout::prelude::RenderTree;
use common::{Input, attr, frame, layout_with, serial};
use fs_components_widgets::grid::{Column, Grid, Row, STYLES};

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

fn show(g: &Grid) -> RenderTree {
    layout_with(g.build(), 600.0, 400.0, STYLES)
}

fn has(tree: &RenderTree, id: &str, class: &str) -> bool {
    let node = tree.query_by_id(id).expect("the element");
    tree.element_registry().has_class(node, class)
}

fn family() -> Vec<Row> {
    vec![
        row("p1", 1).with_children(vec![
            row("c1", 11),
            row("c2", 12).with_children(vec![row("g1", 121)]),
        ]),
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
    assert_eq!(
        n("tg-row-g1", "aria-expanded"),
        None,
        "a leaf row has no aria-expanded"
    );
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
    assert!(
        g.is_open("p1"),
        "Right on the first cell of a closed parent opens it"
    );
    assert_eq!(g.cursor().unwrap().1, "name", "and the cursor stays");
    assert_eq!(
        attr(&tree, "tg-row-p1", "aria-expanded").as_deref(),
        Some("true")
    );
    assert!(tree.query_by_id("tg-row-c1").is_some());
    input.key(&mut tree, 39);
    assert_eq!(
        g.cursor().unwrap().1,
        "size",
        "Right on an open parent moves one cell"
    );
    input.key(&mut tree, 37);
    input.key(&mut tree, 37);
    frame(&mut tree, 600.0, 400.0);
    assert!(
        !g.is_open("p1"),
        "Left on the first cell of an open parent closes it"
    );
    assert!(tree.query_by_id("tg-row-c1").is_none());
    input.key(&mut tree, 40);
    input.key(&mut tree, 39);
    assert_eq!(
        g.cursor().unwrap(),
        ("p2".to_owned(), "size".to_owned()),
        "a leaf row: Right moves a cell"
    );
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
    assert_eq!(
        g.cursor().unwrap().0,
        "p1",
        "the cursor goes to the closed ancestor"
    );
    g.set_open("p2", true);
    g.set_open("nothing", true);
    assert!(!g.is_open("p2") && !g.is_open("nothing"));
}

#[test]
fn right_and_left_move_a_cell_off_the_first_column_even_on_a_parent_row() {
    let _g = serial();
    let g = tg();
    let mut tree = show(&g);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "tg"));
    input.key(&mut tree, 39);
    input.key(&mut tree, 37);
    assert!(!g.is_open("p1"), "Right opened it, Left closed it again");
    input.key(&mut tree, 39);
    assert!(g.is_open("p1"));
    input.key(&mut tree, 39);
    assert_eq!(g.cursor().unwrap().1, "size");
    input.key(&mut tree, 39);
    assert_eq!(g.cursor().unwrap().1, "kind");
    input.key(&mut tree, 37);
    input.key(&mut tree, 37);
    assert!(
        g.is_open("p1"),
        "Left on the second cell moves, it does not close the row"
    );
    assert_eq!(g.cursor().unwrap().1, "name");
    // from the first cell of a closed parent row, only the first column opens it
    g.set_open("p1", false);
    frame(&mut tree, 600.0, 400.0);
    input.key(&mut tree, 39);
    frame(&mut tree, 600.0, 400.0);
    assert!(g.is_open("p1"), "Right on the first cell opens");
    g.set_open("p1", false);
    input.key(&mut tree, 39);
    input.key(&mut tree, 37);
    assert!(
        !g.is_open("p1"),
        "moving to the second cell and back leaves it closed"
    );
}

#[test]
fn a_plain_grid_is_never_opened_by_the_keys_and_a_page_of_zero_is_one_row() {
    let _g = serial();
    let g = Grid::new(cols(), &family()).id("pg").page(0);
    let mut tree = show(&g);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "pg"));
    assert_eq!(attr(&tree, "pg-row-p1", "aria-expanded"), None);
    input.key(&mut tree, 39);
    assert_eq!(
        g.cursor().unwrap().1,
        "size",
        "Right moves a cell in a plain grid"
    );
    input.key(&mut tree, 37);
    input.key(&mut tree, 34);
    assert_eq!(g.cursor().unwrap().0, "p2", "a page of 0 is one row");
    assert!(!g.is_open("p1"));
}

#[test]
fn the_counts_of_a_treegrid_count_the_shown_rows_and_the_head() {
    let _g = serial();
    let g = Grid::treegrid(cols(), &family()).id("tc");
    let tree = show(&g);
    assert_eq!(
        attr(&tree, "tc-table", "aria-rowcount").as_deref(),
        Some("3"),
        "two shown rows and the head"
    );
    g.set_open("p1", true);
    let tree = show(&g);
    assert_eq!(
        attr(&tree, "tc-table", "aria-rowcount").as_deref(),
        Some("5"),
        "open rows count"
    );
}
