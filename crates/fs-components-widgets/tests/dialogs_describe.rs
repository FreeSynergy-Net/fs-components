//! `aria-describedby` of the dialogs (APG dialog-modal: only where the content is a short message, not a list or a long text) and
//! the destructive look of the answer button of a `danger` alertdialog.
//!
//! Tests: attribute and class reads on the built dialogs.

mod common;

use common::{attr, layout, serial};
use fs_components_widgets::dialogs::{
    AlertDialog, ChoiceDialog, ChoiceOption, ConfirmDialog, Labels, LicenseDialog,
};

#[test]
fn a_short_message_is_described_a_list_and_a_long_text_are_not() {
    let _g = serial();
    let l = Labels::default();
    let confirm = ConfirmDialog::new(&l, "Remove?", "Sure.").id("cd");
    let tree = layout(confirm.build(), 480.0, 260.0);
    assert_eq!(
        attr(&tree, "cd", "aria-describedby").as_deref(),
        Some("cd-body")
    );
    let alert = AlertDialog::new(&l, "Lost", "Kept here.").id("ad");
    let tree = layout(alert.build(), 480.0, 260.0);
    assert_eq!(
        attr(&tree, "ad", "aria-describedby").as_deref(),
        Some("ad-body")
    );
    let choice = ChoiceDialog::new(&l, "Which?")
        .id("ch")
        .option(ChoiceOption::new("a", "A"))
        .option(ChoiceOption::new("b", "B"));
    let tree = layout(choice.build(), 480.0, 360.0);
    assert_eq!(attr(&tree, "ch", "role").as_deref(), Some("dialog"));
    assert_eq!(
        attr(&tree, "ch", "aria-describedby"),
        None,
        "a list is not described"
    );
    let licence =
        LicenseDialog::new(&l, "Licence", "MIT", "Permission is hereby granted.").id("ld");
    let tree = layout(licence.build(), 520.0, 420.0);
    assert_eq!(
        attr(&tree, "ld", "aria-describedby"),
        None,
        "a long text is not described"
    );
    assert_eq!(
        attr(&tree, "ld", "aria-labelledby").as_deref(),
        Some("ld-title")
    );
}

/// Whether the element with this id, or one of its parts, has the class.
fn has_class_within(tree: &blinc_layout::prelude::RenderTree, id: &str, class: &str) -> bool {
    let reg = tree.element_registry();
    let node = tree.query_by_id(id).expect("the element");
    reg.has_class(node, class)
        || reg
            .get_descendants(node)
            .into_iter()
            .any(|n| reg.has_class(n, class))
}

#[test]
fn the_answer_of_a_danger_alertdialog_is_the_destructive_button() {
    let _g = serial();
    let l = Labels::default();
    let danger = AlertDialog::new(&l, "Delete?", "No way back.")
        .id("ad")
        .with_cancel(None)
        .danger(true);
    let tree = layout(danger.build(), 480.0, 260.0);
    assert!(has_class_within(&tree, "ad-ok", "cn-button--destructive"));
    assert!(!has_class_within(
        &tree,
        "ad-cancel",
        "cn-button--destructive"
    ));
    let plain = AlertDialog::new(&l, "Lost", "Kept.").id("ad2");
    let tree = layout(plain.build(), 480.0, 260.0);
    assert!(!has_class_within(&tree, "ad2-ok", "cn-button--destructive"));
}
