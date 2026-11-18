//! The context menu is a WAI-ARIA `menu` (APG menu pattern) with a tab stop; its lines are menu items. The real shell, no pixels.
//!
//! Tests: attribute reads on the open panel.

#[path = "context_common/mod.rs"]
mod context_common;
#[path = "menu_common/mod.rs"]
mod menu_common;

use context_common::{files, rig, right_click};

#[test]
fn the_open_context_menu_is_a_menu_and_a_tab_stop() {
    let mut r = rig(800.0, 500.0);
    r.open(&right_click(300.0, 200.0), files(&["a"]));
    let node =
        r.h.tree
            .query_by_id("fs-app-context-panel")
            .expect("the panel");
    let read = |name: &str| r.h.tree.element_registry().attribute(node, name);
    assert_eq!(read("role").as_deref(), Some("menu"));
    assert_eq!(read("tabindex").as_deref(), Some("0"));
}
