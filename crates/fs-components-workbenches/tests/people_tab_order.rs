//! Tab reaches every field and every button of the form, in reading order.
//!
//! Tests: the real tree of the workbench in edit mode, its tab stops counted like the window does.

#[path = "people_common/mod.rs"]
mod people_common;

use std::sync::Arc;

use fs_components_workbenches::people::{PeopleLabels, PeopleWidget};
use people_common::{Book, harness::Harness};

#[test]
fn the_form_has_a_tab_stop_for_each_field_and_each_button() {
    let book = Arc::new(Book::with(&["ada", "bob"]));
    let widget = PeopleWidget::new(book, PeopleLabels::default());
    let mut h = Harness::new(widget, 900.0, 600.0);
    h.click("edit");
    h.rebuild();
    let stops = h.tree.tab_stops();
    let reg = h.tree.element_registry();
    let owners: Vec<String> = stops
        .iter()
        .filter_map(|node| {
            std::iter::once(*node)
                .chain(h.tree.layout_tree.ancestors(*node))
                .find_map(|n| reg.get_id(n))
        })
        .collect();
    let name = h.widget.element_name();
    let wanted: Vec<String> = [
        "search",
        "ada",
        "bob",
        "field-name",
        "field-mail/1",
        "add-mail",
        "save",
        "cancel",
    ]
    .iter()
    .map(|what| format!("{name}-{what}"))
    .collect();
    // The bar of the list: choose, then the menu (the dropdown names its own trigger); no plus while editing.
    assert_eq!(owners[0], format!("{name}-select"));
    assert!(owners[1].starts_with("fs-dropdown-trigger"), "{}", owners[1]);
    assert_eq!(&owners[2..], &wanted[..], "the order Tab walks through the list and the form");
}
