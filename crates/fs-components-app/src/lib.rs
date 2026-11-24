//! `fs-components-app`: the shell every FreeSynergy program puts around its workbenches.
//!
//! A program describes its [`Screen`]s, each a [`Surface`], and calls [`launch`]. The shell brings
//! the window, the title bar with the menus, the navigation and the status line (ADR 0005: the
//! difference between widget and app is the shell). It brings no colours of its own (they come from the theme) and only the
//! words of the standard menus (`texts/`, German and English, ADR 0065); every other label comes from the program. Programs see no Blinc type except [`Element`] (ADR 0009).
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

mod args;
mod context_menu;
mod context_size;
mod context_trigger;
mod error;
mod frame_log;
mod key_events;
mod language;
mod menu;
mod menu_line;
mod menu_model;
mod menu_overlay;
mod menu_view;
#[cfg(test)]
mod menu_view_tests;
mod new_file;
#[cfg(feature = "play")]
mod play;
mod quit;
mod shell;
mod shortcuts;
mod standard;
mod standard_widget;
mod status_panel;
mod styles;
mod surface;
mod texts;
mod view;
mod widget;
mod widget_menu;
mod window;

pub use args::parse_size;
pub use context_trigger::context_trigger;
pub use error::AppError;
/// The plan and the codes of `AppWidget::play` (a described input, W3C Perform Actions).
#[cfg(feature = "play")]
pub use fs_components_replay as replay;
pub use frame_log::{FrameLog, VARIABLE as FRAME_LOG_VARIABLE};
pub use fs_components_widgets::Element;
pub use language::{FileLanguageControl, LanguageChoice, LanguageControl, Scope};
pub use menu::{Mark, Menu, MenuEntry, MenuHandler};
pub use menu_model::{ModelTexts, key_names, shell_words};

/// The command model the menus can be built from (`Command`, `Arrangement`, `resolve`, shortcuts, conflicts).
pub use fs_components_commands as commands;
pub use new_file::{write_new_file, write_new_private_file};
pub use shell::{Shell, Status};
pub use standard::{FrontendItem, StandardCallbacks, StandardMenus};
pub use styles::{MIN_WINDOW, min_surface};
pub use surface::{Screen, Surface};
pub use texts::TEXTS;
pub use widget::AppWidget;
pub use window::{MOTION_PROMOTION, WindowOptions, enable_motion_promotion, launch, open_more};
