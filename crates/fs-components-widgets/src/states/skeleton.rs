//! `skeleton`: grey bars in the shape of what is coming, shown while it loads, so nothing jumps when it arrives.
//!
//! Pattern: Builder + Adapter. Marked busy (`aria-busy`) with the word of the labels for a screen reader. Limit: the bars do not
//! pulse (no animation in the contract yet), they only hold the place.

use blinc_layout::div::{Div, div};

use crate::Element;

/// One shape of the placeholder.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Shape {
    /// A bar of text: the width in percent of the room (100, 70 or 60: the classes `fs-skel-w100`, `-w70`, `-w60`).
    Line(u8),
    /// A block of a fixed size.
    Block(f32, f32),
    /// A round mark (an avatar, an icon).
    Circle(f32),
}

/// The placeholder.
#[derive(Clone)]
pub struct Skeleton {
    id: String,
    word: String,
    shapes: Vec<Vec<Shape>>,
}

impl Skeleton {
    /// An empty placeholder; `word` is what a screen reader hears ("Loading").
    pub fn new(word: &str) -> Self {
        Self {
            id: "skeleton".to_owned(),
            word: word.to_owned(),
            shapes: Vec::new(),
        }
    }

    /// The prefix of the ids of the parts.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// `n` lines of text, the last one shorter.
    #[must_use]
    pub fn lines(mut self, n: usize) -> Self {
        for i in 0..n {
            let share = if i + 1 == n && n > 1 { 60 } else { 100 };
            self.shapes.push(vec![Shape::Line(share)]);
        }
        self
    }

    /// `n` rows of a round mark and two lines (a list of people, programs, appointments).
    #[must_use]
    pub fn rows(mut self, n: usize) -> Self {
        for _ in 0..n {
            self.shapes.push(vec![Shape::Circle(32.0), Shape::Line(70)]);
        }
        self
    }

    /// A block of `w` by `h` pixels (a picture).
    #[must_use]
    pub fn block(mut self, w: f32, h: f32) -> Self {
        self.shapes.push(vec![Shape::Block(w, h)]);
        self
    }

    /// How many bars there are.
    pub fn bars(&self) -> usize {
        self.shapes.iter().map(Vec::len).sum()
    }

    /// The element.
    pub fn build(&self) -> impl Element + use<> {
        self.render()
    }

    pub(super) fn render(&self) -> Div {
        let mut root = div()
            .id(&self.id)
            .class("fs-skel")
            .attr("aria-busy", "true")
            .attr("aria-label", &self.word);
        for (r, row) in self.shapes.iter().enumerate() {
            let mut line = div().class("fs-skel-row");
            for (i, shape) in row.iter().enumerate() {
                let id = format!("{}-bar-{r}-{i}", self.id);
                line = line.child(match *shape {
                    Shape::Line(share) => div()
                        .id(id)
                        .class("fs-skel-bar")
                        .class("fs-skel-line")
                        .class(format!("fs-skel-w{share}")),
                    Shape::Block(w, h) => div().id(id).class("fs-skel-bar").w(w).h(h),
                    Shape::Circle(d) => div()
                        .id(id)
                        .class("fs-skel-bar")
                        .class("fs-skel-circle")
                        .w(d)
                        .h(d),
                });
            }
            root = root.child(line);
        }
        root
    }
}
