//! Where everything of the surface stands, and the questions asked of it: where is the caret, what is at this point.
//!
//! Pattern: Value object — a [`Geometry`] is computed from the tree and the width ([`crate::place`]) and read by the drawing
//! ([`crate::view`]), by the caret and by the click, so all three agree by construction. It is data: no Blinc, no state.

use crate::measure::Face;

/// One line of a leaf: the bytes of its text it holds, and where it stands.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// First byte of the line in the text of the leaf.
    pub start: usize,
    /// One past the last byte.
    pub end: usize,
    /// The top of the line, from the top of the surface.
    pub y: f32,
}

/// What a leaf is, for the drawing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sort {
    /// Running text.
    Text,
    /// A heading.
    Heading,
    /// Code or source text.
    Code,
    /// A horizontal rule (no text).
    Rule,
}

/// A leaf of the tree placed on the surface.
#[derive(Clone, Debug, PartialEq)]
pub struct Leaf {
    /// The path of the leaf in the tree.
    pub path: Vec<usize>,
    /// What it is.
    pub sort: Sort,
    /// The text.
    pub text: String,
    /// The left edge of its text.
    pub x: f32,
    /// The top of its first line.
    pub y: f32,
    /// How wide its text may be.
    pub w: f32,
    /// How its text is set.
    pub face: Face,
    /// Its lines.
    pub lines: Vec<Line>,
    /// The height of one line (and of the caret).
    pub line_h: f32,
    /// The height of the whole leaf.
    pub h: f32,
}

impl Leaf {
    /// The bottom of the leaf.
    #[must_use]
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
}

/// A marker in the margin of a list item.
#[derive(Clone, Debug, PartialEq)]
pub enum Marker {
    /// A bullet, centred on `(x, y)`.
    Bullet {
        /// Centre x.
        x: f32,
        /// Centre y.
        y: f32,
    },
    /// The number of a numbered item, its text right-aligned to `x + w`.
    Number {
        /// Left edge of the column.
        x: f32,
        /// Top of the line.
        y: f32,
        /// Width of the column.
        w: f32,
        /// The text (`3.`).
        text: String,
    },
    /// A task box.
    Task {
        /// Left.
        x: f32,
        /// Top.
        y: f32,
        /// The side.
        side: f32,
        /// Ticked.
        done: bool,
        /// The path of the text leaf of the item (what a click toggles).
        path: Vec<usize>,
    },
}

/// A shape behind or beside text.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    /// The ground of a code block, with its language hint.
    Ground {
        /// Left.
        x: f32,
        /// Top.
        y: f32,
        /// Width.
        w: f32,
        /// Height.
        h: f32,
        /// The language (may be empty).
        lang: String,
    },
    /// The bar at the left of a quotation.
    Bar {
        /// Left.
        x: f32,
        /// Top.
        y: f32,
        /// Height.
        h: f32,
    },
    /// A horizontal rule.
    Rule {
        /// Left.
        x: f32,
        /// Top.
        y: f32,
        /// Width.
        w: f32,
    },
}

/// The caret: where it stands and how tall it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Caret {
    /// Left.
    pub x: f32,
    /// Top.
    pub y: f32,
    /// Height.
    pub h: f32,
}

/// The whole surface placed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Geometry {
    /// The leaves in reading order.
    pub leaves: Vec<Leaf>,
    /// The markers of the list items.
    pub markers: Vec<Marker>,
    /// The shapes.
    pub shapes: Vec<Shape>,
    /// The height of everything.
    pub height: f32,
}
