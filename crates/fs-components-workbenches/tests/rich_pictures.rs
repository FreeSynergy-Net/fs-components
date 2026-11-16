//! Spike pictures for writing like Notion (U3): what the person sees while typing a heading, a task box and a code block.
//!
//! Three kinds of pictures, each in the style of the process (`FS_ABNAHME_STYLE`, as in the acceptance pictures):
//!  1. `heute`: the block editor as it is, the shortcut stays visible as raw Markdown text;
//!  2. `rich`: the typed blocks of the `RichEditor` (after the scripted typing) drawn by the reader's own text view
//!     (heading size, real task boxes, code block ground);
//!  3. `blinc`: the Blinc `rich_text_editor` widget over a `RichDocument` that an adapter fills from the same typed blocks,
//!     the rules of the `RichEditor` run on its text (what a Blinc-adapter route would look like).
//!
//! Tests: one test that only draws; it writes pictures when `FS_RICH_DIR` names a directory (else it does nothing).

mod common;

use std::sync::Arc;

use blinc_core::Color;
use blinc_layout::widgets::rich_text_editor::{
    RichDocument, RichTextTheme, rich_text_editor, rich_text_state,
};
use common::Editable;
use common::draw::{Scratch, fonts_work};
use common::pages::Pages;
use fs_components_editor::rich::{Node, RichBlock, RichEditor};
use fs_components_editor::{Doc, Edit, TextEditor};
use fs_components_render::{Canvas, draw_to_png};
use fs_components_workbenches::{
    Block, Document, DocumentSource, Entry, Hit, Inline, Layout, LinkTarget, ListItem, Picture,
    SourceError, Step, snapshot,
};

/// The keys the person presses, in order: a heading, a paragraph, two task items, a code block, a rule.
fn script(with_space: bool) -> RichEditor {
    let mut e = RichEditor::new();
    e.load(&Doc::new(vec![String::new()]));
    let h = if with_space {
        "## Plan for Monday"
    } else {
        "##Plan for Monday"
    };
    let mut at = 0;
    let mut block = 0;
    let type_in = |e: &mut RichEditor, block: usize, text: &str, at: &mut usize| {
        e.edit(&Edit::Type {
            block,
            at: *at,
            text: text.into(),
        });
    };
    type_in(&mut e, block, h, &mut at);
    let end = |e: &RichEditor, b: usize| e.document().blocks[b].len();
    e.edit(&Edit::Enter {
        block,
        at: end(&e, block),
    });
    block += 1;
    type_in(&mut e, block, "Things to do before the meeting:", &mut 0);
    e.edit(&Edit::Enter {
        block,
        at: end(&e, block),
    });
    block += 1;
    type_in(&mut e, block, "[] write the summary", &mut 0);
    e.edit(&Edit::Enter {
        block,
        at: end(&e, block),
    });
    block += 1;
    type_in(&mut e, block, "write the agenda", &mut 0);
    e.edit(&Edit::Enter {
        block,
        at: end(&e, block),
    });
    e.edit(&Edit::Enter {
        block: block + 1,
        at: 6,
    });
    block += 1;
    type_in(&mut e, block, "```rust", &mut 0);
    e.edit(&Edit::Enter { block, at: 7 });
    type_in(&mut e, block, "fn main() {", &mut 8);
    e.edit(&Edit::Enter {
        block,
        at: end(&e, block) - 4,
    });
    let n = end(&e, block) - 4;
    type_in(&mut e, block, "    println!(\"hello\");", &mut n.clone());
    let _ = n;
    e
}

fn text_inline(t: &str) -> Inline {
    Inline::plain(t)
}

/// The typed tree as blocks of the reader's text view.
fn view_node(n: &Node) -> Block {
    match n {
        Node::Heading(level, t) => Block::Heading {
            level: *level,
            text: text_inline(t),
        },
        Node::Para(t) | Node::Raw(t) => Block::Paragraph(text_inline(t)),
        Node::Quote(c) => Block::Quote(c.iter().map(view_node).collect()),
        Node::Code(l, t) => Block::Code {
            lang: l.clone(),
            text: t.clone(),
        },
        Node::Rule => Block::Rule,
        Node::List(l) => Block::List {
            ordered: l.ordered,
            start: u64::from(l.start),
            items: l
                .items
                .iter()
                .map(|i| {
                    let it = ListItem::new(i.children.iter().map(view_node).collect());
                    i.task.map_or(it.clone(), |d| it.task(d))
                })
                .collect(),
        },
    }
}

fn view_blocks(blocks: &[RichBlock]) -> Vec<Block> {
    blocks.iter().map(|b| view_node(&b.node)).collect()
}

