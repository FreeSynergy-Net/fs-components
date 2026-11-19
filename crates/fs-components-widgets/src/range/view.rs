//! The range slider as an element and its handle.
//!
//! Pattern: Observer + Facade — a clonable handle over the two values; every change draws the thumbs again through `Live`.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::div::div;
use blinc_layout::prelude::text;

use crate::Element;
use crate::keys::KeyEvent;
use crate::live::Live;
use crate::slider::{Scale, THUMB, number};

type OnChange = Arc<dyn Fn(f64, f64) + Send + Sync>;

/// Which thumb.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Which {
    /// The low one.
    Low,
    /// The high one.
    High,
}

impl Which {
    /// The value `to` as this thumb may have it: never past the other thumb `(low, high)` and inside the scale.
    pub fn clamp(self, to: f64, (low, high): (f64, f64)) -> f64 {
        match self {
            Self::Low => to.min(high),
            Self::High => to.max(low),
        }
    }
}

/// A range slider. Clones share the values and the callback.
#[derive(Clone)]
pub struct Range {
    pub(super) id: String,
    pub(super) names: (String, String),
    pub(super) scale: Scale,
    values: Arc<Mutex<(f64, f64)>>,
    grabbed: Arc<Mutex<Option<Which>>>,
    pub(super) focus: Arc<Mutex<Option<Which>>>,
    pub(super) live: Live,
    on_change: Option<OnChange>,
}

impl Range {
    /// A range on `scale`; the thumbs are named `low` and `high` ("Lowest price", "Highest price") and start at the two ends.
    /// Ids: `<id>-low`, `<id>-high`, `<id>-track`.
    pub fn new(scale: Scale, low: &str, high: &str) -> Self {
        Self {
            id: "range".to_owned(),
            names: (low.to_owned(), high.to_owned()),
            scale,
            values: Arc::new(Mutex::new((scale.min(), scale.max()))),
            grabbed: Arc::new(Mutex::new(None)),
            focus: Arc::new(Mutex::new(None)),
            live: Live::new(),
            on_change: None,
        }
    }

    /// The prefix of the ids.
    #[must_use]
    pub fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self
    }

    /// Start with these values (snapped; the low one is never above the high one).
    #[must_use]
    pub fn values(self, low: f64, high: f64) -> Self {
        let (low, high) = (self.scale.snap(low), self.scale.snap(high));
        *self.slot() = (low.min(high), high.max(low));
        self
    }

    /// Call `handler` with `(low, high)` after every change by the person.
    #[must_use]
    pub fn on_change(mut self, handler: impl Fn(f64, f64) + Send + Sync + 'static) -> Self {
        self.on_change = Some(Arc::new(handler));
        self
    }

    fn slot(&self) -> MutexGuard<'_, (f64, f64)> {
        self.values.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The values `(low, high)` now.
    pub fn get(&self) -> (f64, f64) {
        *self.slot()
    }

    /// Move one thumb to `to` (snapped, and held back by the other thumb); true when it changed.
    pub fn change(&self, which: Which, to: f64) -> bool {
        let now = {
            let mut v = self.slot();
            let to = which.clamp(self.scale.snap(to), *v);
            let was = *v;
            match which {
                Which::Low => v.0 = to,
                Which::High => v.1 = to,
            }
            if *v == was {
                return false;
            }
            *v
        };
        self.live.refresh_now();
        if let Some(handler) = &self.on_change {
            handler(now.0, now.1);
        }
        true
    }

    /// A press of the keyboard on one thumb.
    pub fn press(&self, which: Which, event: KeyEvent) -> bool {
        let (low, high) = self.get();
        let value = if which == Which::Low { low } else { high };
        match self.scale.key(value, event.key) {
            Some(to) => {
                self.change(which, to);
                true
            }
            None => false,
        }
    }

    /// The pixel where the centre of a thumb is, from the left edge of the track: the low thumb sits after its spacer, the high one
    /// after the low thumb, the filled part and its own spacer, so it has one more thumb width in front of it.
    fn centre(&self, which: Which, run: f32) -> f32 {
        let (low, high) = self.get();
        #[allow(clippy::cast_possible_truncation)]
        match which {
            Which::Low => self.scale.fraction(low) as f32 * run + THUMB / 2.0,
            Which::High => self.scale.fraction(high) as f32 * run + THUMB * 1.5,
        }
    }

    /// The thumb nearer to the pixel `x` (the low one on a tie when `x` is below its centre, else the high one).
    fn nearer(&self, x: f32, run: f32) -> Which {
        let (a, b) = (
            (x - self.centre(Which::Low, run)).abs(),
            (x - self.centre(Which::High, run)).abs(),
        );
        if a < b || (a <= b && x < self.centre(Which::Low, run)) {
            Which::Low
        } else {
            Which::High
        }
    }

    /// Move `which` so that its centre is at the pixel `x`.
    fn to_x(&self, which: Which, x: f32, run: f32) {
        let lead = if which == Which::Low { 0.5 } else { 1.5 };
        let f = f64::from(((x - THUMB * lead) / run).clamp(0.0, 1.0));
        self.change(which, self.scale.at(f));
    }

    /// The range slider, `width` pixels wide, to place in a layout. Layers as in `Slider::build`: the two keyboard layers (one per
    /// thumb, each the size of the track, invisible, the pointer goes through) only change their attributes; what moves is drawn in
    /// the `visual` region; the track stays and carries the pointer.
    pub fn build(&self, width: f32) -> impl Element + use<> {
        let run = (width - THUMB * 2.0).max(1.0);
        let (down, drag, up, aria, visual, shown) = (
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
                let which = down.nearer(ctx.local_x, run);
                *down.grabbed.lock().unwrap_or_else(PoisonError::into_inner) = Some(which);
                down.to_x(which, ctx.local_x, run);
            })
            .on_drag(move |ctx| {
                let held = *drag.grabbed.lock().unwrap_or_else(PoisonError::into_inner);
                if let Some(which) = held {
                    drag.to_x(which, ctx.mouse_x - ctx.bounds_x, run);
                }
            })
            .on_mouse_up(move |_| {
                *up.grabbed.lock().unwrap_or_else(PoisonError::into_inner) = None;
            })
            .child(self.live.region("aria", move || aria.keyboard(width)))
            .child(self.live.region("visual", move || visual.thumbs(run)));
        div()
            .class("fs-slider")
            .class("fs-range")
            .child(track)
            .child(
                div()
                    .w(160.0)
                    .flex_shrink_0()
                    .child(self.live.region("value", move || {
                        let (low, high) = shown.get();
                        div().class("fs-slider-value").child(text(format!(
                            "{} - {}",
                            number(low),
                            number(high)
                        )))
                    })),
            )
    }
}
