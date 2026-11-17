//! The command model of FreeSynergy programs (plan `PLAN-components-menue`, M1): what a program can do, and where it is offered.
//!
//! Three parts, none of them knows Blinc or a look:
//! - a [`Command`] is what the program publishes: an id, the key of its MF2 text (ADR 0057), a symbol, shortcuts, a
//!   [`State`] (a disabled command always has a reason) and a [`Toggle`] (check or radio);
//! - an [`Arrangement`] is a tree of [`MenuItem`]s per place (menu bar, context menu, ...): command, submenu, separator,
//!   group, placeholder;
//! - [`resolve`] checks both against each other and gives the tree a view draws.
//!
//! M2 adds the dispatcher: a [`KeyPress`] and a [`FocusContext`] go into a [`Keymap`] and come out as an [`Outcome`] (run, disabled with
//! reason, no match, conflict). A locked command never runs, a conflict runs nothing, and a text field keeps its own keys.
//!
//! M3 adds the typed parameters and the result of a command ([`Param`], [`ParamType`], [`ResultType`]), the exporter to building blocks
//! of a workflow ([`export_blocks`]), the standard menu as fixed commands `app.*` ([`standard`]) and the pure model of the context menu
//! ([`ContextState`], [`Trigger`], [`stages`]).
//!
//! Shortcuts are stored as W3C `aria-keyshortcuts` plus the one extension `Primary` (Ctrl, or Cmd on macOS); a conflict
//! between two commands is reported ([`conflicts`]), never decided silently.

mod arrangement;
mod blocks;
mod command;
mod commands;
mod conflicts;
mod context;
mod context_state;
mod display;
mod keymap;
mod keypress;
mod parameter;
mod problem;
mod radio;
mod resolve;
mod shortcut;
pub mod standard;
mod text_keys;
mod validate;

pub use arrangement::{Arrangement, MenuItem};
pub use blocks::{Block, export_blocks};
pub use command::{Command, CommandId, State, TextKey, Toggle};
pub use commands::Commands;
pub use conflicts::{Conflict, conflicts};
pub use context::{Area, MORE_KEY, Point, Stage, Stages, Target, Trigger, stages, target_for};
pub use context_state::{Choice, ContextState};
pub use display::{KeyNames, Platform};
pub use keymap::{Keymap, Outcome};
pub use keypress::{FocusContext, KeyPress};
pub use parameter::{Param, ParamType, ResultType};
pub use problem::Problem;
pub use radio::RadioError;
pub use resolve::{Resolved, resolve};
pub use shortcut::{Modifiers, Shortcut, ShortcutError};
pub use standard::{standard_commands, standard_menubar, with_standard};
pub use validate::{validate_arrangement, validate_commands};
