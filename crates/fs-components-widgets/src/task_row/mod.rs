//! `task-row`: one task as a line - tick | priority | title (and a sub line) | due | a mark that there is a description.
//!
//! Pattern: Composite - the line is made of the same small parts the card and the checklist use (`fs-check-box`, the tag chip of
//! `task-card`); `TaskRow` is a plain description (Builder) and the element has no state of its own: what is ticked comes from the
//! data, the person's tick or open is only reported (`on_toggle`, `on_open`). The row has no words and no colours of its own and does
//! not know what a task is. The list around it is `list_view`'s business (TR2); the form adapter is TR2 too.
//! WAI-ARIA APG, checkbox: the row is a `checkbox` with `aria-checked`, one tab stop. A click on the tick ticks, a click anywhere else
//! on the line opens, the raw key Space ticks, Enter opens. Limit: in a Blinc window Enter and Space reach a widget only as
//! `Key::Activate` (the two cannot be told apart), which ticks.

mod row;
mod styles;

pub use row::{TaskRow, Tone};
pub use styles::STYLES;
