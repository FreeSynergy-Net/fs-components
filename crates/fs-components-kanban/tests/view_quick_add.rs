//! Tests of the quick-add row: the element tree, locked and unlocked, the keyboard, blanks and refusals.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod host;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_kanban::*;
use fs_components_styles::StyleId;
use host::{H, W, board, cards_in, host};
use ui::{Input, attr, frame, layout, serial};

const ENTER: u32 = 13;

#[test]
fn without_the_operation_the_row_is_one_locked_control_in_every_column_and_does_nothing() {
    let _g = serial();
    for ops in [vec![], vec![Operation::Move, Operation::AddColumn]] {
        let h = host(&ops);
        let b = board(&h, StyleId::Admin);
        let mut tree = layout(b.build(W, H), W, H);
        for col in ["todo", "doing", "done"] {
            let id = format!("kb-add-card-{col}");
            assert!(tree.query_by_id(&id).is_some(), "{id} stays visible");
            assert_eq!(attr(&tree, &id, "data-locked").as_deref(), Some("true"));
            assert!(tree.query_by_id(&format!("kb-quick-field-{col}")).is_none());
            assert!(tree.query_by_id(&format!("kb-quick-ok-{col}")).is_none());
        }
        let mut input = Input::new();
        input.click(&mut tree, "kb-add-card-todo");
        frame(&mut tree, W, H);
        assert!(h.log.lock().unwrap().is_empty());
        b.set_quick_text("todo", "Sneaky");
        assert!(!b.submit_quick("todo"), "the API refuses as well");
        assert!(!b.add_card("todo", "Sneaky"));
        assert_eq!(cards_in(&h, "todo"), ["a", "b"]);
        assert!(h.log.lock().unwrap().is_empty(), "the host is never asked");
        assert_eq!(b.snapshot().notice.as_deref(), Some("locked"));
    }
}

#[test]
fn with_the_operation_every_column_has_a_field_and_a_button_and_no_lock() {
    let _g = serial();
    let h = host(&[Operation::AddCard]);
    let b = board(&h, StyleId::Windows);
    let tree = layout(b.build(W, H), W, H);
    for col in ["todo", "doing", "done"] {
        for part in ["kb-quick-", "kb-quick-field-", "kb-quick-ok-"] {
            assert!(
                tree.query_by_id(&format!("{part}{col}")).is_some(),
                "{part}{col}"
            );
        }
        assert!(tree.query_by_id(&format!("kb-add-card-{col}")).is_none());
    }
    // the other controls stay locked: only the asked operation unlocked
    assert_eq!(
        attr(&tree, "kb-add-column", "data-locked").as_deref(),
        Some("true")
    );
}

#[test]
fn the_button_adds_the_trimmed_title_empties_the_field_and_the_card_shows() {
    let _g = serial();
    let h = host(&[Operation::AddCard]);
    let b = board(&h, StyleId::Apple);
    let mut tree = layout(b.build(W, H), W, H);
    let mut input = Input::new();
    b.set_quick_text("doing", "  Ship it ");
    input.click(&mut tree, "kb-quick-ok-doing");
    frame(&mut tree, W, H);
    assert_eq!(
        host::last(&h),
        "add_card doing Ship it",
        "trimmed, in the column"
    );
    assert_eq!(cards_in(&h, "doing"), ["new-0"]);
    assert_eq!(b.quick_text("doing"), "", "the field empties");
    assert!(tree.query_by_id("kb-card-new-0").is_some());
    assert!(tree.query_by_id("kb-notice").is_none());
}

#[test]
fn enter_in_the_field_adds_in_that_column_only_and_other_texts_stay() {
    let _g = serial();
    let h = host(&[Operation::AddCard]);
    let b = board(&h, StyleId::Admin);
    let mut tree = layout(b.build(W, H), W, H);
    let mut input = Input::new();
    b.set_quick_text("done", "Other");
    b.set_quick_text("todo", "Two");
    input.key(&mut tree, ENTER);
    assert!(h.log.lock().unwrap().is_empty(), "no field has the keyboard");
    b.focus_quick("todo");
    blinc_layout::widgets::text_input::process_pending_input_focus();
    input.key(&mut tree, ENTER);
    frame(&mut tree, W, H);
    assert_eq!(host::last(&h), "add_card todo Two");
    assert_eq!(
        h.log.lock().unwrap().len(),
        1,
        "Enter adds in the field that has the keyboard only"
    );
    assert_eq!(b.quick_text("todo"), "");
    assert_eq!(b.quick_text("done"), "Other", "a redraw keeps the other text");
}

#[test]
fn blanks_add_nothing_and_a_refusal_keeps_the_text_and_says_why() {
    let _g = serial();
    let h = host(&[Operation::AddCard]);
    let b = board(&h, StyleId::Windows);
    let mut tree = layout(b.build(W, H), W, H);
    let mut input = Input::new();
    b.set_quick_text("todo", "   ");
    input.click(&mut tree, "kb-quick-ok-todo");
    frame(&mut tree, W, H);
    assert!(h.log.lock().unwrap().is_empty(), "blanks are no title");
    b.set_quick_text("todo", "refuse");
    input.click(&mut tree, "kb-quick-ok-todo");
    frame(&mut tree, W, H);
    assert_eq!(cards_in(&h, "todo"), ["a", "b"]);
    assert_eq!(b.quick_text("todo"), "refuse", "the text stays");
    assert!(tree.query_by_id("kb-notice").is_some());
}

#[test]
fn a_click_in_the_row_does_not_put_a_held_card_down_on_the_column() {
    let _g = serial();
    let h = host(&[Operation::AddCard, Operation::Move]);
    let b = board(&h, StyleId::Admin);
    let mut tree = layout(b.build(W, H), W, H);
    let mut input = Input::new();
    assert_eq!(b.grab("a"), Step::Changed);
    frame(&mut tree, W, H);
    input.click(&mut tree, "kb-quick-field-doing");
    frame(&mut tree, W, H);
    assert_eq!(cards_in(&h, "doing"), Vec::<String>::new());
    assert!(b.snapshot().held.is_some(), "the card is still held");
}

#[test]
fn the_row_unlocks_by_itself_and_locks_again() {
    let _g = serial();
    let h = host(&[]);
    let b = board(&h, StyleId::Admin);
    let mut tree = layout(b.build(W, H), W, H);
    assert!(tree.query_by_id("kb-quick-field-todo").is_none());
    h.ops.lock().unwrap().push(Operation::AddCard);
    b.refresh();
    frame(&mut tree, W, H);
    assert!(tree.query_by_id("kb-quick-field-todo").is_some());
    assert!(tree.query_by_id("kb-add-card-todo").is_none());
    h.ops.lock().unwrap().clear();
    b.refresh();
    frame(&mut tree, W, H);
    assert!(tree.query_by_id("kb-add-card-todo").is_some());
    assert!(tree.query_by_id("kb-quick-field-todo").is_none());
}
