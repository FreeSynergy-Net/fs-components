//! The pure state of a splitter.
//!
//! Pattern: State — the position of the divider in percent of the whole, its limits, the size before a collapse, and what a key does.

use crate::keys::Key;

/// What a key did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    /// Not a key of the splitter.
    Ignored,
    /// The divider moved or the pane collapsed or came back: the new position.
    Moved(f64),
}

/// The divider.
#[derive(Clone, Debug)]
pub struct State {
    position: f64,
    min: f64,
    max: f64,
    step: f64,
    before: Option<f64>,
    vertical: bool,
}

impl State {
    /// A divider at `position` percent, between `min` and `max` percent, moving by `step` percent. A divider between left and
    /// right panes is `vertical` (Left and Right move it).
    pub fn new(position: f64, min: f64, max: f64, step: f64, vertical: bool) -> Self {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        let (min, max) = (min.clamp(0.0, 100.0), max.clamp(0.0, 100.0));
        Self {
            position: position.clamp(min, max),
            min,
            max,
            step: step.max(0.1),
            before: None,
            vertical,
        }
    }

    /// The first pane's size in percent.
    pub fn position(&self) -> f64 {
        self.position
    }

    /// The least size.
    pub fn min(&self) -> f64 {
        self.min
    }

    /// The greatest size.
    pub fn max(&self) -> f64 {
        self.max
    }

    /// Whether the divider runs between left and right panes.
    pub fn vertical(&self) -> bool {
        self.vertical
    }

    /// Whether the first pane is collapsed (the divider is at 0).
    pub fn collapsed(&self) -> bool {
        self.before.is_some()
    }

    /// Put the divider at `to` percent (held to the limits); a move ends a collapse. True when it moved.
    pub fn set(&mut self, to: f64) -> bool {
        let to = to.clamp(self.min, self.max);
        self.before = None;
        if (to - self.position).abs() < f64::EPSILON {
            return false;
        }
        self.position = to;
        true
    }

    /// What a key does.
    pub fn press(&mut self, key: Key) -> Outcome {
        let (less, more) = if self.vertical {
            (Key::Left, Key::Right)
        } else {
            (Key::Up, Key::Down)
        };
        let to = match key {
            k if k == less => self.position - self.step,
            k if k == more => self.position + self.step,
            Key::Home => self.min,
            Key::End => self.max,
            // Enter reaches a Blinc window as `Key::Enter` (on release; Space comes as `Key::Space`)
            Key::Enter | Key::Activate => return self.collapse(),
            _ => return Outcome::Ignored,
        };
        self.before = None;
        self.position = to.clamp(self.min, self.max);
        Outcome::Moved(self.position)
    }

    fn collapse(&mut self) -> Outcome {
        if let Some(was) = self.before.take() {
            self.position = was;
        } else {
            self.before = Some(self.position);
            self.position = 0.0;
        }
        Outcome::Moved(self.position)
    }
}
