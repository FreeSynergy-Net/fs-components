//! `popover`: a small panel that opens next to its trigger, holds a little content and closes again.
//!
//! Pattern: Facade + Observer (`view.rs`: a clonable handle, `Live` draws it again) + Adapter. WAI-ARIA has no popover pattern; the
//! panel follows the APG dialog pattern (non-modal): the trigger carries `aria-haspopup="dialog"`, `aria-expanded` and
//! `aria-controls`, the panel has role `dialog` and a name, Escape closes it and the keyboard goes BACK to the trigger. A widget
//! cannot move the keyboard, so closing asks the host through `FocusTo` (see `focus.rs`). GNOME HIG: Popover. Limits: the panel
//! sits under the trigger in the flow of its host (no overlay, no flip at the window edge).

mod labels;
mod styles;
mod view;

pub use labels::{Labels, TEXTS};
pub use styles::STYLES;
pub use view::Popover;
