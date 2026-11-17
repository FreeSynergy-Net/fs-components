//! `fs-components-filepicker`: the file chooser of FreeSynergy: "wherever a path is needed, a file chooser".
//!
//! One trait ([`FileChooser`]); the system dialog over xdg-desktop-portal ([`PortalChooser`], Linux, feature `portal`, crate
//! `ashpd`); the path field as a marked, provisional fallback ([`PathFieldChooser`], Windows or no portal); a fake for tests
//! ([`FakeChooser`]). A path only ever comes out of [`Picked`], which only a chooser makes. A cancel is
//! [`ChooserError::Cancelled`], no problem; problems are a code with parameters ([`ProblemCode`]) and their words are in the
//! MF2 files of [`Labels`]. `open` blocks: the interface calls it only through [`worker::pick_in_worker`]. The Blinc parts
//! (feature `ui`): [`ui::FilePickButton`], [`ui::PathPromptView`].
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

mod chooser;
pub mod fake;
pub mod filters;
mod labels;
mod pathfield;
#[cfg(all(feature = "portal", target_os = "linux"))]
pub mod portal;
pub mod portal_uri;
mod problem;
mod prompt;
mod request;
mod select;
#[cfg(feature = "ui")]
pub mod ui;
mod validate;
pub mod worker;

pub use chooser::FileChooser;
pub use fake::{Answer, FakeChooser, Gate};
pub use labels::{Labels, TEXTS};
pub use pathfield::PathFieldChooser;
#[cfg(all(feature = "portal", target_os = "linux"))]
pub use portal::PortalChooser;
pub use problem::{ChooserError, Problem, ProblemCode};
pub use prompt::PathPrompt;
pub use request::{Filter, Mode, OpenRequest, Picked};
pub use select::{FallbackChooser, default_chooser, session_bus_present};
pub use validate::validate_path;
