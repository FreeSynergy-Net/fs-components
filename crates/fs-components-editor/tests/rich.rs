//! The rich editor over the contract: the shortcuts turn into typed blocks at once, the contract holds, untouched blocks stay as
//! they were. Each rule has a case that must fire and a counter-case that must not.
//!
//! Tests: model tests, no window.

use fs_components_editor::rich::{Node, RichEditor};
use fs_components_editor::{Doc, Edit, TextEditor, apply, conformance};

fn typed(start: &str, block: usize, at: usize, text: &str) -> RichEditor {
    let mut e = RichEditor::new();
    e.load(&if start.trim().is_empty() {
        Doc::new(vec![String::new()])
    } else {
        Doc::parse(start)
    });
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

fn is_list(e: &RichEditor, i: usize, ordered: bool, start: u32) -> bool {
    matches!(node(e, i), Node::List(l) if l.ordered == ordered && l.start == start)
}

#[test]
fn it_passes_the_conformance_collection() {
    conformance::check(&|| -> Box<dyn TextEditor> { Box::new(RichEditor::new()) });
}

#[test]
fn heading_shortcuts_fire_by_level_and_not_with_seven_hashes() {
    for n in 1..=6u8 {
        let e = typed("\n", 0, 0, &format!("{} Title", "#".repeat(usize::from(n))));
        assert_eq!(node(&e, 0), &Node::Heading(n, "Title".into()));
    }
    let e = typed("\n", 0, 0, "####### x");
    assert_eq!(
        node(&e, 0),
        &Node::Para("####### x".into()),
        "seven hashes are text"
    );
}

#[test]
fn a_shortcut_inside_text_does_not_fire() {
    let e = typed("word\n", 0, 4, " # ");
    assert!(matches!(node(&e, 0), Node::Para(_)));
    let e = typed("a\n", 0, 0, "## ");
    assert_eq!(
        node(&e, 0),
        &Node::Heading(2, "a".into()),
        "at the start it fires and keeps the text behind"
    );
}

#[test]
fn list_task_quote_and_numbered_shortcuts() {
    for (keys, ordered, start) in [
        ("- ", false, 1),
        ("* ", false, 1),
        ("+ ", false, 1),
        ("3. ", true, 3),
    ] {
        let e = typed("\n", 0, 0, &format!("{keys}item"));
        assert!(is_list(&e, 0, ordered, start), "{keys}");
        assert_eq!(e.text(&[0, 0, 0]), "item", "{keys}");
    }
    let e = typed("\n", 0, 0, "> said");
    assert_eq!(node(&e, 0), &Node::Quote(vec![Node::Para("said".into())]));
    for (keys, done) in [
        ("[] ", false),
        ("[ ] ", false),
        ("[x] ", true),
        ("- [] ", false),
    ] {
        let e = typed("\n", 0, 0, &format!("{keys}item"));
        assert_eq!(
            e.item_of(&[0, 0, 0]).map(|i| i.task),
            Some(Some(done)),
            "{keys}"
        );
        assert_eq!(e.text(&[0, 0, 0]), "item");
    }
    for keys in ["-x ", "1.5 ", "1.", "- x"] {
        let e = typed("\n", 0, 0, keys);
        assert!(
            matches!(node(&e, 0), Node::Para(_) | Node::List(_)),
            "{keys}"
        );
    }
    assert!(
        matches!(node(&typed("\n", 0, 0, "-x "), 0), Node::Para(_)),
        "a dash glued to a word is not a list"
    );
    assert!(
        matches!(node(&typed("\n", 0, 0, "1.5 "), 0), Node::Para(_)),
        "a number with a dot but no space is not a list"
    );
}

#[test]
fn a_list_is_one_container_with_items() {
    let mut e = typed("\n", 0, 0, "- one");
    e.edit(&Edit::Enter { block: 0, at: 5 });
    e.edit(&Edit::Type {
        block: 0,
        at: 8,
        text: "two".into(),
    });
    assert_eq!(e.blocks().len(), 1, "two items, one block");
    assert!(matches!(node(&e, 0), Node::List(l) if l.items.len() == 2));
    assert_eq!(e.document().blocks, ["- one\n- two"]);
    let mut e = typed("\n", 0, 0, "4. a");
    e.edit(&Edit::Enter { block: 0, at: 4 });
    e.edit(&Edit::Type {
        block: 0,
        at: 8,
        text: "b".into(),
    });
    assert_eq!(
        e.document().blocks,
        ["4. a\n5. b"],
        "the numbers follow from the start"
    );
}

#[test]
fn the_task_box_writes_and_toggles() {
    let mut e = typed("\n", 0, 0, "[] buy milk");
    assert_eq!(e.document().to_markdown(), "- [ ] buy milk\n");
    e.toggle_task(&[0, 0, 0]);
    assert_eq!(e.document().to_markdown(), "- [x] buy milk\n");
    e.toggle_task(&[0, 0, 0]);
    assert_eq!(e.document().to_markdown(), "- [ ] buy milk\n");
    let mut p = typed("plain\n", 0, 0, "");
    p.toggle_task(&[0]);
    assert_eq!(
        p.document().to_markdown(),
        "plain\n",
        "no box, nothing to toggle"
    );
}

#[test]
fn a_fence_opens_a_code_block_with_space_or_enter_and_enter_stays_inside() {
    let e = typed("\n", 0, 0, "``` rust");
    assert_eq!(
        node(&e, 0),
        &Node::Code(String::new(), "rust".into()),
        "a space after the ticks: no language, text behind"
    );
    let mut e = typed("\n", 0, 0, "```rust");
    e.edit(&Edit::Enter { block: 0, at: 7 });
    assert_eq!(node(&e, 0), &Node::Code("rust".into(), String::new()));
    e.edit(&Edit::Type {
        block: 0,
        at: 8,
        text: "fn a() {}".into(),
    });
    e.edit(&Edit::Enter { block: 0, at: 17 });
    e.edit(&Edit::Type {
        block: 0,
        at: 18,
        text: "fn b() {}".into(),
    });
    assert_eq!(
        e.blocks().len(),
        1,
        "Enter in code makes a line, not a block"
    );
    assert_eq!(e.document().blocks[0], "```rust\nfn a() {}\nfn b() {}\n```");
    // Counter-case: ticks with other text are no fence.
    let mut e = typed("```a b\n", 0, 6, "");
    e.edit(&Edit::Enter { block: 0, at: 6 });
    assert!(matches!(node(&e, 0), Node::Para(_)));
}

#[test]
fn three_dashes_and_enter_make_a_rule_and_a_new_paragraph() {
    let mut e = typed("\n", 0, 0, "---");
    e.edit(&Edit::Enter { block: 0, at: 3 });
    assert_eq!(e.document().blocks, ["---", ""]);
    assert_eq!(node(&e, 0), &Node::Rule);
}

#[test]
fn enter_in_lists_continues_and_an_empty_item_leaves_the_list() {
    let mut e = typed("\n", 0, 0, "[] one");
    e.edit(&Edit::Enter { block: 0, at: 12 });
    assert_eq!(
        e.document().blocks,
        ["- [ ] one\n- [ ] "],
        "the next item is an open task"
    );
    e.edit(&Edit::Enter { block: 0, at: 16 });
    assert_eq!(
        e.document().blocks,
        ["- [ ] one", ""],
        "an empty item leaves the list"
    );
    let mut e = typed("\n", 0, 0, "## Head");
    e.edit(&Edit::Enter { block: 0, at: 7 });
    assert_eq!(
        node(&e, 1),
        &Node::Para(String::new()),
        "after a heading comes a paragraph"
    );
}

#[test]
fn an_empty_item_in_the_middle_splits_the_list() {
    let mut e = RichEditor::new();
    e.load(&Doc::parse("1. a\n2. \n3. c\n"));
    e.edit(&Edit::Enter { block: 0, at: 8 });
    assert_eq!(
        e.document().blocks,
        ["1. a", "", "3. c"],
        "the numbers go on where they were"
    );
}

#[test]
fn backspace_gives_the_shortcut_back_once_then_a_heading_turns_into_a_paragraph() {
    let mut e = typed("\n", 0, 0, "## ");
    assert_eq!(node(&e, 0), &Node::Heading(2, String::new()));
    e.edit(&Edit::Backspace { block: 0, at: 3 });
    assert_eq!(
        node(&e, 0),
        &Node::Para("## ".into()),
        "the typed characters come back"
    );
    let mut e = typed("## Old\n", 0, 6, "!");
    e.edit(&Edit::Backspace { block: 0, at: 3 });
    assert_eq!(
        node(&e, 0),
        &Node::Para("Old!".into()),
        "no rule fired: the kind goes, the text stays"
    );
}

#[test]
fn untouched_blocks_stay_byte_for_byte_and_make_no_operation() {
    let text = "* odd bullet\n\n_italic_ and __bold__\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n\n- one\n- two\n\n#   spaced\n";
    let mut e = RichEditor::new();
    let doc = Doc::parse(text);
    e.load(&doc);
    assert_eq!(e.document().to_markdown(), text);
    assert_eq!(e.take_ops().len(), 0);
    assert!(
        matches!(node(&e, 2), Node::Raw(_)),
        "a table is kept as source, not turned into something"
    );
    assert!(
        matches!(node(&e, 3), Node::List(l) if l.items.len() == 2),
        "a list is a list, with its items"
    );
    e.edit(&Edit::Type {
        block: 1,
        at: 0,
        text: "x".into(),
    });
    let ops = e.take_ops();
    assert_eq!(ops.len(), 1, "one block changed, one operation");
    let mut replay = doc;
    apply(&mut replay, &ops).unwrap();
    assert_eq!(replay, e.document());
    assert_eq!(replay.blocks[0], "* odd bullet");
}

#[test]
fn multibyte_text_is_never_cut_inside_a_character() {
    let mut e = typed("## Gr\u{fc}\u{df}e \u{1f468}\u{200d}\u{1f469}\n", 0, 0, "");
    for at in 0..e.document().blocks[0].len() + 2 {
        e.edit(&Edit::Enter { block: 0, at });
        e.edit(&Edit::Type {
            block: 0,
            at,
            text: "\u{e4}".into(),
        });
        e.edit(&Edit::Backspace { block: 0, at });
    }
    assert!(
        !e.blocks().is_empty(),
        "no panic is the claim: offsets inside a character are rounded down"
    );
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
        "2026",
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
