//! The type `reference`: the rule of the contract crate through the trait (absolute IRI, kept as written), the registry entry and the
//! block, the input: the line, the candidates from the source, a click or Enter on the only one picks, a missing source is said in
//! words (ADR 0097), the problem only after a change.
//!
//! Tests: model tests over the trait; the input built offscreen and driven with the window's own router.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use std::sync::{Arc, Mutex};

use common::{Input, attr, frame, layout, serial};
use fs_components_fields::{
    Candidate, KeysAsText, Kind, Look, Problem, ReferenceInput, ReferenceSource,
};
use fs_components_styles::StyleId;

struct Known;

impl ReferenceSource for Known {
    fn find(&self, typed: &str, limit: usize) -> Vec<Candidate> {
        [
            ("https://example.org/tasks/7", "Write the report"),
            ("https://example.org/tasks/8", "Write the plan"),
            ("https://example.org/notes/1", "Notes"),
        ]
        .into_iter()
        .filter(|(iri, label)| {
            iri.contains(typed) || label.to_lowercase().contains(&typed.to_lowercase())
        })
        .take(limit)
        .map(|(i, l)| Candidate::new(i, l))
        .collect()
    }
}

fn field(source: bool) -> ReferenceInput {
    let src: Option<Arc<dyn ReferenceSource>> =
        source.then(|| Arc::new(Known) as Arc<dyn ReferenceSource>);
    ReferenceInput::new(
        Look::new(StyleId::Admin, false),
        Arc::new(KeysAsText),
        "rf",
        src,
    )
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
        frame(tree, 600.0, 400.0);
    }
}

#[test]
fn the_trait_takes_an_absolute_iri_as_written() {
    let f = Kind::Reference.field();
    assert_eq!(f.key(), "reference");
    assert_eq!(f.segments().len(), 1);
    assert_eq!(Kind::Reference.block(), "field-reference");
    assert_eq!(Kind::from_key("reference"), Some(Kind::Reference));
    assert_eq!(f.problem(&["https://example.org/a".to_owned()]), None);
    assert_eq!(f.problem(&["urn:x:1".to_owned()]), None);
    assert_eq!(
        f.problem(&["  ".to_owned()]),
        None,
        "nothing typed is no problem"
    );
    assert_eq!(
        f.problem(&["tasks/7".to_owned()]),
        Some(Problem::NotAbsolute)
    );
    assert_eq!(
        f.problem(&["x:".to_owned()]),
        Some(Problem::BadReference),
        "nothing after the scheme"
    );
    assert_eq!(
        f.problem(&["http://exa mple".to_owned()]),
        Some(Problem::BadReference)
    );
    assert_eq!(
        f.problem(&[format!("https://e.org/{}", "a".repeat(2100))]),
        Some(Problem::BadReference),
        "too long"
    );
    assert_eq!(
        f.join(&["  HTTPS://Example.org/A ".to_owned()]),
        "HTTPS://Example.org/A",
        "kept as written, only trimmed"
    );
    assert_eq!(f.split(" a "), ["a"]);
    assert_eq!(Problem::NotAbsolute.key(), "problem.reference.not-absolute");
    assert_eq!(Problem::BadReference.key(), "problem.reference.bad");
}

#[test]
fn typing_shows_the_candidates_and_a_click_picks_one() {
    let _g = serial();
    let log = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&log);
    let r = field(true).on_change(move |t| sink.lock().unwrap().push(t.to_owned()));
    let mut tree = layout(r.build(), 600.0, 400.0);
    let mut input = Input::new();
    assert!(
        tree.query_by_id("rf-list").is_none(),
        "nothing typed, no candidates"
    );
    type_into(&mut tree, &mut input, "rf", "tasks");
    assert_eq!(r.candidates().len(), 2);
    assert_eq!(attr(&tree, "rf-list", "role").as_deref(), Some("listbox"));
    assert!(
        tree.query_by_id("rf-list-opt-https://example.org/tasks/8")
            .is_some()
    );
    input.click(&mut tree, "rf-list-opt-https://example.org/tasks/8");
    frame(&mut tree, 600.0, 400.0);
    assert_eq!(r.text(), "https://example.org/tasks/8");
    assert_eq!(
        r.picked().map(|c| c.label),
        Some("Write the plan".to_owned())
    );
    assert!(
        tree.query_by_id("rf-picked").is_some(),
        "the label of the picked one is shown"
    );
    assert!(
        tree.query_by_id("rf-list").is_none(),
        "the list is gone once one is picked"
    );
    assert_eq!(
        log.lock().unwrap().last().map(String::as_str),
        Some("https://example.org/tasks/8")
    );
    assert_eq!(r.problem(), None);
}

