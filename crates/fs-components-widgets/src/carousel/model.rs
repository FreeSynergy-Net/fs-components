//! The pure state of a carousel.
//!
//! Pattern: State — how many slides, which is shown, whether the ends join, what a key does.

use crate::keys::Key;

/// What a key did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Not a key of the carousel, or nothing to do.
    Ignored,
    /// This slide is shown now.
    Shown(usize),
}

/// The slides and the one shown.
#[derive(Clone, Debug)]
pub struct State {
    count: usize,
    current: usize,
    wrap: bool,
}

impl State {
    /// `count` slides, the first shown; the ends join when `wrap`.
    pub fn new(count: usize, wrap: bool) -> Self {
        Self {
            count,
            current: 0,
            wrap,
        }
    }

    /// The slides.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Whether the ends join.
    pub fn wraps(&self) -> bool {
        self.wrap
    }

    /// The index of the slide shown.
    pub fn current(&self) -> usize {
        self.current
    }

    /// Show slide `to`; false when there is none or it is shown already.
    pub fn go(&mut self, to: usize) -> bool {
        if to >= self.count || to == self.current {
            return false;
        }
        self.current = to;
        true
    }

    /// The next slide (the first after the last when wrapping).
    pub fn forward(&mut self) -> Outcome {
        let to = if self.current + 1 < self.count {
            self.current + 1
        } else if self.wrap {
            0
        } else {
            return Outcome::Ignored;
        };
        self.shown(to)
    }

    /// The previous slide (the last before the first when wrapping).
    pub fn back(&mut self) -> Outcome {
        let to = if self.current > 0 {
            self.current - 1
        } else if self.wrap && self.count > 0 {
            self.count - 1
        } else {
            return Outcome::Ignored;
        };
        self.shown(to)
    }

    fn shown(&mut self, to: usize) -> Outcome {
        if self.go(to) {
            Outcome::Shown(to)
        } else {
            Outcome::Ignored
        }
    }

    /// What a key does.
    pub fn press(&mut self, key: Key) -> Outcome {
        match key {
            Key::Right => self.forward(),
            Key::Left => self.back(),
            Key::Home if self.count > 0 => self.shown(0),
            Key::End if self.count > 0 => self.shown(self.count - 1),
            _ => Outcome::Ignored,
        }
    }
}
