//! The style sheet of all widgets of this crate.
//!
//! Pattern: none — one pure function that joins the sheets of the single widgets.

use crate::{
    agent_tree, avatar_group, badge, banner, breadcrumb, calendar_view, column_table, dialogs,
    field_card, file_preview, file_tile, head_bar, lightbox, list_view, message_composer,
    message_list, name_dialog, participant, person_face, person_picker, plan, popover,
    share_dialog, sortable_list, states, task_card, text_view, toast, tool_call, tooltip,
    transfer_list, tree_view,
};

/// The CSS of all widgets in this crate; the shell adds it to the theme once.
pub fn styles() -> String {
    [
        fs_components_styles::styles::craft_sheet::SHEET,
        text_view::STYLES,
        tree_view::STYLES,
        list_view::STYLES,
        avatar_group::STYLES,
        badge::STYLES,
        person_face::STYLES,
        head_bar::STYLES,
        field_card::STYLES,
        fs_components_primitives::SEARCH_FIELD_STYLES,
        transfer_list::STYLES,
        sortable_list::STYLES,
        lightbox::STYLES,
        share_dialog::STYLES,
        person_picker::STYLES,
        name_dialog::STYLES,
        dialogs::STYLES,
        toast::STYLES,
        tooltip::STYLES,
        popover::STYLES,
        banner::STYLES,
        states::STYLES,
        tool_call::STYLES,
        participant::STYLES,
        message_list::STYLES,
        message_composer::STYLES,
        agent_tree::STYLES,
        calendar_view::STYLES,
        task_card::STYLES,
        plan::STYLES,
        column_table::STYLES,
        [breadcrumb::STYLES, file_tile::STYLES, file_preview::STYLES]
            .concat()
            .as_str(),
    ]
    .concat()
}
