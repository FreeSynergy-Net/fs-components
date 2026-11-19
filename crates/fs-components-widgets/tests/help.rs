//! `help`: the blocks come through the contract (`HelpSource`), the chain takes the highest edition, and EVERY outcome has words on the
//! screen (ADR 0097: an entry with content or a visible fallback, never greyed out, never empty).
//!
//! Tests: model tests over fake sources, attribute and id reads on the built element, an in-process click on the documentation button.

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, layout, serial, size_of};
use fs_components_mf2::check_pairs;
use fs_components_widgets::help::{
    Chain, Help, HelpBlock, HelpError, HelpSource, Labels, Outcome, TEXTS, paragraphs, purpose_of,
    resolve,
};
use fs_components_words::{Locale, Localized};

/// A source with one answer for every question.
struct Fixed(Result<Option<HelpBlock>, HelpError>);

impl HelpSource for Fixed {
    fn block(&self, _: &str, _: &str, _: &[String]) -> Result<Option<HelpBlock>, HelpError> {
        self.0.clone()
    }
}

/// A source that remembers what it was asked.
struct Asked(Mutex<Vec<(String, String, Vec<String>)>>);

impl HelpSource for Asked {
    fn block(
        &self,
        program: &str,
        purpose: &str,
        langs: &[String],
    ) -> Result<Option<HelpBlock>, HelpError> {
        self.0
            .lock()
            .unwrap()
            .push((program.to_owned(), purpose.to_owned(), langs.to_vec()));
        Ok(None)
    }
}

fn block(rev: u32, text: &str) -> HelpBlock {
    HelpBlock {
        id: format!("b{rev}"),
        lang: "en".to_owned(),
        text: text.to_owned(),
        rev,
        fallback_from: None,
        outdated: false,
    }
}

fn fixed(answer: Result<Option<HelpBlock>, HelpError>) -> Arc<dyn HelpSource> {
    Arc::new(Fixed(answer))
}

fn langs() -> Vec<String> {
    vec!["en".to_owned()]
}

#[test]
fn every_word_is_in_german_and_english() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let de = Labels::for_locale(&Locale::parse("de").unwrap());
    assert_eq!(de.open_docs, "Dokumentation öffnen");
    assert!(
        de.missing("versions").contains("versions")
            && de.missing("versions").contains("keine Hilfe")
    );
    assert!(Labels::default().missing("versions").contains("no help"));
    assert!(de.shown_in("English").contains("English") && de.outdated().contains("älter"));
}

#[test]
fn the_question_is_program_purpose_and_a_wish_list_of_languages() {
    let spy = Arc::new(Asked(Mutex::default()));
    let labels = Labels::for_locale(&Locale::parse("de-AT").unwrap());
    let help = Help::new(&labels, "fs-files", Arc::clone(&spy) as Arc<dyn HelpSource>);
    let _ = help.outcome("versions");
    let asked = spy.0.lock().unwrap().clone();
    assert_eq!(asked.len(), 1);
    assert_eq!(
        (asked[0].0.as_str(), asked[0].1.as_str()),
        ("fs-files", "help.versions")
    );
    assert_eq!(
        asked[0].2.first().map(String::as_str),
        Some("de-AT"),
        "the wish starts with the language of the person"
    );
    assert!(
        asked[0].2.iter().any(|l| l == "en"),
        "and ends in English: {:?}",
        asked[0].2
    );
    assert_eq!(purpose_of("a.b"), "help.a.b");
    let wished = Help::new(
        &Labels::default(),
        "p",
        Arc::clone(&spy) as Arc<dyn HelpSource>,
    )
    .languages(&["fr", "en"]);
    let _ = wished.outcome("x");
    assert_eq!(spy.0.lock().unwrap().last().unwrap().2, ["fr", "en"]);
}

#[test]
fn an_entry_a_missing_one_an_empty_one_and_a_dead_source_are_four_different_outcomes() {
    let found = resolve(
        fixed(Ok(Some(block(1, "Hello.")))).as_ref(),
        "p",
        "t",
        &langs(),
    );
    assert!(matches!(found, Outcome::Found(b) if b.text == "Hello."));
    assert_eq!(
        resolve(fixed(Ok(None)).as_ref(), "p", "t", &langs()),
        Outcome::Missing
    );
    assert_eq!(
        resolve(
            fixed(Ok(Some(block(1, " \n\n \u{7}")))).as_ref(),
            "p",
            "t",
            &langs()
        ),
        Outcome::Missing,
        "a block with no text is no help"
    );
    assert_eq!(
        resolve(
            fixed(Err(HelpError("no texts".into()))).as_ref(),
            "p",
            "t",
            &langs()
        ),
        Outcome::Unavailable("no texts".into())
    );
}

