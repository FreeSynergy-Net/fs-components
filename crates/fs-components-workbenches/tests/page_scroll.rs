//! Findings U4 and U5 of the user test of 2026-11-14: the icon of the page was cut off at the top, as if the page had been
//! scrolled (a page that is shorter than the window cannot be scrolled; the wheel over it moved it and it stayed there).
//!
//! Tests: the real widget laid out, the wheel turned over the page like a window does it (router, hit test, scroll chain), the
//! place of the icon read before and after.

mod common;

use std::sync::Arc;

use blinc_layout::event_router::EventRouter;
use blinc_layout::prelude::RenderTree;
use common::{Editable, Fake};
use fs_components_workbenches::{Labels, Layout, Picture, ReaderWidget, WikiLabels, snapshot};

fn init() {
    let tmp = std::env::temp_dir().join(format!("fs-wheel-{}.png", std::process::id()));
    let _ = snapshot(Picture::new(Arc::new(Fake), 200, 100), &tmp);
    let _ = std::fs::remove_file(&tmp);
    blinc_theme::ThemeState::init_default();
    blinc_core::reactive::set_stateful_deps_notifier(|ids| {
        blinc_layout::check_stateful_deps(ids);
    });
}

/// The page is turned with the wheel over its middle, `turns` times; the vertical place of the icon after.
fn icon_after_wheel(layout: Layout, turns: u32) -> f32 {
    init();
    let source = Editable::with(&[("a.md", "# A\n\nShort page.\n")]);
    source
        .heading_titles
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let widget = ReaderWidget::new(Arc::new(source), Labels::default(), None)
        .layout(layout, WikiLabels::default());
    widget.with_reader(|r| r.open("a.md"));
    let mut tree = RenderTree::from_element(&widget.build(1200.0, 800.0));
    tree.compute_layout(1200.0, 800.0);
    let mut router = EventRouter::new();
    for _ in 0..turns {
        let (x, y) = (700.0, 400.0);
        let _ = router.on_mouse_move(&tree, x, y);
        if let Some(hit) = router.on_scroll_nested(&tree, 0.0, 120.0) {
            tree.dispatch_scroll_chain(hit.node, &hit.ancestors, x, y, 0.0, 120.0);
        }
        tree.tick_scroll_physics(1_000);
        let updates = tree.take_pending_property_updates();
        tree.apply_partial_property_updates(updates);
        tree.compute_layout(1200.0, 800.0);
    }
    let node = tree.query_by_id("fs-wk-page-icon").expect("the icon");
    tree.get_absolute_bounds(node).expect("bounds").y
}

#[test]
fn a_page_shorter_than_the_window_stays_where_it_is_when_the_wheel_is_turned() {
    let before = icon_after_wheel(Layout::Notion, 0);
    for turns in [1, 3, 8] {
        let after = icon_after_wheel(Layout::Notion, turns);
        assert!(
            (after - before).abs() < 0.5,
            "the icon moved from {before} to {after} after {turns} turns of the wheel"
        );
    }
}
