//! The controls `switch` and `segmented`: a person sets one value with one gesture.
//!
//! Pattern: State (a pure value per control) + Observer (`Live` draws it again) + Adapter (`activatable`: the one place that makes
//! an element a tab stop and joins the keyboard and the pointer without doubling a press). A control owns no wording: the labels
//! are the caller's. Both are one tab stop (WAI-ARIA `switch`, `radiogroup`).

mod segmented;
mod styles;
mod switch;

pub use segmented::{Segment as SegmentOption, Segmented};
pub use styles::STYLES;
pub use switch::{Switch, SwitchState};

use std::sync::Arc;

use blinc_layout::div::Div;
use blinc_layout::event_router::keyboard_focused_node;
use fs_components_widgets::keys::{Handled, KeyEvent, bind_keys};

/// `root` as a tab stop. `keys` gets every key while `root` has the keyboard (Enter and Space arrive as `Key::Activate`);
/// `press` runs on a click of the pointer. A press of the keyboard is NOT also a click: the window turns Enter and Space into a
/// click on the focused element, and `bind_keys` reports that as `Key::Activate`; the pointer branch looks at who has the
/// keyboard and stays out then.
pub(crate) fn activatable(
    root: Div,
    keys: Arc<dyn Fn(KeyEvent) -> Handled + Send + Sync>,
    press: Arc<dyn Fn() + Send + Sync>,
) -> Div {
    bind_keys(root, keys).on_click(move |ctx| {
        if keyboard_focused_node() != Some(ctx.node_id) {
            press();
        }
    })
}