#[test]
fn the_chain_takes_the_highest_edition_skips_a_dead_source_and_fails_only_when_all_do() {
    let (low, high) = (
        fixed(Ok(Some(block(2, "old")))),
        fixed(Ok(Some(block(5, "new")))),
    );
    let dead = fixed(Err(HelpError("down".into())));
    let chain = Chain::new()
        .then(Arc::clone(&low))
        .then(Arc::clone(&dead))
        .then(Arc::clone(&high));
    assert_eq!(
        chain.block("p", "help.t", &langs()).unwrap().unwrap().text,
        "new"
    );
    let tie = Chain::new()
        .then(fixed(Ok(Some(block(3, "first")))))
        .then(fixed(Ok(Some(block(3, "second")))));
    assert_eq!(
        tie.block("p", "help.t", &langs()).unwrap().unwrap().text,
        "first",
        "equal editions: the nearer source wins"
    );
    let none = Chain::new().then(fixed(Ok(None))).then(Arc::clone(&dead));
    assert_eq!(
        none.block("p", "help.t", &langs()),
        Ok(None),
        "one source answered with nothing: missing, not unavailable"
    );
    let all_dead = Chain::new()
        .then(Arc::clone(&dead))
        .then(fixed(Err(HelpError("also".into()))));
    assert_eq!(
        all_dead.block("p", "help.t", &langs()),
        Err(HelpError("down".into()))
    );
    assert_eq!(
        Chain::new().block("p", "help.t", &langs()),
        Ok(None),
        "an empty chain has nothing"
    );
}

#[test]
fn paragraphs_are_plain_text_without_control_characters() {
    assert_eq!(
        paragraphs("One\u{7} line\nwraps.\n\n\n  Two.  \r\n\r\nThree."),
        ["One line wraps.", "Two.", "Three."]
    );
    assert!(paragraphs("\u{0}\u{1b}[31m").len() <= 1);
    assert!(paragraphs("  \n\n ").is_empty());
}

/// The size of the element with this id: it must be drawn (more than nothing).
fn drawn(tree: &blinc_layout::prelude::RenderTree, id: &str) -> bool {
    tree.query_by_id(id).is_some() && {
        let (w, h) = size_of(tree, id);
        w > 0.0 && h > 0.0
    }
}

#[test]
fn every_outcome_puts_words_on_the_screen_the_rule_of_adr_0097() {
    let _g = serial();
    let docs_open = Arc::new(Mutex::new(0u32));
    let cases: [(&str, Arc<dyn HelpSource>, &str); 4] = [
        (
            "found",
            fixed(Ok(Some(block(1, "How to start.\n\nThen go on.")))),
            "found",
        ),
        ("missing", fixed(Ok(None)), "missing"),
        ("empty", fixed(Ok(Some(block(1, "  ")))), "missing"),
        (
            "dead",
            fixed(Err(HelpError("nobody".into()))),
            "unavailable",
        ),
    ];
    for (name, source, kind) in cases {
        let opened = Arc::clone(&docs_open);
        let help = Help::new(&Labels::default(), "fs-files", source)
            .id("hp")
            .on_docs(move || *opened.lock().unwrap() += 1);
        assert!(
            !help.visible_text("versions").trim().is_empty(),
            "{name}: nothing to read"
        );
        let tree = layout(help.build("versions"), 480.0, 300.0);
        assert!(drawn(&tree, "hp"), "{name}: the help is not drawn");
        assert!(drawn(&tree, "hp-text"), "{name}: its words are not drawn");
        let reg = tree.element_registry();
        let root = tree.query_by_id("hp").unwrap();
        assert!(
            reg.has_class(root, &format!("fs-help-{kind}")),
            "{name}: class fs-help-{kind}"
        );
        assert_eq!(
            tree.query_by_id("hp-docs").is_some(),
            kind != "found",
            "{name}: the way to the documentation"
        );
    }
}

#[test]
fn the_documentation_button_runs_the_handler_and_is_absent_without_one() {
    let _g = serial();
    let opened = Arc::new(Mutex::new(0u32));
    let count = Arc::clone(&opened);
    let help = Help::new(&Labels::default(), "p", fixed(Ok(None)))
        .id("hp")
        .on_docs(move || *count.lock().unwrap() += 1);
    let mut tree = layout(help.build("versions"), 480.0, 300.0);
    Input::new().click(&mut tree, "hp-docs");
    assert_eq!(*opened.lock().unwrap(), 1);
    let bare = Help::new(&Labels::default(), "p", fixed(Ok(None))).id("hp");
    let tree = layout(bare.build("versions"), 480.0, 300.0);
    assert!(
        tree.query_by_id("hp-docs").is_none(),
        "no handler: no button, the sentence stays"
    );
    assert!(drawn(&tree, "hp-text"));
}

#[test]
fn a_translation_that_is_not_there_or_older_says_so() {
    let _g = serial();
    let mut b = block(4, "Words.");
    b.lang = "en".into();
    b.fallback_from = Some("de".into());
    b.outdated = true;
    let both = Help::new(
        &Labels::for_locale(&Locale::parse("de").unwrap()),
        "p",
        fixed(Ok(Some(b))),
    )
    .id("hp");
    let tree = layout(both.build("t"), 480.0, 300.0);
    assert!(drawn(&tree, "hp-note"), "shown in another language: said");
    assert!(drawn(&tree, "hp-outdated"), "older than the original: said");
    let plain = Help::new(&Labels::default(), "p", fixed(Ok(Some(block(1, "Words."))))).id("hp");
    let tree = layout(plain.build("t"), 480.0, 300.0);
    assert!(
        tree.query_by_id("hp-note").is_none() && tree.query_by_id("hp-outdated").is_none(),
        "no note when nothing is wrong"
    );
}
