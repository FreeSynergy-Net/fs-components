//! The cards of the fields as the widget draws them: the symbol and the actions follow `PersonField::kind`, never the value.
//!
//! Tests: input-level tests over the real element (the actions of a card are elements with ids `<name>-field-<section>-<field>-action-<n>`).
//!
//! Mutants (each must turn a test red):
//! - the view guesses a mail address from an `@` in the value -> `a_text_field_with_an_at_sign_gets_no_mail_action`
//! - the kind said by the source is ignored by the view        -> `a_mail_field_gets_the_mail_action_and_copy`

#[path = "people_common/mod.rs"]
mod people_common;

use std::sync::Arc;

use fs_components_workbenches::people::{FieldKind, PeopleLabels, PeopleWidget};
use people_common::Book;
use people_common::harness::Harness;

fn harness(kind: FieldKind) -> Harness {
    let book = Book::with(&["ada"]);
    *book.mail_kind.lock().unwrap() = kind;
    let widget = PeopleWidget::new(Arc::new(book), PeopleLabels::default());
    Harness::new(widget, 900.0, 600.0)
}

#[test]
fn a_text_field_with_an_at_sign_gets_no_mail_action() {
    // The value is "ada@x.org", but the source did not say it is an address: only Copy.
    let h = harness(FieldKind::Text);
    assert!(h.has("field-0-0-action-0"), "Copy is there");
    assert!(!h.has("field-0-0-action-1"), "no mail action from a guess");
}

#[test]
fn a_mail_field_gets_the_mail_action_and_copy() {
    let h = harness(FieldKind::Mail);
    assert!(h.has("field-0-0-action-0") && h.has("field-0-0-action-1"));
    assert!(!h.has("field-0-0-action-2"));
}
