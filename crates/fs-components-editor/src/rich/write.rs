//! Writing the typed tree as canonical Markdown source.
//!
//! Pattern: Adapter — the inverse of [`super::read`]: a [`Node`] becomes source text, and the offset of the text of each leaf
//! in that source is given again in reading order. A block nobody changed is not written at all (the editor keeps its source).

use super::node::{List, Node};
use super::read::{is_block_marker, task_box};

/// Where a paragraph is written, which decides what it must be kept from being read as.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ctx {
    /// A block of the page or of a quote: any block marker at its start.
    Block,
    /// The text of a list item that has no task box: only a task box.
    Item,
    /// The text of a task item: nothing.
    Text,
}

/// A paragraph written so that reading it gives the same paragraph: a backslash in front of what would be a marker
/// (`## x` as `\## x`, `- x` as `\- x`, `1. x` as `\1. x`, `[x] x` as `\[x] x`), and where the text starts after it.
fn para(text: &str, ctx: Ctx) -> (String, Vec<usize>) {
    let marker: fn(&str) -> bool = match ctx {
        Ctx::Block => is_block_marker,
        Ctx::Item => |t| task_box(t).is_some(),
        Ctx::Text => |_| false,
    };
    let first = text.split('\n').next().unwrap_or("");
    if marker(first.trim_start_matches('\\')) {
        (format!("\\{text}"), vec![1])
    } else {
        (text.to_owned(), vec![0])
    }
}

/// The canonical source of `node` and the offsets of the text of its leaves in it.
#[must_use]
pub fn write(node: &Node) -> (String, Vec<usize>) {
    emit(node, false)
}

/// The source of `node` to be stored: as [`write`], but a paragraph that would be read as something else gets a backslash in
/// front (`## x` is `\## x`), so that reading it gives the paragraph again.
#[must_use]
pub fn write_saved(node: &Node) -> (String, Vec<usize>) {
    emit(node, true)
}

fn emit(node: &Node, mask: bool) -> (String, Vec<usize>) {
    match node {
        Node::Para(t) if mask => para(t, Ctx::Block),
        Node::Para(t) | Node::Raw(t) => (t.clone(), vec![0]),
        Node::Heading(n, t) => {
            let head = format!("{} ", "#".repeat(usize::from(*n)));
            (format!("{head}{t}"), vec![head.len()])
        }
        Node::Code(lang, t) => {
            let head = format!("```{lang}\n");
            (format!("{head}{t}\n```"), vec![head.len()])
        }
        Node::Rule => ("---".to_owned(), vec![0]),
        Node::Quote(children) => quote(children, mask),
        Node::List(l) => list(l, mask),
    }
}

/// `text` with `first` before its first line and `rest` before the others (an empty line gets the prefix without its trailing
/// space), and `starts` moved to where they are then.
fn prefixed(text: &str, starts: &[usize], first: &str, rest: &str) -> (String, Vec<usize>) {
    let mut out = String::new();
    let mut cuts = Vec::new();
    let mut at = 0;
    for (k, line) in text.split('\n').enumerate() {
        let prefix = if k == 0 { first } else { rest };
        let prefix = if line.is_empty() {
            prefix.trim_end()
        } else {
            prefix
        };
        if k > 0 {
            out.push('\n');
        }
        cuts.push((at, out.len() + prefix.len()));
        out.push_str(prefix);
        out.push_str(line);
        at += line.len() + 1;
    }
    let moved = starts
        .iter()
        .map(|&s| {
            let (line_at, new_at) = cuts
                .iter()
                .rev()
                .find(|(a, _)| *a <= s)
                .copied()
                .unwrap_or((0, 0));
            new_at + (s - line_at)
        })
        .collect();
    (out, moved)
}

fn quote(children: &[Node], mask: bool) -> (String, Vec<usize>) {
    let mut text = String::new();
    let mut starts = Vec::new();
    for (k, c) in children.iter().enumerate() {
        if k > 0 {
            text.push_str("\n\n");
        }
        let (t, s) = emit(c, mask);
        starts.extend(s.iter().map(|o| o + text.len()));
        text.push_str(&t);
    }
    prefixed(&text, &starts, "> ", "> ")
}

fn list(l: &List, mask: bool) -> (String, Vec<usize>) {
    let mut text = String::new();
    let mut starts = Vec::new();
    for (k, item) in l.items.iter().enumerate() {
        if k > 0 {
            text.push('\n');
        }
        let number = l.start.saturating_add(u32::try_from(k).unwrap_or(0));
        let mut marker = if l.ordered {
            format!("{number}. ")
        } else {
            "- ".to_owned()
        };
        if let Some(done) = item.task {
            marker.push_str(if done { "[x] " } else { "[ ] " });
        }
        let width = if l.ordered {
            format!("{number}. ").len()
        } else {
            2
        };
        text.push_str(&marker);
        for (j, child) in item.children.iter().enumerate() {
            let (t, s) = match child {
                Node::Para(t) if j == 0 && mask => para(
                    t,
                    if item.task.is_none() {
                        Ctx::Item
                    } else {
                        Ctx::Text
                    },
                ),
                _ => emit(child, mask),
            };
            if j == 0 {
                starts.extend(s.iter().map(|o| o + text.len()));
                text.push_str(&t);
            } else {
                // A child after the text (a nested list) goes on the lines below, indented by the marker.
                let pad = " ".repeat(width);
                let (t, s) = prefixed(&t, &s, &pad, &pad);
                text.push('\n');
                starts.extend(s.iter().map(|o| o + text.len()));
                text.push_str(&t);
            }
        }
    }
    (text, starts)
}
