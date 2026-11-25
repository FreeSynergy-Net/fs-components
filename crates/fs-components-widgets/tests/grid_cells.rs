//! `grid` with cells that hold elements (P2 of the board-in-grid draft): the Strategy `CellView` (text stays the default), a row of ONE
//! cell over all columns (`Row::spanning`), `Row::min_height` (a theme token), rows of different heights MEASURED in the laid-out tree
//! (never a guessed number), and the cursor over a spanning row (the column aim is kept).
//!
//! Mutants (each turns the named test red): rows with a fixed height instead of the content's -> `a_row_is_as_high_as_its_highest_cell`;
//! the cells of a row not stretched -> `the_cells_of_a_row_are_equally_high`; `min_height` ignored -> `min_height_is_a_floor_from_the_theme`;
//! the spanning cell one column wide -> `a_spanning_row_has_one_cell_over_all_columns`; the column aim dropped -> `the_cursor_keeps_its_column_across_a_spanning_row`;
//! a `CellView` that answers `None` still replacing the text -> `without_a_strategy_every_cell_is_text`.
//!
//! Pattern: none (tests).

mod common;

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::RenderTree;
use blinc_theme::{SpacingToken, ThemeState};
use common::{attr, layout_with, serial, size_of};
use fs_components_widgets::grid::{CellAt, CellView, Column, Grid, Row, STYLES};
use fs_components_widgets::{Key, KeyEvent};

const W: f32 = 100.0;

/// A strategy that answers nothing.
struct Nothing;

impl CellView for Nothing {}

/// Boxes of fixed height in the column `a` (by row: `tall` 90, `short` 24) and a head of its own in column `b`.
struct Boxes;

impl CellView for Boxes {
    fn cell(&self, at: &CellAt<'_>) -> Option<Div> {
        let h = match (at.col, at.row) {
            ("a", "tall") => 90.0,
            ("a", "short") => 24.0,
            _ => return None,
        };
        Some(div().id(format!("box-{}", at.row)).h(h).w(40.0))
    }

    fn head(&self, col: &str) -> Option<Div> {
        (col == "b").then(|| div().id("own-head").h(10.0).w(10.0))
    }
}

fn cols() -> Vec<Column> {
    ["a", "b", "c"]
        .iter()
        .map(|k| Column::new(*k, k.to_uppercase()))
        .collect()
}

fn text_row(key: &str) -> Row {
    Row::new(key, vec!["x".into(), "y".into(), "z".into()])
}

fn grid(rows: &[Row]) -> Grid {
    Grid::new(cols(), rows)
        .id("gr")
        .column_width(W)
        .cells(Arc::new(Boxes))
}

fn show(g: &Grid) -> RenderTree {
    layout_with(g.build(), 600.0, 600.0, STYLES)
}

fn at(g: &Grid) -> String {
    let (r, c) = g.cursor().expect("a cursor");
    format!("{r}/{c}")
}

fn key(g: &Grid, k: Key) -> bool {
    g.press(KeyEvent::new(k))
}

fn rows() -> Vec<Row> {
    vec![text_row("tall"), text_row("short"), text_row("plain")]
}

#[test]
fn a_row_is_as_high_as_its_highest_cell() {
    let _g = serial();
    let tree = show(&grid(&rows()));
    let (tall, short, plain) = (
        size_of(&tree, "gr-row-tall").1,
        size_of(&tree, "gr-row-short").1,
        size_of(&tree, "gr-row-plain").1,
    );
    assert!(tall >= 90.0, "the tall row holds its 90 px box: {tall}");
    assert!(
        short < tall && plain < tall,
        "rows differ: {tall} {short} {plain}"
    );
    // the same padding and edge in both rows: the difference is exactly the difference of the boxes
    assert!(
        ((tall - short) - (90.0 - 24.0)).abs() < 0.5,
        "{tall} - {short} is not 66"
    );
    assert!(size_of(&tree, "box-tall").1 > size_of(&tree, "box-short").1);
}

#[test]
fn the_cells_of_a_row_are_equally_high() {
    let _g = serial();
    let tree = show(&grid(&rows()));
    let row = size_of(&tree, "gr-row-tall").1;
    for col in ["a", "b", "c"] {
        let h = size_of(&tree, &format!("gr-cell-tall-{col}")).1;
        assert!(
            (h - row).abs() < 0.5,
            "cell {col} is {h} high, the row {row}"
        );
    }
}

