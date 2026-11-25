//! The contract of the views: the blocks of the catalog and `views()` say the same (A2 of ENTSCHEID-zeichnen). A block with `forms=`
//! and a `widget` has a view, except the ones listed here as not built; every widget of the catalog is either a view or listed as
//! having none (stated, not silently dropped); every view is the widget of a block.
#![cfg(feature = "views")]

use std::collections::BTreeSet;

use fs_components::views;
use fs_components_catalog::BlockCatalog;

const CONTRACT: &str = include_str!("../../../contract/contract.toml");
const FILES: [(&str, &str); 6] = [
    (
        "dialogs.toml",
        include_str!("../../../contract/blocks/dialogs.toml"),
    ),
    (
        "fields.toml",
        include_str!("../../../contract/blocks/fields.toml"),
    ),
    (
        "frames.toml",
        include_str!("../../../contract/blocks/frames.toml"),
    ),
    (
        "general.toml",
        include_str!("../../../contract/blocks/general.toml"),
    ),
    (
        "programs.toml",
        include_str!("../../../contract/blocks/programs.toml"),
    ),
    (
        "widgets.toml",
        include_str!("../../../contract/blocks/widgets.toml"),
    ),
];

/// Blocks with `forms=` and a `widget` whose component has no view yet (not built, said openly).
const FORM_BLOCKS_WITHOUT_VIEW: &[&str] =
    &["timeline", "file-tile", "calendar-grid", "calendar-view"];

/// Widgets of the catalog that draw no form of the contract yet, so no view is due (a value, a frame, a part, or a form not yet named).
const WIDGETS_WITHOUT_VIEW: &[&str] = &[
    "AgentTree",
    "AvatarGroup",
    "CalendarView",
    "FileTile",
    "FilePickButton",
    "FilePreview",
    "HeadBar",
    "Lightbox",
    "MessageComposer",
    "MessageList",
    "Participant",
    "PersonFace",
    "PersonPicker",
    "ShareDialog",
    "SortableList",
    "TaskCard",
    "TextView",
    "ToolCall",
    "TransferList",
    "column_table",
    "fs-components-calendar-grid",
    "fs-components-map",
    "fs-components-timeline",
];

fn catalog() -> BlockCatalog {
    BlockCatalog::read(CONTRACT, &FILES).expect("the contract files read")
}

fn view_widgets() -> BTreeSet<String> {
    views().iter().map(|v| v.widget().to_owned()).collect()
}

#[test]
fn a_widget_is_the_view_of_one_component_only() {
    assert_eq!(view_widgets().len(), views().len());
}

#[test]
fn every_block_with_forms_and_a_widget_has_a_view_except_the_ones_not_built() {
    let (c, have) = (catalog(), view_widgets());
    let missing: BTreeSet<String> = c
        .iter()
        .filter(|b| !b.forms.is_empty())
        .filter(|b| b.widget.as_ref().is_some_and(|w| !have.contains(w)))
        .map(|b| b.name.clone())
        .collect();
    let expected: BTreeSet<String> = FORM_BLOCKS_WITHOUT_VIEW
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    assert_eq!(
        missing, expected,
        "blocks that name a form and a widget but have no view"
    );
}

#[test]
fn every_widget_of_the_catalog_is_a_view_or_stated_as_having_none_and_every_view_has_a_block() {
    let (c, have) = (catalog(), view_widgets());
    let widgets: BTreeSet<String> = c.iter().filter_map(|b| b.widget.clone()).collect();
    let stated: BTreeSet<String> = WIDGETS_WITHOUT_VIEW
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    for w in &have {
        assert!(widgets.contains(w), "view {w} is the widget of no block");
        assert!(
            !stated.contains(w),
            "{w} has a view, drop it from the list of the ones without"
        );
    }
    let without: BTreeSet<String> = widgets.difference(&have).cloned().collect();
    assert_eq!(
        without, stated,
        "widgets of the catalog without a view must be listed, and only those"
    );
}

#[test]
fn every_view_block_names_a_form_to_be_fed_with() {
    let (c, have) = (catalog(), view_widgets());
    for b in c
        .iter()
        .filter(|b| b.widget.as_ref().is_some_and(|w| have.contains(w)))
    {
        assert!(
            !b.forms.is_empty(),
            "block {} has a view but names no form",
            b.name
        );
    }
}

/// E4: the block `code-editor` has its view (widget `CodeEditor`) and names the form it reads, `single-record` from 1.1.0 on (the
/// word exists in fs-interfaces only after the `forms.txt` appendix; a pin follows then).
#[test]
fn the_code_editor_block_has_its_view_and_names_the_form_it_reads() {
    let c = catalog();
    let block = c
        .iter()
        .find(|b| b.name == "code-editor")
        .expect("the block code-editor");
    assert_eq!(block.widget.as_deref(), Some("CodeEditor"));
    assert!(view_widgets().contains("CodeEditor"));
    assert!(
        block.forms.iter().any(|f| f.name() == "single-record"),
        "forms: {:?}",
        block.forms
    );
}
