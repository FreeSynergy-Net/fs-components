//! Replay of a described input against an offscreen UI.
//!
//! Pattern: Adapter (`Driver`, what an offscreen UI can be told) and Command (every action is a data object;
//! [`perform`] executes them tick by tick). Deviation from Command: no undo, a step has only "execute".
//!
//! The file format is the body of the W3C `WebDriver` *Perform Actions* command (Level 1 section 17, wheel from Level 2),
//! unchanged. [`parse`] reads it with the crate `webdriver` (one file, `translate.rs`) into our own [`Plan`]; a [`Driver`]
//! plays the plan. Everything the format can say and a driver cannot do is an [`ReplayError`] with a [`Code`], never skipped.
//!
//! Supported subset: pointer (mouse: move, down, up, pause), key (down, up, pause), wheel (scroll, pause), none (pause).
//! Not supported: pen and touch, `pointerCancel`, special keys outside the table in [`Named`].
#![forbid(unsafe_code)]

mod driver;
mod error;
#[cfg(feature = "headless")]
mod headless;
mod perform;
mod step;
mod translate;

pub use driver::{Caps, Driver};
pub use error::{Code, ReplayError};
#[cfg(feature = "headless")]
pub use headless::HeadlessDriver;
pub use perform::{FRAME_MS, perform};
pub use step::{Button, Key, Named, Origin, Plan, Step, Tick};
pub use translate::{MAX_DURATION_MS, parse, translate};
