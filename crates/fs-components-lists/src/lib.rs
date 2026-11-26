//! `fs-components-lists`: the lists of the widgets, cut out of `fs-components-widgets` (split1, step C).
//!
//! Listbox, list view, grid, table, tree, tree view, feed, sortable list and chip. They build on
//! `fs-components-core`; `fs-components-widgets` re-exports them under the old paths.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

// The modules reach the core by `crate::<name>` as they did inside the widgets.
use fs_components_core::{Element, icon, keys, live, roving};

pub mod chip;
pub mod feed;
pub mod grid;
pub mod list_view;
pub mod listbox;
pub(crate) mod rows_form;
pub mod sortable_list;
pub mod table;
pub mod tree;
pub mod tree_view;

#[cfg(test)]
mod rows_form_cursor_tests;
#[cfg(test)]
mod rows_form_props;
#[cfg(test)]
mod rows_form_tests;
