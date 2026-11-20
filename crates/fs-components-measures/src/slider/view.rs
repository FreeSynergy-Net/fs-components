//! The slider as an element and its handle.
//!
//! Pattern: Observer + Facade — a clonable handle over the value; every change draws the slider again through `Live` (the thumb
//! keeps the keyboard by its id).

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::scale::{Scale, number};
use crate::Element;
use crate::keys::{KeyEvent, bind_keys};
use crate::live::Live;

type OnChange = Arc<dyn Fn(f64) + Send + Sync>;
type Words = Arc<dyn Fn(f64) -> String + Send + Sync>;

/// The width of the thumb, in pixels (the track has this much less to run on).
pub const THUMB: f32 = 20.0;
/// The room the number beside the track has, in pixels.
const VALUE: f32 = 80.0;
/// The height of the rail, in pixels (the sheet gives the same: `--space-2`).
const RAIL: f32 = 8.0;

/// The rail the filled part runs on: as wide as the track, centred on the thumb's height (the thumb is taller than the rail).
pub(crate) fn rail(width: f32) -> Div {
    div()
        .absolute()
        .left(0.0)
        .top((THUMB - RAIL) / 2.0)
        .w(width)
        .child(div().class("fs-slider-rail").w(width))
}

/// A slider. Clones share the value and the callbacks.
#[derive(Clone)]
pub struct Slider {
    id: String,
    name: String,
    scale: Scale,
    value: Arc<Mutex<f64>>,
    active: Arc<Mutex<bool>>,
    live: Live,
    words: Option<Words>,
    on_change: Option<OnChange>,
}

impl Slider {
    /// A slider on `scale` named `name` (`aria-label`: "Volume"), at the least value. Ids: `<id>` (the thumb), `<id>-track`.
    pub fn new(scale: Scale, name: &str) -> Self {
        Self {
            id: "slider".to_owned(),
            name: name.to_owned(),
            scale,
            value: Arc::new(Mutex::new(scale.min())),
            active: Arc::new(Mutex::new(false)),
            live: Live::new(),
            words: None,
            on_change: None,
        }
    }

    /// The prefix of the ids.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// Start at this value (snapped to the scale).
    #[must_use]
    pub fn value(self, value: f64) -> Self {
        *self.slot() = self.scale.snap(value);
        self
    }

    /// How the value is said ("40 %", "3 of 10"): `aria-valuetext` and the shown number.
    #[must_use]
    pub fn words(mut self, words: impl Fn(f64) -> String + Send + Sync + 'static) -> Self {
        self.words = Some(Arc::new(words));
        self
    }

