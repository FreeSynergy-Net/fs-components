//! The handle of a timeline: shared state, source, events, and the redraw.
//!
//! Pattern: Facade + Observer — one clonable handle over [`TimelineState`]; each call changes the
//! state, tells the host through [`TimelineEvents`], and asks the live area to draw again (only
//! the timeline is built again, never per frame).

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use blinc_layout::selector::ScrollRef;
use fs_components_calendar_grid::texts::Chain;
use fs_components_fields::{Look, Say};
use fs_components_mf2::NeutralDateNames;
use fs_components_widgets::Key;
use fs_components_widgets::live::Live;
use fs_components_words::{DateNames, Locale};

use crate::model::Zoom;
use crate::source::{NoEvents, TimelineEvents, TimelineSource};
use crate::state::{Step, TimelineState};

/// A timeline. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct Timeline {
    state: Arc<Mutex<TimelineState>>,
    pub(crate) source: Arc<dyn TimelineSource>,
    events: Arc<dyn TimelineEvents>,
    host_say: Arc<dyn Say>,
    pub(crate) say: Arc<Chain>,
    pub(crate) dates: Arc<dyn DateNames>,
    pub(crate) look: Look,
    live: Live,
    pub(crate) scroll: ScrollRef,
    /// Width of the view as built last, in pixels (as `f32` bits).
    pub(crate) width: Arc<AtomicU32>,
    /// A scroll asked for and not yet drawn: the position and the offset the view had then.
    pub(crate) target: Arc<Mutex<Option<(f32, f32, Instant)>>>,
    /// The scroll asked for before the first build (the width was not known), whether read since or not.
    pub(crate) early: Arc<Mutex<Option<f32>>>,
    /// The view has been built at least once.
    pub(crate) built: Arc<AtomicBool>,
}

impl Timeline {
    /// A timeline over `source`, words from `say` (the catalog of the calendar components where it has none: the buttons are those of the calendar grid), in `look`.
    pub fn new(
        state: TimelineState,
        source: Arc<dyn TimelineSource>,
        say: Arc<dyn Say>,
        look: Look,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
            source,
            events: Arc::new(NoEvents),
            say: Arc::new(Chain::new(Arc::clone(&say), &Locale::english())),
            host_say: say,
            dates: Arc::new(NeutralDateNames),
            look,
            live: Live::new(),
            scroll: fs_components_primitives::scroll_ref(),
            width: Arc::new(AtomicU32::new(0)),
            target: Arc::default(),
            early: Arc::default(),
            built: Arc::default(),
        }
    }

    /// The language of the reader: the catalog words in `locale`, the names of the months from `dates` (`DrawCtx.dates`).
    #[must_use]
    pub fn localised(mut self, locale: &Locale, dates: Arc<dyn DateNames>) -> Self {
        self.say = Arc::new(Chain::new(Arc::clone(&self.host_say), locale));
        self.dates = dates;
        self
    }

    /// Tell `events` of the reader's choices.
    #[must_use]
    pub fn events(mut self, events: Arc<dyn TimelineEvents>) -> Self {
        self.events = events;
        self
    }

    /// A copy of the state.
    pub fn snapshot(&self) -> TimelineState {
        self.lock().clone()
    }

    pub(crate) fn lock(&self) -> MutexGuard<'_, TimelineState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Carry out a step. `old` is the zoom before it: a changed zoom is rescaled here, so the
    /// toolbar, the keys and the API all take the one way.
    fn apply(&self, step: Step, old: Zoom) -> Step {
        match &step {
            Step::Select(k) => self.events.select_item(k),
            Step::Zoomed(z) => {
                self.rescale(old, *z);
                self.events.zoomed(*z);
            }
            Step::Ignored => return step,
        }
        self.live.refresh();
        step
    }

    /// Choose an item.
    pub fn select(&self, key: &str) -> Step {
        let step = self.lock().select(key);
        let zoom = self.lock().zoom;
        self.apply(step, zoom)
    }

    /// Set the zoom level.
    pub fn set_zoom(&self, zoom: Zoom) -> Step {
        let (old, step) = {
            let mut state = self.lock();
            (state.zoom, state.set_zoom(zoom))
        };
        self.apply(step, old)
    }

    /// A key press; true when the timeline used it.
    pub fn key(&self, key: Key) -> bool {
        let items = self.source.items();
        let (old, step) = {
            let mut state = self.lock();
            (state.zoom, state.key(&items, key))
        };
        self.apply(step, old) != Step::Ignored
    }

    /// Read the source again and draw: the host changed its data.
    pub fn refresh(&self) {
        self.live.refresh();
    }

    /// The element of `width` x `height` pixels. The keyboard binding sits on a stable outer box;
    /// inside it a live area is built again after each change, nothing else.
    pub fn build(
        &self,
        width: f32,
        height: f32,
    ) -> impl blinc_layout::prelude::ElementBuilder + use<> {
        self.width.store(width.to_bits(), Ordering::Relaxed);
        self.reclamp();
        let (me, keys) = (self.clone(), self.clone());
        let inner = self.live.region("timeline", move || {
            crate::view::timeline(&me, width, height)
        });
        crate::view::bind_keys(
            blinc_layout::div::div().id("tl-root").w(width).h(height),
            Arc::new(move |k| keys.key(k)),
        )
        .child(inner)
    }
}

impl Timeline {
    /// The first and last day of the axis at `zoom` for a view `width` pixels wide: all items with a margin, the titles that
    /// stand right of their bar, and at least up to the right edge of the view (so the grid fills it).
    pub(crate) fn axis_range(
        &self,
        zoom: Zoom,
        width: f32,
    ) -> (crate::model::Day, crate::model::Day) {
        use crate::layout::{Scale, extend_to, item_rect, outside_label_x, range};
        let items = self.source.items();
        let today = self.lock().today;
        let (first, mut last) = range(
            &items,
            7,
            today.unwrap_or(crate::model::Day::from_number(0)),
        );
        let scale = Scale {
            origin: first,
            px: zoom.px_per_day(),
        };
        let g = self.rows();
        let pad = self.look.style.spacing.xs;
        for it in &items {
            let r = item_rect(it, 0, &scale, g);
            if matches!(it.kind, crate::model::Kind::Bar { .. }) {
                let w = self.title_width(&it.title);
                if let Some(x) = outside_label_x(r, w, pad) {
                    last = extend_to(last, &scale, x + w + pad);
                }
            }
        }
        (first, extend_to(last, &scale, width))
    }

    /// The heights of the rows of this style.
    pub(crate) fn rows(&self) -> crate::layout::Rows {
        let s = self.look.style;
        crate::layout::Rows {
            row: s.heights.row_dense + s.spacing.xs,
            bar: s.heights.row_dense - s.spacing.xs,
        }
    }

    /// The width of a title in the caption size, in pixels.
    pub(crate) fn title_width(&self, title: &str) -> f32 {
        blinc_layout::text_measure::measure_text(title, self.look.style.text.caption).width
    }
}

impl Timeline {
    /// The words on the axis (months with their year, day numbers) for a view `width` pixels wide, as drawn.
    pub fn axis_texts(&self, width: f32) -> Vec<String> {
        let zoom = self.snapshot().zoom;
        let range = self.axis_range(zoom, width);
        let scale = crate::layout::Scale {
            origin: range.0,
            px: zoom.px_per_day(),
        };
        crate::view::axis::texts(self, zoom, range, &scale)
    }
}
