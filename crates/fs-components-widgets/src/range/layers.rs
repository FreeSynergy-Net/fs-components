//! The layers of the range slider: what moves (the drawn thumbs) and what holds the keyboard (one invisible layer per thumb).
//!
//! Pattern: Template method — the same split as `Slider::build` (see there for why); this file only draws.

use std::sync::{Arc, PoisonError};

use blinc_layout::div::{Div, div};

use super::view::{Range, Which};
use crate::keys::bind_keys;
use crate::slider::{THUMB, number, rail};

impl Range {
    pub(super) fn thumbs(&self, run: f32) -> Div {
        let (low, high) = self.get();
        let focus = *self.focus.lock().unwrap_or_else(PoisonError::into_inner);
        #[allow(clippy::cast_possible_truncation)]
        let (a, b) = (
            self.scale.fraction(low) as f32 * run,
            self.scale.fraction(high) as f32 * run,
        );
        let drawn = |which: Which, id: &str| {
            let mut t = div()
                .id(format!("{}-{id}-thumb", self.id))
                .class("fs-slider-thumb");
            if focus == Some(which) {
                t = t.class("fs-slider-thumb-focus");
            }
            t
        };
        div()
            .w(run + THUMB * 2.0)
            .h(THUMB)
            .child(rail(run + THUMB * 2.0))
            .child(
                div()
                    .absolute()
                    .left(0.0)
                    .top(0.0)
                    .h(THUMB)
                    .flex_row()
                    .items_center()
                    .child(div().w(a))
                    .child(drawn(Which::Low, "low"))
                    .child(div().class("fs-slider-fill").w((b - a).max(0.0)))
                    .child(drawn(Which::High, "high")),
            )
    }

    /// The two keyboard layers.
    pub(super) fn keyboard(&self, width: f32) -> Div {
        let (low, high) = self.get();
        div()
            .h(0.0)
            .child(self.layer(Which::Low, (low, high), width))
            .child(self.layer(Which::High, (low, high), width))
    }

    fn layer(&self, which: Which, (low, high): (f64, f64), width: f32) -> Div {
        let (id, name, value, min, max) = match which {
            Which::Low => ("low", &self.names.0, low, self.scale.min(), high),
            Which::High => ("high", &self.names.1, high, low, self.scale.max()),
        };
        let (keys, on, off) = (self.clone(), self.clone(), self.clone());
        bind_keys(
            div()
                .id(format!("{}-{id}", self.id))
                .class("fs-slider-keys")
                .absolute()
                .left(0.0)
                .top(0.0)
                .w(width)
                .h(THUMB)
                .attr("role", "slider")
                .attr("aria-label", name)
                .attr("aria-orientation", "horizontal")
                .attr("aria-valuenow", number(value))
                .attr("aria-valuemin", number(min))
                .attr("aria-valuemax", number(max))
                .pointer_events_none(),
            Arc::new(move |event| keys.press(which, event)),
        )
        .on_focus(move |_| on.mark(Some(which)))
        .on_blur(move |_| off.mark(None))
    }

    /// Which thumb has the keyboard (the drawn thumb gets the ring); drawn again only when it changed.
    fn mark(&self, which: Option<Which>) {
        let changed = {
            let mut f = self.focus.lock().unwrap_or_else(PoisonError::into_inner);
            let changed = *f != which;
            *f = which;
            changed
        };
        if changed {
            self.live.refresh_now();
        }
    }
}
