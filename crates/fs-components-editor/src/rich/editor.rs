//! The editor over the typed blocks: typing, Enter, Backspace, the task box.

use super::block::{Kind, RichBlock};
use super::rules::{self, Turn};
use crate::{Doc, Edit, Op, TextEditor, diff};

/// The last shortcut that fired, so that Backspace right after gives the typed characters back (as in tiptap).
#[derive(Clone, Debug)]
struct Fired {
    block: usize,
    typed: String,
}

/// The rich editor: typed blocks, and the document as it was loaded to find the operations against.
#[derive(Debug, Default)]
pub struct RichEditor {
    blocks: Vec<RichBlock>,
    loaded: Doc,
    fired: Option<Fired>,
}

fn floor(s: &str, mut at: usize) -> usize {
    at = at.min(s.len());
    while !s.is_char_boundary(at) {
        at -= 1;
    }
    at
}

impl RichEditor {
    /// An editor with no document.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The typed blocks as they are now (what a frontend draws).
    #[must_use]
    pub fn blocks(&self) -> &[RichBlock] {
        &self.blocks
    }

    /// The offset in the text of a block for the offset `at` in its source (the marker is not typed into).
    fn text_at(&self, block: usize, at: usize) -> usize {
        let b = &self.blocks[block];
        floor(&b.text, at.saturating_sub(b.marker_len()))
    }

    fn turn_into(&mut self, block: usize, turn: Turn, typed: String) {
        let b = &mut self.blocks[block];
        b.kind = turn.kind;
        b.text = turn.text;
        b.source = None;
        self.fired = Some(Fired { block, typed });
    }

    /// Type `c` at offset `t` of the text of `block`; the offset of the cursor afterwards.
    fn type_char(&mut self, block: usize, t: usize, c: char) -> usize {
        let b = &mut self.blocks[block];
        if b.kind == Kind::Rule {
            return t;
        }
        b.text.insert(t, c);
        b.source = None;
        if c == ' ' {
            let (typed, rest) = b.text.split_at(t + 1);
            if let Some(turn) = rules::on_space(&b.kind, typed, rest) {
                let typed = typed.to_owned();
                self.turn_into(block, turn, typed);
                return 0;
            }
        }
        t + c.len_utf8()
    }

    /// Enter in `block` at the source offset `at`.
    fn enter(&mut self, block: usize, at: usize) {
        let t = self.text_at(block, at);
        let b = &mut self.blocks[block];
        match b.kind.clone() {
            Kind::Code(_) => {
                b.text.insert(t, '\n');
                b.source = None;
                return;
            }
            Kind::Rule => {}
            Kind::Raw => {
                let (left, right) = b.text.split_at(floor(&b.text, at));
                let (left, right) = (RichBlock::parse(left), RichBlock::parse(right));
                self.blocks.splice(block..=block, [left, right]);
                return;
            }
            kind if t == b.text.len() => {
                if let Some(turn) = rules::on_enter(&kind, &b.text) {
                    let typed = b.text.clone();
                    let rule = turn.kind == Kind::Rule;
                    self.turn_into(block, turn, typed);
                    if rule {
                        self.blocks
                            .insert(block + 1, RichBlock::new(Kind::Paragraph, ""));
                    }
                    return;
                }
                // An empty item leaves the list.
                if b.text.is_empty()
                    && matches!(
                        kind,
                        Kind::Bullet | Kind::Numbered(_) | Kind::Task(_) | Kind::Quote
                    )
                {
                    b.kind = Kind::Paragraph;
                    b.source = None;
                    return;
                }
            }
            _ => {}
        }
        let b = &mut self.blocks[block];
        let rest = b.text.split_off(t);
        if !rest.is_empty() || t == 0 {
            b.source = None;
        }
        let next = match &b.kind {
            Kind::Bullet => Kind::Bullet,
            Kind::Numbered(n) => Kind::Numbered(n + 1),
            Kind::Task(_) => Kind::Task(false),
            Kind::Quote => Kind::Quote,
            _ => Kind::Paragraph,
        };
        self.blocks.insert(block + 1, RichBlock::new(next, rest));
    }

    /// Backspace at the source offset `at`. At the start of a block: the shortcut that has just fired is given back; else a
    /// typed block becomes a paragraph; else the block joins the one above. Elsewhere one character goes.
    pub fn backspace(&mut self, block: usize, at: usize) {
        if block >= self.blocks.len() {
            return;
        }
        let t = self.text_at(block, at);
        if t > 0 {
            let b = &mut self.blocks[block];
            if let Some(c) = b.text[..t].chars().next_back() {
                b.text.replace_range(t - c.len_utf8()..t, "");
                b.source = None;
            }
            return;
        }
        if let Some(f) = self.fired.take().filter(|f| f.block == block) {
            let b = &mut self.blocks[block];
            b.text = format!("{}{}", f.typed, b.text);
            b.kind = Kind::Paragraph;
            b.source = None;
        } else if !matches!(self.blocks[block].kind, Kind::Paragraph | Kind::Raw) {
            let b = &mut self.blocks[block];
            if b.kind == Kind::Rule {
                self.blocks.remove(block);
            } else {
                b.kind = Kind::Paragraph;
                b.source = None;
            }
        } else if block > 0
            && !matches!(
                self.blocks[block - 1].kind,
                Kind::Rule | Kind::Code(_) | Kind::Raw
            )
        {
            let gone = self.blocks.remove(block);
            let above = &mut self.blocks[block - 1];
            above.text.push_str(&gone.text);
            above.source = None;
        }
    }

    /// Check or uncheck the task at `block`.
    pub fn toggle_task(&mut self, block: usize) {
        if let Some(b) = self.blocks.get_mut(block)
            && let Kind::Task(done) = b.kind
        {
            b.kind = Kind::Task(!done);
            b.source = None;
        }
    }
}

impl TextEditor for RichEditor {
    fn name(&self) -> &'static str {
        "rich"
    }

    fn load(&mut self, doc: &Doc) {
        self.blocks = doc.blocks.iter().map(|s| RichBlock::parse(s)).collect();
        self.loaded = doc.clone();
        self.fired = None;
    }

    fn document(&self) -> Doc {
        Doc::new(self.blocks.iter().map(RichBlock::to_source).collect())
    }

    fn edit(&mut self, edit: &Edit) {
        match edit {
            Edit::Type { block, at, text } if *block < self.blocks.len() => {
                // Typed key by key: the rules see every Space.
                let mut t = self.text_at(*block, *at);
                for c in text.chars().filter(|&c| c != '\n') {
                    t = self.type_char(*block, t, c);
                }
            }
            Edit::Enter { block, at } if *block < self.blocks.len() => self.enter(*block, *at),
            _ => {}
        }
    }

    fn take_ops(&mut self) -> Vec<Op> {
        let now = self.document();
        let ops = diff(&self.loaded, &now);
        self.loaded = now;
        ops
    }
}
