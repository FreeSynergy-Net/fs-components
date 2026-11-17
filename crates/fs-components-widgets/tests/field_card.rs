//! `field-card`: the card of one field, with its actions at the right.
//!
//! Tests: pure value tests (the words and the order of the actions, whether the label is drawn).
//!
//! Mutants (each must turn a test red):
//! - the actions come out in another order than given -> `the_actions_keep_their_order_and_their_words`
//! - an empty label is drawn as an empty line         -> `an_empty_label_shows_only_the_value`
//! - an action word is dropped (a symbol nobody can read aloud) -> `every_action_says_in_words_what_it_does`

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use fs_components_widgets::{CardAction, FieldCard, Glyph};

fn act() -> Arc<dyn Fn() + Send + Sync> {
    Arc::new(|| {})
}

#[test]
fn the_actions_keep_their_order_and_their_words() {
    let card = FieldCard::new(Glyph::Mail, "E-mail", "ada@example.org")
        .action(CardAction::new(Glyph::Share, "Write mail", act()))
        .action(CardAction::new(Glyph::Copy, "Copy", act()));
    assert_eq!(card.spoken(), ["Write mail", "Copy"]);
}

#[test]
fn an_empty_label_shows_only_the_value() {
    assert!(FieldCard::new(Glyph::Info, "Note", "x").shows_label());
    assert!(!FieldCard::new(Glyph::Info, "", "x").shows_label());
    assert!(!FieldCard::new(Glyph::Info, "  ", "x").shows_label());
}

#[test]
fn every_action_says_in_words_what_it_does() {
    let ran = Arc::new(AtomicU32::new(0));
    let seen = Arc::clone(&ran);
    let card = FieldCard::new(Glyph::Phone, "Phone", "+49 30 1234567").action(CardAction::new(
        Glyph::Copy,
        "Copy",
        Arc::new(move || {
            seen.fetch_add(1, Ordering::Relaxed);
        }),
    ));
    assert!(card.spoken().iter().all(|w| !w.trim().is_empty()));
    assert_eq!(
        ran.load(Ordering::Relaxed),
        0,
        "building a card runs nothing"
    );
}
