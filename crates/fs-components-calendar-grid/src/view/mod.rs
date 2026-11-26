//! The views of the grid.
//!
//! Pattern: none — declares the modules and re-exports.

mod agenda;
mod bars;
mod body;
mod grid;
mod head;
mod keys;
mod marks;
mod month;
mod painter;
mod parts;
mod timed;
mod toolbar;
mod week_numbers;

pub use grid::grid;
pub use keys::bind_keys;
