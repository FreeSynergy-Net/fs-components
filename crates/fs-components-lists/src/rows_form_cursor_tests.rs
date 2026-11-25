//! Tests of the grid view on `refresh`: the cursor the reader made stays where the cell is still there.
//!
//! Pattern: Test double – fixed rows stand in for the module; the contract is `Mounted::refresh` of fs-components-core.

use fs_components_core::form_view::FormView;
use fs_components_core::keys::Key;

use crate::grid::form_view::{View, keep_cursor};
use crate::grid::{Column, Grid, Row};
use crate::rows_form_tests::{ctx, input};

fn grid(keys: &[&str]) -> Grid {
    let rows: Vec<Row> = keys
        .iter()
        .map(|k| Row::new(*k, vec![(*k).to_owned()]))
        .collect();
    Grid::new(vec![Column::new("label", "Label")], &rows)
}

#[test]
fn a_refresh_keeps_the_cursor_on_its_row() {
    let mounted = View.mount(input(), ctx());
    assert!(mounted.key(Key::Down), "the cursor goes to the second row");
    mounted.refresh(input());
    assert!(
        !mounted.key(Key::Down),
        "still on the last row, not back at the top"
    );
    assert!(mounted.key(Key::Up), "and the row above is there");
}

#[test]
fn the_cursor_follows_its_row_when_rows_come_before_it() {
    let old = grid(&["a", "b"]);
    assert!(old.press(fs_components_core::keys::KeyEvent::new(Key::Down)));
    let kept = keep_cursor(&old, grid(&["x", "a", "b"]));
    assert_eq!(kept.cursor(), Some(("b".to_owned(), "label".to_owned())));
}

#[test]
fn a_row_that_is_gone_leaves_the_cursor_at_the_top() {
    let old = grid(&["a", "b"]);
    old.press(fs_components_core::keys::KeyEvent::new(Key::Down));
    let kept = keep_cursor(&old, grid(&["a"]));
    assert_eq!(kept.cursor(), Some(("a".to_owned(), "label".to_owned())));
}