    /// Call `handler` with the new value after every change by the person.
    #[must_use]
    pub fn on_change(mut self, handler: impl Fn(f64) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    fn slot(&self) -> MutexGuard<'_, f64> {
        self.value.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The value now.
    pub fn get(&self) -> f64 {
        *self.slot()
    }

    /// Set the value from the program (snapped; no callback).
    pub fn set(&self, value: f64) {
        let to = self.scale.snap(value);
        if (*self.slot() - to).abs() > f64::EPSILON {
            *self.slot() = to;
            self.live.refresh_now();
        }
    }

    fn change(&self, to: f64) -> bool {
        let to = self.scale.snap(to);
        {
            let mut v = self.slot();
            if (*v - to).abs() <= f64::EPSILON {
                return false;
            }
            *v = to;
        }
        self.live.refresh_now();
        if let Some(handler) = &self.on_change {
            handler(to);
        }
        true
    }

    /// A press of the keyboard.
    pub fn press(&self, event: KeyEvent) -> bool {
        match self.scale.key(self.get(), event.key) {
            Some(to) => {
                self.change(to);
                true
            }
            None => false,
        }
    }

    fn said(&self, value: f64) -> String {
        self.words
            .as_ref()
            .map_or_else(|| number(value), |w| w(value))
    }

    /// The slider, `width` pixels wide, to place in a layout.
    ///
    /// Three layers, because of how Blinc draws a live area again: an area whose sizes changed keeps the old attributes, and an area
    /// whose structure changed loses the keyboard. So the thumb that the keyboard stays on (`aria` region) has a size that never
    /// changes and only its attributes change; what moves (`visual` region: the filled part and the drawn thumb) holds no keyboard;
    /// and the track around both stays and carries the pointer, so a drag is not cut off by the redraw it causes.
    pub fn build(&self, width: f32) -> impl Element + use<> {
        let run = (width - THUMB).max(1.0);
        let (down, drag, up, aria, visual, value) = (
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
            self.clone(),
        );
        let track = div()
            .id(format!("{}-track", self.id))
            .class("fs-slider-track")
            .w(width)
            .h(THUMB)
            .flex_shrink_0()
            .on_mouse_down(move |ctx| {
                down.hold(true);
                down.to_x(ctx.local_x, run);
            })
            .on_drag(move |ctx| drag.to_x(ctx.mouse_x - ctx.bounds_x, run))
            .on_mouse_up(move |_| up.hold(false))
            .child(self.live.region("aria", move || aria.keyboard(width)))
            .child(self.live.region("visual", move || visual.visual(width)));
        div()
            .class("fs-slider")
            .child(track)
            .child(
                div()
                    .w(VALUE)
                    .flex_shrink_0()
                    .child(self.live.region("value", move || {
                        div()
                            .class("fs-slider-value")
                            .child(text(value.said(value.get())))
                    })),
            )
    }

    fn hold(&self, on: bool) {
        *self.active.lock().unwrap_or_else(PoisonError::into_inner) = on;
        self.live.refresh_now();
    }

    /// The pointer is at `x` along the track (pixels from its left edge): the thumb's centre goes there.
    fn to_x(&self, x: f32, run: f32) {
        let f = f64::from(((x - THUMB / 2.0) / run).clamp(0.0, 1.0));
        self.change(self.scale.at(f));
    }

    /// The layer the keyboard and the ARIA state live on: as big as the track, never moves, invisible; the pointer goes through it.
    fn keyboard(&self, width: f32) -> Div {
        let value = self.get();
        let keys = self.clone();
        let layer = bind_keys(
            div()
                .id(&self.id)
                .class("fs-slider-keys")
                .absolute()
                .left(0.0)
                .top(0.0)
                .w(width)
                .h(THUMB)
                .attr("role", "slider")
                .attr("aria-label", &self.name)
                .attr("aria-orientation", "horizontal")
                .attr("aria-valuenow", number(value))
                .attr("aria-valuemin", number(self.scale.min()))
                .attr("aria-valuemax", number(self.scale.max()))
                .attr("aria-valuetext", self.said(value))
                .pointer_events_none(),
            Arc::new(move |event| keys.press(event)),
        );
        div().h(0.0).child(layer)
    }

    /// The layer that moves: the filled part and the drawn thumb. The ring of the keyboard is drawn on the thumb by the class the
    /// keys layer sets through its focus (see `fs-slider-keys:focus`), which the sheet reads.
    fn visual(&self, width: f32) -> Div {
        let run = (width - THUMB).max(1.0);
        #[allow(clippy::cast_possible_truncation)]
        let fill = self.scale.fraction(self.get()) as f32 * run;
        let mut thumb = div()
            .id(format!("{}-thumb", self.id))
            .class("fs-slider-thumb");
        if *self.active.lock().unwrap_or_else(PoisonError::into_inner) {
            thumb = thumb.class("fs-slider-thumb-active");
        }
        div().w(width).h(THUMB).child(rail(width)).child(
            div()
                .absolute()
                .left(0.0)
                .top(0.0)
                .h(THUMB)
                .flex_row()
                .items_center()
                .child(div().class("fs-slider-fill").w(fill))
                .child(thumb),
        )
    }
}
