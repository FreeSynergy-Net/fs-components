//! The type `choice`: the rule against the list (the contract crate's `Choice::pick`), the registry entry, the input as segments for a
//! few entries and as a list in a popover for more, the labels through `Say`, the problem only after a change, and the sentence
//! (never a dead control) when the list is empty (ADR 0097).
//!
//! Tests: model tests over `check` and the trait; the input built offscreen and driven with the window's own router.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use std::sync::{Arc, Mutex};

use common::{Input, attr, frame, layout, serial};
use fs_components_fields::kinds::check_choice;
use fs_components_fields::{ChoiceInput, Kind, Look, Problem, Say};
use fs_components_styles::StyleId;

fn keys(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("k{i}")).collect()
}

fn say() -> Arc<dyn Say> {
    Arc::new(|key: &str| match key {
        "color.k1" => "Blue".to_owned(),
        other => other.to_owned(),
    })
}

fn input(n: usize) -> ChoiceInput {
    ChoiceInput::new(Look::new(StyleId::Admin, false), say(), "color", keys(n)).name("ch")
}

#[test]
fn the_rule_asks_with_the_list_and_nothing_typed_is_no_problem() {
    let list = keys(3);
    assert_eq!(check_choice("k1", &list), None);
    assert_eq!(check_choice("  k2 ", &list), None, "trimmed");
    assert_eq!(
        check_choice("", &list),
        None,
        "nothing typed is no value, no problem"
    );
    assert_eq!(check_choice("k9", &list), Some(Problem::NotInList));
    assert_eq!(
        check_choice("K1", &list),
        Some(Problem::NotInList),
        "keys are matched exactly"
    );
    assert_eq!(
        check_choice("k0", &[]),
        Some(Problem::NotInList),
        "an empty list holds nothing"
    );
    assert_eq!(Problem::NotInList.key(), "problem.choice.not-in-list");
}

#[test]
fn the_type_is_one_part_in_the_registry_and_has_its_block() {
    let f = Kind::Choice.field();
    assert_eq!(f.key(), "choice");
    assert_eq!(f.segments().len(), 1);
    assert_eq!(f.segments()[0].key, "key");
    assert_eq!(Kind::Choice.block(), "field-choice");
    assert_eq!(Kind::from_key("choice"), Some(Kind::Choice));
    assert_eq!(Kind::from_block("field-choice"), Some(Kind::Choice));
    assert_eq!(f.split("  k1 "), ["k1"]);
    assert_eq!(f.join(&[" k1 ".to_owned()]), "k1");
    assert_eq!(f.join(&[]), "");
    assert_eq!(f.compact(&["k1".to_owned()]), "k1");
    assert_eq!(
        f.problem(&["anything".to_owned()]),
        None,
        "the trait does not know the list"
    );
}

#[test]
fn up_to_five_entries_are_segments_labelled_through_say_and_a_click_chooses_once() {
    let _g = serial();
    let log = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&log);
    let c = input(3).on_change(move |k| sink.lock().unwrap().push(k.to_owned()));
    assert!(c.is_segmented());
    let mut tree = layout(c.build(), 600.0, 200.0);
    assert_eq!(attr(&tree, "ch", "role").as_deref(), Some("radiogroup"));
    assert_eq!(attr(&tree, "ch-k1", "role").as_deref(), Some("radio"));
    assert_eq!(
        attr(&tree, "ch-k1", "aria-checked").as_deref(),
        Some("false"),
        "nothing is chosen for the person"
    );
    assert_eq!(c.text(), "");
    let mut input = Input::new();
    input.click(&mut tree, "ch-k1");
    frame(&mut tree, 600.0, 200.0);
    assert_eq!(c.text(), "k1");
    assert_eq!(*log.lock().unwrap(), ["k1"]);
    assert_eq!(
        attr(&tree, "ch-k1", "aria-checked").as_deref(),
        Some("true")
    );
    assert!(
        tree.query_by_id("ch-k1-label").is_some(),
        "the label of the entry is drawn"
    );
}

