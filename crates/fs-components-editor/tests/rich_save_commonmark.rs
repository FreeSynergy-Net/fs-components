//! The saved form against a real `CommonMark` reader (pulldown-cmark): a paragraph, wherever it stands, must come out as
//! paragraph text and nothing else (no heading, list, quote, rule, code in it).

use fs_components_editor::rich::{Item, List, Node, write::write_saved};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};

/// The texts that look like a marker, and plain ones.
const TEXTS: &[&str] = &[
    "## x",
    "# x",
    "###### x",
    "#",
    "##",
    "#\tx",
    "- x",
    "-",
    "-\tx",
    "* x",
    "+ x",
    "1. x",
    "12. x",
    "123456789. x",
    "1) x",
    "1.",
    "1)",
    "1.\tx",
    "1)\tx",
    "> x",
    ">x",
    "---",
    "***",
    "___",
    "```rust",
    "~~~",
    "[x] x",
    "[ ] x",
    "\\## x",
    "\\- x",
    "\\1. x",
    "1\\. x",
    "1\\) x",
    "plain",
    "a - b",
    "2026",
    "1.x",
    "1234567890. x",
];

/// The events of `source` as `P(text)`; anything but a paragraph shows as its tag. `paragraphs` is what the text of the
/// paragraph should be (one for the single case).
fn shape(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    for e in Parser::new(source) {
        match e {
            Event::Start(Tag::Paragraph) => out.push("P[".to_owned()),
            Event::End(TagEnd::Paragraph) => out.push("]".to_owned()),
            Event::Text(t) => out.push(t.to_string()),
            Event::Start(t) => out.push(format!("<{t:?}>")),
            Event::End(t) => out.push(format!("</{t:?}>")),
            other => out.push(format!("{other:?}")),
        }
    }
    out
}

/// The text of the one paragraph in `source`, or the reason it is not one.
fn paragraph_text(source: &str) -> Result<String, Vec<String>> {
    let s = shape(source);
    if s.first().map(String::as_str) == Some("P[") && s.last().map(String::as_str) == Some("]") {
        let inner = &s[1..s.len() - 1];
        if inner
            .iter()
            .all(|e| !e.starts_with('<') && !e.starts_with("Start") && !e.starts_with("Code"))
        {
            return Ok(inner.concat());
        }
    }
    Err(s)
}

#[test]
fn a_saved_paragraph_is_a_paragraph_with_its_text_in_a_commonmark_reader() {
    for t in TEXTS {
        let (source, _) = write_saved(&Node::Para((*t).to_owned()));
        assert_eq!(
            paragraph_text(&source),
            Ok((*t).to_owned()),
            "text {t:?}, saved as {source:?}"
        );
    }
}

#[test]
fn the_text_of_a_list_item_is_paragraph_text_in_a_commonmark_reader() {
    for t in TEXTS {
        for ordered in [false, true] {
            let node = Node::List(List::one(ordered, 1, Item::text(None, *t)));
            let (source, _) = write_saved(&node);
            let s = shape(&source);
            // One list, one item, and the text of the item alone (the shape of the item is `<Item>` and the text, or in a
            // tight list the text without a paragraph).
            let want = if ordered {
                "<List(Some(1))>"
            } else {
                "<List(None)>"
            };
            assert_eq!(
                s.first().map(String::as_str),
                Some(want),
                "text {t:?}, saved as {source:?}: {s:?}"
            );
            assert_eq!(
                s.get(1).map(String::as_str),
                Some("<Item>"),
                "text {t:?}, saved as {source:?}: {s:?}"
            );
            assert_eq!(
                s[2..s.len() - 2].concat(),
                *t,
                "the text of the item {t:?}, saved as {source:?}: {s:?}"
            );
            assert!(
                s[2..s.len() - 2].iter().all(|e| !e.starts_with('<')),
                "nothing but the text in the item {t:?}, saved as {source:?}: {s:?}"
            );
        }
    }
}

#[test]
fn the_text_of_a_quote_is_paragraph_text_in_a_commonmark_reader() {
    for t in TEXTS {
        let (source, _) = write_saved(&Node::Quote(vec![Node::Para((*t).to_owned())]));
        let s = shape(&source);
        assert_eq!(
            s.first().map(String::as_str),
            Some("<BlockQuote(None)>"),
            "text {t:?}, saved as {source:?}: {s:?}"
        );
        assert_eq!(
            s.get(1).map(String::as_str),
            Some("P["),
            "text {t:?}, saved as {source:?}: {s:?}"
        );
        assert_eq!(
            s[2..s.len() - 2].concat(),
            *t,
            "the text {t:?}, saved as {source:?}: {s:?}"
        );
    }
}

/// The counter-case: the same sources unmasked are not paragraphs (the check can fail).
#[test]
fn the_check_sees_a_marker_that_is_not_masked() {
    for source in ["## x", "- x", "1. x", "1) x", "1.", "> x", "-", "#"] {
        assert!(
            paragraph_text(source).is_err(),
            "{source:?} is not a paragraph in CommonMark"
        );
    }
}