struct One(Vec<Block>);

impl DocumentSource for One {
    fn entries(&self, _p: &str) -> Result<Vec<Entry>, SourceError> {
        Ok(vec![Entry::new("seite.md", "Seite")])
    }
    fn read(&self, path: &str) -> Result<Document, SourceError> {
        Ok(Document::new(path, self.0.clone()))
    }
    fn resolve(&self, _f: &str, h: &str) -> LinkTarget {
        LinkTarget::Missing(h.into())
    }
    fn find(&self, _t: &str) -> Result<Vec<Hit>, SourceError> {
        Ok(vec![])
    }
}

/// The adapter route: the typed tree flattened into Blinc's flat blocks (it has no task box and no code kind).
fn blinc_document(blocks: &[RichBlock]) -> RichDocument {
    use blinc_layout::widgets::rich_text_editor::Block as B;
    let c = Color::BLACK;
    let mut doc = RichDocument::from_blocks(vec![]);
    for b in blocks {
        flatten(&b.node, c, &mut doc.blocks);
    }
    let _ = B::divider;
    doc
}

fn flatten(n: &Node, c: Color, out: &mut Vec<blinc_layout::widgets::rich_text_editor::Block>) {
    use blinc_layout::widgets::rich_text_editor::Block as B;
    match n {
        Node::Heading(l, t) => out.push(B::heading(*l, t.clone(), c)),
        Node::Para(t) | Node::Raw(t) | Node::Code(_, t) => out.push(B::paragraph(t.clone(), c)),
        Node::Rule => out.push(B::divider()),
        Node::Quote(ch) => {
            for x in ch {
                if let Some(t) = x.text() {
                    out.push(B::quote(t.to_owned(), c));
                }
            }
        }
        Node::List(l) => {
            for i in &l.items {
                let t = i.children.first().and_then(Node::text).unwrap_or("");
                let line = match i.task {
                    Some(d) => format!("{} {t}", if d { "[x]" } else { "[ ]" }),
                    None => t.to_owned(),
                };
                out.push(if l.ordered {
                    B::numbered(line, c)
                } else {
                    B::bullet(line, c)
                });
            }
        }
    }
}

fn out_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("FS_RICH_DIR").map(Into::into)
}

fn style() -> String {
    std::env::var("FS_ABNAHME_STYLE")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "standard".into())
}

fn keep(tmp: &Scratch, from: &str, name: &str) {
    let dir = out_dir().unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(tmp.file(from), dir.join(format!("{}-{name}", style()))).unwrap();
}

#[test]
fn draw_the_spike_pictures() {
    if out_dir().is_none() {
        return;
    }
    fs_components_render::use_style(fs_components_styles::StyleId::from_name(&style()));
    let tmp = Scratch::new();
    assert!(fonts_work(&tmp), "no fonts or no graphics");
    let _ = Pages;
    let typed = script(true);
    let markdown = typed.document().to_markdown();
    std::fs::write(tmp.file("keys.md"), &markdown).unwrap();

    // 1. today: the same text in the block editor, as Markdown characters.
    let source = Editable::with(&[("seite.md", &markdown)]);
    let mut p = Picture::new(Arc::new(source), 1000, 700);
    p.layout = Layout::Notion;
    p.steps = vec![Step::Open("seite.md".into()), Step::Edit];
    snapshot(p, &tmp.file("a.png")).expect("picture a");
    keep(&tmp, "a.png", "01-heute-block-editor.png");

    // 2. the typed blocks, drawn formatted; and the same keys without the Space that fires the rule.
    for (name, with_space) in [
        ("02-rich-formatiert.png", true),
        ("03-rich-ohne-leerzeichen.png", false),
    ] {
        let e = script(with_space);
        let mut p = Picture::new(Arc::new(One(view_blocks(e.blocks()))), 1000, 700);
        p.layout = Layout::Notion;
        p.steps = vec![Step::Open("seite.md".into())];
        snapshot(p, &tmp.file("b.png")).expect("picture b");
        keep(&tmp, "b.png", name);
    }

    // 3. the Blinc widget.
    let doc = blinc_document(typed.blocks());
    let state = rich_text_state(doc);
    draw_to_png(
        &Canvas::new(800, 600, "").with_window_styles(),
        |w, _| {
            blinc_layout::div::div()
                .w(w)
                .h(600.0)
                .bg(Color::WHITE)
                .p(24.0)
                .child(rich_text_editor(&state, RichTextTheme::default(), w - 48.0))
        },
        || {},
        &tmp.file("c.png"),
    )
    .expect("picture c");
    keep(&tmp, "c.png", "04-blinc-rich-text-editor.png");
}
