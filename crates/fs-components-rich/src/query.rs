//! The questions asked of a [`Geometry`]: where is the caret, what is at this point, where does an arrow key go.
//!
//! Pattern: none — pure queries over plain data (see [`crate::geometry`]); the drawing, the click and the keys all ask here.

use crate::geometry::{Caret, Geometry, Leaf, Marker, Sort};

/// Where a line-wise move puts the cursor.
#[derive(Clone, Debug, PartialEq)]
pub struct Landing {
    /// The leaf.
    pub path: Vec<usize>,
    /// The byte offset in its text.
    pub at: usize,
    /// The x the person is after (kept over a run of Up and Down).
    pub x: f32,
}

/// The byte offset in the line `k` of `leaf` nearest to the horizontal position `x`.
fn offset_in(leaf: &Leaf, k: usize, x: f32) -> usize {
    let line = &leaf.lines[k];
    let text = leaf.text.get(line.start..line.end).unwrap_or("");
    // A line that wraps does not end on the space it broke at: that one is no place for the cursor.
    let text = if k + 1 < leaf.lines.len() {
        text.trim_end()
    } else {
        text
    };
    let mut best = (0, f32::MAX);
    for i in text
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(text.len()))
    {
        let d = (leaf.x + leaf.face.width(&text[..i]) - x).abs();
        if d < best.1 {
            best = (i, d);
        }
    }
    line.start + best.0
}

impl Geometry {
    /// The leaf at `path`.
    #[must_use]
    pub fn leaf(&self, path: &[usize]) -> Option<&Leaf> {
        self.leaves.iter().find(|l| l.path == path)
    }

    /// The line of the leaf at `path` the byte `at` is on.
    fn line_of(leaf: &Leaf, at: usize) -> usize {
        leaf.lines.iter().rposition(|l| l.start <= at).unwrap_or(0)
    }

    /// The caret for byte `at` of the leaf at `path`.
    #[must_use]
    pub fn caret(&self, path: &[usize], at: usize) -> Option<Caret> {
        let leaf = self.leaf(path)?;
        let line = &leaf.lines[Self::line_of(leaf, at)];
        let to = at.min(line.end).min(leaf.text.len());
        let prefix = leaf.text.get(line.start..to).unwrap_or("");
        Some(Caret {
            x: leaf.x + leaf.face.width(prefix),
            y: line.y,
            h: leaf.line_h,
        })
    }

    /// The line (and its leaf) nearest to height `y`.
    fn nearest_line(&self, y: f32) -> Option<(&Leaf, usize)> {
        let distance = |l: &&Leaf| {
            if y < l.y {
                l.y - y
            } else {
                (y - l.bottom()).max(0.0)
            }
        };
        // A click on or next to a rule lands in the text nearest to it, not on the rule.
        let leaf = self
            .leaves
            .iter()
            .filter(|l| l.sort != Sort::Rule)
            .min_by(|a, b| distance(a).total_cmp(&distance(b)))?;
        let k = leaf.lines.iter().rposition(|l| l.y <= y).unwrap_or(0);
        Some((leaf, k))
    }

    /// The leaf and the byte offset nearest to the point `(x, y)`.
    #[must_use]
    pub fn hit(&self, x: f32, y: f32) -> Option<(Vec<usize>, usize)> {
        let (leaf, k) = self.nearest_line(y)?;
        Some((leaf.path.clone(), offset_in(leaf, k, x)))
    }

    /// The task item whose box is at the point `(x, y)`; the box and a little room around it count.
    #[must_use]
    pub fn task_at(&self, x: f32, y: f32) -> Option<&[usize]> {
        const ROOM: f32 = 4.0;
        self.markers.iter().find_map(|m| match m {
            Marker::Task {
                x: bx,
                y: by,
                side,
                path,
                ..
            } if x >= bx - ROOM
                && x <= bx + side + ROOM
                && y >= by - ROOM
                && y <= by + side + ROOM =>
            {
                Some(path.as_slice())
            }
            _ => None,
        })
    }

    /// The lines of the whole surface in reading order: (leaf, line).
    fn rows(&self) -> impl Iterator<Item = (&Leaf, usize)> {
        self.leaves
            .iter()
            .flat_map(|l| (0..l.lines.len()).map(move |k| (l, k)))
    }

    /// One visual line up or down from the cursor, at the horizontal position `want` (the one of the caret when none): the
    /// lines of a wrapped paragraph count, so does each block. `None` at the first line going up and at the last going down.
    #[must_use]
    pub fn step(
        &self,
        path: &[usize],
        at: usize,
        down: bool,
        want: Option<f32>,
    ) -> Option<Landing> {
        let caret = self.caret(path, at)?;
        let x = want.unwrap_or(caret.x);
        let rows: Vec<_> = self.rows().collect();
        let here = {
            let leaf = self.leaf(path)?;
            let k = Self::line_of(leaf, at);
            rows.iter()
                .position(|(l, j)| l.path == leaf.path && *j == k)?
        };
        // A rule is not a place to stand in: Up and Down walk over it to the next line of text.
        let mut next = if down { here + 1 } else { here.checked_sub(1)? };
        while rows.get(next)?.0.sort == Sort::Rule {
            next = if down { next + 1 } else { next.checked_sub(1)? };
        }
        let (leaf, k) = *rows.get(next)?;
        Some(Landing {
            path: leaf.path.clone(),
            at: offset_in(leaf, k, x),
            x,
        })
    }

    /// The start (`end` false) or the end of the visual line the cursor is on.
    #[must_use]
    pub fn line_edge(&self, path: &[usize], at: usize, end: bool) -> Option<usize> {
        let leaf = self.leaf(path)?;
        let k = Self::line_of(leaf, at);
        let line = &leaf.lines[k];
        if !end {
            return Some(line.start);
        }
        let text = leaf.text.get(line.start..line.end).unwrap_or("");
        Some(if k + 1 < leaf.lines.len() {
            line.start + text.trim_end().len()
        } else {
            line.end
        })
    }
}
