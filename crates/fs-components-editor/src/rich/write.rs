//! Writing the typed tree as canonical Markdown source.
//!
//! Pattern: Adapter — the inverse of [`super::read`]: a [`Node`] becomes source text, and the offset of the text of each leaf
//! in that source is given again in reading order. A block nobody changed is not written at all (the editor keeps its source).

use super::marker::{is_block_marker, mask_closing, number_escape};
use super::node::{List, Node};

/// A paragraph written so that reading it gives the same paragraph, in any reader of `CommonMark`: a backslash in front of what
/// would be a marker (`## x` as `\## x`, `- x` as `\- x`, `[x] x` as `\[x] x`), a backslash in front of the dot or bracket of
/// a number (`1. x` as `1\. x`, `1) x` as `1\) x`; one before the digit is no escape), and where the text starts after it.
/// `mask` is false for the text of a task item (after its box nothing is a marker).
fn para(text: &str, mask: bool) -> (String, Vec<usize>) {
    let first = text.split('\n').next().unwrap_or("");
    if mask && let Some((digits, _)) = number_escape(first) {
        return (format!("{}\\{}", &text[..digits], &text[digits..]), vec![0]);
    }
    if mask && is_block_marker(first.trim_start_matches('\\')) {
        (format!("\\{text}"), vec![1])
    } else {
        (text.to_owned(), vec![0])
    }
}

/// The fence of a code block: backticks, more than the longest run of them in the code (three at least), so that no line of the
/// code closes the block.
fn fence_for(code: &str) -> String {
    let mut longest = 0;
    let mut run = 0;
    for c in code.chars() {
        run = if c == '`' { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    "`".repeat((longest + 1).max(3))
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
        Node::Para(t) if mask => para(t, true),
        Node::Para(t) | Node::Raw(t) => (t.clone(), vec![0]),
        Node::Heading(n, t) => {
            let head = format!("{} ", "#".repeat(usize::from(*n)));
            let text = if mask { mask_closing(t) } else { t.clone() };
            (format!("{head}{text}"), vec![head.len()])
        }
        Node::Code(lang, t) => {
            let fence = fence_for(t);
            let head = format!("{fence}{lang}\n");
            (format!("{head}{t}\n{fence}"), vec![head.len()])
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
                Node::Para(t) if j == 0 && mask => para(t, item.task.is_none()),
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
