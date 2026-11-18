//! `field-card`: one field of a person as a rounded card, the symbol of the kind of field at the left and the actions at the right
//! (write a mail, copy), as in GNOME Contacts.
//!
//! Pattern: Composite (symbol, text, actions) + Command (each action is a callback with its own words). The card knows no
//! person: the caller gives what the field is called, what it holds and which actions it offers. The label is small and
//! above the value; an empty label shows only the value. Every action is a tab stop with its words as `aria-label`.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;

use crate::icon::{Glyph, Tone, icon_stroked};

/// The style sheet of the card.
pub const STYLES: &str = r"
.fs-field-card { align-items: center; gap: var(--space-3); padding: var(--space-3) var(--space-4); min-height: var(--space-14); border-radius: var(--radius-lg); background: var(--surface-elevated); border-width: 1px; border-color: var(--border); }
.fs-field-card-body { flex-grow: 1; min-width: 0; gap: var(--space-1); }
.fs-field-card-label { color: var(--text-secondary); font-size: var(--text-sm); }
.fs-field-card-value { color: var(--text-primary); font-size: var(--text-base); }
.fs-field-card-actions { flex-shrink: 0; align-items: center; gap: var(--space-1); }
.fs-field-card-action { width: var(--space-9); height: var(--space-9); border-radius: var(--radius-default); align-items: center; justify-content: center; cursor: pointer; }
.fs-field-card-action:hover { background: var(--accent-subtle); }
.fs-field-card-action:focus { background: var(--accent-subtle); }
";

/// What an action does.
pub type Act = Arc<dyn Fn() + Send + Sync>;

/// One action at the right of a card.
#[derive(Clone)]
pub struct CardAction {
    /// The symbol.
    pub glyph: Glyph,
    /// What it does, in words (the program's language).
    pub spoken: String,
    /// The press.
    pub act: Act,
}

impl CardAction {
    /// An action.
    pub fn new(glyph: Glyph, spoken: impl Into<String>, act: Act) -> Self {
        Self {
            glyph,
            spoken: spoken.into(),
            act,
        }
    }
}

/// One field.
#[derive(Clone)]
pub struct FieldCard {
    glyph: Glyph,
    label: String,
    value: String,
    actions: Vec<CardAction>,
}

impl FieldCard {
    /// A card for `value` of the field `label`, with the symbol `glyph`.
    pub fn new(glyph: Glyph, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            glyph,
            label: label.into(),
            value: value.into(),
            actions: Vec::new(),
        }
    }

    /// One more action at the right.
    #[must_use]
    pub fn action(mut self, action: CardAction) -> Self {
        self.actions.push(action);
        self
    }

    /// The words of the actions, in order (for a test or a screen reader).
    pub fn spoken(&self) -> Vec<&str> {
        self.actions.iter().map(|a| a.spoken.as_str()).collect()
    }

    /// Is the label drawn? (An empty label shows only the value.)
    pub fn shows_label(&self) -> bool {
        !self.label.trim().is_empty()
    }

    /// The element; `id` is the base of the ids of the card (`<id>`, `<id>-action-<n>`).
    pub fn build(&self, id: &str) -> Div {
        let mut body = div().class("fs-field-card-body").flex_col();
        if self.shows_label() {
            body = body.child(div().class("fs-field-card-label").child(text(&self.label)));
        }
        body = body.child(div().class("fs-field-card-value").child(text(&self.value)));
        let mut actions = div().class("fs-field-card-actions").flex_row();
        for (n, action) in self.actions.iter().enumerate() {
            let act = Arc::clone(&action.act);
            actions = actions.child(
                div()
                    .id(format!("{id}-action-{n}"))
                    .class("fs-field-card-action")
                    .flex_row()
                    .attr("tabindex", "0")
                    .attr("role", "button")
                    .attr("aria-label", &action.spoken)
                    .attr("title", &action.spoken)
                    .on_click(move |_| act())
                    .child(icon_stroked(action.glyph, 18.0, Tone::Normal, Some(1.6))),
            );
        }
        // A group named by what it holds: the label, or the value when there is no label (APG: a card has no role of its own).
        let name = if self.shows_label() {
            &self.label
        } else {
            &self.value
        };
        div()
            .id(id)
            .class("fs-field-card")
            .attr("role", "group")
            .attr("aria-label", name)
            .flex_row()
            .w_full()
            .child(icon_stroked(self.glyph, 20.0, Tone::Normal, Some(1.6)))
            .child(body)
            .child(actions)
    }
}
