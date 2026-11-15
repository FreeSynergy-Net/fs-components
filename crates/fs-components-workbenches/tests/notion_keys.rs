//! Notion frontend, editing: the arrow keys and the selection go across the borders of blocks as on one continuous surface
//! (finding U3 of 2026-11-14): one key press goes from a block to the next (the blank line that separates two blocks is no
//! stop), Shift extends a selection over several blocks, and deleting such a selection joins what is left.
//!
//! Tests: event tests through the built widget, keys broadcast to the built tree like the window runner does.

mod common;

use std::sync::Arc;

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::prelude::RenderTree;
use common::{Editable, Fake};
use fs_components_workbenches::{Labels, Layout, ReaderWidget, WikiLabels};

const BACKSPACE: u32 = 8;
const UP: u32 = 38;
const DOWN: u32 = 40;

fn editing() -> (ReaderWidget, RenderTree) {
    let tmp = std::env::temp_dir().join(format!("fs-keys-{}.png", std::process::id()));
    let _ = fs_components_workbenches::snapshot(
        fs_components_workbenches::Picture::new(Arc::new(Fake), 200, 100),
        &tmp,
    );
    let _ = std::fs::remove_file(&tmp);
    blinc_theme::ThemeState::init_default();
    blinc_core::reactive::set_stateful_deps_notifier(|ids| {
        blinc_layout::check_stateful_deps(ids);
    });
    let source = Editable::with(&[("m.md", "# M\n\nalpha\n\nbeta\n\ngamma\n")]);
    source
        .heading_titles
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let widget = ReaderWidget::new(Arc::new(source), Labels::default(), None)
        .layout(Layout::Notion, WikiLabels::default());
    widget.with_reader(|r| {
        r.open("m.md");
        r.begin_edit();
    });
    let tree = RenderTree::from_element(&widget.build(1000.0, 600.0));
    (widget, tree)
}

fn key(tree: &mut RenderTree, code: u32, shift: bool) {
    tree.broadcast_key_event(KEY_DOWN, code, shift, false, false, false);
    tree.process_pending_subtree_rebuilds();
}

#[test]
fn one_arrow_press_goes_from_a_block_to_the_next_and_back() {
    let (widget, mut tree) = editing();
    widget.focus_block(0, 2);
    assert_eq!(widget.cursor_in_block(), Some((0, 2)));
    key(&mut tree, DOWN, false);
    assert_eq!(
        widget.cursor_in_block(),
        Some((1, 2)),
        "one Down: the next block, not the blank line between"
    );
    key(&mut tree, DOWN, false);
    assert_eq!(widget.cursor_in_block(), Some((2, 2)));
    key(&mut tree, UP, false);
    assert_eq!(widget.cursor_in_block(), Some((1, 2)), "one Up: back");
    key(&mut tree, UP, false);
    key(&mut tree, UP, false);
    assert_eq!(
        widget.cursor_in_block(),
        Some((0, 2)),
        "stays in the first block at the top"
    );
}

#[test]
fn a_selection_runs_over_blocks_and_deleting_it_joins_what_is_left() {
    let (widget, mut tree) = editing();
    widget.focus_block(0, 2);
    key(&mut tree, DOWN, true);
    let selected = widget.selected_text().expect("a selection");
    assert_eq!(selected, "pha\n\nbe", "from block alpha into block beta");
    key(&mut tree, BACKSPACE, false);
    assert_eq!(widget.block_texts(), ["alta", "gamma"]);
}
