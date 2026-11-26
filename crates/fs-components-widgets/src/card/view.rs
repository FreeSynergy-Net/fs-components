//! The element of a card: the frame, the classes of its state, the handlers of the pointer and the keys.
//!
//! Pattern: Adapter (split from `widget.rs`): the Blinc events are turned into the calls of the card's own methods; no rule lives here.

use std::sync::Arc;

use blinc_layout::div::{Div, div};

use super::keys::Mode;
use super::widget::{Card, Outcome};
use crate::keys::{KeyEvent, bind_keys};

impl Card {
    /// The element: the parts (header, title, body, footer, as the caller made them with the classes `fs-card-header` ...) inside the
    /// frame of the card. The look follows the state at this moment; the owner draws again when [`CardEvents::changed`] says so.
    pub fn build(&self, parts: Vec<Div>) -> Div {
        let mut root = div().id(&self.id).class("fs-card").cursor_pointer();
        if self.panel {
            root = root.class("fs-card-panel");
        }
        if self.shared {
            root = root.class("fs-card-shared");
        }
        match self.mode() {
            Mode::Held => root = root.class("fs-card-held"),
            Mode::Carried => root = root.class("fs-card-carried"),
            Mode::Locked => root = root.class("fs-card-locked"),
            Mode::Rest => {}
        }
        let (down, drag, end, click, menu) = (
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
        );
        let mut inner = div().flex_col().w_full();
        for part in parts {
            inner = inner.child(part);
        }
        inner = inner
            .on_mouse_down(move |ctx| {
                if ctx.mouse_button == 0 {
                    down.press((ctx.mouse_x, ctx.mouse_y));
                }
            })
            .on_drag(move |ctx| {
                drag.pointer((ctx.mouse_x, ctx.mouse_y));
            })
            .on_drag_end(move |_| {
                end.release();
            })
            .on_click(move |_| {
                click.release();
            })
            .on_right_click(move |ctx| {
                menu.events.menu(&menu.key, (ctx.mouse_x, ctx.mouse_y));
            });
        let keys = self.clone();
        bind_keys(
            root.child(inner),
            Arc::new(move |e: KeyEvent| keys.key_press(&e) != Outcome::Nothing),
        )
    }
}
