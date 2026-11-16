//! Saving and reading the rich editor: every node kind survives write and read, a raw block can be typed in, a shortcut given back
//! stays text after saving.
//!
//! Tests: model tests, no window.

use fs_components_editor::rich::{Node, RichEditor};
use fs_components_editor::{Doc, Edit, TextEditor};

fn typed(start: &str, block: usize, at: usize, text: &str) -> RichEditor {
    let mut e = RichEditor::new();
    e.load(&Doc::parse(start));
    e.edit(&Edit::Type {
        block,
        at,
        text: text.into(),
    });
    e
}

fn node(e: &RichEditor, i: usize) -> &Node {
    &e.blocks()[i].node
}

/// Every kind of node survives writing and reading: what is written reads back as the same tree (and writing that again gives
/// the same source). The paragraphs that look like a marker are the ones that need a mask.
#[test]
fn write_then_read_gives_the_same_tree() {
    use fs_components_editor::rich::{Item, List, read::read, write::write_saved as write};
    let para = |t: &str| Node::Para(t.into());
    let mut nodes = vec![
        Node::Heading(2, "## in a heading".into()),
        Node::Code("rust".into(), "## code\n- code".into()),
        Node::Rule,
        Node::List(List::one(true, 3, Item::text(None, "x"))),
        Node::List(List::one(false, 1, Item::text(Some(true), "[x] inner"))),
        Node::Quote(vec![para("a"), para("## q"), para("- q")]),
    ];
    for t in [
        "## x", "# x", "- x", "* x", "+ x", "1. x", "12. x", "[x] x", "[ ] x", "[X] x", "> x",
        "---", "```rust", "| a |", "\\## x", "\\\\- x", "\\[x] x", "plain", "a - b", "##x", "1.x",
        "2026", "1\\. x", "1\\\\. x", "12\\. x",
    ] {
        nodes.push(para(t));
        nodes.push(Node::List(List::one(false, 1, Item::text(None, t))));
        nodes.push(Node::List(List::one(false, 1, Item::text(Some(false), t))));
        nodes.push(Node::Quote(vec![para(t)]));
    }
    for n in nodes {
        let (source, starts) = write(&n);
        let (back, got) = read(&source);
        assert_eq!(back, n, "read(write(n)) is n; source {source:?}");
        assert_eq!(got, starts, "the leaf starts agree; source {source:?}");
        assert_eq!(write(&back).0, source, "writing again changes nothing");
    }
}

/// The mask of a number is the valid CommonMark escape: the dot is escaped (`1\. x`), not the digit (a backslash in front of a
/// digit is no escape, any other reader would show it). The expectation is fixed, as a CommonMark reader gives it: a backslash
/// before an ASCII punctuation character is dropped and the character is text, so these sources show `1. x`, `12. x`, `1) x`.
#[test]
fn a_number_is_masked_with_a_valid_commonmark_escape() {
    use fs_components_editor::rich::{read::read, write::write_saved};
    for (text, source) in [
        ("1. x", "1\\. x"),
        ("12. x", "12\\. x"),
        ("1) x", "1) x"),
        ("1\\. x", "1\\\\. x"),
        ("\\1. x", "\\\\1. x"),
    ] {
        let n = Node::Para(text.into());
        assert_eq!(write_saved(&n).0, source, "the source of {text:?}");
        assert_eq!(read(source).0, n, "{source:?} reads as {text:?}");
        let (again, _) = read(&write_saved(&read(source).0).0);
        assert_eq!(again, n, "read, write, read of {source:?}");
    }
}

/// The finding of the review: a lazy line of a list item and a table in a quote are raw, and typing in them is not lost.
#[test]
fn a_raw_block_can_be_typed_in() {
    for src in ["- a\n- b\nlazy", "> a\n>\n> | x | y |"] {
        let e = typed(src, 0, 0, "Z");
        assert!(matches!(node(&e, 0), Node::Raw(_)), "{src:?} is raw");
        assert!(
            e.document().blocks[0].starts_with('Z'),
            "{src:?}: typing was lost"
        );
    }
}

/// A shortcut given back stays text for good: written and read again it is a paragraph, not the block the shortcut made.
#[test]
fn a_given_back_shortcut_stays_text_after_saving() {
    for keys in ["## ", "- ", "1. ", "[x] ", "> "] {
        let mut s2 = fs_components_editor::rich::Session::new(&Doc::new(vec![String::new()]));
        for ch in keys.chars() {
            s2.key(fs_components_editor::rich::Key::Char(ch));
        }
        s2.key(fs_components_editor::rich::Key::Backspace);
        s2.key(fs_components_editor::rich::Key::Char('x'));
        let saved = s2.save_form();
        assert_eq!(
            s2.document().blocks[0],
            format!("{keys}x"),
            "the live form is what was typed"
        );
        let mut again = RichEditor::new();
        again.load(&saved);
        assert!(
            matches!(node(&again, 0), Node::Para(t) if t == &format!("{keys}x")),
            "{keys:?}: {saved:?}"
        );
    }
}
