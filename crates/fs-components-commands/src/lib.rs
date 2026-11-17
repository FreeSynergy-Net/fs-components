//! The command model of FreeSynergy programs (plan `PLAN-components-menue`, M1): what a program can do, and where it is offered.
//!
//! Three parts, none of them knows Blinc or a look:
//! - a [`Command`] is what the program publishes: an id, the key of its MF2 text (ADR 0057), a symbol, shortcuts, a
//!   [`State`] (a disabled command always has a reason) and a [`Toggle`] (check or radio);
//! - an [`Arrangement`] is a tree of [`MenuItem`]s per place (menu bar, context menu, ...): command, submenu, separator,
//!   group, placeholder;
//! - [`resolve`] checks both against each other and gives the tree a view draws.
//!
//! Shortcuts are stored as W3C `aria-keyshortcuts` plus the one extension `Primary` (Ctrl, or Cmd on macOS); a conflict
//! between two commands is reported ([`conflicts`]), never decided silently.

mod arrangement;
mod command;
mod commands;
mod conflicts;
mod display;
mod problem;
mod radio;
mod resolve;
mod shortcut;
mod validate;

pub use arrangement::{Arrangement, MenuItem};
pub use command::{Command, CommandId, State, TextKey, Toggle};
pub use commands::Commands;
pub use conflicts::{Conflict, conflicts};
pub use display::{KeyNames, Platform};
pub use problem::Problem;
pub use radio::RadioError;
pub use resolve::{Resolved, resolve};
pub use shortcut::{Modifiers, Shortcut, ShortcutError};
pub use validate::{validate_arrangement, validate_commands};
