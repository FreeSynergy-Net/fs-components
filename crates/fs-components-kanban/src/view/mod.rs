//! The views of the board.
//!
//! Pattern: none — declares the modules and re-exports.

mod board;
mod card;
mod column;
mod controls;
mod keys;
mod quick_add;

pub use board::board;
pub use keys::bind_keys;
