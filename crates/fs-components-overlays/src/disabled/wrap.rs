//! `Disabled::wrap`: the decorator.
//!
//! Pattern: Decorator (see `mod.rs`).

use std::sync::Arc;

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
    roving: bool,
    swallow: Option<Arc<dyn Fn() + Send + Sync>>,
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
            roving: false,
            swallow: None,
        }
    }

    /// For a container that must not take the press (a kanban column): the press stops here and runs `f`.
    pub fn swallow(mut self, f: impl Fn() + Send + Sync + 'static) -> Self {
        self.swallow = Some(Arc::new(f));
        self
    }

    /// Menu lines move the keyboard with the arrows (roving tabindex): keep `-1` instead of a tab stop.
    pub fn roving(mut self) -> Self {
        self.roving = true;
        self
    }

    /// The control as it is, same size, plus: class `fs-disabled`, `aria-disabled="true"`, `data-state`, the tab stop kept, and the
    /// reason in a tooltip (hover, focus, and a press: the press opens it and runs nothing). The control must carry no handler of its own.
    pub fn wrap(&self, control: Div) -> Div {
        let tip = Tooltip::new(&self.words).id(&format!("{}-why", self.id));
        if self.why == Why::Unsupported {
            super::observer::unsupported_drawn();
        }
        let press = tip.clone();
        let swallow = self.swallow.clone();
        let control = control
            .class("fs-disabled")
            .attr("aria-disabled", "true")
            .attr("data-state", self.why.state())
            .attr("tabindex", if self.roving { "-1" } else { "0" })
            .on_click(move |_| {
                if let Some(f) = &swallow {
                    f();
                }
                press.open();
            });
        tip.attach(control)
    }
}
