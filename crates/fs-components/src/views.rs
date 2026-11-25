//! The views: the bridge from the name of a widget (`widget` of a block in the contract) to the code that draws it. One line per
//! component; the test `tests/views.rs` holds this list and the contract equal, so a host needs no list of its own.
//!
//! Pattern: Registry by an explicit function (no `inventory`, no linker section: a crate nobody names is not linked anyway).

use fs_components_core::form_view::FormView;

/// Every component that draws a form, one view each, in no special order.
#[must_use]
pub fn views() -> Vec<Box<dyn FormView>> {
    vec![
        Box::new(fs_components_kanban::View),
        Box::new(fs_components_workbenches::people::View),
        Box::new(fs_components_workbenches::code_editor::View),
        Box::new(fs_components_widgets::field_card::form_view::View),
        Box::new(fs_components_lists::tree_view::form_view::View),
        Box::new(fs_components_lists::table::form_view::View),
        Box::new(fs_components_lists::grid::form_view::View),
        Box::new(fs_components_calendar_grid::form_view::View),
        Box::new(fs_components_widgets::tabs::form_view::View),
        Box::new(fs_components_widgets::calendar_view::form_view::View),
        Box::new(fs_components_timeline::form_view::View),
    ]
}
