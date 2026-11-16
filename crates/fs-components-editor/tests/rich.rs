//! The rich editor spike: the shortcuts turn into typed blocks at once, the contract holds, untouched blocks stay as they
//! were. Each rule has a case that must fire and a counter-case that must not.
//!
//! Tests: model tests, no window.

use fs_components_editor::rich::{Kind, RichEditor};
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

fn kind_text(e: &RichEditor, i: usize) -> (Kind, String) {
    (e.blocks()[i].kind.clone(), e.blocks()[i].text.clone())
}

#[test]
fn it_passes_the_conformance_collection() {
    conformance::check(&|| -> Box<dyn TextEditor> { Box::new(RichEditor::new()) });
}

#[test]
fn heading_shortcuts_fire_by_level_and_not_with_seven_hashes() {
    for n in 1..=6u8 {
        let e = typed("\n", 0, 0, &format!("{} Title", "#".repeat(usize::from(n))));
        assert_eq!(kind_text(&e, 0), (Kind::Heading(n), "Title".into()));
        assert_eq!(
            e.document().blocks[0],
            format!("{} Title", "#".repeat(usize::from(n)))
        );
    }
    let e = typed("\n", 0, 0, "####### x");
    assert_eq!(
        kind_text(&e, 0),
        (Kind::Paragraph, "####### x".into()),
        "seven hashes are text"
    );
}

#[test]
fn a_shortcut_inside_text_does_not_fire() {
    let e = typed("word\n", 0, 4, " # ");
    assert_eq!(kind_text(&e, 0).0, Kind::Paragraph);
    let e = typed("a\n", 0, 0, "## ");
    assert_eq!(
        kind_text(&e, 0),
        (Kind::Heading(2), "a".into()),
        "at the start it fires and keeps the text behind"
    );
}

#[test]
fn list_task_quote_and_numbered_shortcuts() {
    for (typed_text, kind) in [
        ("- ", Kind::Bullet),
        ("* ", Kind::Bullet),
        ("> ", Kind::Quote),
        ("[] ", Kind::Task(false)),
        ("[x] ", Kind::Task(true)),
        ("3. ", Kind::Numbered(3)),
        ("- [] ", Kind::Task(false)),
    ] {
        let e = typed("\n", 0, 0, &format!("{typed_text}item"));
        assert_eq!(kind_text(&e, 0), (kind, "item".into()), "{typed_text}");
    }
    let e = typed("\n", 0, 0, "-x ");
    assert_eq!(
        kind_text(&e, 0).0,
        Kind::Paragraph,
        "a dash glued to a word is not a list"
    );
    let e = typed("\n", 0, 0, "1.5 ");
    assert_eq!(
        kind_text(&e, 0).0,
        Kind::Paragraph,
        "a number with a dot but no space is not a list"
    );
}

#[test]
fn the_task_box_writes_and_toggles() {
    let mut e = typed("\n", 0, 0, "[] buy milk");
    assert_eq!(e.document().to_markdown(), "- [ ] buy milk\n");
    e.toggle_task(0);
    assert_eq!(e.document().to_markdown(), "- [x] buy milk\n");
    e.toggle_task(0);
    assert_eq!(e.document().to_markdown(), "- [ ] buy milk\n");
}

#[test]
fn a_fence_opens_a_code_block_with_space_or_enter_and_enter_stays_inside() {
    let e = typed("\n", 0, 0, "``` rust");
    assert_eq!(
        kind_text(&e, 0),
        (Kind::Code(String::new()), "rust".into()),
        "a space after the ticks: no language, text behind"
    );
    let mut e = typed("\n", 0, 0, "```rust");
    e.edit(&Edit::Enter { block: 0, at: 7 });
    assert_eq!(kind_text(&e, 0), (Kind::Code("rust".into()), String::new()));
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
    assert_eq!(e.blocks()[0].kind, Kind::Paragraph);
}

#[test]
fn three_dashes_and_enter_make_a_rule_and_a_new_paragraph() {
    let mut e = typed("\n", 0, 0, "---");
    e.edit(&Edit::Enter { block: 0, at: 3 });
    assert_eq!(e.document().blocks, ["---", ""]);
    assert_eq!(e.blocks()[0].kind, Kind::Rule);
}

#[test]
fn enter_in_lists_continues_and_an_empty_item_leaves_the_list() {
    let mut e = typed("\n", 0, 0, "[] one");
    e.edit(&Edit::Enter { block: 0, at: 12 });
    assert_eq!(
        kind_text(&e, 1),
        (Kind::Task(false), String::new()),
        "the next item is an open task"
    );
    e.edit(&Edit::Enter { block: 1, at: 6 });
    assert_eq!(
        e.blocks()[1].kind,
        Kind::Paragraph,
        "an empty item leaves the list"
    );
    let mut e = typed("\n", 0, 0, "## Head");
    e.edit(&Edit::Enter { block: 0, at: 7 });
    assert_eq!(
        e.blocks()[1].kind,
        Kind::Paragraph,
        "after a heading comes a paragraph"
    );
    let mut e = typed("\n", 0, 0, "4. a");
    e.edit(&Edit::Enter { block: 0, at: 6 });
    assert_eq!(e.blocks()[1].kind, Kind::Numbered(5));
}

#[test]
fn backspace_gives_the_shortcut_back_once_then_a_heading_turns_into_a_paragraph() {
    let mut e = typed("\n", 0, 0, "## ");
    assert_eq!(e.blocks()[0].kind, Kind::Heading(2));
    e.backspace(0, 3);
    assert_eq!(
        kind_text(&e, 0),
        (Kind::Paragraph, "## ".into()),
        "the typed characters come back"
    );
    let mut e = typed("## Old\n", 0, 6, "!");
    e.backspace(0, 3);
    assert_eq!(
        kind_text(&e, 0),
        (Kind::Paragraph, "Old!".into()),
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
    assert!(e.take_ops().is_empty());
    assert_eq!(
        e.blocks()[2].kind,
        Kind::Raw,
        "a table is kept as source, not turned into something"
    );
    assert_eq!(e.blocks()[3].kind, Kind::Raw, "a multi-line list too");
    // One word changed in block 1: one operation, the others are not touched.
    e.edit(&Edit::Type {
        block: 1,
        at: 0,
        text: "x".into(),
    });
    let ops = e.take_ops();
    assert_eq!(ops.len(), 1);
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
        e.backspace(0, at);
    }
    // No panic is the claim: offsets inside a character are rounded down.
    assert!(!e.blocks().is_empty());
}
