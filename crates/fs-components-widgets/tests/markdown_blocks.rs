//! `markdown_blocks`: `CommonMark` to the blocks `TextView` draws. Pure data, no window.
//!
//! Each test names the mutant it is meant to catch (the list of the plan
//! `plan-commonmark-textview-2026-11-21.md`, section 5): a change in `markdown.rs` of that kind must
//! turn the test red. They are written to be run by Workflow `testlauf` (stage `ende`), not while building.
//!
//! Pattern: none — a test file.
use fs_components_widgets::{Align, Block, Inline, ListItem, Mark, MarkKind, markdown_blocks};

fn t(text: &str) -> Inline {
    Inline::plain(text)
}

fn para(text: &str) -> Block {
    Block::Paragraph(t(text))
}

fn heading(level: u8, text: &str) -> Block {
    Block::Heading {
        level,
        text: t(text),
    }
}

fn item(text: &str) -> ListItem {
    ListItem::new(vec![para(text)])
}

fn list(ordered: bool, start: u64, items: Vec<ListItem>) -> Block {
    Block::List {
        ordered,
        start,
        items,
    }
}

fn mark(kind: MarkKind, start: usize, end: usize) -> Mark {
    Mark::new(kind, start, end)
}

// Mutant "heading level shifted by 1".
#[test]
fn headings_one_to_six_keep_their_level() {
    let got = markdown_blocks("# a\n\n## b\n\n### c\n\n#### d\n\n##### e\n\n###### f\n");
    let want: Vec<Block> = ["a", "b", "c", "d", "e", "f"]
        .iter()
        .enumerate()
        .map(|(i, s)| heading(u8::try_from(i).unwrap() + 1, s))
        .collect();
    assert_eq!(got, want);
}

#[test]
fn a_setext_heading_and_a_paragraph() {
    assert_eq!(
        markdown_blocks("Title\n=====\n\nbody\n"),
        vec![heading(1, "Title"), para("body")]
    );
}

// Mutant "mark end moved by 1".
#[test]
fn a_code_span_covers_exactly_its_characters() {
    assert_eq!(
        markdown_blocks("run `fs-forms` now"),
        vec![Block::Paragraph(Inline::new(
            "run fs-forms now",
            vec![mark(MarkKind::Code, 4, 12)]
        ))]
    );
}

// Mutant "byte offsets instead of character offsets": umlauts are 2 bytes, the emoji 4.
#[test]
fn offsets_count_characters_not_bytes() {
    assert_eq!(
        markdown_blocks("äö `x` 😀 `yz`"),
        vec![Block::Paragraph(Inline::new(
            "äö x 😀 yz",
            vec![mark(MarkKind::Code, 3, 4), mark(MarkKind::Code, 7, 9)]
        ))]
    );
    assert_eq!(
        markdown_blocks("**ü😀** end"),
        vec![Block::Paragraph(Inline::new(
            "ü😀 end",
            vec![mark(MarkKind::Strong, 0, 2)]
        ))]
    );
}

#[test]
fn a_link_keeps_its_target_as_written_and_nests_with_emphasis() {
    assert_eq!(
        markdown_blocks("[a *b*](http://x/?q=1&r=ä \"title\")"),
        vec![Block::Paragraph(Inline::new(
            "a b",
            vec![
                mark(
                    MarkKind::Link {
                        href: "http://x/?q=1&r=ä".to_string()
                    },
                    0,
                    3
                ),
                mark(MarkKind::Emphasis, 2, 3),
            ]
        ))]
    );
}

#[test]
fn emphasis_inside_emphasis_nests() {
    assert_eq!(
        markdown_blocks("*a **b** c*"),
        vec![Block::Paragraph(Inline::new(
            "a b c",
            vec![mark(MarkKind::Emphasis, 0, 5), mark(MarkKind::Strong, 2, 3)]
        ))]
    );
}

// Mutant "start of the numbering ignored".
#[test]
fn a_numbered_list_keeps_its_first_number() {
    assert_eq!(
        markdown_blocks("3. a\n4. b\n"),
        vec![list(true, 3, vec![item("a"), item("b")])]
    );
    assert_eq!(
        markdown_blocks("- a\n- b\n"),
        vec![list(false, 1, vec![item("a"), item("b")])]
    );
}

