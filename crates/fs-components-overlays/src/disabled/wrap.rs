//! `Disabled::wrap`: the decorator.
//!
//! Pattern: Decorator (see `mod.rs`).

use blinc_layout::div::Div;

use super::labels::Labels;
use crate::tooltip::Tooltip;

/// Why a control cannot be used. Both carry words, never silent.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Why {
    /// Not finished yet (contract state `unsupported`): the words are [`Labels::unsupported`].
    Unsupported,
    /// Exists but not now (contract state `disabled`), in the caller's words.
    Now(String),
}

impl Why {
    /// The value of `data-state`.
    fn state(&self) -> &'static str {
        match self {
            Why::Unsupported => "unsupported",
            Why::Now(_) => "disabled",
        }
    }
}

/// A control that cannot be used.
#[must_use]
pub struct Disabled {
    id: String,
    why: Why,
    words: String,
}

impl Disabled {
    /// A disabled control `id` (the tooltip is `{id}-why`, its bubble `{id}-why-bubble`).
    pub fn new(id: &str, why: Why, labels: &Labels) -> Self {
        let words = match &why {
            Why::Unsupported => labels.unsupported.clone(),
            Why::Now(w) => w.clone(),
        };
        Self {
            id: id.to_owned(),
            why,
            words,
        }
    }

    /// The control as it is, same size, plus: class `fs-disabled`, `aria-disabled="true"`, `data-state`, the tab stop kept, and the
    /// reason in a tooltip (hover, focus, and a press: the press opens it and runs nothing). The control must carry no handler of its own.
    pub fn wrap(&self, control: Div) -> Div {
        let tip = Tooltip::new(&self.words).id(&format!("{}-why", self.id));
        let press = tip.clone();
        let control = control
            .class("fs-disabled")
            .attr("aria-disabled", "true")
            .attr("data-state", self.why.state())
            .attr("tabindex", "0")
            .on_click(move |_| press.open());
        tip.attach(control)
    }
}
