//! The Adapter over Blinc's `Headless`: a real UI built, driven and drawn offscreen (needs a graphics adapter).
//!
//! Pattern: Adapter - `blinc_recorder::Headless` behind [`Driver`]. Holes of `Headless`, each caught here and named in an error:
//! no key release (`caps().key_up` is false; Shift is a state kept here), modifiers only Shift, only the left button, and
//! `Headless::centre` panics for a missing element (this Adapter asks `find` first).

use std::path::Path;

use blinc_recorder::Headless;

use crate::driver::{Caps, Driver};
use crate::error::{Code, ReplayError};
use crate::step::{Button, Key, Named};

/// Windows key codes as the window of Blinc sends them (`blinc_app` keyboard input).
const fn code(key: Named) -> Option<u32> {
    Some(match key {
        Named::Backspace => 8,
        Named::Tab => 9,
        Named::Enter => 13,
        Named::Escape => 27,
        Named::Space => 32,
        Named::PageUp => 33,
        Named::PageDown => 34,
        Named::End => 35,
        Named::Home => 36,
        Named::Left => 37,
        Named::Up => 38,
        Named::Right => 39,
        Named::Down => 40,
        Named::Delete => 127,
        Named::Shift | Named::Control | Named::Alt | Named::Meta => return None,
    })
}

/// Plays a plan on a [`Headless`] surface the caller built and mounted.
pub struct HeadlessDriver<'a> {
    ui: &'a mut Headless,
    pointer: (f32, f32),
    shift: bool,
}

impl<'a> HeadlessDriver<'a> {
    /// Take the surface. The pointer is taken to be at the top left, where `Headless` starts it.
    pub fn new(ui: &'a mut Headless) -> Self {
        Self {
            ui,
            pointer: (0.0, 0.0),
            shift: false,
        }
    }
}

impl Driver for HeadlessDriver<'_> {
    fn caps(&self) -> Caps {
        Caps {
            key_up: false,
            modifiers: false,
            extra_buttons: false,
            wheel: true,
        }
    }

    fn pointer(&self) -> (f32, f32) {
        self.pointer
    }

    fn centre(&self, id: &str) -> Result<(f32, f32), ReplayError> {
        let selector = format!("#{id}");
        // `Headless::centre` panics for a missing element or one without bounds: ask first.
        let present = self
            .ui
            .find(&selector)
            .and_then(|node| self.ui.tree().get_absolute_bounds(node))
            .is_some();
        if present {
            Ok(self.ui.centre(&selector))
        } else {
            Err(ReplayError::new(
                Code::ElementMissing,
                format!("no element with id {id}"),
            ))
        }
    }

    fn move_to(&mut self, x: f32, y: f32) -> Result<(), ReplayError> {
        self.ui.move_to(x, y);
        self.pointer = (x, y);
        Ok(())
    }

    fn button(&mut self, button: Button, down: bool) -> Result<(), ReplayError> {
        if button != Button::Left {
            return Err(ReplayError::new(
                Code::Unsupported,
                format!("{button:?} button: Headless presses the left button only"),
            ));
        }
        if down {
            self.ui.mouse_down();
        } else {
            self.ui.mouse_up();
        }
        Ok(())
    }

    fn key_down(&mut self, key: &Key) -> Result<(), ReplayError> {
        match key {
            Key::Char(c) => {
                self.ui.type_text(&c.to_string());
            }
            Key::Named(Named::Shift) => self.shift = true,
            Key::Named(m @ (Named::Control | Named::Alt | Named::Meta)) => {
                return Err(ReplayError::new(
                    Code::Modifier,
                    format!("{m:?}: Headless holds only Shift"),
                ));
            }
            Key::Named(n) => {
                if let Some(c) = code(*n) {
                    self.ui.key_with_shift(c, self.shift);
                }
            }
        }
        Ok(())
    }

    fn key_up(&mut self, key: &Key) -> Result<(), ReplayError> {
        if matches!(key, Key::Named(Named::Shift)) {
            self.shift = false;
            return Ok(());
        }
        Err(ReplayError::new(
            Code::KeyUp,
            format!("keyUp of {key:?}: Headless cannot release a key"),
        ))
    }

    fn wheel(&mut self, dx: f32, dy: f32) -> Result<(), ReplayError> {
        self.ui.scroll(dx, dy);
        Ok(())
    }

    fn advance(&mut self, ms: u32) {
        self.ui.advance(ms);
    }

    fn snapshot(&mut self, path: &Path) -> Result<(), ReplayError> {
        let driver = |e: String| ReplayError::new(Code::Driver, e);
        let frame = self
            .ui
            .snapshot()
            .map_err(|e| driver(format!("cannot draw the surface: {e}")))?;
        blinc_recorder::ScreenshotExporter::save_png(&frame, path)
            .map_err(|e| driver(format!("cannot write {}: {e}", path.display())))
    }
}