#[test]
fn enter_picks_the_only_candidate_and_does_nothing_with_several_or_none() {
    let _g = serial();
    let r = field(true);
    let mut tree = layout(r.build(), 600.0, 400.0);
    let mut input = Input::new();
    type_into(&mut tree, &mut input, "rf", "tasks");
    input.key(&mut tree, 13);
    frame(&mut tree, 600.0, 400.0);
    assert_eq!(r.picked(), None, "two candidates: Enter picks none");
    type_into(&mut tree, &mut input, "rf", "/7");
    assert_eq!(r.candidates().len(), 1);
    input.key(&mut tree, 13);
    frame(&mut tree, 600.0, 400.0);
    assert_eq!(
        r.text(),
        "https://example.org/tasks/7",
        "the only candidate is picked"
    );
    let none = field(true).value("zzz");
    assert!(none.candidates().is_empty(), "nothing matches");
}

#[test]
fn without_a_source_the_missing_search_is_said_and_the_iri_can_still_be_typed() {
    let _g = serial();
    let r = field(false);
    let mut tree = layout(r.build(), 600.0, 400.0);
    assert!(
        tree.query_by_id("rf-nosource").is_some(),
        "ADR 0097: the missing function is said"
    );
    let mut input = Input::new();
    type_into(&mut tree, &mut input, "rf", "https://example.org/x");
    assert_eq!(r.text(), "https://example.org/x");
    assert!(r.candidates().is_empty());
    assert!(tree.query_by_id("rf-list").is_none());
    let with = field(true);
    let tree = layout(with.build(), 600.0, 400.0);
    assert!(
        tree.query_by_id("rf-nosource").is_none(),
        "with a source nothing is missing"
    );
}

#[test]
fn a_problem_is_shown_after_a_change_and_gone_when_mended() {
    let _g = serial();
    let r = field(true);
    let mut tree = layout(r.build(), 600.0, 400.0);
    assert!(
        tree.query_by_id("rf-problem").is_none(),
        "an untouched field shows no error"
    );
    let mut input = Input::new();
    type_into(&mut tree, &mut input, "rf", "tasks/7");
    frame(&mut tree, 600.0, 400.0);
    assert_eq!(r.problem(), Some(Problem::NotAbsolute));
    assert!(tree.query_by_id("rf-problem").is_some());
    let start = ReferenceInput::new(
        Look::new(StyleId::Admin, false),
        Arc::new(KeysAsText),
        "rs",
        None,
    )
    .value(" https://a.org/b ");
    assert_eq!(start.text(), "https://a.org/b");
    assert_eq!(start.problem(), None);
}

struct Many;

impl ReferenceSource for Many {
    fn find(&self, _typed: &str, limit: usize) -> Vec<Candidate> {
        (0..20)
            .take(limit)
            .map(|i| Candidate::new(format!("https://example.org/n/{i}"), format!("n{i}")))
            .collect()
    }
}

#[test]
fn at_most_six_candidates_are_asked_for_and_shown() {
    let _g = serial();
    let r = ReferenceInput::new(
        Look::new(StyleId::Admin, false),
        Arc::new(KeysAsText),
        "rm",
        Some(Arc::new(Many)),
    )
    .value("n");
    assert_eq!(r.candidates().len(), 6);
}

#[test]
fn a_start_value_that_is_wrong_shows_no_sentence_until_a_change_and_a_changed_text_forgets_the_pick()
 {
    let _g = serial();
    let r = ReferenceInput::new(
        Look::new(StyleId::Admin, false),
        Arc::new(KeysAsText),
        "rw",
        None,
    )
    .value("tasks/7");
    let mut tree = layout(r.build(), 600.0, 400.0);
    assert_eq!(r.problem(), Some(Problem::NotAbsolute), "known at once");
    assert!(
        tree.query_by_id("rw-problem").is_none(),
        "but shown only after a change"
    );
    let p = field(true);
    let mut tree = layout(p.build(), 600.0, 400.0);
    let mut input = Input::new();
    p.pick(&Candidate::new(
        "https://example.org/tasks/7",
        "Write the report",
    ));
    assert!(p.picked().is_some());
    type_into(&mut tree, &mut input, "rf", "x");
    assert_eq!(p.picked(), None, "the text is no longer the picked IRI");
}
