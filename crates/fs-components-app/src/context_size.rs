//! How big a context menu is before it is laid out (M3): its width and a floor for its height.
//!
//! Pattern: none (pure functions over the lines). The width decides the side of the point, the height decides above or below it.

use crate::menu::MenuEntry;
use crate::menu_view::content_width_max;
use crate::styles::{MENU_MAX_WIDTH, MENU_WIDTH, ROW_BUTTON};

/// The width of the panel: the usual width, or as wide as the lines need where they have symbols, shortcuts or reasons, and wide
/// enough for a row of symbols.
pub(crate) fn panel_width(entries: &[MenuEntry], window: f32) -> f32 {
    let rich = entries
        .iter()
        .any(|e| e.icon.is_some() || e.shortcut.is_some() || e.reason.is_some());
    let base = if rich {
        content_width_max(entries, window, MENU_MAX_WIDTH).max(MENU_WIDTH)
    } else {
        MENU_WIDTH
    };
    #[allow(clippy::cast_precision_loss)] // a row has a handful of buttons
    let row = entries.iter().map(|e| e.row.len()).max().unwrap_or(0) as f32 * ROW_BUTTON + 12.0;
    base.max(row)
}

/// What a line of a context menu measures once laid out (measured in the real view: 41 px a text line, 14 px more for a reason, 17 px a
/// separator, 48 px a row of symbols).
const LINE: f32 = 41.0;
const REASON: f32 = 14.0;
const SEPARATOR: f32 = 17.0;
const ROW: f32 = 48.0;

/// The height of the panel as far as it is known before it is laid out: what decides above or below the point. A line that
/// wraps makes it taller, so it is a floor.
pub(crate) fn estimate_height(entries: &[MenuEntry]) -> f32 {
    entries
        .iter()
        .map(|e| {
            if e.separator {
                SEPARATOR
            } else if !e.row.is_empty() {
                ROW
            } else if e.reason.is_some() {
                LINE + REASON
            } else {
                LINE
            }
        })
        .sum()
}
