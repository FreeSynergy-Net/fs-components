//! The panel of a dropdown: plain elements with a size known before they are laid out, so the place can be decided at once.
//!
//! Pattern: Builder (rows from entries). Looks are `blinc_cn`'s menu classes, so a style paints it like any other menu.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::overlay_stack::OverlayHandle;
use blinc_theme::{RadiusToken, ThemeState};

/// The height of a line, in pixels.
pub const ROW_HEIGHT: f32 = 32.0;
/// The height of a separator, in pixels.
pub const SEPARATOR_HEIGHT: f32 = 9.0;
/// The border of the panel on each side.
const BORDER: f32 = 1.0;
const TEXT_SIZE: f32 = 14.0;

/// What a chosen line does.
pub type Action = Arc<dyn Fn() + Send + Sync>;

/// One line of a dropdown.
#[derive(Clone)]
pub enum Entry {
    /// A line the person can choose, or not (`action` is `None` when it is disabled).
    Line {
        /// The words.
        label: String,
        /// What happens when it is chosen; `None` for a disabled line.
        action: Option<Action>,
    },
    /// A thin line between groups.
    Separator,
}

/// The height of a panel with these entries.
#[must_use]
pub fn height(entries: &[Entry]) -> f32 {
    let lines: f32 = entries
        .iter()
        .map(|e| match e {
            Entry::Line { .. } => ROW_HEIGHT,
            Entry::Separator => SEPARATOR_HEIGHT,
        })
        .sum();
    lines + 2.0 * BORDER
}

/// The panel `width` x `shown` pixels; a panel shorter than its entries scrolls. A chosen line closes `handle` first, then acts.
/// Runs under the lock of the overlay stack: builds elements only.
pub fn build(
    entries: &[Entry],
    id: &str,
    (width, shown): (f32, f32),
    handle: OverlayHandle,
) -> Div {
    let radius = ThemeState::get().radius(RadiusToken::Md);
    let mut panel = div()
        .class("cn-menu")
        .class("cn-dropdown-menu")
        .attr("role", "menu")
        .id(id)
        .flex_col()
        .rounded(radius)
        .w(width)
        .h(shown);
    if shown < height(entries) {
        panel = panel.overflow_y_scroll();
    } else {
        panel = panel.overflow_clip();
    }
    for entry in entries {
        panel = panel.child(match entry {
            Entry::Separator => separator(),
            Entry::Line { label, action } => line(label, action.clone(), handle),
        });
    }
    panel
}

fn separator() -> Div {
    div()
        .w_full()
        .h(SEPARATOR_HEIGHT)
        .flex_col()
        .justify_center()
        .child(div().class("cn-menu-separator").w_full().h(1.0))
}

fn line(label: &str, action: Option<Action>, handle: OverlayHandle) -> Div {
    let disabled = action.is_none();
    div()
        .class("cn-menu-item")
        .class("cn-dropdown-item")
        .attr("role", "menuitem")
        .attr("tabindex", "-1")
        .attr_when("disabled", "", disabled)
        .w_full()
        .h(ROW_HEIGHT)
        .px(12.0)
        .flex_row()
        .items_center()
        .child(
            text(label)
                .size(TEXT_SIZE)
                .no_wrap()
                .no_cursor()
                .pointer_events_none(),
        )
        .on_click(move |_| {
            if let Some(action) = &action {
                // Close first, so that what the action opens (a confirm dialog) is not closed with the menu.
                handle.close();
                action();
            }
        })
}
