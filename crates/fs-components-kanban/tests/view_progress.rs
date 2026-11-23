//! SW-2: the progress bar of a card (optional channel `progress` of `cards-in-columns@1.1.0`): drawn only with a value, its filled
//! part is the percentage of the bar, a value over 100 is cut to 100.
//!
//! Mutants (each makes the named test fail): the bar is drawn for every card -> `only_a_card_with_progress_has_a_bar`; the fill
//! ignores the value (always full) -> `the_fill_is_the_percentage`.
//!
//! Pattern: none; tests. Layout only, no graphics adapter.
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_kanban::*;
use fs_components_styles::StyleId;
use host::{H, W, board, host};
use ui::{layout, serial};

fn bar_width(progress: Option<u8>) -> Option<(f32, f32)> {
    let h = host(&Operation::ALL);
    h.cols.lock().unwrap()[0].cards[0].progress = progress;
    let tree = layout(board(&h, StyleId::Admin).build(W, H), W, H);
    let bar = tree.get_absolute_bounds(tree.query_by_id("kb-progress-a")?)?;
    let fill = tree.layout().children(tree.query_by_id("kb-progress-a")?)[0];
    let f = tree.get_absolute_bounds(fill)?;
    Some((bar.width, f.width))
}

#[test]
fn only_a_card_with_progress_has_a_bar() {
    let _g = serial();
    assert!(bar_width(None).is_none());
    assert!(bar_width(Some(0)).is_some());
}

#[test]
fn the_fill_is_the_percentage() {
    let _g = serial();
    let (full, fill) = bar_width(Some(25)).unwrap();
    assert!((fill - full / 4.0).abs() < 1.0, "{fill} of {full}");
    let (full, fill) = bar_width(Some(250)).unwrap();
    assert!((fill - full).abs() < 1.0, "cut to 100: {fill} of {full}");
}
