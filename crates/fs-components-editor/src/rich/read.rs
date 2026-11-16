//! Reading the Markdown source of one block into the typed tree.
//!
//! Pattern: Adapter — a block of Markdown source becomes a [`Node`]; at the same time the byte offset in the source where the
//! text of each leaf begins is noted (in reading order), so that an edit of the contract, which names a source offset, finds its
//! leaf. A source this reader does not understand as a typed block (a table, a list with a lazy line) is [`Node::Raw`].

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

pub(super) fn is_rule(line: &str) -> bool {
    let t = line.trim();
    t.len() >= 3
        && ["-", "*", "_"]
            .iter()
            .any(|c| t.chars().all(|x| c.starts_with(x)))
}

/// The heading a line is: level and the length of the marker.
fn heading(line: &str) -> Option<(u8, usize)> {
    let hashes = line.chars().take_while(|&c| c == '#').count();
    ((1..=6).contains(&hashes) && line[hashes..].starts_with(' '))
        .then(|| (u8::try_from(hashes).unwrap_or(6), hashes + 1))
}

/// The list marker a line starts with: ordered, the number, and the length of the marker.
fn list_marker(line: &str) -> Option<(bool, u32, usize)> {
    if ["- ", "* ", "+ "].iter().any(|m| line.starts_with(m)) {
        return Some((false, 1, 2));
    }
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    (digits > 0 && digits < 10 && line[digits..].starts_with(". "))
        .then(|| (true, line[..digits].parse().unwrap_or(1), digits + 2))
}

/// The task box at the start of an item's text: its state and the length (with the space after it).
pub(super) fn task_box(text: &str) -> Option<(bool, usize)> {
    [("[ ] ", false), ("[x] ", true), ("[X] ", true)]
        .iter()
        .find_map(|(b, done)| text.starts_with(b).then_some((*done, b.len())))
}

/// Would `line` be read as something other than a paragraph at the start of a block (heading, list, quote, table, rule, fence,
/// task box)?
pub(super) fn is_block_marker(line: &str) -> bool {
    heading(line).is_some()
        || list_marker(line).is_some()
        || line.starts_with(['>', '|'])
        || is_rule(line)
        || ["```", "~~~"].iter().any(|f| line.starts_with(f))
        || task_box(line).is_some()
}

/// Is `line` the text of a paragraph that begins with a backslash in front of what would be a marker (`\## x`)? The writer puts
/// that backslash there so that the text stays a paragraph; `escaped(line, f)` asks for the marker kind `f`.
pub(super) fn escaped(line: &str, marker: fn(&str) -> bool) -> bool {
    line.strip_prefix('\\')
        .is_some_and(|rest| marker(rest.trim_start_matches('\\')))
}

fn block(lines: &[Line], starts: &mut Vec<usize>) -> Option<Node> {
    let first = lines.first()?;
    if let Some(f) = ["```", "~~~"].iter().find(|f| first.text.starts_with(*f))
        && lines.len() >= 2
        && lines[lines.len() - 1].text.trim_end() == *f
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
        return Some(Node::Heading(level, first.text[len..].to_owned()));
    }
    // The backslash that keeps a paragraph from being read as a marker (`\## x`) is not text.
    let cut = usize::from(escaped(first.text, is_block_marker));
    starts.push(first.off + cut);
    let mut text = join(lines);
    text.replace_range(..cut, "");
    Some(Node::Para(text))
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
            let text = &rest[cut..];
            let hold = usize::from(task.is_none() && escaped(text, |t| task_box(t).is_some()));
            starts.push(line.off + len + cut + hold);
            items.push(Item::text(task, &text[hold..]));
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
