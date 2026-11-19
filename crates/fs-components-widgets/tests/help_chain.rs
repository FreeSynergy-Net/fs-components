//! `help`, the chain and the plain text: the chain takes the highest edition, skips a source that is down and fails only when all do;
//! a block is plain text without control characters.
//!
//! Tests: model tests over fake sources.

use std::sync::Arc;

use fs_components_widgets::help::{Chain, HelpBlock, HelpError, HelpSource, paragraphs};

/// A source with one answer for every question.
struct Fixed(Result<Option<HelpBlock>, HelpError>);

impl HelpSource for Fixed {
    fn block(&self, _: &str, _: &str, _: &[String]) -> Result<Option<HelpBlock>, HelpError> {
        self.0.clone()
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
    assert_eq!(paragraphs("  \n\n "), Vec::<String>::new());
}
