//! The Blinc parts: the pick button and the view of the path field.
//!
//! Pattern: Adapter over `blinc_cn` and the widgets crate (`Live`); this file only declares modules and re-exports. The words
//! come from [`crate::Labels`]; no part here calls a chooser on the window's thread.

mod button;
mod prompt_view;
mod styles;

pub use button::{FilePickButton, Phase};
pub use prompt_view::PathPromptView;
pub use styles::STYLES;