#[test]
fn min_height_is_a_floor_from_the_theme() {
    let _g = serial();
    let floor = ThemeState::get().spacing_value(SpacingToken::Space16);
    let g = grid(&[
        text_row("low"),
        text_row("floored").min_height(SpacingToken::Space16),
    ]);
    let tree = show(&g);
    let (low, floored) = (
        size_of(&tree, "gr-row-low").1,
        size_of(&tree, "gr-row-floored").1,
    );
    assert!(
        low < floor,
        "the plain row is lower than the floor: {low} {floor}"
    );
    assert!(
        (floored - floor).abs() < 0.5,
        "{floored} is not the floor {floor}"
    );
}

#[test]
fn a_spanning_row_has_one_cell_over_all_columns() {
    let _g = serial();
    let g = grid(&[Row::spanning("lane", "Anna"), text_row("r")]);
    let tree = show(&g);
    assert_eq!(
        attr(&tree, "gr-cell-lane-a", "role").as_deref(),
        Some("rowheader")
    );
    assert_eq!(
        attr(&tree, "gr-cell-lane-a", "aria-colspan").as_deref(),
        Some("3")
    );
    assert_eq!(size_of(&tree, "gr-cell-lane-a").0, 3.0 * W);
    assert!(
        tree.query_by_id("gr-cell-lane-b").is_none(),
        "no second cell"
    );
    assert_eq!(
        attr(&tree, "gr-cell-r-b", "aria-colspan"),
        None,
        "a plain cell has no span"
    );
    assert_eq!(
        attr(&tree, "gr-table", "aria-rowcount").as_deref(),
        Some("3"),
        "the head, the lane and the row"
    );
}

#[test]
fn the_cursor_keeps_its_column_across_a_spanning_row() {
    let _g = serial();
    let g = grid(&[
        text_row("r0"),
        Row::spanning("lane", "Anna"),
        text_row("r1"),
    ]);
    assert!(g.go_to("r0", "c"));
    assert!(key(&g, Key::Down));
    assert_eq!(at(&g), "lane/a", "the lane has the one cell");
    assert!(key(&g, Key::Down));
    assert_eq!(at(&g), "r1/c", "the aim was column c");
    assert!(key(&g, Key::Up) && key(&g, Key::Up));
    assert_eq!(at(&g), "r0/c");
    assert!(!g.go_to("lane", "c"), "the lane has no cell in column c");
    assert!(key(&g, Key::Left));
    assert!(key(&g, Key::Down) && key(&g, Key::Down));
    assert_eq!(at(&g), "r1/b", "going left changed the aim");
}

#[test]
fn a_spanning_parent_row_of_a_treegrid_opens_and_closes() {
    let _g = serial();
    let lane = Row::spanning("lane", "Anna").with_children(vec![text_row("cells")]);
    let g = Grid::treegrid(cols(), &[lane]).id("gr").column_width(W);
    let tree = show(&g);
    assert_eq!(
        attr(&tree, "gr-row-lane", "aria-expanded").as_deref(),
        Some("false")
    );
    assert!(tree.query_by_id("gr-row-cells").is_none());
    assert!(key(&g, Key::Right), "Right opens the lane");
    let tree = show(&g);
    assert_eq!(
        attr(&tree, "gr-row-lane", "aria-expanded").as_deref(),
        Some("true")
    );
    assert!(tree.query_by_id("gr-row-cells").is_some());
    assert!(
        !key(&g, Key::Right),
        "no cell to the right of a spanning cell"
    );
    assert!(key(&g, Key::Left), "Left closes it");
    assert!(!g.is_open("lane"));
}

#[test]
fn without_a_strategy_every_cell_is_text() {
    let _g = serial();
    let g = Grid::new(cols(), &rows()).id("gr").column_width(W);
    let tree = show(&g);
    assert!(tree.query_by_id("box-tall").is_none());
    // a strategy that answers nothing changes nothing either
    let g = Grid::new(cols(), &rows())
        .id("gr")
        .column_width(W)
        .cells(Arc::new(Nothing));
    let tree = show(&g);
    assert!(tree.query_by_id("box-tall").is_none());
    let plain = size_of(&tree, "gr-row-tall").1;
    assert!(plain < 90.0, "text rows are low: {plain}");
}

#[test]
fn a_head_cell_may_hold_an_element() {
    let _g = serial();
    let tree = show(&grid(&rows()));
    assert!(
        tree.query_by_id("own-head").is_some(),
        "column b has its own head"
    );
    assert_eq!(
        attr(&tree, "gr-headcell-b", "role").as_deref(),
        Some("columnheader"),
        "the head cell stays a column header"
    );
}
