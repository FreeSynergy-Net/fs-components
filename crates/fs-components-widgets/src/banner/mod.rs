//! `banner`: a bar across the top of a window or a page with one message that stays until it is dealt with or dismissed.
//!
//! Pattern: Builder + Observer (`view.rs`: a clonable handle, `Live` draws it again) + Adapter over the `blinc_cn` button.
//! GNOME HIG: Banner (info bar). It is not the ARIA landmark `banner` (the page header): its role is `status` for info and ok,
//! `alert` for warning and error (`Severity::role`, WAI-ARIA APG alert pattern: it takes no keyboard focus by itself). The
//! optional action is a button, the dismiss mark is a tab stop (Enter, Space or Escape on it dismiss). The step is told by glyph,
//! not by colour alone.

mod labels;
mod styles;
mod view;

pub use labels::{Labels, TEXTS};
pub use styles::STYLES;
pub use view::Banner;
