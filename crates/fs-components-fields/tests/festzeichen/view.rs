//! P1 probe: the picture side: one frame, three Blinc text inputs, the signs between them solid or grey, the suggested ending grey behind the typed text.
//!
//! Tests: `festzeichen.rs` (accessibility tree) and the picture test.

use std::sync::Arc;

use blinc_core::Color;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use blinc_layout::text_measure::measure_text;
use blinc_layout::widgets::text_input::text_input;

use super::address::{AddressField, IDS, Mutation};

const GHOST: Color = Color::rgba(0.62, 0.62, 0.66, 1.0);
const FONT: f32 = 14.0;
const PAD: f32 = 6.0;

impl AddressField {
    /// The element. Signs not yet taken are grey, taken ones solid; the suggested ending stands grey right behind the typed text
    /// (placed by measuring the typed text: it is drawn from the state when built, a live update needs the caller to rebuild: not probed).
    pub fn view(self: &Arc<Self>, error: bool) -> Div {
        let part = |i: usize, w: f32, hint: &str| {
            let (me, me2) = (Arc::clone(self), Arc::clone(self));
            let input = text_input(&self.segs[i])
                .id(IDS[i])
                .w(w)
                .border_width(0.0)
                .padding_x(PAD)
                .text_size(FONT)
                .placeholder(hint)
                .placeholder_color(GHOST)
                .accessibility_label(["Name before the at sign", "Server", "Ending"][i])
                .on_change(move |_| me.typed(i))
                .on_paste_override(move |s| {
                    me2.paste(i, s);
                    true
                });
            let mut cell = div().relative().w(w).h(36.0).child(input);
            let rest = if i == 2 {
                self.ghost_tld()
            } else {
                String::new()
            };
            if !rest.is_empty() {
                let x = PAD + measure_text(&self.value(2), FONT).width;
                cell = cell.child(
                    div()
                        .absolute()
                        .left(x)
                        .top(9.0)
                        .child(text(rest).size(FONT).color(GHOST)),
                );
            }
            cell
        };
        let sign = |k: usize, s: &str| {
            let c = if self.real(k) || self.mutation == Mutation::GhostInCopy {
                Color::rgba(0.1, 0.1, 0.12, 1.0)
            } else {
                GHOST
            };
            div().child(text(s).size(18.0).color(c))
        };
        let colour = if error {
            Color::rgba(0.8, 0.1, 0.1, 1.0)
        } else {
            Color::rgba(0.5, 0.5, 0.55, 1.0)
        };
        let (me, me4, me5) = (Arc::clone(self), Arc::clone(self), Arc::clone(self));
        div()
            .id("addr")
            .flex_row()
            .items_center()
            .w(440.0)
            .h(40.0)
            .rounded(6.0)
            .border(if error { 2.0 } else { 1.0 }, colour)
            .on_key_down(move |c| me.key_down(c.key_code, c.ctrl || c.meta))
            .on_key_up(move |c| me4.key_up(c.key_code, c.ctrl || c.meta))
            .on_mouse_up(move |_| (0..3).for_each(|i| me5.remember(i)))
            .child(part(0, 150.0, "name"))
            .child(sign(0, "@"))
            .child(part(1, 150.0, "example"))
            .child(sign(1, "."))
            .child(part(2, 70.0, ""))
    }
}
