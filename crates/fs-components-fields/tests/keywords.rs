//! The type `keywords`: the rule of the contract crate through the trait (canonical, sorted, lower case, limits), and the input: a comma
//! or Enter makes a chip, Backspace on the empty line removes the last, the chip's button and Delete remove, a word that is too long
//! is refused with a sentence, the stored text is canonical.
//!
//! Tests: model tests over the trait; the input built offscreen and typed into with the window's own router.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use std::sync::{Arc, Mutex};

use common::{Input, attr, frame, layout, serial};
use fs_components_fields::{KeysAsText, KeywordsInput, Kind, Look, Problem};
use fs_components_styles::StyleId;

fn input() -> KeywordsInput {
    KeywordsInput::named(Look::new(StyleId::Admin, false), Arc::new(KeysAsText), "kw")
}

fn type_into(
    tree: &mut blinc_layout::prelude::RenderTree,
    input: &mut Input,
    id: &str,
    text: &str,
) {
    input.click(tree, id);
    for ch in text.chars() {
        tree.broadcast_text_input_event(ch, false, false, false, false);
        frame(tree, 600.0, 300.0);
    }
}

#[test]
fn the_trait_stores_the_canonical_set_and_judges_it() {
    let f = Kind::Keywords.field();
    assert_eq!(f.key(), "keywords");
    assert_eq!(f.segments().len(), 1);
    assert_eq!(Kind::Keywords.block(), "field-keywords");
    assert_eq!(Kind::from_key("keywords"), Some(Kind::Keywords));
    assert_eq!(
        f.join(&[" Work,family ,WORK".to_owned()]),
        "family, work",
        "lower case, no duplicate, sorted"
    );
    assert_eq!(f.compact(&["b, a".to_owned()]), "a, b");
    assert_eq!(f.split("a, b"), ["a, b"]);
    assert_eq!(
        f.split(" a, b "),
        [" a, b "],
        "the typed list is kept as typed; join makes it canonical"
    );
    assert_eq!(f.problem(&["a, b".to_owned()]), None);
    assert_eq!(
        f.problem(&["  ".to_owned()]),
        None,
        "nothing typed is no problem"
    );
    assert_eq!(f.problem(&["x".repeat(65)]), Some(Problem::BadKeyword));
    let many: Vec<String> = (0..257).map(|i| format!("w{i}")).collect();
    assert_eq!(f.problem(&[many.join(",")]), Some(Problem::TooMany));
    let enough: Vec<String> = (0..256).map(|i| format!("w{i}")).collect();
    assert_eq!(
        f.problem(&[enough.join(",")]),
        None,
        "256 words are the most, and fine"
    );
    assert_eq!(
        f.problem(&["x".repeat(64)]),
        None,
        "64 characters are the longest, and fine"
    );
    assert_eq!(
        f.join(&["bad ,".to_owned() + &"y".repeat(65)]),
        ("bad ,".to_owned() + &"y".repeat(65)).trim().to_owned(),
        "a list that is not a set is kept as typed"
    );
    assert_eq!(Problem::BadKeyword.key(), "problem.keywords.bad-word");
    assert_eq!(Problem::TooMany.key(), "problem.keywords.too-many");
}

#[test]
fn words_are_made_like_the_stored_ones_in_the_order_they_came() {
    let _g = serial();
    let log = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&log);
    let k = input().on_change(move |t| sink.lock().unwrap().push(t.to_owned()));
    assert!(k.add_word("  Work ", true));
    assert!(k.add_word("family", true));
    assert!(
        !k.add_word("WORK", true),
        "a word that is there is not added twice"
    );
    assert!(!k.add_word("   ", true), "a blank is no word");
    assert_eq!(
        k.words(),
        ["work", "family"],
        "lower case at once, in the order they came"
    );
    assert_eq!(
        k.text(),
        "family, work",
        "the stored text is the canonical one"
    );
    assert_eq!(
        *log.lock().unwrap(),
        ["work", "family, work"],
        "each change is heard once"
    );
    assert_eq!(k.problem(), None);
}

