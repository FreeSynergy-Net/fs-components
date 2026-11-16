//! Writing the typed tree as canonical Markdown source.
//!
//! Pattern: Adapter — the inverse of [`super::read`]: a [`Node`] becomes source text, and the offset of the text of each leaf
//! in that source is given again in reading order. A block nobody changed is not written at all (the editor keeps its source).

use super::node::{List, Node};

/// The canonical source of `node` and the offsets of the text of its leaves in it.
#[must_use]
pub fn write(node: &Node) -> (String, Vec<usize>) {
    match node {
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
        Node::Quote(children) => quote(children),
        Node::List(l) => list(l),
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

fn quote(children: &[Node]) -> (String, Vec<usize>) {
    let mut text = String::new();
    let mut starts = Vec::new();
    for (k, c) in children.iter().enumerate() {
        if k > 0 {
            text.push_str("\n\n");
        }
        let (t, s) = write(c);
        starts.extend(s.iter().map(|o| o + text.len()));
        text.push_str(&t);
    }
    prefixed(&text, &starts, "> ", "> ")
}

fn list(l: &List) -> (String, Vec<usize>) {
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
            let (t, s) = write(child);
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
