//! What an offscreen UI can be told.
//!
//! Pattern: Adapter (the trait `Driver`).

use std::path::Path;

use crate::error::ReplayError;
use crate::step::{Button, Key};

/// What a driver cannot do, so that `perform` refuses a plan before it plays the first step (no half-played input).
#[allow(clippy::struct_excessive_bools)] // four independent facts of a driver, not a state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Caps {
    /// Releases a key (Shift is always allowed: the driver or `perform` keeps that state).
    pub key_up: bool,
    /// Holds Control, Alt and Meta.
    pub modifiers: bool,
    /// Presses a button other than the left one.
    pub extra_buttons: bool,
    /// Turns the wheel.
    pub wheel: bool,
}

impl Caps {
    /// A driver that can do everything.
    pub const ALL: Self = Self {
        key_up: true,
        modifiers: true,
        extra_buttons: true,
        wheel: true,
    };
}

/// Pattern: Adapter - what an offscreen UI can be told: pointer, buttons, keys, wheel, clock, picture.
///
/// Tests, `fs-forms-run` and later the recorder talk to this trait and never to the UI toolkit directly. Every method that can
/// fail returns a [`ReplayError`]; none panics for a missing element.
pub trait Driver {
    /// What this driver cannot do.
    fn caps(&self) -> Caps;
    /// Where the pointer is now (the start is the top left).
    fn pointer(&self) -> (f32, f32);
    /// The centre of the element with this id, or `Code::ElementMissing`.
    ///
    /// # Errors
    /// `Code::ElementMissing` when nothing matches.
    fn centre(&self, id: &str) -> Result<(f32, f32), ReplayError>;
    /// Move the pointer to a point of the surface (with a button held, this drags).
    ///
    /// # Errors
    /// `Code::Driver` when the surface is gone.
    fn move_to(&mut self, x: f32, y: f32) -> Result<(), ReplayError>;
    /// Press (`down`) or release a button at the pointer.
    ///
    /// # Errors
    /// `Code::Unsupported` for a button the driver does not have.
    fn button(&mut self, button: Button, down: bool) -> Result<(), ReplayError>;
    /// Press a key (a character is typed).
    ///
    /// # Errors
    /// `Code::Key` or `Code::Modifier` for a key the driver cannot send.
    fn key_down(&mut self, key: &Key) -> Result<(), ReplayError>;
    /// Release a key.
    ///
    /// # Errors
    /// `Code::KeyUp` when the driver cannot release this key.
    fn key_up(&mut self, key: &Key) -> Result<(), ReplayError>;
    /// Turn the wheel under the pointer.
    ///
    /// # Errors
    /// `Code::Unsupported` when the driver has no wheel.
    fn wheel(&mut self, dx: f32, dy: f32) -> Result<(), ReplayError>;
    /// Let the clock run (frames).
    fn advance(&mut self, ms: u32);
    /// Draw the surface as it is now and write it as a PNG.
    ///
    /// # Errors
    /// `Code::Driver` when it cannot be drawn or written.
    fn snapshot(&mut self, path: &Path) -> Result<(), ReplayError>;
}
