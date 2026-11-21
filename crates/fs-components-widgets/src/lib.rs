//! `fs-components-widgets`: widgets (layer 2) of FreeSynergy.
//!
//! A widget draws something whose data model is known when it is built. Here: [`text`] (the
//! neutral description of a document: blocks and marks), [`text_view`] (draws it) and
//! [`tree_view`] (draws a tree of labels). No widget knows what its data means, and no Blinc type
//! appears in a public signature except where a function must return an element to be placed in
//! a Blinc layout ([`Element`]).
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

pub mod agent_tree;
pub mod avatar_group;
pub mod badge;
pub mod banner;
pub mod breadcrumb;
pub mod calendar_view;
pub mod carousel;
pub mod checklist;
pub mod column_table;
pub mod craft;
pub mod disclosure;
pub use fs_components_core::{
    drafts, focus, icon, icon_theme, keys, landmark, live, roving, source_error, states,
};
pub(crate) use fs_components_core::{picture, softwrap};
pub use fs_components_lists::{
    chip, feed, grid, list_view, listbox, sortable_list, table, tree, tree_view,
};
pub use fs_components_measures::{meter, progress, range, slider};
pub use fs_components_overlays::{dialogs, name_dialog, popover, toast, tooltip};

pub mod drop_zone;
pub mod dropdown;
pub mod field_card;
pub mod file_preview;
pub mod file_tile;
#[cfg(feature = "test-access")]
pub use fs_components_primitives::test_access;
pub mod head_bar;
pub mod help;
pub mod lightbox;
pub mod message_composer;
pub mod message_list;
pub mod page_title;
pub mod participant;
pub mod person_face;
pub mod person_picker;
pub mod plan;
pub mod share_dialog;
pub mod slash_menu;
pub mod splitter;
mod styles;
pub mod tabs;
pub mod task_card;
pub mod text;
pub mod text_view;
pub mod tool_call;
pub mod toolbar;
pub mod transfer_list;

pub use avatar_group::{AvatarGroup, Person, Presence};
pub use avatar_group::{
    MAX_SHOWN as AVATAR_GROUP_MAX_SHOWN, Size, initials as avatar_group_initials,
};
pub use badge::{badge_count, badge_dot};
pub use breadcrumb::{Breadcrumb, Crumb};
pub use calendar_view::{
    CalendarItem, CalendarNames, CalendarPosition, CalendarView, Day, Moment, Picked, ViewRange,
};
pub use disclosure::{Accordion, Disclosure, Section};
pub use field_card::{CardAction, FieldCard};
pub use file_preview::{Fact, FilePreview, PreviewContent, SafeText, picture_fits, safe_text};
pub use file_tile::{FileTile, TileGrid};
pub use fs_components_core::icon::{Glyph, Tone, icon, icon_stroked};
pub use fs_components_core::icon_export::{export_theme, index_theme};
pub use fs_components_core::icon_kind::{PinnedKind, pinned_icon, sheet_edge, sheet_edge_by};
pub use fs_components_core::icon_named::{icon_named, icon_named_stroked, set_icon_theme};
pub use head_bar::{HeadBar, head_button, head_face};
pub use icon_theme::{IconTheme, default_base_dirs};
pub use keys::{Handled, Key, KeyEvent};
pub use landmark::Landmark;
pub use lightbox::{LightBox, Media};
pub use list_view::{
    Column, ColumnLayout, ColumnWidth, FileStore, LayoutStore, ListPosition, ListView, NoStore,
    Row, RowSource, VecRows,
};
pub use name_dialog::{NameDialog, Purpose as NamePurpose};
pub use page_title::{PageTitle, TitleLabels};
pub use participant::{KindLabels, Sender, SenderKind};
pub use person_face::PersonFace;
pub use person_picker::{Candidate, CandidateSource, PersonPicker, PersonRef};
pub use share_dialog::{Choice, Permission, ShareDialog};
pub use slash_menu::SlashLabels;
pub use sortable_list::{Content as SortableContent, Item, Lane, SortableList};
pub use styles::styles;
pub use tabs::{Tab, Tabs};
pub use text::{Align, Block, Document, Inline, ListItem, Mark, MarkKind, markdown_blocks};
pub use text_view::{
    LinkHandler, TaskHandler, TextView, cell_box, count_tasks, fallback_family, missing_glyphs,
    open_external,
};
pub use tool_call::{ToolCall, ToolCallView};
pub use toolbar::{Item as ToolItem, Toolbar};
pub use transfer_list::{Transfer, TransferList};
pub use tree_view::{TreeNode, TreeView};

pub use fs_components_core::Element;
