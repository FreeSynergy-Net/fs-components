//! `fs-components-measures`: the measures of the widgets, cut out of `fs-components-widgets` (split1, step C).
//!
//! Slider, range, meter and progress. They build on `fs-components-core`;
//! `fs-components-widgets` re-exports them under the old paths.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

// The modules reach the core by `crate::<name>` as they did inside the widgets.
use fs_components_core::{Element, keys, live};

pub mod meter;
pub mod progress;
pub mod range;
pub mod slider;