#[test]
fn a_word_that_is_too_long_is_refused_with_a_sentence_and_a_good_word_clears_it() {
    let _g = serial();
    let k = input();
    let mut tree = layout(k.build(), 600.0, 300.0);
    assert!(!k.add_word(&"x".repeat(65), true));
    assert_eq!(k.problem(), Some(Problem::BadKeyword));
    frame(&mut tree, 600.0, 300.0);
    assert!(
        tree.query_by_id("kw-problem").is_some(),
        "the sentence is drawn"
    );
    assert!(k.words().is_empty());
    assert!(k.add_word("fine", true));
    frame(&mut tree, 600.0, 300.0);
    assert_eq!(k.problem(), None);
    assert!(tree.query_by_id("kw-problem").is_none());
    let capped = input();
    for i in 0..256 {
        assert!(capped.add_word(&format!("w{i}"), false));
    }
    assert!(!capped.add_word("one-more", true));
    assert_eq!(capped.problem(), Some(Problem::TooMany));
    assert_eq!(capped.words().len(), 256);
}

#[test]
fn a_comma_typed_makes_a_chip_and_leaves_the_rest_on_the_line() {
    let _g = serial();
    let k = input();
    let mut tree = layout(k.build(), 600.0, 300.0);
    let mut input = Input::new();
    type_into(&mut tree, &mut input, "kw", "Work,fam");
    assert_eq!(
        k.words(),
        ["work"],
        "the word before the comma became a chip"
    );
    assert!(tree.query_by_id("kw-chips-chip-work").is_some());
    type_into(&mut tree, &mut input, "kw", "ily,");
    assert_eq!(k.words(), ["work", "family"]);
    assert_eq!(k.text(), "family, work");
}

#[test]
fn enter_makes_a_chip_of_the_line_and_backspace_on_the_empty_line_removes_the_last() {
    let _g = serial();
    let k = input();
    let mut tree = layout(k.build(), 600.0, 300.0);
    let mut input = Input::new();
    type_into(&mut tree, &mut input, "kw", "alpha");
    input.key(&mut tree, 13);
    frame(&mut tree, 600.0, 300.0);
    assert_eq!(k.words(), ["alpha"], "Enter commits the typed word");
    type_into(&mut tree, &mut input, "kw", "beta");
    input.key(&mut tree, 13);
    assert_eq!(k.words(), ["alpha", "beta"]);
    input.key(&mut tree, 8);
    frame(&mut tree, 600.0, 300.0);
    assert_eq!(
        k.words(),
        ["alpha"],
        "Backspace on the empty line removes the last chip"
    );
    assert_eq!(k.text(), "alpha");
    input.key(&mut tree, 8);
    input.key(&mut tree, 8);
    assert!(
        k.words().is_empty(),
        "and the one before it, and nothing more at the end"
    );
    assert_eq!(k.text(), "");
}

#[test]
fn the_button_and_the_program_remove_a_word_and_the_chips_are_a_named_list() {
    let _g = serial();
    let k = input().value("a, b, c");
    let mut tree = layout(k.build(), 600.0, 300.0);
    assert_eq!(
        k.words(),
        ["a", "b", "c"],
        "the start value is spread into chips"
    );
    assert_eq!(attr(&tree, "kw-chips", "role").as_deref(), Some("list"));
    assert_eq!(
        attr(&tree, "kw-chips-remove-b", "aria-label").as_deref(),
        Some("Remove b")
    );
    let mut input = Input::new();
    input.click(&mut tree, "kw-chips-remove-b");
    frame(&mut tree, 600.0, 300.0);
    assert_eq!(k.words(), ["a", "c"]);
    assert!(tree.query_by_id("kw-chips-chip-b").is_none());
    assert!(!k.remove_word("zz"), "a word that is not there");
    assert!(k.remove_word("a"));
    assert_eq!(k.text(), "c");
}
