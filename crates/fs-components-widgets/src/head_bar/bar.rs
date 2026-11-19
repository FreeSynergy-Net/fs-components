//! `head-bar`: the bar on top of an area, with a button or two at each end and a title between them (the header of the
//! contacts list: plus, title, check, menu; the header of a person: pencil, dots).
//!
//! Pattern: Composite — a bar is three slots (left, title, right) filled with elements; [`head_button`] is the round symbol button
//! that goes into them. The bar knows no program: a press is a callback, and every button says in words what it does
//! (`aria-label`, title) because a symbol alone cannot be read aloud.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;

use blinc_theme::{ColorToken, ThemeState};

use crate::icon::{Glyph, Tone, icon_stroked};

/// The style sheet of the bar and its buttons.
pub const STYLES: &str = r"
.fs-head-bar { flex-shrink: 0; align-items: center; gap: var(--space-2); padding-left: var(--space-3); padding-right: var(--space-3); height: var(--space-14); }
.fs-head-bar-left { align-items: center; gap: var(--space-1); flex-shrink: 0; }
.fs-head-bar-right { align-items: center; gap: var(--space-1); flex-shrink: 0; justify-content: flex-end; }
.fs-head-bar-title { flex-grow: 1; min-width: 0; align-items: center; justify-content: center; font-size: var(--text-base); font-weight: 600; color: var(--text-primary); }
.fs-head-button { width: var(--space-9); height: var(--space-9); border-radius: var(--radius-default); align-items: center; justify-content: center; cursor: pointer; }
.fs-head-button:hover { background: var(--accent-subtle); }
.fs-head-button-on { background: var(--accent-subtle); }
.fs-head-button:focus { background: var(--accent-subtle); }
";

/// What a press on a button does.
pub type Press = Arc<dyn Fn() + Send + Sync>;

/// Pixels of the symbol in a button.
const SYMBOL: f32 = 18.0;

/// The look of a button, with no behaviour: for a trigger that another element (a dropdown) makes pressable and a tab stop.
/// `spoken` says what it does, for a screen reader and as the tooltip. `turned` rotates the symbol a quarter turn (three dots become
/// the vertical ones).
pub fn head_face(glyph: Glyph, spoken: &str, turned: bool) -> Div {
    let symbol = if turned && matches!(glyph, Glyph::More) {
        vertical_dots()
    } else {
        let symbol = div().child(icon_stroked(glyph, SYMBOL, Tone::Normal, Some(1.8)));
        if turned {
            symbol.rotate_deg(90.0)
        } else {
            symbol
        }
    };
    div()
        .class("fs-head-button")
        .flex_row()
        .attr("aria-label", spoken)
        .attr("title", spoken)
        .child(symbol)
}

/// Three dots one above the other, drawn as round boxes: the glyph `More` turned is only dots of a hair's length and cannot be seen.
fn vertical_dots() -> Div {
    let colour = ThemeState::get().color(ColorToken::TextPrimary);
    let mut column = div().flex_col().items_center().gap_px(3.0);
    for _ in 0..3 {
        column = column.child(div().w(4.0).h(4.0).rounded(2.0).bg(colour));
    }
    column
}

/// A round button with a symbol, a tab stop. `id` lets a program or a test find it; see [`head_face`] for the rest.
pub fn head_button(id: &str, glyph: Glyph, spoken: &str, turned: bool, press: &Press) -> Div {
    let press = Arc::clone(press);
    head_face(glyph, spoken, turned)
        .id(id)
        .attr("tabindex", "0")
        .attr("role", "button")
        .on_click(move |_| press())
}

/// The bar.
#[derive(Default)]
pub struct HeadBar {
    id: Option<String>,
    level: Option<u8>,
    left: Vec<Div>,
    title: Option<String>,
    right: Vec<Div>,
}

impl HeadBar {
    /// An empty bar.
    pub fn new() -> Self {
        Self::default()
    }

    /// The id of the bar: the title is `{id}-title`. Without one the bar has no id (a program that does not need to find it).
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        self.id = Some(id.to_owned());
        self
    }

    /// The level of the heading the title is (`aria-level`, 1 to 6); 1 unless told otherwise (a bar inside a page may be a lower level).
    #[must_use]
    pub fn level(mut self, level: u8) -> Self {
        self.level = Some(level.clamp(1, 6));
        self
    }

    /// An element at the left end.
    #[must_use]
    pub fn left(mut self, element: Div) -> Self {
        self.left.push(element);
        self
    }

    /// The title between the ends.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// An element at the right end.
    #[must_use]
    pub fn right(mut self, element: Div) -> Self {
        self.right.push(element);
        self
    }

    /// How many elements stand at the ends (left, right).
    pub fn counts(&self) -> (usize, usize) {
        (self.left.len(), self.right.len())
    }

    /// The element to place in a layout. The slots are always there, so the title stays in the middle whatever stands at the ends.
    pub fn build(self) -> Div {
        // WAI-ARIA APG toolbar: a row of controls with a name (the title). Limit: Tab walks the buttons; the arrow keys of the pattern
        // need the host to move the keyboard (`FocusTo`) and are a follow-up.
        let named = self.title.clone().filter(|t| !t.is_empty());
        let mut bar = div()
            .class("fs-head-bar")
            .flex_row()
            .w_full()
            .attr("role", "toolbar")
            .attr("aria-orientation", "horizontal");
        if let Some(name) = &named {
            bar = bar.attr("aria-label", name);
        }
        let title_id = self.id.as_ref().map(|id| format!("{id}-title"));
        if let Some(id) = &self.id {
            bar = bar.id(id);
        }
        bar = bar.child(
            div()
                .class("fs-head-bar-left")
                .flex_row()
                .children(self.left),
        );
        let mut title = div()
            .class("fs-head-bar-title")
            .flex_row()
            .attr("role", "heading")
            .attr("aria-level", self.level.unwrap_or(1).to_string());
        if let Some(id) = title_id {
            title = title.id(id);
        }
        bar = bar.child(title.child(text(self.title.unwrap_or_default()).no_cursor()));
        bar.child(
            div()
                .class("fs-head-bar-right")
                .flex_row()
                .children(self.right),
        )
    }
}
