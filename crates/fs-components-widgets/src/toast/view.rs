//! The toast stack as elements.
//!
//! Pattern: Adapter — the pure stack in, Blinc layout out. Each toast is a row: the glyph of its kind, the text, an optional
//! action button, a close mark. The close mark is the tab stop of a toast (Escape or Enter on it closes); a pointer over a toast
//! holds its time.

use std::sync::Arc;

use blinc_cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::handle::Toasts;
use super::model::Item;
use crate::icon::{Tone, icon_stroked};
use crate::keys::{Key, bind_keys};
use crate::states::STROKE;

impl Toasts {
    pub(super) fn render(&self) -> Div {
        let (shown, hidden) = {
            let stack = self.stack();
            let shown: Vec<Div> = stack.shown().iter().map(|i| self.row(i)).collect();
            (shown, stack.hidden())
        };
        let mut root = div().id(&self.id).class("fs-toasts");
        if hidden > 0 {
            root = root.child(
                text(self.labels.more(hidden))
                    .id(format!("{}-more", self.id))
                    .class("fs-toasts-more"),
            );
        }
        for row in shown {
            root = root.child(row);
        }
        root
    }

    fn row(&self, item: &Item) -> Div {
        let id = item.id;
        let kind = item.toast.kind;
        let (enter, leave, act) = (self.clone(), self.clone(), self.clone());
        let mut row = div()
            .id(format!("{}-{id}", self.id))
            .class("fs-toast")
            .class(format!("fs-toast-{}", kind.name()))
            .attr("role", kind.role())
            .attr(
                "aria-live",
                if kind.role() == "alert" {
                    "assertive"
                } else {
                    "polite"
                },
            )
            .attr("aria-atomic", "true")
            .child(div().class("fs-toast-icon").child(icon_stroked(
                kind.glyph(),
                16.0,
                Tone::Normal,
                Some(STROKE),
            )))
            .child(text(item.toast.text.clone()).class("fs-toast-text"))
            .on_hover_enter(move |_| enter.hold(id, true))
            .on_hover_leave(move |_| leave.hold(id, false));
        if let Some((label, _)) = &item.toast.action {
            row = row.child(
                ButtonBuilder::with_key(format!("{}-{id}-action", self.id), label.clone())
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Small)
                    .on_click(move |_| {
                        act.run_action(id);
                    })
                    .build_component()
                    .id(&format!("{}-{id}-action", self.id)),
            );
        }
        let closer = div()
            .id(format!("{}-{id}-close", self.id))
            .class("fs-toast-close")
            .attr("role", "button")
            .attr("aria-label", &self.labels.close)
            .child(icon_stroked(
                crate::icon::Glyph::Close,
                14.0,
                Tone::Normal,
                Some(STROKE),
            ));
        let key = self.clone();
        let click = self.clone();
        let (focus_in, focus_out) = (self.clone(), self.clone());
        let closer = bind_keys(
            closer,
            Arc::new(move |event| match event.key {
                Key::Escape | Key::Activate | Key::Enter => key.dismiss(id),
                _ => false,
            }),
        )
        .on_focus(move |_| focus_in.hold_keyboard(id, true))
        .on_blur(move |_| focus_out.hold_keyboard(id, false))
        .on_click(move |_| {
            click.dismiss(id);
        });
        row.child(closer)
    }
}
