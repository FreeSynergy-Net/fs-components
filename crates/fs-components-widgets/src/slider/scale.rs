//! The scale of a slider and what a key does to a value on it.
//!
//! Pattern: State — pure functions over `(min, max, step, page)`; no element, no callback.

use crate::keys::Key;

/// A scale: the least and the greatest value, the step and the page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scale {
    min: f64,
    max: f64,
    step: f64,
    page: f64,
}

impl Scale {
    /// A scale from `min` to `max` (swapped when given the wrong way round) moving by `step` (a step that is not above zero becomes
    /// a hundredth of the span). The page is ten steps.
    pub fn new(min: f64, max: f64, step: f64) -> Self {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        let step = if step > 0.0 {
            step
        } else {
            (max - min) / 100.0
        };
        Self {
            min,
            max,
            step,
            page: step * 10.0,
        }
    }

    /// How far Page Up and Page Down move (not below one step).
    #[must_use]
    pub fn page(mut self, page: f64) -> Self {
        self.page = page.max(self.step);
        self
    }

    /// The least value.
    pub fn min(&self) -> f64 {
        self.min
    }

    /// The greatest value.
    pub fn max(&self) -> f64 {
        self.max
    }

    /// The value inside the scale, on a step.
    pub fn snap(&self, value: f64) -> f64 {
        let clamped = value.clamp(self.min, self.max);
        if self.step <= 0.0 || (self.max - self.min) <= 0.0 {
            return clamped;
        }
        let on = self.min + ((clamped - self.min) / self.step).round() * self.step;
        // the greatest value is always reachable, even when the span is no multiple of the step
        on.clamp(self.min, self.max)
    }

    /// Where on the scale a value is, 0.0 (least) to 1.0 (greatest).
    pub fn fraction(&self, value: f64) -> f64 {
        if self.max <= self.min {
            return 0.0;
        }
        ((value.clamp(self.min, self.max) - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    /// The value at a place of the track, 0.0 to 1.0.
    pub fn at(&self, fraction: f64) -> f64 {
        self.snap(self.min + fraction.clamp(0.0, 1.0) * (self.max - self.min))
    }

    /// The value a key leads to from `value`, or `None` when the key is not a key of a slider.
    pub fn key(&self, value: f64, key: Key) -> Option<f64> {
        let to = match key {
            Key::Left | Key::Down => value - self.step,
            Key::Right | Key::Up => value + self.step,
            Key::PageDown => value - self.page,
            Key::PageUp => value + self.page,
            Key::Home => self.min,
            Key::End => self.max,
            _ => return None,
        };
        Some(self.snap(to))
    }
}

/// A value as `aria-valuenow` says it: no trailing zeros.
pub fn number(value: f64) -> String {
    let text = format!("{value:.4}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}
