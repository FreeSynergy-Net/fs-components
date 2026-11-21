//! `CommonMark` to the blocks `TextView` draws: [`markdown_blocks`].
//!
//! Pattern: Adapter — a standard parser (`pulldown-cmark`, `CommonMark` with tables, strike-through
//! and task lists, the media type `text/markdown`) is mapped onto the neutral block model. It never
//! fails: what the model cannot express becomes [`Block::Plain`] with the source text. Raw HTML is
//! never interpreted, it is text. Marks are counted in characters (not bytes), like the model says.

use std::ops::Range;

use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use fs_components_core::text::{Align, Block, Inline, ListItem, Mark, MarkKind};

/// How many quotes and lists may sit inside each other; deeper text becomes `Block::Plain`
/// (the text of a person must not overflow the stack).
const MAX_DEPTH: usize = 32;

/// `CommonMark` source to the blocks `TextView` draws. Never fails: what the model cannot express
/// (raw HTML, unknown constructs, nesting deeper than 32 levels) becomes `Block::Plain`.
#[must_use]
pub fn markdown_blocks(source: &str) -> Vec<Block> {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let mut walker = Walker {
        events: Parser::new_ext(source, options).into_offset_iter(),
        source,
    };
    walker.blocks(0).blocks
}

struct Parsed {
    blocks: Vec<Block>,
    task: Option<bool>,
}

struct Walker<'a, I> {
    events: I,
    source: &'a str,
}

impl<'a, I: Iterator<Item = (Event<'a>, Range<usize>)>> Walker<'a, I> {
    /// Blocks up to the end of the container that began just before (or the end of the input).
    fn blocks(&mut self, depth: usize) -> Parsed {
        let mut out = Vec::new();
        let mut run = Run::default();
        let mut task = None;
        while let Some((event, range)) = self.events.next() {
            if run.feed(&event) {
                continue;
            }
            match event {
                Event::Start(Tag::Paragraph | Tag::Heading { .. }) => run.flush(&mut out),
                Event::End(TagEnd::Paragraph) => {
                    if !run.is_empty() {
                        out.push(Block::Paragraph(run.take()));
                    }
                }
                Event::End(TagEnd::Heading(level)) => out.push(Block::Heading {
                    level: level_number(level),
                    text: run.take(),
                }),
                Event::End(_) => {
                    run.flush(&mut out);
                    break;
                }
                Event::Start(tag) => {
                    run.flush(&mut out);
                    out.push(self.container(tag, range, depth));
                }
                Event::Rule => {
                    run.flush(&mut out);
                    out.push(Block::Rule);
                }
                Event::TaskListMarker(done) => task = Some(done),
                _ => {}
            }
        }
        run.flush(&mut out);
        Parsed { blocks: out, task }
    }

    /// One block that starts with `tag`.
    fn container(&mut self, tag: Tag<'a>, range: Range<usize>, depth: usize) -> Block {
        match tag {
            Tag::BlockQuote(_) if depth < MAX_DEPTH => Block::Quote(self.blocks(depth + 1).blocks),
            Tag::List(first) if depth < MAX_DEPTH => self.list(first, depth),
            Tag::CodeBlock(kind) => self.code(&kind),
            Tag::HtmlBlock => {
                let mut text = String::new();
                for (event, _) in &mut self.events {
                    match event {
                        Event::End(TagEnd::HtmlBlock) => break,
                        Event::Html(t) | Event::Text(t) => text.push_str(&t),
                        _ => {}
                    }
                }
                Block::Plain(text.trim_end().to_string())
            }
            Tag::Table(aligns) => self.table(&aligns),
            _ => self.skip(&range),
        }
    }

    fn list(&mut self, first: Option<u64>, depth: usize) -> Block {
        let mut items = Vec::new();
        while let Some((event, _)) = self.events.next() {
            match event {
                Event::Start(Tag::Item) => {
                    let parsed = self.blocks(depth + 1);
                    let mut item = ListItem::new(parsed.blocks);
                    if let Some(done) = parsed.task {
                        item = item.task(done);
                    }
                    items.push(item);
                }
                Event::End(TagEnd::List(_)) => break,
                _ => {}
            }
        }
        Block::List {
            ordered: first.is_some(),
            start: first.unwrap_or(1),
            items,
        }
    }

    fn code(&mut self, kind: &CodeBlockKind<'_>) -> Block {
        let mut text = String::new();
        for (event, _) in &mut self.events {
            match event {
                Event::End(TagEnd::CodeBlock) => break,
                Event::Text(t) => text.push_str(&t),
                _ => {}
            }
        }
        if text.ends_with('\n') {
            text.pop();
        }
        let lang = match kind {
            CodeBlockKind::Fenced(info) => info.split_whitespace().next().unwrap_or("").to_string(),
            CodeBlockKind::Indented => String::new(),
        };
        Block::Code { lang, text }
    }

    fn table(&mut self, aligns: &[Alignment]) -> Block {
        let mut rows = Vec::new();
        let mut row = Vec::new();
        let mut cell = Run::default();
        for (event, _) in &mut self.events {
            if cell.feed(&event) {
                continue;
            }
            match event {
                Event::Start(Tag::TableHead | Tag::TableRow) => row = Vec::new(),
                Event::End(TagEnd::TableHead | TagEnd::TableRow) => {
                    rows.push(std::mem::take(&mut row));
                }
                Event::End(TagEnd::TableCell) => row.push(cell.take()),
                Event::End(TagEnd::Table) => break,
                _ => {}
            }
        }
        Block::Table {
            align: aligns.iter().map(|a| align(*a)).collect(),
            rows,
        }
    }

    /// Skips to the end of what began with a start event; the source text is kept as `Plain`.
    fn skip(&mut self, range: &Range<usize>) -> Block {
        let mut open = 1_usize;
        while open > 0 {
            match self.events.next() {
                Some((Event::Start(_), _)) => open += 1,
                Some((Event::End(_), _)) => open -= 1,
                Some(_) => {}
                None => break,
            }
        }
        Block::Plain(
            self.source
                .get(range.clone())
                .unwrap_or("")
                .trim()
                .to_string(),
        )
    }
}

