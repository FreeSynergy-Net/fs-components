//! The look of the people workbench: pure functions from a snapshot of the state to elements.
//!
//! Pattern: Adapter (state to Blinc elements) — each function draws one area and reports what the
//! person does through [`PeopleActions`]; nothing here changes state itself. No function here is
//! ever called with a lock held.

mod actions;
mod detail;
mod form;
mod frame;
mod keys;
mod list;
mod person;
mod styles;

pub(super) use actions::PeopleActions;
pub use actions::MenuFor;
pub(super) use detail::detail;
pub(super) use frame::{Areas, frame};
pub(super) use list::list;
pub use styles::styles;
