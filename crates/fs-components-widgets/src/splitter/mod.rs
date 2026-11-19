//! `splitter`: a movable divider between two panes (the window splitter).
//!
//! Pattern: State (`model.rs`, pure: the position in percent, the limits, collapse) + Observer (`view.rs`: `Live` draws the panes again
//! after every move, the divider's own element stays so a drag is not cut off) + Adapter. WAI-ARIA APG, window splitter: the divider
//! is a focusable `separator` with `aria-valuenow` (the size of the first pane in percent), `aria-valuemin`, `aria-valuemax`,
//! `aria-controls` (the first pane) and `aria-orientation` (`vertical` for a divider between left and right panes, as ARIA says),
//! named by `aria-label`. Left and Right (Up and Down for panes one above the other) move it by a step, Home and End go to the least and
//! the greatest size, Enter collapses the first pane and Enter again gives it back (in a Blinc window Space does the same: the window hands Enter and Space over as one key). The pointer drags it. Limits: two panes only;
//! the size is computed from the width given to `build`; the first pane's collapsed state is only the position 0.

mod model;
mod styles;
mod view;

pub use model::{Outcome, State};
pub use styles::STYLES;
pub use view::Splitter;
