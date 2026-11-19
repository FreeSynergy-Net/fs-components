//! The sidebar of the file window is a named navigation landmark (WAI-ARIA APG landmark regions; GNOME HIG: Sidebar).
//!
//! Tests: attribute reads on the built window.

#[path = "files_common/mod.rs"]
mod files_common;

use std::sync::Arc;

use files_common::Shelf;
use files_common::harness::Harness;
use fs_components_workbenches::file_manager::{FileLabels, FileManagerWidget};

#[test]
fn the_sidebar_is_a_navigation_named_places() {
    let w = FileManagerWidget::new(
        Arc::new(Shelf::new().folder("d", None)),
        FileLabels::default(),
    );
    let name = w.element_name();
    let h = Harness::new(w, 1300.0, 700.0);
    let node = h
        .tree
        .query_by_id(&format!("{name}-sidebar"))
        .expect("the sidebar");
    let read = |name: &str| h.tree.element_registry().attribute(node, name);
    assert_eq!(read("role").as_deref(), Some("navigation"));
    assert_eq!(
        read("aria-label").as_deref(),
        Some(FileLabels::default().chrome.places.as_str())
    );
}
