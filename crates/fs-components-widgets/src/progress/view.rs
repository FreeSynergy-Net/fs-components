//! The progress bar as an element and its handle.
//!
//! Pattern: Observer + Facade — a clonable handle over the value (`None`: unknown); `set` draws the bar again through `Live`.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::Element;
use crate::live::Live;

/// A progress bar. Clones share the value.
#[derive(Clone)]
pub struct Progress {
    id: String,
    name: String,
    value: Arc<Mutex<Option<f64>>>,
    live: Live,
}

/// A value as the bar says it: whole percent.
fn percent(value: f64) -> u32 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let whole = value.clamp(0.0, 100.0).round() as u32;
    whole
}

impl Progress {
    /// A bar named `name` ("Copying") at 0 %. Ids: `<id>`, `<id>-fill`.
    pub fn new(name: &str) -> Self {
        Self {
            id: "progress".to_owned(),
            name: name.to_owned(),
            value: Arc::new(Mutex::new(Some(0.0))),
            live: Live::new(),
        }
    }

    /// The prefix of the ids.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// The end is not known.
    #[must_use]
    pub fn unknown(self) -> Self {
        *self.slot() = None;
        self
    }

    fn slot(&self) -> MutexGuard<'_, Option<f64>> {
        self.value.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// How far it has come, 0 to 100 (a value outside is held to it); the bar is drawn again when the whole percent changed.
    pub fn set(&self, value: f64) {
        let to = value.clamp(0.0, 100.0);
        let changed = {
            let mut v = self.slot();
            let was = v.map(percent);
            *v = Some(to);
            was != Some(percent(to))
        };
        if changed {
            self.live.refresh_now();
        }
    }

    /// The end is not known (any more).
    pub fn set_unknown(&self) {
        let changed = self.slot().take().is_some();
        if changed {
            self.live.refresh_now();
        }
    }

    /// The value now, `None` while the end is not known.
    pub fn get(&self) -> Option<f64> {
        *self.slot()
    }

    /// The bar, `width` pixels wide, to place in a layout.
    pub fn build(&self, width: f32) -> impl Element + use<> {
        let this = self.clone();
        self.live.region("bar", move || this.render(width))
    }

    fn render(&self, width: f32) -> Div {
        let value = self.get();
        let mut bar = div()
            .id(&self.id)
            .class("fs-progress")
            .attr("role", "progressbar")
            .attr("aria-label", &self.name);
        let (fill, words) = if let Some(v) = value {
            {
                let p = percent(v);
                bar = bar
                    .attr("aria-valuemin", "0")
                    .attr("aria-valuemax", "100")
                    .attr("aria-valuenow", p.to_string())
                    .attr("aria-valuetext", format!("{p} %"));
                if p >= 100 {
                    bar = bar.class("fs-progress-done");
                }
                (
                    width * f32::from(u16::try_from(p).unwrap_or(100)) / 100.0,
                    format!("{p} %"),
                )
            }
        } else {
            bar = bar.class("fs-progress-unknown").attr("aria-busy", "true");
            (width, "...".to_owned())
        };
        bar.child(
            div().class("fs-progress-track").w(width).child(
                div()
                    .id(format!("{}-fill", self.id))
                    .class("fs-progress-fill")
                    .w(fill),
            ),
        )
        .child(text(words).class("fs-progress-text"))
    }
}
