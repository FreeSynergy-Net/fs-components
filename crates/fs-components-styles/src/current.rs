//! The style the process draws with now.
//!
//! Pattern: none — one process-wide value (an atomic index into [`StyleId::ALL`]). Why here: the colours and the icon set of a
//! style are read by widgets that do not know the window (`fs-components-widgets` icons), and the window may change the style while
//! it runs; the Blinc theme is changed by `fs_components_render::set_style`, which sets this value too.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::style::StyleId;

/// Index of the chosen style in [`StyleId::ALL`]; Admin (0) until one is chosen.
static CURRENT: AtomicUsize = AtomicUsize::new(0);

/// The style in use now; [`StyleId::Admin`] until [`set_current_style`] says another.
pub fn current_style() -> StyleId {
    StyleId::ALL[CURRENT.load(Ordering::Relaxed).min(StyleId::ALL.len() - 1)]
}

/// Say which style is in use. A widget that is built afterwards reads its colours and icons from it.
pub fn set_current_style(id: StyleId) {
    let at = StyleId::ALL.iter().position(|s| *s == id).unwrap_or(0);
    CURRENT.store(at, Ordering::Relaxed);
}
