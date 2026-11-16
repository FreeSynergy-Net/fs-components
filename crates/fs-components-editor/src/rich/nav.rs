//! Moving the cursor over the leaves: by character, by line, to the start and the end of a line.
//!
//! Pattern: none — pure functions from a cursor to a cursor. A line is a run of text up to a line break of the leaf itself (code
//! and source have them); the wrapped lines of a long paragraph are the business of the drawing, which knows the width.

use super::editor::{Cursor, RichEditor};

fn put(path: Vec<usize>, at: usize) -> Cursor {
    Cursor { path, at }
}

/// One character left; at the start of a leaf the end of the leaf before.
pub(super) fn left(e: &RichEditor, c: &Cursor) -> Cursor {
    let text = e.text(&c.path);
    if let Some((i, _)) = text[..c.at.min(text.len())].char_indices().next_back() {
        return put(c.path.clone(), i);
    }
    match e.leaf_before(&c.path) {
        Some(p) => put(p.clone(), e.text(&p).len()),
        None => c.clone(),
    }
}

/// One character right; at the end of a leaf the start of the leaf after.
pub(super) fn right(e: &RichEditor, c: &Cursor) -> Cursor {
    let text = e.text(&c.path);
    if let Some(ch) = text.get(c.at..).and_then(|t| t.chars().next()) {
        return put(c.path.clone(), c.at + ch.len_utf8());
    }
    match e.leaf_after(&c.path) {
        Some(p) => put(p, 0),
        None => c.clone(),
    }
}

fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map_or(0, |i| i + 1)
}

fn line_end(text: &str, at: usize) -> usize {
    text[at..].find('\n').map_or(text.len(), |i| at + i)
}

/// The start of the line the cursor is on.
pub(super) fn home(e: &RichEditor, c: &Cursor) -> Cursor {
    put(
        c.path.clone(),
        line_start(e.text(&c.path), c.at.min(e.text(&c.path).len())),
    )
}

/// The end of the line the cursor is on.
pub(super) fn end(e: &RichEditor, c: &Cursor) -> Cursor {
    put(
        c.path.clone(),
        line_end(e.text(&c.path), c.at.min(e.text(&c.path).len())),
    )
}

/// The byte offset `col` characters into the line that starts at `start` (at most the end of the line).
fn column(text: &str, start: usize, col: usize) -> usize {
    let end = line_end(text, start);
    text[start..end]
        .char_indices()
        .nth(col)
        .map_or(end, |(i, _)| start + i)
}

/// One line up or down, keeping the column the person is after (`want`, in characters; the column of the cursor when none).
/// The column that is kept is given back with the new cursor.
pub(super) fn vertical(
    e: &RichEditor,
    c: &Cursor,
    down: bool,
    want: Option<usize>,
) -> (Cursor, usize) {
    let text = e.text(&c.path);
    let at = c.at.min(text.len());
    let start = line_start(text, at);
    let col = want.unwrap_or_else(|| text[start..at].chars().count());
    if down {
        let end = line_end(text, at);
        if end < text.len() {
            return (put(c.path.clone(), column(text, end + 1, col)), col);
        }
        if let Some(p) = e.leaf_after(&c.path) {
            let at = column(e.text(&p), 0, col);
            return (put(p, at), col);
        }
        return (put(c.path.clone(), text.len()), col);
    }
    if start > 0 {
        let above = line_start(text, start - 1);
        return (put(c.path.clone(), column(text, above, col)), col);
    }
    if let Some(p) = e.leaf_before(&c.path) {
        let t = e.text(&p);
        let at = column(t, line_start(t, t.len()), col);
        return (put(p, at), col);
    }
    (put(c.path.clone(), 0), col)
}
