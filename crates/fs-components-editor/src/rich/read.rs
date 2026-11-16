//! Reading the Markdown source of one block into the typed tree.
//!
//! Pattern: Adapter — a block of Markdown source becomes a [`Node`]; at the same time the byte offset in the source where the
//! text of each leaf begins is noted (in reading order), so that an edit of the contract, which names a source offset, finds its
//! leaf. A source this reader does not understand as a typed block (a table, a list with a lazy line) is [`Node::Raw`].

use super::marker::{
    escaped, fence, heading, is_block_marker, is_rule, list_marker, number_escape, task_box,
    unmask_closing,
};
use super::node::{Item, List, Node};

/// One line of the source and where it starts.
#[derive(Clone, Copy)]
struct Line<'a> {
    text: &'a str,
    off: usize,
}

/// Read one block: the tree and the source offsets of the text of its leaves.
#[must_use]
pub fn read(source: &str) -> (Node, Vec<usize>) {
    let mut off = 0;
    let lines: Vec<Line> = source
        .split('\n')
        .map(|text| {
            let line = Line { text, off };
            off += text.len() + 1;
            line
        })
        .collect();
    let mut starts = Vec::new();
    let node = block(&lines, &mut starts).unwrap_or_else(|| {
        // A reading that failed half way has left starts of leaves that do not exist: the raw block has exactly one.
        starts.clear();
        raw(&lines, &mut starts)
    });
    (node, starts)
}

fn raw(lines: &[Line], starts: &mut Vec<usize>) -> Node {
    starts.push(lines.first().map_or(0, |l| l.off));
    Node::Raw(join(lines))
}

fn join(lines: &[Line]) -> String {
    lines.iter().map(|l| l.text).collect::<Vec<_>>().join("\n")
}

fn block(lines: &[Line], starts: &mut Vec<usize>) -> Option<Node> {
    let first = lines.first()?;
    if let Some(f) = fence(first.text)
        && lines.len() >= 2
        && lines[lines.len() - 1].text.trim_end() == f
    {
        let inner = &lines[1..lines.len() - 1];
        starts.push(lines[1].off);
        return Some(Node::Code(
            first.text[f.len()..].trim().to_owned(),
            join(inner),
        ));
    }
    if lines.len() == 1 && is_rule(first.text) {
        starts.push(first.off);
        return Some(Node::Rule);
    }
    if lines.iter().all(|l| l.text.starts_with('>')) {
        return quote(lines, starts);
    }
    if list_marker(first.text).is_some() {
        return list(lines, starts).map(Node::List);
    }
    if lines
        .iter()
        .any(|l| l.text.starts_with('|') || is_rule(l.text) || list_marker(l.text).is_some())
    {
        return None;
    }
    if lines.len() == 1
        && let Some((level, len)) = heading(first.text)
    {
        starts.push(first.off + len);
        return Some(Node::Heading(level, unmask_closing(&first.text[len..])));
    }
    let (text, cut) = unmask(&join(lines));
    starts.push(first.off + cut);
    Some(Node::Para(text))
}

/// The text of a paragraph without the backslash the writer put in front of what would be read as a marker (`\## x` is
/// `## x`, `1\. x` is `1. x`), and how far the start of the text has moved by the first of them.
pub(super) fn unmask(source: &str) -> (String, usize) {
    let first = source.split('\n').next().unwrap_or("");
    let cut = usize::from(escaped(first, is_block_marker));
    let mut text = source[cut..].to_owned();
    // A number (`1\. x`): the text after the digits has moved by the backslash.
    if let Some((digits, _)) = number_escape(&text).filter(|&(_, k)| k > 0) {
        text.replace_range(digits..=digits, "");
    }
    (text, cut)
}

fn quote(lines: &[Line], starts: &mut Vec<usize>) -> Option<Node> {
    let inner: Vec<Line> = lines
        .iter()
        .map(|l| {
            let cut = if l.text[1..].starts_with(' ') { 2 } else { 1 };
            Line {
                text: &l.text[cut..],
                off: l.off + cut,
            }
        })
        .collect();
    let mut children = Vec::new();
    for group in inner
        .split(|l| l.text.trim().is_empty())
        .filter(|g| !g.is_empty())
    {
        children.push(block(group, starts)?);
    }
    (!children.is_empty()).then_some(Node::Quote(children))
}

fn list(lines: &[Line], starts: &mut Vec<usize>) -> Option<List> {
    let (ordered, start, _) = list_marker(lines[0].text)?;
    let mut items: Vec<Item> = Vec::new();
    let mut nested: Vec<Line> = Vec::new();
    let mut width = 0;
    let flush = |items: &mut Vec<Item>, nested: &mut Vec<Line>, starts: &mut Vec<usize>| {
        if nested.is_empty() {
            return Some(());
        }
        let inner = list(nested, starts)?;
        items.last_mut()?.children.push(Node::List(inner));
        nested.clear();
        Some(())
    };
    for line in lines {
        if let Some((o, _, len)) = list_marker(line.text) {
            (o == ordered).then_some(())?;
            flush(&mut items, &mut nested, starts)?;
            let rest = &line.text[len..];
            let (task, cut) = task_box(rest).map_or((None, 0), |(d, n)| (Some(d), n));
            let (text, hold) = if task.is_none() {
                unmask(rest)
            } else {
                (rest[cut..].to_owned(), 0)
            };
            starts.push(line.off + len + cut + hold);
            items.push(Item::text(task, text));
            width = len;
        } else {
            let indent = line.text.len() - line.text.trim_start().len();
            (indent >= width && !items.is_empty()).then_some(())?;
            nested.push(Line {
                text: &line.text[width..],
                off: line.off + width,
            });
        }
    }
    flush(&mut items, &mut nested, starts)?;
    Some(List {
        ordered,
        start,
        items,
    })
}
