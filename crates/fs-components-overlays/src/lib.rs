//! `fs-components-overlays`: the overlays of the widgets, cut out of `fs-components-widgets` (split1, step C).
//!
//! Popover, tooltip, toast, dialogs and the name dialog: things that stand above the page.
//! They build on `fs-components-core`; `fs-components-widgets` re-exports them under the old paths.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

// The modules reach the core by `crate::<name>` as they did inside the widgets.
use fs_components_core::{Element, focus, icon, keys, live, states};

pub mod dialogs;
pub mod disabled;
pub mod name_dialog;
pub mod popover;
pub mod toast;
pub mod tooltip;