#[test]
fn a_nested_list_sits_inside_its_item() {
    assert_eq!(
        markdown_blocks("- a\n  - b\n- c\n"),
        vec![list(
            false,
            1,
            vec![
                ListItem::new(vec![para("a"), list(false, 1, vec![item("b")])]),
                item("c"),
            ]
        )]
    );
}

// Mutant "task state swapped".
#[test]
fn task_items_keep_their_state() {
    assert_eq!(
        markdown_blocks("- [x] done\n- [ ] open\n- plain\n"),
        vec![list(
            false,
            1,
            vec![
                item("done").task(true),
                item("open").task(false),
                item("plain")
            ]
        )]
    );
}

#[test]
fn a_quote_can_hold_a_list() {
    assert_eq!(
        markdown_blocks("> - a\n> - b\n"),
        vec![Block::Quote(vec![list(
            false,
            1,
            vec![item("a"), item("b")]
        )])]
    );
}

// Mutant "language not cut to the first word" and "last line break stays".
#[test]
fn a_fence_gives_the_first_word_as_language_and_no_trailing_break() {
    assert_eq!(
        markdown_blocks("```rust ignore\nfn x() {}\nfn y() {}\n```\n"),
        vec![Block::Code {
            lang: "rust".to_string(),
            text: "fn x() {}\nfn y() {}".to_string()
        }]
    );
    assert_eq!(
        markdown_blocks("```\na\n```\n"),
        vec![Block::Code {
            lang: String::new(),
            text: "a".to_string()
        }]
    );
    assert_eq!(
        markdown_blocks("    indented\n"),
        vec![Block::Code {
            lang: String::new(),
            text: "indented".to_string()
        }]
    );
}

#[test]
fn a_rule_separates_two_paragraphs() {
    assert_eq!(
        markdown_blocks("a\n\n---\n\nb\n"),
        vec![para("a"), Block::Rule, para("b")]
    );
}

// Mutant "strike-through and table option switched off": without them the text stays a paragraph.
#[test]
fn a_table_has_alignment_and_a_header_row_and_cells_hold_marks() {
    assert_eq!(
        markdown_blocks("| a | b |\n|:--|--:|\n| 1 | ~~2~~ |\n"),
        vec![Block::Table {
            align: vec![Align::Left, Align::Right],
            rows: vec![
                vec![t("a"), t("b")],
                vec![t("1"), Inline::new("2", vec![mark(MarkKind::Strike, 0, 1)])],
            ],
        }]
    );
    assert_eq!(
        markdown_blocks("~~gone~~ stays"),
        vec![Block::Paragraph(Inline::new(
            "gone stays",
            vec![mark(MarkKind::Strike, 0, 4)]
        ))]
    );
}

// Mutant "soft break stays a line break".
#[test]
fn a_soft_break_is_a_space_and_a_hard_break_a_line_break() {
    assert_eq!(markdown_blocks("a\nb\n"), vec![para("a b")]);
    assert_eq!(markdown_blocks("a  \nb\n"), vec![para("a\nb")]);
}

// Mutant "HTML dropped silently instead of Plain".
#[test]
fn raw_html_is_text_and_never_lost() {
    assert_eq!(
        markdown_blocks("<script>x</script>\n"),
        vec![Block::Plain("<script>x</script>".to_string())]
    );
    assert_eq!(
        markdown_blocks("<div>\nhi\n</div>\n\ntext\n"),
        vec![Block::Plain("<div>\nhi\n</div>".to_string()), para("text")]
    );
    assert_eq!(markdown_blocks("a <b>x</b>"), vec![para("a <b>x</b>")]);
}

#[test]
fn an_image_shows_its_alt_text() {
    assert_eq!(markdown_blocks("![alt](u.png)"), vec![para("alt")]);
}

#[test]
fn nothing_gives_no_blocks_and_odd_input_does_not_panic() {
    assert_eq!(markdown_blocks(""), vec![]);
    assert_eq!(markdown_blocks("\n\n"), vec![]);
    assert_eq!(markdown_blocks("   \n  "), vec![]);
    assert_eq!(markdown_blocks("a\0b").len(), 1);
}
