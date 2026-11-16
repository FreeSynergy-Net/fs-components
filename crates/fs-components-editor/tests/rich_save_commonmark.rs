//! The saved form against a real `CommonMark` reader (pulldown-cmark): a paragraph, wherever it stands, must come out as
//! paragraph text and nothing else (no heading, list, quote, rule, code in it).

mod rich_commonmark;

use fs_components_editor::rich::{Item, List, Node, write::write_saved};
use rich_commonmark::{TEXTS, paragraph_text, shape};

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
                "\u{27e8}List(Some(1))\u{27e9}"
            } else {
                "\u{27e8}List(None)\u{27e9}"
            };
            assert_eq!(
                s.first().map(String::as_str),
                Some(want),
                "text {t:?}, saved as {source:?}: {s:?}"
            );
            assert_eq!(
                s.get(1).map(String::as_str),
                Some("\u{27e8}Item\u{27e9}"),
                "text {t:?}, saved as {source:?}: {s:?}"
            );
            assert_eq!(
                s[2..s.len() - 2].concat(),
                *t,
                "the text of the item {t:?}, saved as {source:?}: {s:?}"
            );
            assert!(
                s[2..s.len() - 2].iter().all(|e| !e.starts_with('\u{27e8}')),
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
            Some("\u{27e8}BlockQuote(None)\u{27e9}"),
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
