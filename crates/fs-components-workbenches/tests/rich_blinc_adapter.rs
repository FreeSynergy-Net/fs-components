//! Spike, the adapter route: the Blinc `rich_text_editor` model (`RichDocument`, its own edit functions) with the input rules of
//! the `RichEditor` run on top. What works, and the two limits that show in the tests (no task and no code kind in Blinc).
//!
//! Tests: model tests on Blinc's public edit functions, no window.

use blinc_core::Color;
use blinc_layout::widgets::rich_text_editor::cursor::{ActiveFormat, DocPosition, Selection};
use blinc_layout::widgets::rich_text_editor::{
    BlockKind, RichDocument, delete_selection, insert_char, set_block_kind, split_block,
};
use fs_components_editor::rich::rules::{self, Place, Turn};

/// One key into the document: insert it; at a Space let the rules look at the text in front of the cursor.
fn key(doc: &mut RichDocument, pos: DocPosition, c: char) -> DocPosition {
    let next = insert_char(doc, pos, c, &ActiveFormat::default());
    if c != ' ' {
        return next;
    }
    let block = &doc.blocks[next.block];
    let typed: String = block.lines[0].text.chars().take(next.col).collect();
    let Some(turn) = rules::on_space(&typed, Place::Page) else {
        return next;
    };
    let kind = match turn {
        Turn::Heading(n) => BlockKind::Heading(n),
        Turn::Bullet => BlockKind::BulletItem,
        Turn::Numbered(_) => BlockKind::NumberedItem,
        Turn::Quote => BlockKind::Quote,
        // No kind in Blinc for these: the rule is not applied, the text stays as typed.
        _ => return next,
    };
    let start = DocPosition::new(next.block, 0, 0);
    let after = delete_selection(
        doc,
        Selection {
            anchor: start,
            head: next,
        },
    );
    set_block_kind(doc, Selection::empty(after), kind);
    after
}

fn type_all(doc: &mut RichDocument, mut pos: DocPosition, s: &str) -> DocPosition {
    for c in s.chars() {
        pos = key(doc, pos, c);
    }
    pos
}

fn fresh() -> RichDocument {
    RichDocument::from_blocks(vec![
        blinc_layout::widgets::rich_text_editor::Block::paragraph_empty(),
    ])
}

#[test]
fn heading_list_and_quote_shortcuts_work_on_blinc_blocks() {
    for (keys, kind, text) in [
        ("## Title", BlockKind::Heading(2), "Title"),
        ("- item", BlockKind::BulletItem, "item"),
        ("3. step", BlockKind::NumberedItem, "step"),
        ("> said", BlockKind::Quote, "said"),
    ] {
        let mut doc = fresh();
        type_all(&mut doc, DocPosition::ZERO, keys);
        assert_eq!(doc.blocks[0].kind, kind, "{keys}");
        assert_eq!(doc.blocks[0].plain_text(), text, "{keys}");
    }
}

#[test]
fn counter_cases_stay_text() {
    let mut doc = fresh();
    type_all(&mut doc, DocPosition::ZERO, "a ## b");
    assert_eq!(doc.blocks[0].kind, BlockKind::Paragraph);
    assert_eq!(doc.blocks[0].plain_text(), "a ## b");
}

#[test]
fn task_and_code_have_no_kind_in_blinc() {
    let mut doc = fresh();
    type_all(&mut doc, DocPosition::ZERO, "[] todo");
    assert_eq!(
        doc.blocks[0].kind,
        BlockKind::Paragraph,
        "no task kind: the text '[] todo' stays"
    );
    let mut doc = fresh();
    type_all(&mut doc, DocPosition::ZERO, "``` x");
    assert_eq!(doc.blocks[0].kind, BlockKind::Paragraph, "no code kind");
}

#[test]
fn enter_after_a_heading_is_a_paragraph_and_an_empty_item_leaves_the_list() {
    let mut doc = fresh();
    let p = type_all(&mut doc, DocPosition::ZERO, "# Big");
    split_block(&mut doc, p);
    assert_eq!(
        doc.blocks[1].kind,
        BlockKind::Paragraph,
        "Blinc itself starts a paragraph after a heading"
    );
    let mut doc = fresh();
    let p = type_all(&mut doc, DocPosition::ZERO, "- one");
    let p = split_block(&mut doc, p);
    assert_eq!(doc.blocks[1].kind, BlockKind::BulletItem);
    split_block(&mut doc, p);
    assert_eq!(
        doc.blocks[1].kind,
        BlockKind::Paragraph,
        "an empty item leaves the list"
    );
}

#[test]
fn spans_are_byte_offsets_and_columns_are_characters() {
    // The adapter has to convert: the same position is a different number in Blinc's spans and in a column.
    let mut doc = fresh();
    type_all(&mut doc, DocPosition::ZERO, "\u{fc}\u{fc}x");
    let line = &doc.blocks[0].lines[0];
    assert_eq!(line.text.chars().count(), 3);
    assert_eq!(line.text.len(), 5, "bytes differ from characters");
    let _ = Color::BLACK;
}
