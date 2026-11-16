//! The saved form against a real `CommonMark` reader, part two: the round trip, headings, code blocks and the counter-cases.

mod rich_commonmark;

use fs_components_editor::rich::{Item, List, Node, read::read, write::write_saved};
use rich_commonmark::{TEXTS, paragraph_text, shape};

/// The nodes the typed reader gives back from the saved form are the nodes that were saved.
fn round(node: &Node) {
    let (source, _) = write_saved(node);
    assert_eq!(&read(&source).0, node, "saved as {source:?}");
}

#[test]
fn saving_and_loading_gives_the_same_node() {
    for t in TEXTS {
        round(&Node::Para((*t).to_owned()));
        round(&Node::Quote(vec![Node::Para((*t).to_owned())]));
        for ordered in [false, true] {
            round(&Node::List(List::one(ordered, 1, Item::text(None, *t))));
        }
    }
    for t in HEADINGS {
        round(&Node::Heading(2, (*t).to_owned()));
    }
    for (lang, code) in CODES {
        round(&Node::Code((*lang).to_owned(), (*code).to_owned()));
    }
}

const HEADINGS: &[&str] = &[
    "x ##", "x #", "#", "##", "x \t##", "a # b", "x#", "x ###", "x ## ", "# #", "x #\\#", "x \\#",
];

const CODES: &[(&str, &str)] = &[
    ("rust", "a\n```\nb"),
    ("", "```"),
    ("", "```\n"),
    ("rust", "a\n````\nb\n```"),
    ("", "~~~\nx"),
    ("", "``` x"),
    ("", "a ``` b\n``````"),
    ("", "plain"),
    ("", ""),
];

/// A heading comes out of a `CommonMark` reader as a heading with its text (without trailing spaces), a closing `#` included.
#[test]
fn a_saved_heading_keeps_its_closing_hashes_in_a_commonmark_reader() {
    for t in HEADINGS.iter().filter(|t| !t.contains('\\')) {
        let (source, _) = write_saved(&Node::Heading(2, (*t).to_owned()));
        let s = shape(&source);
        assert_eq!(s.len(), 3, "text {t:?}, saved as {source:?}: {s:?}");
        assert!(s[0].starts_with("\u{27e8}Heading"), "{source:?}: {s:?}");
        assert_eq!(s[1], t.trim_end(), "text {t:?}, saved as {source:?}");
    }
}

/// A code block, whatever lines its code has, is one code block with exactly that code and nothing after it.
#[test]
fn a_saved_code_block_is_one_code_block_in_a_commonmark_reader() {
    for (lang, code) in CODES {
        let (source, _) = write_saved(&Node::Code((*lang).to_owned(), (*code).to_owned()));
        let s = shape(&source);
        assert!(
            s.first()
                .is_some_and(|e| e.starts_with("\u{27e8}CodeBlock")),
            "{source:?}: {s:?}"
        );
        assert!(
            s.last()
                .is_some_and(|e| e.starts_with("\u{27e8}/CodeBlock")),
            "{source:?}: {s:?}"
        );
        let inner = s[1..s.len() - 1].concat();
        let want = format!("{code}\n");
        assert_eq!(inner, want, "the code of {source:?}: {s:?}");
        assert!(
            s[1..s.len() - 1].iter().all(|e| !e.starts_with('\u{27e8}')),
            "{source:?}: {s:?}"
        );
    }
}

/// The counter-cases of the new checks: the same texts written without the mask are not what they should be.
#[test]
fn the_check_sees_the_new_gaps_when_they_are_not_closed() {
    for source in [
        "_ _ _",
        "* * *",
        "- - -",
        "<!-- x",
        "<script>",
        "<div>x",
        "<?php x",
        "<![CDATA[x",
        "[a]: b",
    ] {
        assert!(
            paragraph_text(source).is_err(),
            "{source:?} is not a paragraph"
        );
    }
    // A heading with a closing sequence loses it.
    assert_eq!(shape("## x ##")[1], "x");
    // A code block whose code has a line of three backticks, fenced with three: the block ends early and the rest is not code.
    let s = shape("```\na\n```\nb\n```");
    assert!(s.iter().any(|e| e == "P["), "{s:?}");
}
