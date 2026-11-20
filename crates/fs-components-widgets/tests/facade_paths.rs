//! The facade `fs_components_widgets`: every public module and every re-export of today stays reachable
//! under its path, and `styles()` stays byte-equal (the order of the sheets is the cascade).
//!
//! The crate is cut into smaller crates; its paths are the contract of the consumers. This file is
//! the list: a path that goes away makes it fail to COMPILE.
//!
//! Pattern: none — a test file.
#![allow(unused_imports)]

// Every `pub mod` of lib.rs.
use fs_components_widgets::{
    agent_tree, avatar_group, badge, banner, breadcrumb, calendar_view, carousel, checklist, chip,
    column_table, craft, dialogs, disclosure, drafts, drop_zone, dropdown, feed, field_card,
    file_preview, file_tile, focus, grid, head_bar, help, icon, icon_theme, keys, landmark,
    lightbox, list_view, listbox, live, message_composer, message_list, meter, name_dialog,
    page_title, participant, person_face, person_picker, plan, popover, progress, range, roving,
    share_dialog, slash_menu, slider, sortable_list, source_error, splitter, states, table, tabs,
    task_card, text, text_view, toast, tool_call, toolbar, tooltip, transfer_list, tree, tree_view,
};

// Every `pub use` of lib.rs, name for name (renames included).
use fs_components_widgets::{
    AVATAR_GROUP_MAX_SHOWN, Accordion, Align, AvatarGroup, Block, Breadcrumb, CalendarItem,
    CalendarNames, CalendarPosition, CalendarView, Candidate, CandidateSource, CardAction, Choice,
    Column, ColumnLayout, ColumnWidth, Crumb, Day, Disclosure, Document, Element, Fact, FieldCard,
    FileStore, FilePreview, FileTile, Glyph, Handled, HeadBar, IconTheme, Inline, Item,
    KindLabels, Key, KeyEvent, Landmark, Lane, LayoutStore, LightBox, LinkHandler, ListItem,
    ListPosition, ListView, Mark, MarkKind, Media, Moment, NameDialog, NamePurpose, NoStore,
    PageTitle, Permission, Person, PersonFace, PersonPicker, PersonRef, PinnedKind, Presence,
    PreviewContent, Picked, Row, RowSource, SafeText, Section, Sender, SenderKind, ShareDialog,
    Size, SlashLabels, SortableContent, SortableList, Tab, Tabs, TaskHandler, TextView, TileGrid,
    TitleLabels, Tone, ToolCall, ToolCallView, ToolItem, Toolbar, Transfer, TransferList,
    TreeNode, TreeView, VecRows, ViewRange,
};

// The functions are reached through `let`, so a changed signature shows too.
#[test]
fn the_re_exported_functions_are_reachable() {
    use fs_components_widgets as w;
    let _ = (
        w::avatar_group_initials,
        w::badge_count,
        w::badge_dot,
        w::cell_box,
        w::count_tasks,
        w::default_base_dirs,
        w::export_theme,
        w::fallback_family,
        w::head_button,
        w::head_face,
        w::icon,
        w::icon_named,
        w::icon_named_stroked,
        w::icon_stroked,
        w::index_theme,
        w::missing_glyphs,
        w::open_external,
        w::picture_fits,
        w::pinned_icon,
        w::safe_text,
        w::set_icon_theme,
        w::sheet_edge,
        w::sheet_edge_by,
        w::styles,
    );
}

/// FNV-1a, 64 bit: enough to see one changed byte; no dependency.
fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3))
}

/// Golden: the style sheet of all widgets is byte-equal to the one before the split. The order of the
/// parts is the cascade; moving a module into another crate must not change a byte.
#[test]
fn styles_are_byte_equal_to_the_golden() {
    let css = fs_components_widgets::styles();
    assert_eq!((css.len(), fnv(css.as_bytes())), (GOLDEN_LEN, GOLDEN_FNV), "styles() changed");
}

/// Measured on main (v0.0.32, 62976de) before the split.
const GOLDEN_LEN: usize = 64274;
const GOLDEN_FNV: u64 = 17_314_955_352_328_549_746;
