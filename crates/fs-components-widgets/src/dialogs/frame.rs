//! The frame every dialog of the family shares: title, body, footer, and the two keys.
//!
//! Pattern: Template method (a dialog fills the parts, the frame puts them in order and binds Enter and Escape) + Adapter over
//! the `blinc_cn` button. Enter and Escape reach the frame as broadcast keys; when a button has the keyboard the window turns
//! Enter into a click on it first, so a focused button always wins over the default action.

use std::sync::Arc;

use blinc_cn::components::button::{Button, ButtonBuilder, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

/// What a button or a key does.
pub(super) type Act = Arc<dyn Fn() + Send + Sync>;

/// One footer button; its element id is `{id}-{tag}`.
pub(super) fn button(
    id: &str,
    tag: &str,
    word: &str,
    variant: ButtonVariant,
    disabled: bool,
    act: Act,
) -> Button {
    let full = format!("{id}-{tag}");
    ButtonBuilder::with_key(full.clone(), word.to_owned())
        .variant(variant)
        .disabled(disabled)
        .on_click(move |_| act())
        .build_component()
        .id(&full)
}

/// The parts of one dialog.
pub(super) struct Parts {
    /// The element id; the parts are `{id}-title`, `{id}-body`, `{id}-footer`.
    pub id: String,
    /// The kind (`confirm`, `choice`, `license`), a class `fs-dlg-{kind}`.
    pub kind: &'static str,
    /// The state classes of the root (`danger`, `busy`).
    pub states: Vec<&'static str>,
    /// The title.
    pub title: String,
    /// The body.
    pub body: Div,
    /// The buttons, left to right.
    pub footer: Vec<Button>,
    /// Enter, when no button has the keyboard.
    pub enter: Option<Act>,
    /// Escape.
    pub escape: Option<Act>,
}

/// The dialog as one element.
pub(super) fn frame(p: Parts) -> Div {
    let mut footer = div().id(format!("{}-footer", p.id)).class("fs-dlg-footer");
    for b in p.footer {
        footer = footer.child(b);
    }
    let mut root = div()
        .id(&p.id)
        .class("fs-dlg")
        .class(format!("fs-dlg-{}", p.kind))
        .attr("role", "dialog")
        .attr("aria-modal", "true")
        .attr("aria-labelledby", format!("{}-title", p.id));
    for state in p.states {
        root = root.class(format!("fs-dlg-{state}"));
    }
    let (enter, escape) = (p.enter, p.escape);
    root.child(
        text(p.title)
            .id(format!("{}-title", p.id))
            .class("fs-dlg-title"),
    )
    .child(
        div()
            .id(format!("{}-body", p.id))
            .class("fs-dlg-body")
            .child(p.body),
    )
    .child(footer)
    .on_key_down(move |ctx| match ctx.key_code {
        13 => {
            if let Some(act) = &enter {
                act();
            }
        }
        27 => {
            if let Some(act) = &escape {
                act();
            }
        }
        _ => {}
    })
}