#[test]
fn the_start_value_is_chosen_and_a_key_outside_the_list_is_a_problem_after_a_change() {
    let _g = serial();
    let c = input(3).value("k2");
    let tree = layout(c.build(), 600.0, 200.0);
    assert_eq!(
        attr(&tree, "ch-k2", "aria-checked").as_deref(),
        Some("true")
    );
    assert_eq!(c.problem(), None);
    let bad = input(3).value("zz");
    assert_eq!(
        bad.problem(),
        Some(Problem::NotInList),
        "the problem is known at once; it is SHOWN after a change"
    );
    let mut tree = layout(bad.build(), 600.0, 200.0);
    assert!(
        tree.query_by_id("ch-problem").is_none(),
        "an untouched field shows no error"
    );
    let mut input = Input::new();
    input.click(&mut tree, "ch-k0");
    frame(&mut tree, 600.0, 200.0);
    assert_eq!(bad.problem(), None, "choosing an entry mends it");
}

#[test]
fn more_than_five_entries_are_a_list_in_a_popover_and_picking_closes_it() {
    let _g = serial();
    let log = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&log);
    let c = input(8).on_change(move |k| sink.lock().unwrap().push(k.to_owned()));
    assert!(!c.is_segmented());
    let mut tree = layout(c.build(), 600.0, 400.0);
    assert!(
        tree.query_by_id("ch-k1").is_none(),
        "no segments for a long list"
    );
    assert_eq!(attr(&tree, "ch-trigger", "role").as_deref(), Some("button"));
    assert_eq!(
        attr(&tree, "ch-trigger", "aria-haspopup").as_deref(),
        Some("dialog")
    );
    assert_eq!(
        attr(&tree, "ch-trigger", "aria-expanded").as_deref(),
        Some("false")
    );
    let mut input = Input::new();
    input.click(&mut tree, "ch-trigger");
    frame(&mut tree, 600.0, 400.0);
    assert_eq!(
        attr(&tree, "ch-trigger", "aria-expanded").as_deref(),
        Some("true")
    );
    assert_eq!(attr(&tree, "ch-list", "role").as_deref(), Some("listbox"));
    assert_eq!(
        attr(&tree, "ch-list-opt-k1", "role").as_deref(),
        Some("option")
    );
    input.click(&mut tree, "ch-list-opt-k1");
    frame(&mut tree, 600.0, 400.0);
    assert_eq!(c.text(), "k1");
    assert_eq!(*log.lock().unwrap(), ["k1"]);
    assert_eq!(
        attr(&tree, "ch-trigger", "aria-expanded").as_deref(),
        Some("false"),
        "picking closes the list"
    );
    assert!(tree.query_by_id("ch-list").is_none());
}

#[test]
fn an_empty_list_says_so_in_words_and_draws_no_control() {
    let _g = serial();
    let c = input(0);
    let tree = layout(c.build(), 600.0, 200.0);
    assert!(
        tree.query_by_id("ch-empty").is_some(),
        "the sentence is there (ADR 0097)"
    );
    assert!(tree.query_by_id("ch").is_none(), "no dead control");
    assert!(tree.query_by_id("ch-trigger").is_none());
    assert_eq!(c.text(), "");
}

#[test]
fn five_entries_are_still_segments_and_six_are_a_list() {
    let _g = serial();
    assert!(input(5).is_segmented(), "five: segments");
    assert!(!input(6).is_segmented(), "six: a list");
    assert!(input(1).is_segmented());
    assert_eq!(fs_components_fields::SEGMENTS_UP_TO, 5);
}

#[test]
fn every_label_is_asked_through_say_with_the_label_key_and_the_entry() {
    let _g = serial();
    let asked = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&asked);
    let say: Arc<dyn Say> = Arc::new(move |key: &str| {
        sink.lock().unwrap().push(key.to_owned());
        key.to_owned()
    });
    let c = ChoiceInput::new(Look::new(StyleId::Admin, false), say, "color", keys(3)).name("ch");
    let _tree = layout(c.build(), 600.0, 200.0);
    let asked = asked.lock().unwrap().clone();
    for k in ["color.k0", "color.k1", "color.k2", "field.choice"] {
        assert!(asked.iter().any(|a| a == k), "{k} was asked, got {asked:?}");
    }
}

#[test]
fn a_split_keeps_the_trimmed_key_only() {
    assert_eq!(Kind::Choice.field().split("\t k1 \n"), ["k1"]);
}