fn level_number(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn align(a: Alignment) -> Align {
    match a {
        Alignment::None => Align::None,
        Alignment::Left => Align::Left,
        Alignment::Center => Align::Center,
        Alignment::Right => Align::Right,
    }
}

/// A run of inline text being built: characters counted, marks open until their end event.
#[derive(Default)]
struct Run {
    text: String,
    chars: usize,
    marks: Vec<Mark>,
    open: Vec<Option<(MarkKind, usize)>>,
}

impl Run {
    fn push(&mut self, s: &str) {
        self.chars += s.chars().count();
        self.text.push_str(s);
    }

    fn is_empty(&self) -> bool {
        self.text.is_empty() && self.marks.is_empty()
    }

    fn take(&mut self) -> Inline {
        let mut run = std::mem::take(self);
        run.marks
            .sort_by_key(|m| (m.start, std::cmp::Reverse(m.end)));
        Inline::new(run.text, run.marks)
    }

    /// The text so far as a paragraph (when there is some).
    fn flush(&mut self, out: &mut Vec<Block>) {
        if !self.is_empty() {
            out.push(Block::Paragraph(self.take()));
        }
    }

    /// Takes an inline event; false when it is not one.
    fn feed(&mut self, event: &Event<'_>) -> bool {
        match event {
            Event::Text(t) | Event::Html(t) | Event::InlineHtml(t) => self.push(t),
            Event::Code(t) => {
                let start = self.chars;
                self.push(t);
                self.mark(MarkKind::Code, start);
            }
            Event::SoftBreak => self.push(" "),
            Event::HardBreak => self.push("\n"),
            Event::FootnoteReference(label) => self.push(&format!("[^{label}]")),
            Event::Start(Tag::Emphasis) => self.open.push(Some((MarkKind::Emphasis, self.chars))),
            Event::Start(Tag::Strong) => self.open.push(Some((MarkKind::Strong, self.chars))),
            Event::Start(Tag::Strikethrough) => {
                self.open.push(Some((MarkKind::Strike, self.chars)));
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                let kind = MarkKind::Link {
                    href: dest_url.to_string(),
                };
                self.open.push(Some((kind, self.chars)));
            }
            Event::Start(Tag::Image { .. }) => self.open.push(None),
            Event::End(
                TagEnd::Emphasis
                | TagEnd::Strong
                | TagEnd::Strikethrough
                | TagEnd::Link
                | TagEnd::Image,
            ) => {
                if let Some(Some((kind, start))) = self.open.pop() {
                    self.mark(kind, start);
                }
            }
            _ => return false,
        }
        true
    }

    fn mark(&mut self, kind: MarkKind, start: usize) {
        if self.chars > start {
            self.marks.push(Mark::new(kind, start, self.chars));
        }
    }
}
