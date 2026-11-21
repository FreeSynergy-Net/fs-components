//! `markdown_blocks`: CommonMark to the blocks `TextView` draws. Pure data, no window.
//!
//! Each test names the mutant it is meant to catch (the list of the plan
//! `plan-commonmark-textview-2026-11-21.md`, section 5): a change in `markdown.rs` of that kind must
//! turn the test red. They are written to be run by Workflow `testlauf` (stage `ende`), not while building.
//!
//! Pattern: none — a test file.
use std::time::{Duration, Instant};

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

/// The oracle of the tests: text with `code` spans written as backticks to text + marks in characters.
fn coded(raw: &str) -> Inline {
    let mut text = String::new();
    let mut marks = Vec::new();
    for (i, part) in raw.split('`').enumerate() {
        let start = text.chars().count();
        text.push_str(part);
        if i % 2 == 1 {
            marks.push(mark(MarkKind::Code, start, text.chars().count()));
        }
    }
    Inline::new(text, marks)
}

// Mutant "heading level shifted by 1".
#[test]
fn headings_one_to_six_keep_their_level() {
    let got = markdown_blocks("# a\n\n## b\n\n### c\n\n#### d\n\n##### e\n\n###### f\n");
    let want: Vec<Block> = ["a", "b", "c", "d", "e", "f"]
        .iter()
        .enumerate()
        .map(|(i, s)| heading(i as u8 + 1, s))
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

#[test]
fn a_megabyte_paragraph_is_quick() {
    let big = "a".repeat(1 << 20);
    let start = Instant::now();
    let got = markdown_blocks(&big);
    assert!(
        start.elapsed() < Duration::from_secs(2),
        "took {:?}",
        start.elapsed()
    );
    assert_eq!(got.len(), 1);
}

fn depth(blocks: &[Block], quotes: bool) -> usize {
    blocks
        .iter()
        .map(|b| match b {
            Block::Quote(inner) if quotes => 1 + depth(inner, quotes),
            Block::List { items, .. } if !quotes => {
                1 + items
                    .iter()
                    .map(|i| depth(&i.blocks, quotes))
                    .max()
                    .unwrap_or(0)
            }
            _ => 0,
        })
        .max()
        .unwrap_or(0)
}

fn has_plain(blocks: &[Block]) -> bool {
    blocks.iter().any(|b| match b {
        Block::Plain(_) => true,
        Block::Quote(inner) => has_plain(inner),
        Block::List { items, .. } => items.iter().any(|i| has_plain(&i.blocks)),
        _ => false,
    })
}

// Mutant "depth limit removed": the nesting is cut at 32 and the rest stays as text.
#[test]
fn deep_nesting_is_cut_and_the_rest_stays_text() {
    let quotes = markdown_blocks(&format!("{}x\n", "> ".repeat(5000)));
    assert!(
        depth(&quotes, true) <= 33,
        "quote depth {}",
        depth(&quotes, true)
    );
    assert!(has_plain(&quotes));

    let mut nested = String::new();
    for i in 0..200 {
        nested.push_str(&format!("{}- a\n", "  ".repeat(i)));
    }
    let lists = markdown_blocks(&nested);
    assert!(
        depth(&lists, false) <= 33,
        "list depth {}",
        depth(&lists, false)
    );
    assert!(has_plain(&lists));
}

/// The content of the window of `fs-forms-simple` (texts `en.mf2`: tagline, today 1-4, coming 1-9) as the
/// document it will be: the picture must not be worse than today.
#[test]
fn the_content_of_simple_gives_the_expected_blocks() {
    let today = [
        "Opens this window with `fs-forms`, finds its server and checks that both speak the same contract version.",
        "The server `fs-forms serve` answers what it offers (nothing yet) and the support report; everything else says \"unimplemented\".",
        "Says plainly what is not there yet; nothing here pretends to work.",
        "The core (surveys with questions and conditions, anonymous ballots in a separate box, results in batches, counting methods, CSV export, widening over groups) exists as a library and is not connected to this window yet.",
    ];
    let coming = [
        "A list of my surveys with name, state and number of answers, and a search; create, rename, duplicate and delete (the warning says that the answers go with it).",
        "An editor for questions with a preview: add, change, move and delete questions; single and multiple choice, drop-down, short and long text, number, date and time; required questions and checks of the input.",
        "States draft, open, closed and archived, with start and end.",
        "Answering in a window of its own: fill in, save a draft, continue later, submit; a test answer in the preview without saving.",
        "The level of anonymity is chosen and shown to the person who answers, with its limits stated.",
        "The result as a summary with charts and the number of answers, and a CSV export.",
        "Inviting: choose people and groups; at first only invited people can answer.",
        "A public link without an account, protected against mass answers, and embedding in other programs: comes with the server part. Planned, not scheduled.",
        "Date polls (Doodle style) belong to the program Decisions; Forms supplies the input fields that are embedded there.",
    ];
    let tagline = "Goal: Ask questions, collect answers and read the result, and an anonymous answer is really anonymous, not just labelled so.";
    let intro =
        "Everything below is not there yet. The first ones are the plan for the next versions.";

    let mut source = String::from("# FreeSynergy Forms\n\n");
    source.push_str(&format!(
        "{}\n\n{}\n\n> {tagline}\n\n## What it can do today\n\n",
        today[0], today[1]
    ));
    for line in today {
        source.push_str(&format!("- {line}\n"));
    }
    source.push_str(&format!("\n# What comes\n\n{intro}\n\n"));
    for line in coming {
        source.push_str(&format!("- {line}\n"));
    }

    let as_item = |raw: &str| ListItem::new(vec![Block::Paragraph(coded(raw))]);
    let want = vec![
        heading(1, "FreeSynergy Forms"),
        Block::Paragraph(coded(today[0])),
        Block::Paragraph(coded(today[1])),
        Block::Quote(vec![para(tagline)]),
        heading(2, "What it can do today"),
        list(false, 1, today.iter().map(|l| as_item(l)).collect()),
        heading(1, "What comes"),
        para(intro),
        list(false, 1, coming.iter().map(|l| as_item(l)).collect()),
    ];
    assert_eq!(markdown_blocks(&source), want);
}

fn check_inline(inline: &Inline) {
    let chars = inline.text.chars().count();
    for m in &inline.marks {
        assert!(
            m.start < m.end && m.end <= chars,
            "mark {m:?} outside {chars} characters in {:?}",
            inline.text
        );
    }
}

fn check_blocks(blocks: &[Block]) {
    for block in blocks {
        match block {
            Block::Heading { text, .. } | Block::Paragraph(text) => check_inline(text),
            Block::List { items, .. } => items.iter().for_each(|i| check_blocks(&i.blocks)),
            Block::Quote(inner) => check_blocks(inner),
            Block::Table { rows, .. } => rows.iter().flatten().for_each(check_inline),
            _ => {}
        }
    }
}

/// Property over many texts (no new dependency): no mark outside its text, `start < end`.
#[test]
fn no_mark_lies_outside_its_text() {
    let texts = [
        "**a** *b* `c` ~~d~~ [e](f) ![g](h)",
        "äöü **ß😀** `😀😀` [ü](ä)",
        "***a** b*",
        "*a **b*** c",
        "`` `x` `` and ``y``",
        "[a [b](c) d](e)",
        "| ä | `b` |\n|---|:-:|\n| **x** | [y](z) |\n",
        "- [ ] **a**\n  1. *b*\n> `c` ~~d~~\n",
        "<b>**a**</b> &amp; &#128512; \\* \\`",
        "a  \nb\\\nc **d\ne** f",
        "**unclosed *emphasis `code",
    ];
    for text in texts {
        check_blocks(&markdown_blocks(text));
    }
}
