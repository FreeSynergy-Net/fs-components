//! `markdown_blocks`, second part: size, depth, the content of `fs-forms-simple` and the property of the marks.
//! The first part is `markdown_blocks.rs`; same rules (each test names its mutant, run by `testlauf` stage `ende`).
//!
//! Pattern: none — a test file.
use std::time::{Duration, Instant};

use fs_components_widgets::{Block, Inline, ListItem, Mark, MarkKind, markdown_blocks};

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
